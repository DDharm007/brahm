//! Core data models for Stage 03 Decoy Generation Engine.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Category of generated synthetic decoy artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecoyCategory {
    /// Inert language-specific source code file (e.g., `.rs`, `.ts`, `.py`, `.js`).
    SyntheticSource,
    /// Synthetic architecture or design documentation (`.md`).
    SyntheticDocumentation,
    /// Inert synthetic configuration or schema example (`.yaml`, `.json`, `.sql`).
    SyntheticConfiguration,
    /// Uniquely identifiable inert canary anchor artifact.
    CanaryArtifact,
    /// Synthetic dependency inventory isolated from production manifests.
    SyntheticDependency,
}

impl std::fmt::Display for DecoyCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SyntheticSource => write!(f, "synthetic_source"),
            Self::SyntheticDocumentation => write!(f, "synthetic_documentation"),
            Self::SyntheticConfiguration => write!(f, "synthetic_configuration"),
            Self::CanaryArtifact => write!(f, "canary_artifact"),
            Self::SyntheticDependency => write!(f, "synthetic_dependency"),
        }
    }
}

/// Lifecycle installation state of a decoy artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecoyInstallState {
    /// Generated in memory / validated.
    Generated,
    /// Written to isolated decoy staging layer on disk.
    Staged,
    /// Injected into an isolated inspection mirror workspace.
    DeployedInMirror,
    /// Explicitly deactivated via registry.
    Inactive,
    /// Removed safely during cleanup.
    Removed,
}

/// Monitoring status of a decoy artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecoyMonitoringState {
    Registered,
    ActiveWatch,
    Triggered,
    Disabled,
}

/// Risk / signal classification for telemetry prioritization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskClassification {
    /// Low-signal background structural artifact.
    InertLow,
    /// Medium-signal synthetic configuration or architecture doc.
    MonitoredMedium,
    /// High-signal registered canary anchor.
    CanaryHighSignal,
}

/// Configurable generation profile controlling artifact volume and types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DecoyGenerationProfile {
    /// No decoy generation permitted.
    Disabled,
    /// Minimal documentation + single canary anchor.
    Conservative,
    /// Balanced source modules, architecture docs, configs, and canaries.
    #[default]
    Standard,
    /// Expanded multi-component decoy suite for controlled research studies.
    Research,
}

impl std::fmt::Display for DecoyGenerationProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disabled => write!(f, "disabled"),
            Self::Conservative => write!(f, "conservative"),
            Self::Standard => write!(f, "standard"),
            Self::Research => write!(f, "research"),
        }
    }
}

/// Persistent metadata record for a registered decoy artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecoyArtifact {
    /// Stable deterministic identifier (`decoy_<blake3_prefix>`).
    pub artifact_id: String,
    /// Decoy category.
    pub artifact_type: DecoyCategory,
    /// Synthetic filename.
    pub synthetic_filename: String,
    /// Relative path inside the isolated decoy layer (forward slashes).
    pub synthetic_relative_path: String,
    /// Identifier of template used to render content.
    pub template_id: String,
    /// Version string of template used.
    pub template_version: String,
    /// Timestamp when generated (UTC).
    pub generated_at: DateTime<Utc>,
    /// BRAHM engine version that generated this artifact.
    pub generation_version: String,
    /// Related genuine Stage 02 architecture node, language, or framework indicator.
    pub related_architecture_node: Option<String>,
    /// Always `true`; distinguishes synthetic items from genuine software components.
    pub synthetic_status: bool,
    /// Explicit ownership marker ensuring cleanup only removes BRAHM-owned items.
    pub brahm_owned: bool,
    /// Signal classification.
    pub risk_classification: RiskClassification,
    /// Hexadecimal BLAKE3 digest of rendered artifact content.
    pub integrity_fingerprint: String,
    /// Current installation state.
    pub install_state: DecoyInstallState,
    /// Current monitoring state.
    pub monitoring_state: DecoyMonitoringState,
    /// Non-secret inert canary identifier if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canary_token: Option<String>,
    /// Arbitrary deterministic metadata pairs.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, String>,
}

/// Pair of registered metadata and rendered synthetic content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedDecoy {
    pub artifact: DecoyArtifact,
    pub content: String,
}

/// Template definition used by the deterministic TemplateEngine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecoyTemplate {
    pub template_id: String,
    pub version: String,
    pub category: DecoyCategory,
    pub target_language: Option<String>,
    pub default_relative_path: String,
    pub risk_classification: RiskClassification,
    pub description: String,
    pub body_template: String,
}

/// Persistent decoy registry stored in `BRAHM_DECOY_REGISTRY.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecoyRegistryData {
    pub schema_version: String,
    pub repository_id: String,
    pub repository_name: String,
    pub decoy_layer_root: String,
    pub generation_profile: DecoyGenerationProfile,
    pub updated_at: DateTime<Utc>,
    pub artifacts: Vec<DecoyArtifact>,
}

/// Explicit manifest (`BRAHM_DECOY_MANIFEST.json`) enabling Stage 04 production build isolation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecoyManifest {
    pub schema_version: String,
    pub generated_at: DateTime<Utc>,
    pub repository_id: String,
    pub total_artifacts: usize,
    pub production_build_excluded: bool,
    pub entries: Vec<DecoyManifestEntry>,
}

/// Single entry inside `BRAHM_DECOY_MANIFEST.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecoyManifestEntry {
    pub artifact_id: String,
    pub relative_path: String,
    pub category: DecoyCategory,
    pub integrity_fingerprint: String,
    pub brahm_owned: bool,
    pub synthetic_status: bool,
    pub canary_token: Option<String>,
}

/// Severity of a decoy validation check failure or notice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationSeverity {
    Error,
    Warning,
}

/// Single issue identified by `DecoyValidator`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecoyValidationIssue {
    pub artifact_id: String,
    pub severity: ValidationSeverity,
    pub check: String,
    pub message: String,
}

/// Aggregate result of validating generated/staged decoys.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecoyValidationReport {
    pub valid: bool,
    pub checked_count: usize,
    pub issues: Vec<DecoyValidationIssue>,
}

/// Result summary of removing BRAHM-owned decoys (real or dry-run).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecoyCleanupResult {
    pub dry_run: bool,
    pub removed_artifact_ids: Vec<String>,
    pub removed_paths: Vec<String>,
    pub skipped_unverified_paths: Vec<String>,
}
