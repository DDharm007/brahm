//! Stage 06 Security Intelligence Report Generator (Milestone 06.8).
//!
//! Produces all 5 required deliverables:
//! - BRAHM_MONITORING_REPORT.json
//! - BRAHM_SECURITY_EVENTS.json
//! - BRAHM_SESSION_TIMELINE.json
//! - BRAHM_INTEGRITY_REPORT.json
//! - BRAHM_INTELLIGENCE_REPORT.md

use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};

use super::classifier::RuleBasedClassifier;
use super::correlator::TimelineCorrelator;
use super::models::{
    EventType, IntegrityRecord, IntelligenceReport, MonitoringEvent, SessionTimeline,
};
use crate::errors::Result;

/// Bundle of generated Stage 06 artifacts and file paths.
pub struct GeneratedMonitoringReports {
    pub report: IntelligenceReport,
    pub timelines: Vec<SessionTimeline>,
    pub monitoring_report_path: PathBuf,
    pub events_path: PathBuf,
    pub timeline_path: PathBuf,
    pub integrity_path: PathBuf,
    pub markdown_path: PathBuf,
}

/// Synthesize monitoring events, timelines, rule classifications, and integrity checks
/// into the complete suite of Stage 06 JSON and Markdown reports.
pub fn generate_all_monitoring_reports(
    repository_id: &str,
    repository_name: &str,
    events: &[MonitoringEvent],
    integrity_status: IntegrityRecord,
    session_count: usize,
    classifier: &RuleBasedClassifier,
    output_dir: &Path,
) -> Result<GeneratedMonitoringReports> {
    fs::create_dir_all(output_dir)?;

    let correlator = TimelineCorrelator::new();
    let timelines = correlator.correlate_all_sessions(repository_id, events);
    let (findings, aggregate_risk_score) = classifier.classify(events);

    let monitoring_start = events.iter().map(|e| e.timestamp).min();
    let monitoring_end = events.iter().map(|e| e.timestamp).max();

    let canary_events_count = events
        .iter()
        .filter(|e| e.event_type == EventType::CanaryAccessed)
        .count();
    let decoy_interactions_count = events
        .iter()
        .filter(|e| {
            matches!(
                e.event_type,
                EventType::DecoyAccessed
                    | EventType::CanaryAccessed
                    | EventType::SyntheticArtifactModified
            )
        })
        .count();

    let known_limitations = vec![
        "BRAHM cannot claim prevention of reverse engineering or copying from observed decoy hits alone.".to_string(),
        "Canary interactions establish observed artifact access inside the telemetry boundary; actor attribution (human vs. AI model) remains uncertain without external corroboration.".to_string(),
        "File integrity checks validate cryptographic BLAKE3 match against baseline snapshots, not runtime semantic safety.".to_string(),
    ];

    let mut recommended_actions = Vec::new();
    if !integrity_status.baseline_matches || !integrity_status.decoy_integrity_valid {
        recommended_actions.push(
            "Inspect integrity diffs carefully and verify authorized changes before updating baseline."
                .to_string(),
        );
    }
    if !findings.is_empty() {
        recommended_actions.push(
            "Review supporting rule evidence and correlate session timelines before taking defensive action."
                .to_string(),
        );
    }
    if recommended_actions.is_empty() {
        recommended_actions.push("Continue scheduled baseline integrity verification.".to_string());
    }

    let report = IntelligenceReport {
        schema_version: "1.0".to_string(),
        generated_at: Utc::now(),
        repository_id: repository_id.to_string(),
        repository_name: repository_name.to_string(),
        monitoring_start,
        monitoring_end,
        total_events: events.len(),
        canary_events_count,
        decoy_interactions_count,
        session_count,
        aggregate_risk_score,
        integrity_status: integrity_status.clone(),
        findings,
        known_limitations,
        recommended_actions,
    };

    let monitoring_report_path = output_dir.join("BRAHM_MONITORING_REPORT.json");
    fs::write(
        &monitoring_report_path,
        serde_json::to_string_pretty(&report)?,
    )?;

    let events_path = output_dir.join("BRAHM_SECURITY_EVENTS.json");
    fs::write(&events_path, serde_json::to_string_pretty(events)?)?;

    let timeline_path = output_dir.join("BRAHM_SESSION_TIMELINE.json");
    fs::write(&timeline_path, serde_json::to_string_pretty(&timelines)?)?;

    let integrity_path = output_dir.join("BRAHM_INTEGRITY_REPORT.json");
    fs::write(
        &integrity_path,
        serde_json::to_string_pretty(&integrity_status)?,
    )?;

    let markdown = render_intelligence_markdown(&report, &timelines);
    let markdown_path = output_dir.join("BRAHM_INTELLIGENCE_REPORT.md");
    fs::write(&markdown_path, markdown)?;

    Ok(GeneratedMonitoringReports {
        report,
        timelines,
        monitoring_report_path,
        events_path,
        timeline_path,
        integrity_path,
        markdown_path,
    })
}

/// Render the comprehensive 11-section `BRAHM_INTELLIGENCE_REPORT.md`.
pub fn render_intelligence_markdown(
    report: &IntelligenceReport,
    timelines: &[SessionTimeline],
) -> String {
    let mut md = String::new();
    md.push_str("# BRAHM Stage 06 — Security Monitoring & Intelligence Report\n\n");
    md.push_str("> Generated deterministically by **BRAHM Monitoring & Intelligence Engine**\n\n");

    md.push_str("## 1. Repository Identity\n\n");
    md.push_str(&format!("- **Name:** `{}`\n", report.repository_name));
    md.push_str(&format!(
        "- **Repository ID:** `{}`\n\n",
        report.repository_id
    ));

    md.push_str("## 2. Monitoring Period\n\n");
    let start_str = report
        .monitoring_start
        .map(|t| t.to_rfc3339())
        .unwrap_or_else(|| "N/A".to_string());
    let end_str = report
        .monitoring_end
        .map(|t| t.to_rfc3339())
        .unwrap_or_else(|| "N/A".to_string());
    md.push_str(&format!(
        "- **Start:** `{}`\n- **End:** `{}`\n\n",
        start_str, end_str
    ));

    md.push_str("## 3. Total Events\n\n");
    md.push_str(&format!(
        "- **Total Collected Events:** `{}`\n",
        report.total_events
    ));
    md.push_str(&format!(
        "- **Aggregate Rule Risk Score:** `{}`\n\n",
        report.aggregate_risk_score
    ));

    md.push_str("## 4. Canary Events\n\n");
    md.push_str(&format!(
        "- **Canary Trigger Events Observed:** `{}`\n\n",
        report.canary_events_count
    ));

    md.push_str("## 5. Decoy Interactions\n\n");
    md.push_str(&format!(
        "- **Total Decoy & Synthetic Interactions:** `{}`\n\n",
        report.decoy_interactions_count
    ));

    md.push_str("## 6. Session History\n\n");
    md.push_str(&format!(
        "- **Recorded Inspection Sessions:** `{}`\n\n",
        report.session_count
    ));
    for tl in timelines {
        md.push_str(&format!(
            "### Session `{}` ({} events, {} canary hits)\n",
            tl.session_id, tl.event_count, tl.canary_access_count
        ));
        for entry in &tl.entries {
            md.push_str(&format!(
                "- `{}` | `[{}]` {}\n",
                entry.timestamp.to_rfc3339(),
                entry.severity,
                entry.summary
            ));
        }
        md.push('\n');
    }

    md.push_str("## 7. Integrity Status\n\n");
    md.push_str(&format!(
        "- **Genuine Baseline Matches:** `{}`\n",
        report.integrity_status.baseline_matches
    ));
    md.push_str(&format!(
        "- **Staged Decoy Integrity Valid:** `{}`\n",
        report.integrity_status.decoy_integrity_valid
    ));
    md.push_str(&format!(
        "- **Unexpected Drift Items:** `{}`\n\n",
        report.integrity_status.unexpected_change_count
    ));

    md.push_str("## 8. Suspicious Activity Classifications\n\n");
    if report.findings.is_empty() {
        md.push_str("_No rule-based suspicious activity thresholds triggered._\n\n");
    } else {
        for f in &report.findings {
            md.push_str(&format!(
                "- **{}** (`{}` | Severity: `{}` | Confidence: `{:.0}%` | Weight: `{}`)\n",
                f.rule_name,
                f.rule_id,
                f.severity,
                f.confidence * 100.0,
                f.score_contribution
            ));
        }
        md.push('\n');
    }

    md.push_str("## 9. Rule-Based Evidence\n\n");
    if report.findings.is_empty() {
        md.push_str("_No evidentiary rule triggers recorded._\n\n");
    } else {
        for f in &report.findings {
            for ev in &f.evidence {
                md.push_str(&format!("- `[{}]` {}\n", f.rule_id, ev));
            }
            md.push_str(&format!("  - _Note:_ {}\n", f.attribution_disclaimer));
        }
        md.push('\n');
    }

    md.push_str("## 10. Known Limitations\n\n");
    for lim in &report.known_limitations {
        md.push_str(&format!("- {}\n", lim));
    }
    md.push('\n');

    md.push_str("## 11. Recommended Defensive Actions\n\n");
    for act in &report.recommended_actions {
        md.push_str(&format!("- {}\n", act));
    }
    md.push('\n');

    md
}
