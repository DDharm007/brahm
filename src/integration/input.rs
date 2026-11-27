//! Repository Input System (Milestone 05.1).
//!
//! Handles connecting local repositories and validating/cloning authorized GitHub repositories
//! strictly without logging, storing, or exposing credentials.

use chrono::Utc;
use std::fs;
use std::path::Path;
use std::process::Command;
use tracing::info;

use super::models::{ProtectionProfile, RepositoryRecord, RepositorySourceKind};
use crate::analysis::run_repository_analysis;
use crate::architecture::{ArchitectureAnalysisBundle, ArchitectureEngine};
use crate::config::BrahmConfig;
use crate::deception::decoy::generate_repository_id;
use crate::deception::{GenuineLayer, ProductionIntegrityGuard};
use crate::errors::{BrahmError, Result};
use crate::models::RepositoryReport;
use crate::reporting::architecture_json::{
    generate_architecture_json, generate_dependency_graph_json, generate_symbol_index_json,
};

/// Validated metadata parsed from a GitHub repository URL without any credential material.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedGitHubUrl {
    pub sanitized_url: String,
    pub owner: String,
    pub repo_name: String,
}

/// Outcome returned when connecting a repository into BRAHM's protection management.
pub struct RepositoryConnectionResult {
    pub record: RepositoryRecord,
    pub already_registered: bool,
    pub stage1_report: RepositoryReport,
    pub architecture: ArchitectureAnalysisBundle,
    pub genuine_layer: GenuineLayer,
}

/// Manager handling repository onboarding (Local + Authorized GitHub).
#[derive(Debug, Default, Clone)]
pub struct RepositoryInputManager {
    integrity_guard: ProductionIntegrityGuard,
}

impl RepositoryInputManager {
    pub fn new() -> Self {
        Self {
            integrity_guard: ProductionIntegrityGuard::new(),
        }
    }

    /// Register or inspect a local repository path without modifying any source files.
    pub fn connect_local(
        &self,
        repo_path: &Path,
        state_dir: &Path,
        config: Option<BrahmConfig>,
    ) -> Result<RepositoryConnectionResult> {
        self.connect_internal(
            repo_path,
            state_dir,
            RepositorySourceKind::LocalPath,
            None,
            config,
        )
    }

    /// Validate a GitHub repository URL strictly (rejects embedded credentials and shell characters).
    pub fn validate_github_url(raw_url: &str) -> Result<ValidatedGitHubUrl> {
        let trimmed = raw_url.trim();
        if trimmed.is_empty() {
            return Err(BrahmError::Integration(
                "Repository URL cannot be empty".to_string(),
            ));
        }

        // Security check: never accept embedded plaintext tokens/passwords in URL (`https://token@github.com/...`)
        if trimmed.contains('@') && trimmed.starts_with("http") {
            return Err(BrahmError::Unauthorized(
                "Security error: do not pass plaintext credentials/tokens inside repository URLs; configure OS git credentials or environment authorization instead"
                    .to_string(),
            ));
        }

        // Reject shell injection metacharacters or whitespace
        for bad in [' ', '\t', '\n', ';', '|', '&', '$', '`', '(', ')', '<', '>'] {
            if trimmed.contains(bad) {
                return Err(BrahmError::Integration(format!(
                    "Invalid character '{}' in repository URL",
                    bad
                )));
            }
        }

        let clean_path = if let Some(rest) = trimmed.strip_prefix("https://github.com/") {
            rest
        } else if let Some(rest) = trimmed.strip_prefix("git@github.com:") {
            rest
        } else {
            return Err(BrahmError::Integration(
                "Only explicit GitHub URLs (https://github.com/<owner>/<repo> or git@github.com:<owner>/<repo>.git) are supported"
                    .to_string(),
            ));
        };

        let no_git = clean_path.trim_end_matches('/').trim_end_matches(".git");
        let segments: Vec<&str> = no_git.split('/').collect();
        if segments.len() != 2 || segments[0].is_empty() || segments[1].is_empty() {
            return Err(BrahmError::Integration(format!(
                "Invalid GitHub repository format '{}': expected owner/repo",
                trimmed
            )));
        }

        let owner = segments[0];
        let repo_name = segments[1];
        if !is_valid_git_slug(owner) || !is_valid_git_slug(repo_name) {
            return Err(BrahmError::Integration(
                "Repository owner and name may only contain alphanumeric characters, '-', '_', or '.'"
                    .to_string(),
            ));
        }

        let sanitized_url = format!("https://github.com/{}/{}.git", owner, repo_name);
        Ok(ValidatedGitHubUrl {
            sanitized_url,
            owner: owner.to_string(),
            repo_name: repo_name.to_string(),
        })
    }

    /// Perform an explicit shallow clone of an authorized GitHub repository using system `git`.
    /// Never pushes changes, never logs credentials, and disables interactive prompts.
    pub fn connect_github(
        &self,
        url: &str,
        clone_dest: &Path,
        state_dir: &Path,
    ) -> Result<RepositoryConnectionResult> {
        let validated = Self::validate_github_url(url)?;

        if clone_dest.exists() && clone_dest.read_dir()?.next().is_some() {
            return Err(BrahmError::InvalidPath {
                path: clone_dest.to_path_buf(),
                reason: "Clone destination already exists and is non-empty".to_string(),
            });
        }
        fs::create_dir_all(clone_dest)?;

        Self::git_clone_readonly(&validated.sanitized_url, clone_dest)?;

        self.connect_internal(
            clone_dest,
            state_dir,
            RepositorySourceKind::GitHubUrl,
            Some(validated.sanitized_url),
            None,
        )
    }

    /// Execute `git clone --depth 1` via direct process arguments without shell interpolation.
    pub fn git_clone_readonly(remote_spec: &str, dest_dir: &Path) -> Result<()> {
        let output = Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                "--single-branch",
                remote_spec,
                &dest_dir.to_string_lossy(),
            ])
            .env("GIT_TERMINAL_PROMPT", "0")
            .output()
            .map_err(|e| BrahmError::Integration(format!("Failed executing system git: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Redact any accidental token-like strings from git diagnostics
            return Err(BrahmError::Integration(format!(
                "Git clone failed (exit {}): {}",
                output.status,
                stderr.trim()
            )));
        }
        Ok(())
    }

    fn connect_internal(
        &self,
        repo_path: &Path,
        state_dir: &Path,
        source_kind: RepositorySourceKind,
        source_url: Option<String>,
        explicit_config: Option<BrahmConfig>,
    ) -> Result<RepositoryConnectionResult> {
        if !repo_path.exists() || !repo_path.is_dir() {
            return Err(BrahmError::InvalidPath {
                path: repo_path.to_path_buf(),
                reason: "Repository path must exist and be a directory".to_string(),
            });
        }

        let canon_repo = fs::canonicalize(repo_path)?;
        let cfg = match explicit_config {
            Some(c) => c,
            None => BrahmConfig::find_or_default(&canon_repo).0,
        };

        // Stage 01 scan + Stage 02 architecture baseline + genuine layer fingerprint
        let stage1_report = run_repository_analysis(&canon_repo, cfg)?;
        let architecture = ArchitectureEngine::new().analyze(&canon_repo, &stage1_report)?;
        let genuine_layer = self.integrity_guard.capture_genuine_baseline(&canon_repo)?;

        let repo_id = generate_repository_id(&canon_repo, &stage1_report.repository.name);
        fs::create_dir_all(state_dir)?;
        let config_record_path = state_dir.join("BRAHM_REPOSITORY_CONFIG.json");

        let (
            already_registered,
            existing_profile,
            registered_at,
            decoy_reg,
            active_sess,
            stored_hash,
        ) = if config_record_path.is_file() {
            if let Ok(raw) = fs::read_to_string(&config_record_path) {
                if let Ok(prev) = serde_json::from_str::<RepositoryRecord>(&raw) {
                    if prev.repository_id == repo_id {
                        (
                            true,
                            prev.active_profile,
                            prev.registered_at,
                            prev.decoy_registry_path,
                            prev.active_session_id,
                            Some(prev.baseline_fingerprint),
                        )
                    } else {
                        (
                            false,
                            ProtectionProfile::AnalysisOnly,
                            Utc::now(),
                            None,
                            None,
                            None,
                        )
                    }
                } else {
                    (
                        false,
                        ProtectionProfile::AnalysisOnly,
                        Utc::now(),
                        None,
                        None,
                        None,
                    )
                }
            } else {
                (
                    false,
                    ProtectionProfile::AnalysisOnly,
                    Utc::now(),
                    None,
                    None,
                    None,
                )
            }
        } else {
            (
                false,
                ProtectionProfile::AnalysisOnly,
                Utc::now(),
                None,
                None,
                None,
            )
        };

        let baseline_fp = stored_hash.unwrap_or_else(|| genuine_layer.baseline_fingerprint.clone());
        let baseline_path = state_dir.join("BRAHM_BASELINE.json");
        if !already_registered || !baseline_path.is_file() {
            fs::write(
                &baseline_path,
                serde_json::to_string_pretty(&genuine_layer)?,
            )?;
        }

        // Persist Stage 02 architectural intelligence artifacts into state_dir
        let _ = generate_architecture_json(
            &architecture.report,
            Some(&state_dir.join("BRAHM_ARCHITECTURE.json")),
        )?;
        let _ = generate_dependency_graph_json(
            &architecture.graph_export,
            Some(&state_dir.join("BRAHM_DEPENDENCY_GRAPH.json")),
        )?;
        let _ = generate_symbol_index_json(
            &architecture.symbol_index,
            Some(&state_dir.join("BRAHM_SYMBOL_INDEX.json")),
        )?;

        let record = RepositoryRecord {
            schema_version: "1.0".to_string(),
            repository_id: repo_id,
            name: stage1_report.repository.name.clone(),
            source_kind,
            local_path: canon_repo,
            source_url,
            registered_at,
            updated_at: Utc::now(),
            baseline_fingerprint: baseline_fp,
            active_profile: existing_profile,
            state_dir: state_dir.to_path_buf(),
            architecture_pattern: Some(architecture.report.pattern.primary_pattern.clone()),
            decoy_registry_path: decoy_reg,
            active_session_id: active_sess,
        };

        fs::write(&config_record_path, serde_json::to_string_pretty(&record)?)?;
        info!(
            "Connected repository '{}' ({}, registered={})",
            record.name, record.repository_id, already_registered
        );

        Ok(RepositoryConnectionResult {
            record,
            already_registered,
            stage1_report,
            architecture,
            genuine_layer,
        })
    }
}

fn is_valid_git_slug(s: &str) -> bool {
    !s.is_empty()
        && s != "."
        && s != ".."
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}
