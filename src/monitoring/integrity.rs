//! Continuous BLAKE3 Repository & Decoy Integrity Monitor (Milestone 06.6).
//!
//! Compares live genuine files against their Stage 01/04 BLAKE3 baseline and verifies staged
//! synthetic decoys without ever automatically mutating or deleting user files.

use chrono::Utc;
use std::collections::{BTreeMap, HashSet};
use std::path::Path;
use uuid::Uuid;

use super::models::{
    EventSeverity, EventSource, EventType, IntegrityChangeKind, IntegrityFileDiff, IntegrityRecord,
    MonitoringEvent,
};
use crate::deception::decoy::DecoyRegistry;
use crate::deception::{GenuineLayer, ProductionIntegrityGuard};
use crate::errors::Result;

/// Integrity verification engine integrating Stage 01 BLAKE3 hashing into Stage 06 monitoring.
#[derive(Debug, Default, Clone)]
pub struct IntegrityMonitor {
    guard: ProductionIntegrityGuard,
}

impl IntegrityMonitor {
    pub fn new() -> Self {
        Self {
            guard: ProductionIntegrityGuard::new(),
        }
    }

    /// Compare live repository files against `expected_baseline` and staged `decoy_registry`.
    /// Files listed in `expected_change_paths` are explicitly marked as expected maintenance.
    ///
    /// Returns an `IntegrityRecord` plus derived `MonitoringEvent` alerts.
    pub fn check_integrity(
        &self,
        repository_id: &str,
        repo_root: &Path,
        expected_baseline: &GenuineLayer,
        decoy_registry: Option<(&DecoyRegistry, &Path)>,
        expected_change_paths: &HashSet<String>,
        correlation_id: &str,
    ) -> Result<(IntegrityRecord, Vec<MonitoringEvent>)> {
        let live_layer = self.guard.capture_genuine_baseline(repo_root)?;

        let mut diffs = Vec::new();
        let mut events = Vec::new();

        // 1. Check removed and modified genuine files
        for (rel_path, old_hash) in &expected_baseline.file_hashes {
            let expected = expected_change_paths.contains(rel_path);
            match live_layer.file_hashes.get(rel_path) {
                None => {
                    diffs.push(IntegrityFileDiff {
                        relative_path: rel_path.clone(),
                        change_kind: IntegrityChangeKind::Removed,
                        expected_change: expected,
                        detail: "File present in genuine baseline is missing on disk".to_string(),
                    });
                    if !expected {
                        events.push(build_integrity_event(
                            repository_id,
                            EventType::RepositoryIntegrityChanged,
                            EventSeverity::High,
                            rel_path,
                            "removed",
                            correlation_id,
                        ));
                    }
                }
                Some(new_hash) if new_hash != old_hash => {
                    diffs.push(IntegrityFileDiff {
                        relative_path: rel_path.clone(),
                        change_kind: IntegrityChangeKind::Modified,
                        expected_change: expected,
                        detail: format!(
                            "BLAKE3 digest changed ({} -> {})",
                            &old_hash[..10],
                            &new_hash[..10]
                        ),
                    });
                    if !expected {
                        events.push(build_integrity_event(
                            repository_id,
                            EventType::RepositoryIntegrityChanged,
                            EventSeverity::High,
                            rel_path,
                            "modified",
                            correlation_id,
                        ));
                    }
                }
                _ => {}
            }
        }

        // 2. Check added files
        for rel_path in live_layer.file_hashes.keys() {
            if !expected_baseline.file_hashes.contains_key(rel_path) {
                let expected = expected_change_paths.contains(rel_path);
                diffs.push(IntegrityFileDiff {
                    relative_path: rel_path.clone(),
                    change_kind: IntegrityChangeKind::Added,
                    expected_change: expected,
                    detail: "New file absent from baseline snapshot".to_string(),
                });
                if !expected {
                    events.push(build_integrity_event(
                        repository_id,
                        EventType::RepositoryIntegrityChanged,
                        EventSeverity::Medium,
                        rel_path,
                        "added",
                        correlation_id,
                    ));
                }
            }
        }

        // 3. Verify staged decoys if provided
        let mut decoy_integrity_valid = true;
        if let Some((registry, decoy_root)) = decoy_registry {
            for (art_id, ok, msg) in registry.verify_staged_integrity(decoy_root) {
                if !ok {
                    decoy_integrity_valid = false;
                    let rel = registry
                        .get(&art_id)
                        .map(|a| a.synthetic_relative_path.clone())
                        .unwrap_or_else(|| art_id.clone());

                    diffs.push(IntegrityFileDiff {
                        relative_path: rel.clone(),
                        change_kind: IntegrityChangeKind::DecoyModified,
                        expected_change: false,
                        detail: msg.clone(),
                    });

                    let mut meta = BTreeMap::new();
                    meta.insert("relative_path".to_string(), rel);
                    meta.insert("detail".to_string(), msg);

                    events.push(MonitoringEvent {
                        event_id: format!("evt_{}", &Uuid::new_v4().simple().to_string()[..12]),
                        event_type: EventType::SyntheticArtifactModified,
                        repository_id: repository_id.to_string(),
                        session_id: None,
                        artifact_id: Some(art_id),
                        timestamp: Utc::now(),
                        source: EventSource::IntegrityMonitor,
                        severity: EventSeverity::High,
                        correlation_id: correlation_id.to_string(),
                        metadata: meta,
                    });
                }
            }
        }

        let unexpected_change_count = diffs.iter().filter(|d| !d.expected_change).count();
        let baseline_matches =
            live_layer.baseline_fingerprint == expected_baseline.baseline_fingerprint;

        let guidance = if unexpected_change_count == 0 && baseline_matches && decoy_integrity_valid
        {
            "No unexpected integrity drift detected.".to_string()
        } else {
            "Unexpected file or decoy drift detected. Review diffs and obtain explicit operator approval before modifying or restoring files.".to_string()
        };

        let record = IntegrityRecord {
            record_id: format!("int_{}", &Uuid::new_v4().simple().to_string()[..12]),
            repository_id: repository_id.to_string(),
            checked_at: Utc::now(),
            baseline_matches,
            decoy_integrity_valid,
            unexpected_change_count,
            diffs,
            remediation_guidance: guidance,
        };

        Ok((record, events))
    }
}

fn build_integrity_event(
    repository_id: &str,
    event_type: EventType,
    severity: EventSeverity,
    rel_path: &str,
    change_kind: &str,
    correlation_id: &str,
) -> MonitoringEvent {
    let mut meta = BTreeMap::new();
    meta.insert("relative_path".to_string(), rel_path.to_string());
    meta.insert("change".to_string(), change_kind.to_string());
    MonitoringEvent {
        event_id: format!("evt_{}", &Uuid::new_v4().simple().to_string()[..12]),
        event_type,
        repository_id: repository_id.to_string(),
        session_id: None,
        artifact_id: None,
        timestamp: Utc::now(),
        source: EventSource::IntegrityMonitor,
        severity,
        correlation_id: correlation_id.to_string(),
        metadata: meta,
    }
}
