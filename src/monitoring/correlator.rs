//! Deterministic Timeline & Event Correlation Engine (Milestone 06.4).

use std::collections::{BTreeMap, HashSet};

use super::models::{EventType, MonitoringEvent, SessionTimeline, TimelineEntry};

/// Builds ordered chronological timelines grouped by inspection session.
#[derive(Debug, Default, Clone)]
pub struct TimelineCorrelator;

impl TimelineCorrelator {
    pub fn new() -> Self {
        Self
    }

    /// Correlate a slice of events for a specific `session_id` into an ordered timeline.
    pub fn build_session_timeline(
        &self,
        session_id: &str,
        repository_id: &str,
        events: &[MonitoringEvent],
    ) -> SessionTimeline {
        let mut matching: Vec<&MonitoringEvent> = events
            .iter()
            .filter(|e| e.session_id.as_deref() == Some(session_id))
            .collect();

        matching.sort_by_key(|e| (e.timestamp, e.event_id.clone()));

        let mut seen_ids = HashSet::new();
        let mut entries = Vec::new();
        let mut canary_count = 0;

        for evt in matching {
            if !seen_ids.insert(evt.event_id.clone()) {
                continue;
            }
            if evt.event_type == EventType::CanaryAccessed {
                canary_count += 1;
            }

            let detail_hint = evt
                .metadata
                .get("requested_path")
                .or_else(|| evt.metadata.get("context"))
                .or_else(|| evt.metadata.get("relative_path"))
                .map(|v| format!(" ({})", v))
                .unwrap_or_default();

            let art_hint = evt
                .artifact_id
                .as_ref()
                .map(|id| format!(" [{}]", id))
                .unwrap_or_default();

            entries.push(TimelineEntry {
                timestamp: evt.timestamp,
                event_id: evt.event_id.clone(),
                event_type: evt.event_type,
                severity: evt.severity,
                summary: format!("{}{}{}", evt.event_type, art_hint, detail_hint),
            });
        }

        SessionTimeline {
            session_id: session_id.to_string(),
            repository_id: repository_id.to_string(),
            event_count: entries.len(),
            canary_access_count: canary_count,
            entries,
        }
    }

    /// Build timelines for every distinct session present in `events`.
    pub fn correlate_all_sessions(
        &self,
        repository_id: &str,
        events: &[MonitoringEvent],
    ) -> Vec<SessionTimeline> {
        let mut by_session: BTreeMap<String, Vec<MonitoringEvent>> = BTreeMap::new();
        for evt in events {
            let sid = evt
                .session_id
                .clone()
                .unwrap_or_else(|| "unscoped_global".to_string());
            by_session.entry(sid).or_default().push(evt.clone());
        }

        by_session
            .into_iter()
            .map(|(sid, evts)| self.build_session_timeline(&sid, repository_id, &evts))
            .collect()
    }
}
