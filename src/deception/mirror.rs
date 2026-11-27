//! Controlled Repository Mirror Manager (Milestone 04.2).
//!
//! Creates isolated, read-safe inspection mirrors combining permitted genuine files
//! (strictly stripped of secrets/credentials) and policy-selected synthetic artifacts.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tracing::info;

use super::layers::{is_ignored_repo_dir, GenuineLayer, InspectionLayer, ProductionIntegrityGuard};
use super::policy::{DeceptionPolicy, DeceptionPolicyEngine, PolicyEvaluationResult};
use crate::deception::decoy::{compute_content_fingerprint, DecoyRegistry};
use crate::detection::is_sensitive_credential_file;
use crate::errors::{BrahmError, Result};

/// Entry inside `BRAHM_INSPECTION_MANIFEST.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InspectionManifestEntry {
    pub relative_path: String,
    pub is_synthetic: bool,
    pub artifact_id: Option<String>,
    pub sha_or_blake3: String,
}

/// Deliverable `BRAHM_INSPECTION_MANIFEST.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InspectionManifest {
    pub schema_version: String,
    pub session_id: String,
    pub repository_id: String,
    pub generated_at: DateTime<Utc>,
    pub workspace_root: String,
    pub baseline_fingerprint: String,
    pub inspection_view_fingerprint: String,
    pub policy_id: String,
    pub stripped_secret_count: usize,
    pub genuine_file_count: usize,
    pub synthetic_file_count: usize,
    pub entries: Vec<InspectionManifestEntry>,
}

/// Complete result returned when assembling a controlled mirror workspace.
#[derive(Debug, Clone)]
pub struct MirrorPreparationResult {
    pub genuine_layer: GenuineLayer,
    pub inspection_layer: InspectionLayer,
    pub manifest: InspectionManifest,
    pub policy_evaluation: PolicyEvaluationResult,
}

/// Manages creation and safe removal of isolated inspection mirrors.
#[derive(Debug, Default, Clone)]
pub struct ControlledMirrorManager {
    policy_engine: DeceptionPolicyEngine,
    integrity_guard: ProductionIntegrityGuard,
}

impl ControlledMirrorManager {
    pub fn new() -> Self {
        Self {
            policy_engine: DeceptionPolicyEngine::new(),
            integrity_guard: ProductionIntegrityGuard::new(),
        }
    }

    /// Assemble an isolated inspection workspace for `session_id` at `workspace_root`.
    ///
    /// Guarantees:
    /// 1. `workspace_root` must NOT reside inside `genuine_repo_root` or overwrite it.
    /// 2. Secrets (`.env`, `*.pem`, `*.key`, etc.) are strictly omitted from mirror copy.
    /// 3. Symlinks outside canonical genuine repo root are rejected.
    /// 4. Only policy-permitted decoys from `decoy_layer_root` are injected.
    pub fn assemble_mirror(
        &self,
        session_id: &str,
        genuine_repo_root: &Path,
        decoy_registry: &DecoyRegistry,
        decoy_layer_root: &Path,
        policy: &DeceptionPolicy,
        workspace_root: &Path,
    ) -> Result<MirrorPreparationResult> {
        let canon_genuine =
            fs::canonicalize(genuine_repo_root).map_err(|e| BrahmError::InvalidPath {
                path: genuine_repo_root.to_path_buf(),
                reason: format!("Invalid genuine repository root: {}", e),
            })?;

        fs::create_dir_all(workspace_root)?;
        let canon_workspace = fs::canonicalize(workspace_root)?;

        if canon_workspace == canon_genuine || canon_workspace.starts_with(&canon_genuine) {
            return Err(BrahmError::Deception(
                "Security violation: inspection mirror workspace must be isolated outside genuine repository root"
                    .to_string(),
            ));
        }

        // 1. Capture genuine baseline
        let genuine_layer = self
            .integrity_guard
            .capture_genuine_baseline(&canon_genuine)?;

        // 2. Evaluate policy deterministically
        let evaluation = self
            .policy_engine
            .evaluate(policy, decoy_registry.artifacts())?;

        let mut entries = Vec::new();
        let mut genuine_copied = Vec::new();
        let mut stripped_secrets = Vec::new();

        // 3. Copy safe genuine files (strictly non-secret, non-excluded, non-escaping)
        for walk_entry in walkdir::WalkDir::new(&canon_genuine)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| !is_ignored_repo_dir(e.file_name().to_str().unwrap_or("")))
        {
            let walk_entry = walk_entry
                .map_err(|e| BrahmError::Deception(format!("Mirror walk error: {}", e)))?;

            if !walk_entry.file_type().is_file() {
                continue;
            }

            let abs_src = walk_entry.path();
            if let Ok(canon_src) = fs::canonicalize(abs_src) {
                if !canon_src.starts_with(&canon_genuine) {
                    continue;
                }
            }

            let rel_path = abs_src
                .strip_prefix(&canon_genuine)
                .map_err(|e| BrahmError::Deception(format!("Path strip failed: {}", e)))?;
            let rel_str = rel_path.to_string_lossy().replace('\\', "/");

            let filename = walk_entry.file_name().to_str().unwrap_or("");
            if is_sensitive_credential_file(filename) {
                stripped_secrets.push(rel_str);
                continue;
            }

            let dest = canon_workspace.join(rel_path);
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(abs_src, &dest)?;

            let hash = genuine_layer
                .file_hashes
                .get(&rel_str)
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());

            genuine_copied.push(rel_str.clone());
            entries.push(InspectionManifestEntry {
                relative_path: rel_str,
                is_synthetic: false,
                artifact_id: None,
                sha_or_blake3: hash,
            });
        }

        // 4. Inject permitted synthetic artifacts without overwriting copied genuine files
        let mut injected_decoy_ids = Vec::new();
        for art_id in &evaluation.included_ids {
            let Some(art) = decoy_registry.get(art_id) else {
                continue;
            };
            let rel = &art.synthetic_relative_path;
            if rel.contains("..") || rel.starts_with('/') {
                continue;
            }

            let src_decoy = decoy_layer_root.join(rel);
            if !src_decoy.is_file() {
                return Err(BrahmError::Deception(format!(
                    "Staged decoy '{}' missing at '{}'",
                    art.artifact_id,
                    src_decoy.display()
                )));
            }

            let content = fs::read_to_string(&src_decoy)?;
            let actual_hash = compute_content_fingerprint(&content);
            if actual_hash != art.integrity_fingerprint {
                return Err(BrahmError::Deception(format!(
                    "Integrity check failed for staged decoy '{}' before injection",
                    art.artifact_id
                )));
            }

            // Determine safe target path inside mirror: avoid silently clobbering genuine code
            let target_rel = if canon_workspace.join(rel).exists() {
                format!("_synthetic_overlay/{}", rel)
            } else {
                rel.clone()
            };

            let dest = canon_workspace.join(&target_rel);
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&dest, content)?;

            injected_decoy_ids.push(art.artifact_id.clone());
            entries.push(InspectionManifestEntry {
                relative_path: target_rel,
                is_synthetic: true,
                artifact_id: Some(art.artifact_id.clone()),
                sha_or_blake3: actual_hash,
            });
        }

        // 5. Compute unified inspection-view BLAKE3 fingerprint
        entries.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
        let mut view_hasher = blake3::Hasher::new();
        view_hasher.update(b"brahm-inspection-view-v1:");
        for e in &entries {
            view_hasher.update(e.relative_path.as_bytes());
            view_hasher.update(b":");
            view_hasher.update(e.sha_or_blake3.as_bytes());
            view_hasher.update(b"\n");
        }
        let inspection_view_fingerprint = view_hasher.finalize().to_hex().to_string();

        let inspection_layer = InspectionLayer {
            workspace_root: canon_workspace.clone(),
            session_id: session_id.to_string(),
            genuine_files_copied: genuine_copied.clone(),
            stripped_sensitive_files: stripped_secrets.clone(),
            injected_decoy_ids: injected_decoy_ids.clone(),
            inspection_view_fingerprint: inspection_view_fingerprint.clone(),
        };

        let manifest = InspectionManifest {
            schema_version: "1.0".to_string(),
            session_id: session_id.to_string(),
            repository_id: decoy_registry.data().repository_id.clone(),
            generated_at: Utc::now(),
            workspace_root: canon_workspace.to_string_lossy().replace('\\', "/"),
            baseline_fingerprint: genuine_layer.baseline_fingerprint.clone(),
            inspection_view_fingerprint,
            policy_id: policy.policy_id.clone(),
            stripped_secret_count: stripped_secrets.len(),
            genuine_file_count: genuine_copied.len(),
            synthetic_file_count: injected_decoy_ids.len(),
            entries,
        };

        info!(
            "Assembled isolated mirror for session '{}' ({} genuine, {} synthetic, {} secrets stripped)",
            session_id,
            genuine_copied.len(),
            injected_decoy_ids.len(),
            stripped_secrets.len()
        );

        Ok(MirrorPreparationResult {
            genuine_layer,
            inspection_layer,
            manifest,
            policy_evaluation: evaluation,
        })
    }

    /// Clean up a prepared mirror workspace cleanly while ensuring genuine repo is never touched.
    pub fn cleanup_mirror(&self, workspace_root: &Path, genuine_repo_root: &Path) -> Result<()> {
        if !workspace_root.exists() {
            return Ok(());
        }
        let canon_ws = fs::canonicalize(workspace_root)?;
        if let Ok(canon_genuine) = fs::canonicalize(genuine_repo_root) {
            if canon_ws == canon_genuine || canon_genuine.starts_with(&canon_ws) {
                return Err(BrahmError::Deception(
                    "Refusing cleanup: workspace path overlaps genuine repository".to_string(),
                ));
            }
        }
        fs::remove_dir_all(&canon_ws)?;
        Ok(())
    }
}
