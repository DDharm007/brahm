//! Inspection Session Lifecycle Manager (Milestone 04.4 & 04.7).
//!
//! Manages isolated inspection sessions (`Created -> Prepared -> Active -> Completed/Expired -> Cleaned`),
//! persistence, concurrency isolation, and Stage 04 deliverable report generation.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use super::layers::{ProductionIntegrityGuard, ProductionIntegrityReport};
use super::mirror::{ControlledMirrorManager, InspectionManifest, MirrorPreparationResult};
use super::policy::{DeceptionPolicy, DeceptionPolicyProfile};
use crate::deception::decoy::DecoyRegistry;
use crate::errors::{BrahmError, Result};

/// Lifecycle state of an inspection session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Created,
    Prepared,
    Active,
    Suspended,
    Completed,
    Expired,
    Failed,
    Cleaned,
}

/// Persistent session record (`BRAHM_SESSION_REPORT.json`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InspectionSession {
    pub session_id: String,
    pub repository_id: String,
    pub genuine_repo_path: PathBuf,
    pub workspace_path: PathBuf,
    pub state_dir: PathBuf,
    pub policy_id: String,
    pub policy_version: String,
    pub policy_profile: DeceptionPolicyProfile,
    pub registered_artifacts: Vec<String>,
    pub baseline_fingerprint: String,
    pub inspection_view_fingerprint: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: SessionStatus,
    pub audit_notes: Vec<String>,
}

/// Full outcome returned after preparing a deception workspace session.
#[derive(Debug, Clone)]
pub struct DeceptionPreparedOutcome {
    pub session: InspectionSession,
    pub mirror: MirrorPreparationResult,
    pub policy: DeceptionPolicy,
    pub integrity_report: ProductionIntegrityReport,
}

/// Manager handling inspection session lifecycle, isolation, persistence, and cleanup.
#[derive(Debug, Clone)]
pub struct InspectionSessionManager {
    sessions_root: PathBuf,
    mirror_manager: ControlledMirrorManager,
    integrity_guard: ProductionIntegrityGuard,
}

impl InspectionSessionManager {
    /// Initialize a session manager rooted under `sessions_root`.
    pub fn new(sessions_root: impl Into<PathBuf>) -> Self {
        Self {
            sessions_root: sessions_root.into(),
            mirror_manager: ControlledMirrorManager::new(),
            integrity_guard: ProductionIntegrityGuard::new(),
        }
    }

    /// Create and fully prepare a new isolated inspection session, emitting Stage 04 deliverables.
    pub fn create_and_prepare(
        &self,
        genuine_repo_root: &Path,
        decoy_registry: &DecoyRegistry,
        decoy_layer_root: &Path,
        policy: &DeceptionPolicy,
        ttl_seconds: i64,
    ) -> Result<DeceptionPreparedOutcome> {
        policy.validate()?;

        // 1. Validate production isolation BEFORE preparing inspection mirror
        let integrity_manifest = crate::deception::decoy::DecoyManifest {
            schema_version: "1.0".to_string(),
            generated_at: Utc::now(),
            repository_id: decoy_registry.data().repository_id.clone(),
            total_artifacts: decoy_registry.artifacts().len(),
            production_build_excluded: true,
            entries: decoy_registry
                .artifacts()
                .iter()
                .map(|a| crate::deception::decoy::DecoyManifestEntry {
                    artifact_id: a.artifact_id.clone(),
                    relative_path: a.synthetic_relative_path.clone(),
                    category: a.artifact_type,
                    integrity_fingerprint: a.integrity_fingerprint.clone(),
                    brahm_owned: a.brahm_owned,
                    synthetic_status: a.synthetic_status,
                    canary_token: a.canary_token.clone(),
                })
                .collect(),
        };

        let integrity_report = self
            .integrity_guard
            .verify_production_isolation(genuine_repo_root, &integrity_manifest)?;
        if !integrity_report.production_isolated {
            return Err(BrahmError::Deception(format!(
                "Production integrity guard blocked session creation: {} contamination error(s)",
                integrity_report.failures_count
            )));
        }

        // 2. Allocate strictly isolated workspace for this session ID
        let session_id = format!("sess_{}", &Uuid::new_v4().simple().to_string()[..12]);
        let session_state_dir = self.sessions_root.join(&session_id);
        let workspace_path = session_state_dir.join("workspace");
        fs::create_dir_all(&workspace_path)?;

        let canon_genuine = fs::canonicalize(genuine_repo_root)?;
        let mirror = self.mirror_manager.assemble_mirror(
            &session_id,
            &canon_genuine,
            decoy_registry,
            decoy_layer_root,
            policy,
            &workspace_path,
        )?;

        let now = Utc::now();
        let expires_at = now + Duration::seconds(ttl_seconds.max(60));

        let session = InspectionSession {
            session_id: session_id.clone(),
            repository_id: decoy_registry.data().repository_id.clone(),
            genuine_repo_path: canon_genuine,
            workspace_path: mirror.inspection_layer.workspace_root.clone(),
            state_dir: session_state_dir.clone(),
            policy_id: policy.policy_id.clone(),
            policy_version: policy.version.clone(),
            policy_profile: policy.profile,
            registered_artifacts: mirror.inspection_layer.injected_decoy_ids.clone(),
            baseline_fingerprint: mirror.genuine_layer.baseline_fingerprint.clone(),
            inspection_view_fingerprint: mirror
                .inspection_layer
                .inspection_view_fingerprint
                .clone(),
            created_at: now,
            expires_at,
            updated_at: now,
            status: SessionStatus::Prepared,
            audit_notes: vec![
                format!("Created session {}", session_id),
                format!(
                    "Prepared mirror with {} genuine and {} synthetic artifacts",
                    mirror.manifest.genuine_file_count, mirror.manifest.synthetic_file_count
                ),
            ],
        };

        // 3. Persist all 4 Stage 04 deliverable JSON artifacts in session state directory
        self.persist_deliverables(&session, policy, &mirror.manifest, &integrity_report)?;

        Ok(DeceptionPreparedOutcome {
            session,
            mirror,
            policy: policy.clone(),
            integrity_report,
        })
    }

    /// Persist Stage 04 deliverables (`BRAHM_DECEPTION_POLICY.json`, `BRAHM_INSPECTION_MANIFEST.json`,
    /// `BRAHM_SESSION_REPORT.json`, `BRAHM_INTEGRITY_REPORT.json`) in `session.state_dir`.
    pub fn persist_deliverables(
        &self,
        session: &InspectionSession,
        policy: &DeceptionPolicy,
        manifest: &InspectionManifest,
        integrity: &ProductionIntegrityReport,
    ) -> Result<()> {
        fs::create_dir_all(&session.state_dir)?;
        fs::write(
            session.state_dir.join("BRAHM_DECEPTION_POLICY.json"),
            serde_json::to_string_pretty(policy)?,
        )?;
        fs::write(
            session.state_dir.join("BRAHM_INSPECTION_MANIFEST.json"),
            serde_json::to_string_pretty(manifest)?,
        )?;
        fs::write(
            session.state_dir.join("BRAHM_SESSION_REPORT.json"),
            serde_json::to_string_pretty(session)?,
        )?;
        fs::write(
            session.state_dir.join("BRAHM_INTEGRITY_REPORT.json"),
            serde_json::to_string_pretty(integrity)?,
        )?;
        Ok(())
    }

    /// Load an existing session by ID from `sessions_root`.
    pub fn load_session(&self, session_id: &str) -> Result<InspectionSession> {
        let report_path = self
            .sessions_root
            .join(session_id)
            .join("BRAHM_SESSION_REPORT.json");
        if !report_path.is_file() {
            return Err(BrahmError::Session(format!(
                "Session '{}' not found at '{}'",
                session_id,
                report_path.display()
            )));
        }
        let raw = fs::read_to_string(&report_path)?;
        let session: InspectionSession = serde_json::from_str(&raw)?;
        Ok(session)
    }

    /// Save updated session metadata.
    pub fn save_session(&self, session: &InspectionSession) -> Result<()> {
        fs::create_dir_all(&session.state_dir)?;
        fs::write(
            session.state_dir.join("BRAHM_SESSION_REPORT.json"),
            serde_json::to_string_pretty(session)?,
        )?;
        Ok(())
    }

    /// Transition session to `Active` (rejecting expired or cleaned sessions).
    pub fn activate_session(&self, session_id: &str) -> Result<InspectionSession> {
        let mut session = self.load_session(session_id)?;
        if Utc::now() > session.expires_at {
            session.status = SessionStatus::Expired;
            session.updated_at = Utc::now();
            session
                .audit_notes
                .push("Session expired prior to activation".to_string());
            self.save_session(&session)?;
            return Err(BrahmError::Session(format!(
                "Cannot activate session '{}': session expired at {}",
                session_id, session.expires_at
            )));
        }
        if matches!(
            session.status,
            SessionStatus::Completed | SessionStatus::Cleaned | SessionStatus::Failed
        ) {
            return Err(BrahmError::Session(format!(
                "Cannot activate terminal session '{}' in state {:?}",
                session_id, session.status
            )));
        }
        session.status = SessionStatus::Active;
        session.updated_at = Utc::now();
        session.audit_notes.push("Session activated".to_string());
        self.save_session(&session)?;
        Ok(session)
    }

    /// Suspend an active session.
    pub fn suspend_session(&self, session_id: &str) -> Result<InspectionSession> {
        let mut session = self.load_session(session_id)?;
        session.status = SessionStatus::Suspended;
        session.updated_at = Utc::now();
        session.audit_notes.push("Session suspended".to_string());
        self.save_session(&session)?;
        Ok(session)
    }

    /// Complete/stop an active or prepared session.
    pub fn stop_session(&self, session_id: &str) -> Result<InspectionSession> {
        let mut session = self.load_session(session_id)?;
        session.status = SessionStatus::Completed;
        session.updated_at = Utc::now();
        session.audit_notes.push("Session completed".to_string());
        self.save_session(&session)?;
        Ok(session)
    }

    /// Check expiration against wall-clock time, updating status if elapsed.
    pub fn check_expiration(&self, session_id: &str) -> Result<InspectionSession> {
        let mut session = self.load_session(session_id)?;
        if session.status != SessionStatus::Cleaned
            && session.status != SessionStatus::Completed
            && Utc::now() > session.expires_at
        {
            session.status = SessionStatus::Expired;
            session.updated_at = Utc::now();
            session.audit_notes.push("Session TTL expired".to_string());
            self.save_session(&session)?;
        }
        Ok(session)
    }

    /// Safely remove isolated session workspace and mark status `Cleaned`.
    pub fn cleanup_session(&self, session_id: &str) -> Result<InspectionSession> {
        let mut session = self.load_session(session_id)?;
        self.mirror_manager
            .cleanup_mirror(&session.workspace_path, &session.genuine_repo_path)?;
        session.status = SessionStatus::Cleaned;
        session.updated_at = Utc::now();
        session
            .audit_notes
            .push("Isolated workspace cleaned safely".to_string());
        self.save_session(&session)?;
        Ok(session)
    }

    /// List all session reports stored under `sessions_root`.
    pub fn list_sessions(&self) -> Result<Vec<InspectionSession>> {
        let mut sessions = Vec::new();
        if !self.sessions_root.is_dir() {
            return Ok(sessions);
        }
        for entry in fs::read_dir(&self.sessions_root)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let report = entry.path().join("BRAHM_SESSION_REPORT.json");
                if report.is_file() {
                    if let Ok(raw) = fs::read_to_string(&report) {
                        if let Ok(sess) = serde_json::from_str::<InspectionSession>(&raw) {
                            sessions.push(sess);
                        }
                    }
                }
            }
        }
        sessions.sort_by_key(|a| a.created_at);
        Ok(sessions)
    }
}
