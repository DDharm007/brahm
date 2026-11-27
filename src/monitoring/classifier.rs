//! Rule-Based Suspicious Activity Classification Engine (Milestone 06.5).
//!
//! Applies explicit documented deterministic rules with configurable scoring weights.
//! Never infers malicious intent from a single informational event and explicitly marks
//! actor attribution as uncertain.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::models::{EventSeverity, EventType, MonitoringEvent, SecurityFinding};

const ATTRIBUTION_DISCLAIMER: &str = "Actor attribution remains explicitly uncertain: telemetry observes interaction with a registered artifact inside the instrumented boundary and does not independently identify human vs. AI agent identity.";

/// Configurable weights for rule-based security scoring.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoringWeights {
    pub repeated_canary_access: u32,
    pub synthetic_artifact_modified: u32,
    pub sandbox_boundary_violation: u32,
    pub expired_session_activity: u32,
    pub genuine_integrity_drift: u32,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            repeated_canary_access: 20,
            synthetic_artifact_modified: 35,
            sandbox_boundary_violation: 40,
            expired_session_activity: 25,
            genuine_integrity_drift: 45,
        }
    }
}

/// Deterministic rule-based classification engine.
#[derive(Debug, Clone)]
pub struct RuleBasedClassifier {
    weights: ScoringWeights,
}

impl Default for RuleBasedClassifier {
    fn default() -> Self {
        Self::new(ScoringWeights::default())
    }
}

impl RuleBasedClassifier {
    pub fn new(weights: ScoringWeights) -> Self {
        Self { weights }
    }

    pub fn weights(&self) -> &ScoringWeights {
        &self.weights
    }

    /// Evaluate monitoring events against all documented rules, returning audit-ready findings
    /// paired with total aggregate risk score.
    pub fn classify(&self, events: &[MonitoringEvent]) -> (Vec<SecurityFinding>, u32) {
        let mut findings = Vec::new();
        let mut total_score = 0u32;

        // Rule R01: Repeated non-duplicate canary accesses (>= 2 signals)
        let canary_events: Vec<&MonitoringEvent> = events
            .iter()
            .filter(|e| {
                e.event_type == EventType::CanaryAccessed
                    && e.metadata
                        .get("duplicate")
                        .map(|d| d == "false")
                        .unwrap_or(true)
                    && e.metadata
                        .get("classification")
                        .map(|c| !c.contains("BenignValidatorScan"))
                        .unwrap_or(true)
            })
            .collect();

        if canary_events.len() >= 2 {
            let sev = if canary_events.len() >= 4 {
                EventSeverity::High
            } else {
                EventSeverity::Medium
            };
            let score = self.weights.repeated_canary_access;
            total_score += score;
            findings.push(SecurityFinding {
                finding_id: format!("fnd_{}", &Uuid::new_v4().simple().to_string()[..12]),
                rule_id: "R01_REPEATED_CANARY_ACCESS".to_string(),
                rule_name: "Repeated Registered Canary Access".to_string(),
                severity: sev,
                confidence: 0.85,
                score_contribution: score,
                evidence: vec![format!(
                    "Observed {} distinct registered canary marker interactions",
                    canary_events.len()
                )],
                supporting_event_ids: canary_events.iter().map(|e| e.event_id.clone()).collect(),
                detected_at: Utc::now(),
                attribution_disclaimer: ATTRIBUTION_DISCLAIMER.to_string(),
            });
        }

        // Rule R02: Unexpected modification of a synthetic decoy artifact
        let modified_decoys: Vec<&MonitoringEvent> = events
            .iter()
            .filter(|e| e.event_type == EventType::SyntheticArtifactModified)
            .collect();
        if !modified_decoys.is_empty() {
            let score = self.weights.synthetic_artifact_modified;
            total_score += score;
            findings.push(SecurityFinding {
                finding_id: format!("fnd_{}", &Uuid::new_v4().simple().to_string()[..12]),
                rule_id: "R02_SYNTHETIC_ARTIFACT_MODIFIED".to_string(),
                rule_name: "Unexpected Synthetic Decoy Modification".to_string(),
                severity: EventSeverity::High,
                confidence: 0.95,
                score_contribution: score,
                evidence: vec![format!(
                    "{} registered decoy artifact(s) exhibited content/fingerprint mutation",
                    modified_decoys.len()
                )],
                supporting_event_ids: modified_decoys.iter().map(|e| e.event_id.clone()).collect(),
                detected_at: Utc::now(),
                attribution_disclaimer: ATTRIBUTION_DISCLAIMER.to_string(),
            });
        }

        // Rule R03: Sandbox boundary traversal or repeated validation failures
        let boundary_violations: Vec<&MonitoringEvent> = events
            .iter()
            .filter(|e| e.event_type == EventType::ValidationFailed)
            .collect();
        if !boundary_violations.is_empty() {
            let sev = if boundary_violations.len() >= 3 {
                EventSeverity::Critical
            } else {
                EventSeverity::High
            };
            let score = self.weights.sandbox_boundary_violation;
            total_score += score;
            findings.push(SecurityFinding {
                finding_id: format!("fnd_{}", &Uuid::new_v4().simple().to_string()[..12]),
                rule_id: "R03_SANDBOX_BOUNDARY_ATTEMPT".to_string(),
                rule_name: "Workspace Boundary Violation / Traversal Attempt".to_string(),
                severity: sev,
                confidence: 0.95,
                score_contribution: score,
                evidence: vec![format!(
                    "{} blocked workspace access / validation failure event(s)",
                    boundary_violations.len()
                )],
                supporting_event_ids: boundary_violations
                    .iter()
                    .map(|e| e.event_id.clone())
                    .collect(),
                detected_at: Utc::now(),
                attribution_disclaimer: ATTRIBUTION_DISCLAIMER.to_string(),
            });
        }

        // Rule R04: Expired session activity
        let expired_evts: Vec<&MonitoringEvent> = events
            .iter()
            .filter(|e| e.event_type == EventType::SessionExpired)
            .collect();
        if !expired_evts.is_empty() {
            let score = self.weights.expired_session_activity;
            total_score += score;
            findings.push(SecurityFinding {
                finding_id: format!("fnd_{}", &Uuid::new_v4().simple().to_string()[..12]),
                rule_id: "R04_EXPIRED_SESSION_ACTIVITY".to_string(),
                rule_name: "Expired Session Activity Observed".to_string(),
                severity: EventSeverity::Medium,
                confidence: 0.90,
                score_contribution: score,
                evidence: vec![format!(
                    "Detected {} expired inspection session event(s)",
                    expired_evts.len()
                )],
                supporting_event_ids: expired_evts.iter().map(|e| e.event_id.clone()).collect(),
                detected_at: Utc::now(),
                attribution_disclaimer: ATTRIBUTION_DISCLAIMER.to_string(),
            });
        }

        // Rule R05: Unexpected genuine repository baseline drift
        let integrity_evts: Vec<&MonitoringEvent> = events
            .iter()
            .filter(|e| e.event_type == EventType::RepositoryIntegrityChanged)
            .collect();
        if !integrity_evts.is_empty() {
            let score = self.weights.genuine_integrity_drift;
            total_score += score;
            findings.push(SecurityFinding {
                finding_id: format!("fnd_{}", &Uuid::new_v4().simple().to_string()[..12]),
                rule_id: "R05_GENUINE_INTEGRITY_DRIFT".to_string(),
                rule_name: "Repository Baseline Integrity Change".to_string(),
                severity: EventSeverity::High,
                confidence: 1.0,
                score_contribution: score,
                evidence: vec![format!(
                    "{} unexpected genuine repository file change(s) detected against BLAKE3 baseline",
                    integrity_evts.len()
                )],
                supporting_event_ids: integrity_evts.iter().map(|e| e.event_id.clone()).collect(),
                detected_at: Utc::now(),
                attribution_disclaimer: ATTRIBUTION_DISCLAIMER.to_string(),
            });
        }

        (findings, total_score)
    }
}
