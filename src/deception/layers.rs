//! Repository Layer Separation and Production Integrity Protection (Milestones 04.1 & 04.5).
//!
//! Enforces strict physical and logical isolation between:
//! - GenuineLayer (original repository & build inputs)
//! - SyntheticLayer (BRAHM-owned decoys & manifest)
//! - InspectionLayer (isolated workspace assembled for authorized inspection)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::analysis::compute_file_hash;
use crate::deception::decoy::{compute_content_fingerprint, DecoyManifest};
use crate::detection::is_sensitive_credential_file;
use crate::errors::{BrahmError, Result};

/// Production build manifests checked to guarantee zero synthetic build contamination.
const PROTECTED_BUILD_MANIFESTS: &[&str] = &[
    "Cargo.toml",
    "package.json",
    "pyproject.toml",
    "go.mod",
    "pom.xml",
    "build.gradle",
];

/// Represents the uncontaminated genuine repository layer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenuineLayer {
    pub root_path: PathBuf,
    pub baseline_fingerprint: String,
    pub file_hashes: HashMap<String, String>,
    pub captured_at: DateTime<Utc>,
}

/// Represents the isolated synthetic decoy layer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyntheticLayer {
    pub decoy_layer_root: PathBuf,
    pub manifest: DecoyManifest,
}

/// Represents an assembled inspection layer combining safe genuine files + policy-selected synthetic files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InspectionLayer {
    pub workspace_root: PathBuf,
    pub session_id: String,
    pub genuine_files_copied: Vec<String>,
    pub stripped_sensitive_files: Vec<String>,
    pub injected_decoy_ids: Vec<String>,
    pub inspection_view_fingerprint: String,
}

/// Finding from checking genuine source/build integrity against synthetic contamination.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrityCheckFinding {
    pub check_name: String,
    pub path: String,
    pub passed: bool,
    pub detail: String,
}

/// Comprehensive Stage 04 `BRAHM_INTEGRITY_REPORT.json` schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductionIntegrityReport {
    pub schema_version: String,
    pub generated_at: DateTime<Utc>,
    pub genuine_repo_path: String,
    pub production_isolated: bool,
    pub checks_run: usize,
    pub failures_count: usize,
    pub findings: Vec<IntegrityCheckFinding>,
}

/// Guard verifying that synthetic decoys never enter the genuine repository or its build inputs.
#[derive(Debug, Default, Clone)]
pub struct ProductionIntegrityGuard;

impl ProductionIntegrityGuard {
    pub fn new() -> Self {
        Self
    }

    /// Capture a baseline snapshot of genuine files inside `repo_root`, skipping build/git dirs
    /// and excluding secret files from mirror transfers while recording baseline hashes of source files.
    pub fn capture_genuine_baseline(&self, repo_root: &Path) -> Result<GenuineLayer> {
        let canonical_root = fs::canonicalize(repo_root).map_err(|e| BrahmError::InvalidPath {
            path: repo_root.to_path_buf(),
            reason: format!("Cannot resolve genuine repository root: {}", e),
        })?;

        let mut file_hashes = HashMap::new();
        for entry in walkdir::WalkDir::new(&canonical_root)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| !is_ignored_repo_dir(e.file_name().to_str().unwrap_or("")))
        {
            let entry = entry.map_err(|e| {
                BrahmError::Deception(format!("Failed scanning genuine repository: {}", e))
            })?;
            if !entry.file_type().is_file() {
                continue;
            }
            let filename = entry.file_name().to_str().unwrap_or("");
            if is_sensitive_credential_file(filename) {
                continue;
            }
            let rel = entry
                .path()
                .strip_prefix(&canonical_root)
                .unwrap_or(entry.path())
                .to_string_lossy()
                .replace('\\', "/");

            if let Ok(hash) = compute_file_hash(entry.path()) {
                file_hashes.insert(rel, hash);
            }
        }

        let mut sorted_pairs: Vec<(&String, &String)> = file_hashes.iter().collect();
        sorted_pairs.sort_by(|a, b| a.0.cmp(b.0));
        let mut root_hasher = blake3::Hasher::new();
        root_hasher.update(b"brahm-genuine-baseline-v1:");
        for (path, hash) in sorted_pairs {
            root_hasher.update(path.as_bytes());
            root_hasher.update(b"=");
            root_hasher.update(hash.as_bytes());
            root_hasher.update(b"\n");
        }

        Ok(GenuineLayer {
            root_path: canonical_root,
            baseline_fingerprint: root_hasher.finalize().to_hex().to_string(),
            file_hashes,
            captured_at: Utc::now(),
        })
    }

    /// Verify that the genuine repository and its production manifests contain zero registered decoy contamination.
    ///
    /// Fails safely if any synthetic decoy signature (`BRAHM-SYNTHETIC-ARTIFACT` or manifest hash match)
    /// is detected inside the genuine production codebase.
    pub fn verify_production_isolation(
        &self,
        genuine_repo_root: &Path,
        decoy_manifest: &DecoyManifest,
    ) -> Result<ProductionIntegrityReport> {
        let canonical_root =
            fs::canonicalize(genuine_repo_root).map_err(|e| BrahmError::InvalidPath {
                path: genuine_repo_root.to_path_buf(),
                reason: format!("Cannot resolve genuine repository root: {}", e),
            })?;

        let mut findings = Vec::new();

        // 1. Verify none of the synthetic relative paths + fingerprints were copied into genuine repo
        for entry in &decoy_manifest.entries {
            let candidate = canonical_root.join(&entry.relative_path);
            if candidate.is_file() {
                let content = fs::read_to_string(&candidate).unwrap_or_default();
                let actual_hash = compute_content_fingerprint(&content);
                if actual_hash == entry.integrity_fingerprint
                    || content.contains("BRAHM-SYNTHETIC-ARTIFACT")
                    || content.contains(&entry.artifact_id)
                {
                    findings.push(IntegrityCheckFinding {
                        check_name: "synthetic_contamination_check".to_string(),
                        path: entry.relative_path.clone(),
                        passed: false,
                        detail: format!(
                            "Contamination: synthetic decoy '{}' ({}) found inside genuine source tree!",
                            entry.relative_path, entry.artifact_id
                        ),
                    });
                } else {
                    findings.push(IntegrityCheckFinding {
                        check_name: "filename_collision_ownership_check".to_string(),
                        path: entry.relative_path.clone(),
                        passed: false,
                        detail: format!(
                            "Ambiguous file conflict: genuine repo also contains '{}' without BRAHM ownership signature; refusing to mix layers",
                            entry.relative_path
                        ),
                    });
                }
            }
        }

        // 2. Verify production build manifests do not reference synthetic artifact IDs or canary tokens
        for manifest_name in PROTECTED_BUILD_MANIFESTS {
            let manifest_path = canonical_root.join(manifest_name);
            if manifest_path.is_file() {
                let content = fs::read_to_string(&manifest_path).unwrap_or_default();
                let mut clean = true;
                for entry in &decoy_manifest.entries {
                    if content.contains(&entry.artifact_id)
                        || entry
                            .canary_token
                            .as_ref()
                            .map(|t| content.contains(t))
                            .unwrap_or(false)
                        || content.contains(&entry.relative_path) && content.contains("synthetic")
                    {
                        clean = false;
                        findings.push(IntegrityCheckFinding {
                            check_name: "build_input_manifest_check".to_string(),
                            path: (*manifest_name).to_string(),
                            passed: false,
                            detail: format!(
                                "Production build manifest '{}' references decoy artifact '{}'!",
                                manifest_name, entry.artifact_id
                            ),
                        });
                    }
                }
                if clean {
                    findings.push(IntegrityCheckFinding {
                        check_name: "build_input_manifest_check".to_string(),
                        path: (*manifest_name).to_string(),
                        passed: true,
                        detail: format!(
                            "Build manifest '{}' verified clean of synthetic references",
                            manifest_name
                        ),
                    });
                }
            }
        }

        // 3. Scan genuine source files defensively for unexpected BRAHM-SYNTHETIC markers
        for entry in walkdir::WalkDir::new(&canonical_root)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| !is_ignored_repo_dir(e.file_name().to_str().unwrap_or("")))
        {
            let entry = entry.map_err(|e| {
                BrahmError::Deception(format!("Error scanning genuine repository: {}", e))
            })?;
            if !entry.file_type().is_file() {
                continue;
            }
            if let Ok(content) = fs::read_to_string(entry.path()) {
                if content.contains("BRAHM-SYNTHETIC-ARTIFACT") || content.contains("BRAHM-CANARY-")
                {
                    let rel = entry
                        .path()
                        .strip_prefix(&canonical_root)
                        .unwrap_or(entry.path())
                        .to_string_lossy()
                        .replace('\\', "/");
                    // Do not double-report if already recorded above
                    if !findings.iter().any(|f| !f.passed && f.path == rel) {
                        findings.push(IntegrityCheckFinding {
                            check_name: "synthetic_marker_scan".to_string(),
                            path: rel.clone(),
                            passed: false,
                            detail: format!(
                                "Found BRAHM synthetic marker inside genuine directory file '{}'",
                                rel
                            ),
                        });
                    }
                }
            }
        }

        let failures_count = findings.iter().filter(|f| !f.passed).count();

        Ok(ProductionIntegrityReport {
            schema_version: "1.0".to_string(),
            generated_at: Utc::now(),
            genuine_repo_path: canonical_root.to_string_lossy().replace('\\', "/"),
            production_isolated: failures_count == 0,
            checks_run: findings.len().max(1),
            failures_count,
            findings,
        })
    }
}

/// Directory names excluded when mirroring or scanning genuine repositories.
pub fn is_ignored_repo_dir(name: &str) -> bool {
    matches!(
        name,
        ".git" | "target" | "node_modules" | "__pycache__" | ".venv" | "venv" | ".brahm_decoys"
    )
}
