//! Safety and integrity validation for generated Stage 03 decoy artifacts.

use std::collections::HashSet;
use std::path::Path;

use super::identifiers::compute_content_fingerprint;
use super::models::{
    DecoyValidationIssue, DecoyValidationReport, GeneratedDecoy, ValidationSeverity,
};

/// Protected production manifests that decoys must never overlap or modify.
const PROTECTED_PRODUCTION_MANIFESTS: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "package.json",
    "package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    "pyproject.toml",
    "requirements.txt",
    "go.mod",
    "go.sum",
    "pom.xml",
    "build.gradle",
    "BRAHM_ARCHITECTURE.json",
    "BRAHM_ARCHITECTURE.md",
];

/// Credential patterns strictly banned from appearing in generated decoys.
const FORBIDDEN_SECRET_PATTERNS: &[&str] = &[
    "AKIA",
    "sk_live_",
    "sk_test_",
    "ghp_",
    "gho_",
    "xoxb-",
    "BEGIN RSA PRIVATE KEY",
    "BEGIN OPENSSH PRIVATE KEY",
    "BEGIN PRIVATE KEY",
];

/// Executable / side-effect constructs forbidden inside synthetic decoy code.
const FORBIDDEN_EXECUTION_PATTERNS: &[&str] = &[
    "std::process::Command",
    "child_process",
    "os.system(",
    "subprocess.",
    "exec.Command",
    "Runtime.getRuntime",
    "unsafe {",
];

/// Validator ensuring decoys obey all safety, uniqueness, and non-contamination constraints.
#[derive(Debug, Default, Clone)]
pub struct DecoyValidator;

impl DecoyValidator {
    pub fn new() -> Self {
        Self
    }

    /// Validate generated decoys against safety policies and genuine repository boundaries.
    pub fn validate(
        &self,
        decoys: &[GeneratedDecoy],
        genuine_repo_root: &Path,
    ) -> DecoyValidationReport {
        let mut issues = Vec::new();
        let mut seen_ids = HashSet::new();
        let mut seen_paths = HashSet::new();

        for item in decoys {
            let art = &item.artifact;

            // 1. Unique artifact IDs
            if !seen_ids.insert(art.artifact_id.clone()) {
                issues.push(DecoyValidationIssue {
                    artifact_id: art.artifact_id.clone(),
                    severity: ValidationSeverity::Error,
                    check: "unique_id".to_string(),
                    message: format!("Duplicate artifact ID '{}'", art.artifact_id),
                });
            }

            // 2. Unique relative path + traversal guard
            let norm_rel = art.synthetic_relative_path.replace('\\', "/");
            if norm_rel.starts_with('/') || norm_rel.contains("..") || norm_rel.contains(':') {
                issues.push(DecoyValidationIssue {
                    artifact_id: art.artifact_id.clone(),
                    severity: ValidationSeverity::Error,
                    check: "safe_path".to_string(),
                    message: format!("Unsafe relative path '{}'", norm_rel),
                });
            }
            if !seen_paths.insert(norm_rel.clone()) {
                issues.push(DecoyValidationIssue {
                    artifact_id: art.artifact_id.clone(),
                    severity: ValidationSeverity::Error,
                    check: "unique_path".to_string(),
                    message: format!("Duplicate target decoy path '{}'", norm_rel),
                });
            }

            // 3. Explicit synthetic & ownership flags
            if !art.synthetic_status || !art.brahm_owned {
                issues.push(DecoyValidationIssue {
                    artifact_id: art.artifact_id.clone(),
                    severity: ValidationSeverity::Error,
                    check: "synthetic_ownership".to_string(),
                    message: "Decoy missing synthetic_status=true or brahm_owned=true".to_string(),
                });
            }

            // 4. Protected production manifests overlap check
            let filename = Path::new(&norm_rel)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if PROTECTED_PRODUCTION_MANIFESTS.contains(&filename) {
                issues.push(DecoyValidationIssue {
                    artifact_id: art.artifact_id.clone(),
                    severity: ValidationSeverity::Error,
                    check: "production_manifest_protection".to_string(),
                    message: format!(
                        "Decoy path '{}' conflicts with protected manifest '{}'",
                        norm_rel, filename
                    ),
                });
            }

            // 5. Genuine source collision check
            if genuine_repo_root.join(&norm_rel).exists() {
                issues.push(DecoyValidationIssue {
                    artifact_id: art.artifact_id.clone(),
                    severity: ValidationSeverity::Warning,
                    check: "genuine_file_overlap".to_string(),
                    message: format!(
                        "Path '{}' exists in genuine repository; decoy will remain isolated in decoy layer",
                        norm_rel
                    ),
                });
            }

            // 6. Secret pattern leakage check
            for pat in FORBIDDEN_SECRET_PATTERNS {
                if item.content.contains(pat) {
                    issues.push(DecoyValidationIssue {
                        artifact_id: art.artifact_id.clone(),
                        severity: ValidationSeverity::Error,
                        check: "secret_leakage_guard".to_string(),
                        message: format!("Generated content matched credential pattern '{}'", pat),
                    });
                }
            }

            // 7. Execution / side-effect check
            for pat in FORBIDDEN_EXECUTION_PATTERNS {
                if item.content.contains(pat) {
                    issues.push(DecoyValidationIssue {
                        artifact_id: art.artifact_id.clone(),
                        severity: ValidationSeverity::Error,
                        check: "inert_code_guard".to_string(),
                        message: format!(
                            "Generated content matched side-effect construct '{}'",
                            pat
                        ),
                    });
                }
            }

            // 8. BLAKE3 fingerprint consistency
            let actual_hash = compute_content_fingerprint(&item.content);
            if actual_hash != art.integrity_fingerprint {
                issues.push(DecoyValidationIssue {
                    artifact_id: art.artifact_id.clone(),
                    severity: ValidationSeverity::Error,
                    check: "fingerprint_integrity".to_string(),
                    message: "Stored BLAKE3 fingerprint does not match rendered content"
                        .to_string(),
                });
            }
        }

        let has_errors = issues
            .iter()
            .any(|i| i.severity == ValidationSeverity::Error);

        DecoyValidationReport {
            valid: !has_errors,
            checked_count: decoys.len(),
            issues,
        }
    }
}
