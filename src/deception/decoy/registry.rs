//! Persistent decoy registry with search, integrity verification, and safe ownership-gated cleanup.

use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};

use super::identifiers::compute_content_fingerprint;
use super::models::{
    DecoyArtifact, DecoyCategory, DecoyCleanupResult, DecoyGenerationProfile, DecoyInstallState,
    DecoyMonitoringState, DecoyRegistryData,
};
use crate::errors::{BrahmError, Result};

/// In-memory wrapper around persisted `BRAHM_DECOY_REGISTRY.json`.
#[derive(Debug, Clone)]
pub struct DecoyRegistry {
    data: DecoyRegistryData,
}

impl DecoyRegistry {
    /// Initialize a new empty decoy registry.
    pub fn new(
        repository_id: impl Into<String>,
        repository_name: impl Into<String>,
        decoy_layer_root: impl Into<String>,
        profile: DecoyGenerationProfile,
    ) -> Self {
        Self {
            data: DecoyRegistryData {
                schema_version: "1.0".to_string(),
                repository_id: repository_id.into(),
                repository_name: repository_name.into(),
                decoy_layer_root: decoy_layer_root.into(),
                generation_profile: profile,
                updated_at: Utc::now(),
                artifacts: Vec::new(),
            },
        }
    }

    /// Load an existing registry file from disk.
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let p = path.as_ref();
        if !p.exists() {
            return Err(BrahmError::InvalidPath {
                path: p.to_path_buf(),
                reason: "Decoy registry file does not exist".to_string(),
            });
        }
        let raw = fs::read_to_string(p)?;
        let data: DecoyRegistryData = serde_json::from_str(&raw)?;
        Ok(Self { data })
    }

    /// Persist registry to JSON on disk cleanly.
    pub fn save_to_path<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let p = path.as_ref();
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(&self.data)?;
        fs::write(p, json)?;
        Ok(())
    }

    /// Register a new decoy artifact or update an existing record matching `artifact_id`.
    pub fn upsert(&mut self, artifact: DecoyArtifact) {
        self.data.updated_at = Utc::now();
        if let Some(existing) = self
            .data
            .artifacts
            .iter_mut()
            .find(|a| a.artifact_id == artifact.artifact_id)
        {
            *existing = artifact;
        } else {
            self.data.artifacts.push(artifact);
        }
    }

    /// List all registered artifacts.
    pub fn artifacts(&self) -> &[DecoyArtifact] {
        &self.data.artifacts
    }

    /// Underlying serializable registry model.
    pub fn data(&self) -> &DecoyRegistryData {
        &self.data
    }

    /// Search artifacts by substring query on path/ID/canary and optional category filter.
    pub fn search(
        &self,
        query: Option<&str>,
        category: Option<DecoyCategory>,
    ) -> Vec<&DecoyArtifact> {
        let q_lower = query.map(|q| q.to_ascii_lowercase());
        self.data
            .artifacts
            .iter()
            .filter(|a| {
                if let Some(cat) = category {
                    if a.artifact_type != cat {
                        return false;
                    }
                }
                if let Some(ref q) = q_lower {
                    return a.artifact_id.to_ascii_lowercase().contains(q)
                        || a.synthetic_relative_path.to_ascii_lowercase().contains(q)
                        || a.canary_token
                            .as_ref()
                            .map(|t| t.to_ascii_lowercase().contains(q))
                            .unwrap_or(false);
                }
                true
            })
            .collect()
    }

    /// Lookup artifact by exact ID.
    pub fn get(&self, artifact_id: &str) -> Option<&DecoyArtifact> {
        self.data
            .artifacts
            .iter()
            .find(|a| a.artifact_id == artifact_id)
    }

    /// Lookup artifact by exact canary marker token.
    pub fn find_by_canary(&self, token: &str) -> Option<&DecoyArtifact> {
        self.data
            .artifacts
            .iter()
            .find(|a| a.canary_token.as_deref() == Some(token))
    }

    /// Update the installation state of an artifact.
    pub fn set_install_state(&mut self, artifact_id: &str, state: DecoyInstallState) -> Result<()> {
        let entry = self
            .data
            .artifacts
            .iter_mut()
            .find(|a| a.artifact_id == artifact_id)
            .ok_or_else(|| {
                BrahmError::Decoy(format!("Unknown decoy artifact ID '{}'", artifact_id))
            })?;
        entry.install_state = state;
        self.data.updated_at = Utc::now();
        Ok(())
    }

    /// Explicitly deactivate monitoring and mark inactive.
    pub fn deactivate(&mut self, artifact_id: &str) -> Result<()> {
        let entry = self
            .data
            .artifacts
            .iter_mut()
            .find(|a| a.artifact_id == artifact_id)
            .ok_or_else(|| {
                BrahmError::Decoy(format!("Unknown decoy artifact ID '{}'", artifact_id))
            })?;
        entry.install_state = DecoyInstallState::Inactive;
        entry.monitoring_state = DecoyMonitoringState::Disabled;
        self.data.updated_at = Utc::now();
        Ok(())
    }

    /// Verify staged files on disk against stored BLAKE3 fingerprints.
    /// Returns list of `(artifact_id, matches, diagnostic_message)`.
    pub fn verify_staged_integrity(&self, decoy_layer_root: &Path) -> Vec<(String, bool, String)> {
        let mut results = Vec::new();

        for art in &self.data.artifacts {
            if art.install_state == DecoyInstallState::Removed {
                continue;
            }
            let target = decoy_layer_root.join(&art.synthetic_relative_path);
            if !target.is_file() {
                results.push((
                    art.artifact_id.clone(),
                    false,
                    format!("Missing staged file '{}'", art.synthetic_relative_path),
                ));
                continue;
            }
            match fs::read_to_string(&target) {
                Ok(content) => {
                    let actual_hash = compute_content_fingerprint(&content);
                    if actual_hash == art.integrity_fingerprint {
                        results.push((
                            art.artifact_id.clone(),
                            true,
                            "Integrity verified".to_string(),
                        ));
                    } else {
                        results.push((
                            art.artifact_id.clone(),
                            false,
                            format!(
                                "Fingerprint mismatch on '{}' (expected {}, found {})",
                                art.synthetic_relative_path,
                                &art.integrity_fingerprint[..12],
                                &actual_hash[..12]
                            ),
                        ));
                    }
                }
                Err(e) => {
                    results.push((
                        art.artifact_id.clone(),
                        false,
                        format!("Failed reading '{}': {}", art.synthetic_relative_path, e),
                    ));
                }
            }
        }

        results
    }

    /// Safely remove ONLY verified BRAHM-owned decoy artifacts from `decoy_layer_root`.
    /// Never removes files outside the decoy layer or files failing fingerprint ownership checks.
    pub fn cleanup_owned_artifacts(
        &mut self,
        decoy_layer_root: &Path,
        dry_run: bool,
    ) -> Result<DecoyCleanupResult> {
        let canonical_root =
            fs::canonicalize(decoy_layer_root).unwrap_or_else(|_| decoy_layer_root.to_path_buf());
        let mut removed_ids = Vec::new();
        let mut removed_paths = Vec::new();
        let mut skipped_unverified = Vec::new();

        for art in &mut self.data.artifacts {
            if art.install_state == DecoyInstallState::Removed {
                continue;
            }
            if !art.brahm_owned || !art.synthetic_status {
                skipped_unverified.push(art.synthetic_relative_path.clone());
                continue;
            }

            let rel = &art.synthetic_relative_path;
            if rel.contains("..") || rel.starts_with('/') {
                skipped_unverified.push(rel.clone());
                continue;
            }

            let target: PathBuf = decoy_layer_root.join(rel);
            if !target.is_file() {
                continue;
            }

            // Enforce directory confinement
            if let Ok(canon_target) = fs::canonicalize(&target) {
                if !canon_target.starts_with(&canonical_root) {
                    skipped_unverified.push(rel.clone());
                    continue;
                }
            }

            // Ownership check via BLAKE3 fingerprint before deletion
            let content = fs::read_to_string(&target)?;
            let actual_hash = compute_content_fingerprint(&content);
            if actual_hash != art.integrity_fingerprint {
                skipped_unverified.push(format!("{} (hash mismatch)", rel));
                continue;
            }

            if !dry_run {
                fs::remove_file(&target)?;
                art.install_state = DecoyInstallState::Removed;
                art.monitoring_state = DecoyMonitoringState::Disabled;
            }
            removed_ids.push(art.artifact_id.clone());
            removed_paths.push(rel.clone());
        }

        if !dry_run {
            self.data.updated_at = Utc::now();
            // Prune empty BRAHM-owned subdirectories under decoy_layer_root
            for rel in &removed_paths {
                let mut curr = decoy_layer_root.join(rel).parent().map(|p| p.to_path_buf());
                while let Some(dir) = curr {
                    if let Ok(canon_dir) = fs::canonicalize(&dir) {
                        if canon_dir == canonical_root || !canon_dir.starts_with(&canonical_root) {
                            break;
                        }
                        if fs::read_dir(&canon_dir)
                            .map(|mut rd| rd.next().is_none())
                            .unwrap_or(false)
                        {
                            let _ = fs::remove_dir(&canon_dir);
                            curr = dir.parent().map(|p| p.to_path_buf());
                            continue;
                        }
                    }
                    break;
                }
            }
        }

        Ok(DecoyCleanupResult {
            dry_run,
            removed_artifact_ids: removed_ids,
            removed_paths,
            skipped_unverified_paths: skipped_unverified,
        })
    }
}
