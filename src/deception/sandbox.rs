//! Controlled Inspection Sandbox Environment (Milestone 04.6).
//!
//! Exposes a strictly bounded view of an `InspectionSession` to authorized inspection agents:
//! - Rejects path traversals and symlink escapes outside `workspace_path`
//! - Disallows access to original secrets or unrelated host paths
//! - Disallows outbound network calls (`outbound_network_allowed = false`)
//! - Enforces session expiration TTL
//! - Records structured `WorkspaceAccessRecord` events for Stage 05/06 correlation

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use super::session::{InspectionSession, SessionStatus};
use crate::deception::decoy::{DecoyArtifact, DecoyRegistry};
use crate::errors::{BrahmError, Result};

/// Record of an authorized agent reading or inspecting a path inside the sandbox.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceAccessRecord {
    pub session_id: String,
    pub timestamp: DateTime<Utc>,
    pub requested_path: String,
    pub allowed: bool,
    pub matched_artifact_id: Option<String>,
    pub matched_canary_token: Option<String>,
    pub reason: String,
}

/// Strictly bounded workspace wrapper for Stage 05 agent adapters.
#[derive(Debug, Clone)]
pub struct ControlledInspectionEnvironment {
    session: InspectionSession,
    canonical_workspace: PathBuf,
    decoy_registry: DecoyRegistry,
    outbound_network_allowed: bool,
    access_log: Vec<WorkspaceAccessRecord>,
}

impl ControlledInspectionEnvironment {
    /// Bind a sandbox environment to an `Active` (or `Prepared`) non-expired `InspectionSession`.
    pub fn open(session: InspectionSession, decoy_registry: DecoyRegistry) -> Result<Self> {
        if Utc::now() > session.expires_at {
            return Err(BrahmError::Session(format!(
                "Inspection environment cannot open expired session '{}'",
                session.session_id
            )));
        }
        if !matches!(
            session.status,
            SessionStatus::Prepared | SessionStatus::Active
        ) {
            return Err(BrahmError::Session(format!(
                "Session '{}' in state {:?} cannot open inspection sandbox",
                session.session_id, session.status
            )));
        }

        let canonical_workspace =
            fs::canonicalize(&session.workspace_path).map_err(|e| BrahmError::InvalidPath {
                path: session.workspace_path.clone(),
                reason: format!("Inspection workspace unavailable: {}", e),
            })?;

        Ok(Self {
            session,
            canonical_workspace,
            decoy_registry,
            outbound_network_allowed: false,
            access_log: Vec::new(),
        })
    }

    pub fn session(&self) -> &InspectionSession {
        &self.session
    }

    pub fn outbound_network_allowed(&self) -> bool {
        self.outbound_network_allowed
    }

    pub fn access_log(&self) -> &[WorkspaceAccessRecord] {
        &self.access_log
    }

    /// List relative file paths available inside the isolated inspection workspace.
    pub fn list_workspace_files(&self) -> Result<Vec<String>> {
        self.ensure_not_expired()?;
        let mut files = Vec::new();
        for entry in walkdir::WalkDir::new(&self.canonical_workspace)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                if let Ok(rel) = entry.path().strip_prefix(&self.canonical_workspace) {
                    files.push(rel.to_string_lossy().replace('\\', "/"));
                }
            }
        }
        files.sort();
        Ok(files)
    }

    /// Read a relative file inside the isolated workspace while enforcing strict boundary confinement
    /// and recording canary/decoy interactions.
    pub fn read_workspace_file(&mut self, relative_path: &str) -> Result<String> {
        self.ensure_not_expired()?;

        let clean_rel = relative_path.replace('\\', "/");
        if clean_rel.starts_with('/') || clean_rel.contains("..") || clean_rel.contains(':') {
            self.record_access(
                &clean_rel,
                false,
                None,
                None,
                "Path traversal attempt rejected",
            );
            return Err(BrahmError::Unauthorized(format!(
                "Rejected unsafe path outside inspection workspace: '{}'",
                relative_path
            )));
        }

        let joined = self.canonical_workspace.join(&clean_rel);
        let canon_target = match fs::canonicalize(&joined) {
            Ok(p) => p,
            Err(_) => {
                self.record_access(
                    &clean_rel,
                    false,
                    None,
                    None,
                    "Requested path does not exist",
                );
                return Err(BrahmError::InvalidPath {
                    path: PathBuf::from(&clean_rel),
                    reason: "File not found in inspection workspace".to_string(),
                });
            }
        };

        if !canon_target.starts_with(&self.canonical_workspace) {
            self.record_access(
                &clean_rel,
                false,
                None,
                None,
                "Symlink boundary escape rejected",
            );
            return Err(BrahmError::Unauthorized(format!(
                "Target '{}' resolves outside bounded workspace",
                relative_path
            )));
        }

        let matched = self
            .match_registered_decoy(&clean_rel)
            .map(|a| (a.artifact_id.clone(), a.canary_token.clone()));
        let reason = if matched.is_some() {
            "Registered decoy/canary artifact accessed inside sandbox".to_string()
        } else {
            "Mirrored genuine file accessed inside sandbox".to_string()
        };

        self.record_access(
            &clean_rel,
            true,
            matched.as_ref().map(|(id, _)| id.clone()),
            matched.and_then(|(_, tok)| tok),
            &reason,
        );

        let content = fs::read_to_string(&canon_target)?;
        Ok(content)
    }

    fn ensure_not_expired(&self) -> Result<()> {
        if Utc::now() > self.session.expires_at {
            return Err(BrahmError::Session(format!(
                "Session '{}' TTL exceeded during sandbox operation",
                self.session.session_id
            )));
        }
        Ok(())
    }

    fn match_registered_decoy(&self, clean_rel: &str) -> Option<&DecoyArtifact> {
        let stripped = clean_rel
            .strip_prefix("_synthetic_overlay/")
            .unwrap_or(clean_rel);
        self.decoy_registry
            .artifacts()
            .iter()
            .find(|a| a.synthetic_relative_path == stripped)
    }

    fn record_access(
        &mut self,
        rel_path: &str,
        allowed: bool,
        matched_artifact_id: Option<String>,
        matched_canary_token: Option<String>,
        reason: &str,
    ) {
        self.access_log.push(WorkspaceAccessRecord {
            session_id: self.session.session_id.clone(),
            timestamp: Utc::now(),
            requested_path: rel_path.to_string(),
            allowed,
            matched_artifact_id,
            matched_canary_token,
            reason: reason.to_string(),
        });
    }

    /// Expose canonical workspace root strictly read-only for local authorized inspection processes.
    pub fn canonical_workspace_root(&self) -> &Path {
        &self.canonical_workspace
    }
}
