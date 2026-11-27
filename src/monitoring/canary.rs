//! Canary Event Detection & Deduplication (Milestone 06.2).

use chrono::{DateTime, Utc};
use std::collections::BTreeMap;
use std::collections::HashMap;
use uuid::Uuid;

use super::models::{
    CanaryEvent, EventSeverity, EventSource, EventType, FalsePositiveStatus, MonitoringEvent,
};
use crate::deception::decoy::DecoyRegistry;
use crate::deception::WorkspaceAccessRecord;

/// Detector matching sandbox accesses or test-log marker observations against registered canaries.
#[derive(Debug, Clone)]
pub struct CanaryDetector {
    dedup_window_secs: i64,
    last_seen: HashMap<String, DateTime<Utc>>,
}

impl Default for CanaryDetector {
    fn default() -> Self {
        Self::new(5)
    }
}

impl CanaryDetector {
    pub fn new(dedup_window_secs: i64) -> Self {
        Self {
            dedup_window_secs,
            last_seen: HashMap::new(),
        }
    }

    /// Evaluate sandbox access logs against `DecoyRegistry` and emit correlated events.
    pub fn process_sandbox_accesses(
        &mut self,
        repository_id: &str,
        registry: &DecoyRegistry,
        records: &[WorkspaceAccessRecord],
        correlation_id: &str,
    ) -> (Vec<CanaryEvent>, Vec<MonitoringEvent>) {
        let mut canary_events = Vec::new();
        let mut mon_events = Vec::new();

        for rec in records {
            if !rec.allowed {
                let mut meta = BTreeMap::new();
                meta.insert("requested_path".to_string(), rec.requested_path.clone());
                meta.insert("reason".to_string(), rec.reason.clone());
                mon_events.push(MonitoringEvent {
                    event_id: format!("evt_{}", &Uuid::new_v4().simple().to_string()[..12]),
                    event_type: EventType::ValidationFailed,
                    repository_id: repository_id.to_string(),
                    session_id: Some(rec.session_id.clone()),
                    artifact_id: None,
                    timestamp: rec.timestamp,
                    source: EventSource::InspectionSandbox,
                    severity: EventSeverity::High,
                    correlation_id: correlation_id.to_string(),
                    metadata: meta,
                });
                continue;
            }

            if let Some(ref token) = rec.matched_canary_token {
                if let Some((canary_evt, mon_evt)) = self.detect_marker_token(
                    repository_id,
                    Some(&rec.session_id),
                    token,
                    &rec.requested_path,
                    registry,
                    false,
                    correlation_id,
                    rec.timestamp,
                ) {
                    canary_events.push(canary_evt);
                    mon_events.push(mon_evt);
                }
            } else if let Some(ref art_id) = rec.matched_artifact_id {
                let mut meta = BTreeMap::new();
                meta.insert("relative_path".to_string(), rec.requested_path.clone());
                mon_events.push(MonitoringEvent {
                    event_id: format!("evt_{}", &Uuid::new_v4().simple().to_string()[..12]),
                    event_type: EventType::DecoyAccessed,
                    repository_id: repository_id.to_string(),
                    session_id: Some(rec.session_id.clone()),
                    artifact_id: Some(art_id.clone()),
                    timestamp: rec.timestamp,
                    source: EventSource::InspectionSandbox,
                    severity: EventSeverity::Low,
                    correlation_id: correlation_id.to_string(),
                    metadata: meta,
                });
            }
        }

        (canary_events, mon_events)
    }

    /// Scan an external text excerpt (e.g., an authorized test harness log) for registered canary markers.
    /// Rejects unregistered markers cleanly; never logs private secrets.
    pub fn scan_text_for_canaries(
        &mut self,
        repository_id: &str,
        session_id: Option<&str>,
        registry: &DecoyRegistry,
        log_text: &str,
        correlation_id: &str,
    ) -> (Vec<CanaryEvent>, Vec<MonitoringEvent>) {
        let mut canary_events = Vec::new();
        let mut mon_events = Vec::new();

        for art in registry.artifacts() {
            if let Some(ref token) = art.canary_token {
                if log_text.contains(token) {
                    if let Some((c_evt, m_evt)) = self.detect_marker_token(
                        repository_id,
                        session_id,
                        token,
                        "test_log_excerpt",
                        registry,
                        false,
                        correlation_id,
                        Utc::now(),
                    ) {
                        canary_events.push(c_evt);
                        mon_events.push(m_evt);
                    }
                }
            }
        }

        (canary_events, mon_events)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn detect_marker_token(
        &mut self,
        repository_id: &str,
        session_id: Option<&str>,
        token: &str,
        context_note: &str,
        registry: &DecoyRegistry,
        is_internal_validator: bool,
        correlation_id: &str,
        observed_at: DateTime<Utc>,
    ) -> Option<(CanaryEvent, MonitoringEvent)> {
        let art = registry.find_by_canary(token)?;

        let dedup_key = format!("{}:{}", session_id.unwrap_or("global"), art.artifact_id);
        let is_duplicate = if let Some(prev) = self.last_seen.get(&dedup_key) {
            (observed_at - *prev).num_seconds().abs() <= self.dedup_window_secs
        } else {
            false
        };
        self.last_seen.insert(dedup_key, observed_at);

        let classification = if is_internal_validator {
            FalsePositiveStatus::BenignValidatorScan
        } else {
            FalsePositiveStatus::ConfirmedSignal
        };

        let sev = match (classification, is_duplicate) {
            (FalsePositiveStatus::BenignValidatorScan, _) => EventSeverity::Informational,
            (FalsePositiveStatus::ConfirmedSignal, true) => EventSeverity::Low,
            (FalsePositiveStatus::ConfirmedSignal, false) => EventSeverity::Medium,
            _ => EventSeverity::Low,
        };

        let canary_evt = CanaryEvent {
            canary_event_id: format!("can_{}", &Uuid::new_v4().simple().to_string()[..12]),
            repository_id: repository_id.to_string(),
            session_id: session_id.map(|s| s.to_string()),
            artifact_id: art.artifact_id.clone(),
            canary_token: token.to_string(),
            observed_at,
            context_note: context_note.to_string(),
            is_duplicate,
            classification,
        };

        let mut meta = BTreeMap::new();
        meta.insert("canary_token".to_string(), token.to_string());
        meta.insert("context".to_string(), context_note.to_string());
        meta.insert("duplicate".to_string(), is_duplicate.to_string());
        meta.insert(
            "classification".to_string(),
            format!("{:?}", classification),
        );

        let mon_evt = MonitoringEvent {
            event_id: format!("evt_{}", &Uuid::new_v4().simple().to_string()[..12]),
            event_type: EventType::CanaryAccessed,
            repository_id: repository_id.to_string(),
            session_id: session_id.map(|s| s.to_string()),
            artifact_id: Some(art.artifact_id.clone()),
            timestamp: observed_at,
            source: EventSource::CanaryDetector,
            severity: sev,
            correlation_id: correlation_id.to_string(),
            metadata: meta,
        };

        Some((canary_evt, mon_evt))
    }
}
