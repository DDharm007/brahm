//! Configuration models and loader for BRAHM.
//!
//! Handles `brahm.toml` loading, validation, and serialization.

use crate::deception::decoy::DecoyGenerationProfile;
use crate::deception::policy::DeceptionPolicyProfile;
use crate::errors::{BrahmError, Result};
use crate::integration::ProtectionProfile;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Root configuration for BRAHM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BrahmConfig {
    #[serde(default = "default_config_schema_version")]
    pub schema_version: String,
    #[serde(default)]
    pub scanner: ScannerConfig,
    #[serde(default)]
    pub decoy: DecoyConfig,
    #[serde(default)]
    pub deception: DeceptionConfig,
    #[serde(default)]
    pub integration: IntegrationConfig,
}

impl Default for BrahmConfig {
    fn default() -> Self {
        Self {
            schema_version: default_config_schema_version(),
            scanner: ScannerConfig::default(),
            decoy: DecoyConfig::default(),
            deception: DeceptionConfig::default(),
            integration: IntegrationConfig::default(),
        }
    }
}

/// Stage 05 Agent Integration & Protection configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntegrationConfig {
    #[serde(default)]
    pub protection_profile: ProtectionProfile,
    #[serde(default = "default_authorized_agent")]
    pub authorized_agent: String,
}

fn default_authorized_agent() -> String {
    "brahm-local-test-agent".to_string()
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self {
            protection_profile: ProtectionProfile::AnalysisOnly,
            authorized_agent: default_authorized_agent(),
        }
    }
}

/// Stage 04 Repository Deception Engine configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeceptionConfig {
    #[serde(default)]
    pub default_policy_profile: DeceptionPolicyProfile,
    #[serde(default = "default_deception_workspace_root")]
    pub workspace_root: String,
    #[serde(default = "default_session_ttl_seconds")]
    pub session_ttl_seconds: i64,
}

fn default_deception_workspace_root() -> String {
    ".brahm_sessions".to_string()
}

fn default_session_ttl_seconds() -> i64 {
    3600
}

impl Default for DeceptionConfig {
    fn default() -> Self {
        Self {
            default_policy_profile: DeceptionPolicyProfile::Standard,
            workspace_root: default_deception_workspace_root(),
            session_ttl_seconds: default_session_ttl_seconds(),
        }
    }
}

fn default_config_schema_version() -> String {
    "2.0".to_string()
}

/// Stage 03 Decoy Generation configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DecoyConfig {
    #[serde(default)]
    pub profile: DecoyGenerationProfile,
    #[serde(default = "default_decoy_staging_dir")]
    pub staging_dir: String,
    #[serde(default = "default_exclude_from_production")]
    pub exclude_from_production_builds: bool,
}

fn default_decoy_staging_dir() -> String {
    ".brahm/decoys".to_string()
}

fn default_exclude_from_production() -> bool {
    true
}

impl Default for DecoyConfig {
    fn default() -> Self {
        Self {
            profile: DecoyGenerationProfile::Standard,
            staging_dir: default_decoy_staging_dir(),
            exclude_from_production_builds: default_exclude_from_production(),
        }
    }
}

/// Scanner subsystem configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScannerConfig {
    /// Whether to follow directory symlinks (default: false for security).
    #[serde(default = "default_follow_symlinks")]
    pub follow_symlinks: bool,

    /// Maximum file size in megabytes to process or hash (default: 100 MB).
    #[serde(default = "default_max_file_size_mb")]
    pub max_file_size_mb: u64,

    /// Whether to calculate BLAKE3 integrity fingerprints (default: true).
    #[serde(default = "default_calculate_hashes")]
    pub calculate_hashes: bool,

    /// Configured exclusion patterns.
    #[serde(default)]
    pub exclusions: ExclusionConfig,
}

fn default_follow_symlinks() -> bool {
    false
}

fn default_max_file_size_mb() -> u64 {
    100
}

fn default_calculate_hashes() -> bool {
    true
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            follow_symlinks: default_follow_symlinks(),
            max_file_size_mb: default_max_file_size_mb(),
            calculate_hashes: default_calculate_hashes(),
            exclusions: ExclusionConfig::default(),
        }
    }
}

/// Exclusion rules for scanning.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExclusionConfig {
    /// Directory names or patterns to skip.
    #[serde(default = "default_excluded_directories")]
    pub directories: Vec<String>,

    /// Specific file names or patterns to skip.
    #[serde(default = "default_excluded_files")]
    pub files: Vec<String>,

    /// File extensions to skip.
    #[serde(default = "default_excluded_extensions")]
    pub extensions: Vec<String>,
}

fn default_excluded_directories() -> Vec<String> {
    vec![
        ".git".to_string(),
        "node_modules".to_string(),
        "target".to_string(),
        "dist".to_string(),
        "build".to_string(),
        ".next".to_string(),
        "__pycache__".to_string(),
        ".venv".to_string(),
        "venv".to_string(),
        ".idea".to_string(),
        ".vscode".to_string(),
        ".turbo".to_string(),
        "coverage".to_string(),
    ]
}

fn default_excluded_files() -> Vec<String> {
    vec![".DS_Store".to_string(), "Thumbs.db".to_string()]
}

fn default_excluded_extensions() -> Vec<String> {
    vec!["tmp".to_string(), "swp".to_string(), "bak".to_string()]
}

impl Default for ExclusionConfig {
    fn default() -> Self {
        Self {
            directories: default_excluded_directories(),
            files: default_excluded_files(),
            extensions: default_excluded_extensions(),
        }
    }
}

impl BrahmConfig {
    /// Loads a configuration from a given file path.
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(BrahmError::InvalidPath {
                path: path.to_path_buf(),
                reason: "Configuration file does not exist".to_string(),
            });
        }

        let content = fs::read_to_string(path)?;
        let config: BrahmConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// Locates `brahm.toml` in the repository or current directory, or returns default.
    pub fn find_or_default<P: AsRef<Path>>(repo_path: P) -> (Self, Option<PathBuf>) {
        let repo = repo_path.as_ref();
        let repo_config = repo.join("brahm.toml");
        if repo_config.is_file() {
            if let Ok(cfg) = Self::load_from_path(&repo_config) {
                return (cfg, Some(repo_config));
            }
        }

        let cwd_config = PathBuf::from("brahm.toml");
        if cwd_config.is_file() {
            if let Ok(cfg) = Self::load_from_path(&cwd_config) {
                return (cfg, Some(cwd_config));
            }
        }

        (Self::default(), None)
    }

    /// Serializes and writes configuration to a given destination.
    pub fn save_to_path<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        self.validate()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let toml_str = toml::to_string_pretty(self)?;
        fs::write(path, toml_str)?;
        Ok(())
    }

    /// Creates a `.bak` backup if `path` exists before saving updated configuration.
    pub fn backup_and_save<P: AsRef<Path>>(&self, path: P) -> Result<Option<PathBuf>> {
        let p = path.as_ref();
        let backup = if p.is_file() {
            let mut bak = p.to_path_buf();
            let filename = p
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("brahm.toml");
            bak.set_file_name(format!("{}.bak", filename));
            fs::copy(p, &bak)?;
            Some(bak)
        } else {
            None
        };
        self.save_to_path(p)?;
        Ok(backup)
    }

    /// Validates configuration values.
    pub fn validate(&self) -> Result<()> {
        if self.scanner.max_file_size_mb == 0 {
            return Err(BrahmError::Config(
                "max_file_size_mb must be greater than 0".to_string(),
            ));
        }
        if self.scanner.max_file_size_mb > 10_000 {
            return Err(BrahmError::Config(
                "max_file_size_mb cannot exceed 10,000 MB (10 GB)".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_validity() {
        let config = BrahmConfig::default();
        assert!(config.validate().is_ok());
        assert!(!config.scanner.follow_symlinks);
        assert_eq!(config.scanner.max_file_size_mb, 100);
        assert!(config.scanner.calculate_hashes);
        assert!(config
            .scanner
            .exclusions
            .directories
            .contains(&".git".to_string()));
    }

    #[test]
    fn test_serialization_roundtrip() {
        let config = BrahmConfig::default();
        let serialized = toml::to_string_pretty(&config).expect("Serialize to TOML");
        let deserialized: BrahmConfig = toml::from_str(&serialized).expect("Deserialize from TOML");
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_invalid_max_file_size() {
        let mut config = BrahmConfig::default();
        config.scanner.max_file_size_mb = 0;
        assert!(config.validate().is_err());
    }
}
