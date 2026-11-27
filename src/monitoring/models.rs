//! Unified Data Models for Stage 06 Monitoring & Intelligence Engine.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Typed event categories emitted across BRAHM Stages 03–06.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    DecoyCreated,
    DecoyAccessed,
    CanaryAccessed,
    InspectionSessionStarted,
    InspectionSessionEnded,
    RepositoryIntegrityChanged,
    SyntheticArtifactModified,
    InspectionPolicyChanged,
    ValidationFailed,
    SessionExpired,
    CleanupCompleted,
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::DecoyCreated => "decoy_created",
            Self::DecoyAccessed => "decoy_accessed",
            Self::CanaryAccessed => "canary_accessed",
            Self::InspectionSessionStarted => "inspection_session_started",
            Self::InspectionSessionEnded => "inspection_session_ended",
            Self::RepositoryIntegrityChanged => "repository_integrity_changed",
            Self::SyntheticArtifactModified => "synthetic_artifact_modified",
            Self::InspectionPolicyChanged => "inspection_policy_changed",
            Self::ValidationFailed => "validation_failed",
            Self::SessionExpired => "session_expired",
            Self::CleanupCompleted => "cleanup_completed",
        };
        write!(f, "{}", s)
    }
}

impl EventType {
    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "decoy_created" => Some(Self::DecoyCreated),
            "decoy_accessed" => Some(Self::DecoyAccessed),
            "canary_accessed" => Some(Self::CanaryAccessed),
            "inspection_session_started" => Some(Self::InspectionSessionStarted),
            "inspection_session_ended" => Some(Self::InspectionSessionEnded),
            "repository_integrity_changed" => Some(Self::RepositoryIntegrityChanged),
            "synthetic_artifact_modified" => Some(Self::SyntheticArtifactModified),
            "inspection_policy_changed" => Some(Self::InspectionPolicyChanged),
            "validation_failed" => Some(Self::ValidationFailed),
            "session_expired" => Some(Self::SessionExpired),
            "cleanup_completed" => Some(Self::CleanupCompleted),
            _ => None,
        }
    }
}

/// Severity rating aligned with Stage 06 classification rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventSeverity {
    Informational,
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for EventSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Informational => write!(f, "informational"),
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

impl EventSeverity {
    pub fn from_db_str(s: &str) -> Self {
        match s {
            "low" => Self::Low,
            "medium" => Self::Medium,
            "high" => Self::High,
            "critical" => Self::Critical,
            _ => Self::Informational,
        }
    }
}

/// Originating subsystem for telemetry events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventSource {
    DecoyEngine,
    InspectionSandbox,
    AgentAdapter,
    IntegrityMonitor,
    PolicyEngine,
    CanaryDetector,
    MonitoringApi,
}

impl std::fmt::Display for EventSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::DecoyEngine => "decoy_engine",
            Self::InspectionSandbox => "inspection_sandbox",
            Self::AgentAdapter => "agent_adapter",
            Self::IntegrityMonitor => "integrity_monitor",
            Self::PolicyEngine => "policy_engine",
            Self::CanaryDetector => "canary_detector",
            Self::MonitoringApi => "monitoring_api",
        };
        write!(f, "{}", s)
    }
}

impl EventSource {
    pub fn from_db_str(s: &str) -> Self {
        match s {
            "decoy_engine" => Self::DecoyEngine,
            "inspection_sandbox" => Self::InspectionSandbox,
            "agent_adapter" => Self::AgentAdapter,
            "integrity_monitor" => Self::IntegrityMonitor,
            "policy_engine" => Self::PolicyEngine,
            "canary_detector" => Self::CanaryDetector,
            "monitoring_api" => Self::MonitoringApi,
            _ => Self::CanaryDetector,
        }
    }
}

/// Core unified telemetry event (`BRAHM_SECURITY_EVENTS.json`).
/// Data minimization rule: never store arbitrary genuine source contents or secrets inside metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonitoringEvent {
    pub event_id: String,
    pub event_type: EventType,
    pub repository_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub source: EventSource,
    pub severity: EventSeverity,
    pub correlation_id: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, String>,
}

/// False-positive classification applied by `CanaryDetector`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FalsePositiveStatus {
    ConfirmedSignal,
    BenignValidatorScan,
    Uncertain,
}

/// Registered canary interaction record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanaryEvent {
    pub canary_event_id: String,
    pub repository_id: String,
    pub session_id: Option<String>,
    pub artifact_id: String,
    pub canary_token: String,
    pub observed_at: DateTime<Utc>,
    pub context_note: String,
    pub is_duplicate: bool,
    pub classification: FalsePositiveStatus,
}

/// Category of file drift detected during Stage 06 integrity monitoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegrityChangeKind {
    Added,
    Removed,
    Modified,
    DecoyModified,
    BaselineMismatch,
}

/// Single file discrepancy detected against Stage 01 BLAKE3 baseline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrityFileDiff {
    pub relative_path: String,
    pub change_kind: IntegrityChangeKind,
    pub expected_change: bool,
    pub detail: String,
}

/// Snapshot check record (`BRAHM_INTEGRITY_REPORT.json` for Stage 06).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrityRecord {
    pub record_id: String,
    pub repository_id: String,
    pub checked_at: DateTime<Utc>,
    pub baseline_matches: bool,
    pub decoy_integrity_valid: bool,
    pub unexpected_change_count: usize,
    pub diffs: Vec<IntegrityFileDiff>,
    pub remediation_guidance: String,
}

/// Rule-based classification output explaining detected security evidence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecurityFinding {
    pub finding_id: String,
    pub rule_id: String,
    pub rule_name: String,
    pub severity: EventSeverity,
    pub confidence: f64,
    pub score_contribution: u32,
    pub evidence: Vec<String>,
    pub supporting_event_ids: Vec<String>,
    pub detected_at: DateTime<Utc>,
    pub attribution_disclaimer: String,
}

/// Single chronological step inside a correlated session timeline (`BRAHM_SESSION_TIMELINE.json`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimelineEntry {
    pub timestamp: DateTime<Utc>,
    pub event_id: String,
    pub event_type: EventType,
    pub severity: EventSeverity,
    pub summary: String,
}

/// Correlated session narrative.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionTimeline {
    pub session_id: String,
    pub repository_id: String,
    pub event_count: usize,
    pub canary_access_count: usize,
    pub entries: Vec<TimelineEntry>,
}

/// Comprehensive Monitoring Intelligence Summary (`BRAHM_MONITORING_REPORT.json`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntelligenceReport {
    pub schema_version: String,
    pub generated_at: DateTime<Utc>,
    pub repository_id: String,
    pub repository_name: String,
    pub monitoring_start: Option<DateTime<Utc>>,
    pub monitoring_end: Option<DateTime<Utc>>,
    pub total_events: usize,
    pub canary_events_count: usize,
    pub decoy_interactions_count: usize,
    pub session_count: usize,
    pub aggregate_risk_score: u32,
    pub integrity_status: IntegrityRecord,
    pub findings: Vec<SecurityFinding>,
    pub known_limitations: Vec<String>,
    pub recommended_actions: Vec<String>,
}
