//! Core unified models for Stage 05 Agent Integration & Hybrid Protection.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Protection profile controlling active BRAHM capabilities for a connected repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProtectionProfile {
    /// Scan + architecture + integrity baseline only; no decoys or mirrors.
    #[default]
    AnalysisOnly,
    /// Decoy generation + controlled inspection workspace + canary registration + session monitoring.
    ResearchMode,
    /// Mirror + deception policies + inspection session + access telemetry.
    ControlledInspection,
    /// Existing decoy registry + canary events + integrity monitoring + reports.
    MonitoringMode,
}

impl std::fmt::Display for ProtectionProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AnalysisOnly => write!(f, "analysis_only"),
            Self::ResearchMode => write!(f, "research_mode"),
            Self::ControlledInspection => write!(f, "controlled_inspection"),
            Self::MonitoringMode => write!(f, "monitoring_mode"),
        }
    }
}

/// Source kind from which a repository was registered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepositorySourceKind {
    LocalPath,
    GitHubUrl,
}

/// Persistent connected repository configuration (`BRAHM_REPOSITORY_CONFIG.json`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryRecord {
    pub schema_version: String,
    pub repository_id: String,
    pub name: String,
    pub source_kind: RepositorySourceKind,
    /// Local filesystem path to genuine repo (for GitHub clones, path to local checkout).
    pub local_path: PathBuf,
    /// Optional non-credentialed remote HTTPS/SSH URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    pub registered_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub baseline_fingerprint: String,
    pub active_profile: ProtectionProfile,
    pub state_dir: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture_pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decoy_registry_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_session_id: Option<String>,
}

/// Capabilities exposed by an inspection agent adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentCapabilities {
    pub can_read_workspace: bool,
    pub can_list_files: bool,
    pub supports_structured_telemetry: bool,
    pub requires_isolated_sandbox: bool,
    pub supported_environments: Vec<String>,
}

/// Session descriptor prepared for an authorized inspection agent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentSession {
    pub agent_session_id: String,
    pub agent_name: String,
    pub inspection_session_id: String,
    pub workspace_path: PathBuf,
    pub prepared_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions_path: Option<PathBuf>,
}

/// Single structured interaction telemetry item recorded during an agent run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentTelemetryEvent {
    pub timestamp: DateTime<Utc>,
    pub session_id: String,
    pub agent_name: String,
    pub action: String,
    pub target_path: String,
    pub allowed: bool,
    pub artifact_id: Option<String>,
    pub canary_token: Option<String>,
}

/// Structured outcome from executing an inspection agent inside a bounded workspace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentResult {
    pub agent_session_id: String,
    pub agent_name: String,
    pub completed_at: DateTime<Utc>,
    pub accessed_files: Vec<String>,
    pub blocked_attempts: Vec<String>,
    pub telemetry_events: Vec<AgentTelemetryEvent>,
    pub notes: Vec<String>,
}

/// Manifest (`BRAHM_AGENT_MANIFEST.json`) documenting registered adapters and sessions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentManifest {
    pub schema_version: String,
    pub generated_at: DateTime<Utc>,
    pub repository_id: String,
    pub registered_adapters: Vec<String>,
    pub sessions: Vec<AgentSession>,
}

/// Explicit protection installation plan (`BRAHM_PROTECTION_PLAN.json`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtectionPlan {
    pub schema_version: String,
    pub plan_id: String,
    pub generated_at: DateTime<Utc>,
    pub repository_id: String,
    pub repository_name: String,
    pub current_profile: ProtectionProfile,
    pub target_profile: ProtectionProfile,
    pub proposed_steps: Vec<String>,
    pub will_generate_decoys: bool,
    pub will_create_mirror: bool,
    pub production_exclusions_verified: bool,
    pub requires_explicit_approval: bool,
}

/// Rollback outcome deliverable (`BRAHM_ROLLBACK_REPORT.json`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RollbackReport {
    pub schema_version: String,
    pub executed_at: DateTime<Utc>,
    pub dry_run: bool,
    pub repository_id: String,
    pub removed_artifacts: Vec<String>,
    pub cleaned_sessions: Vec<String>,
    pub restored_config_backup: bool,
    pub skipped_unowned_files: Vec<String>,
    pub integrity_verified_after: bool,
}
