//! Stage 06 Monitoring & Intelligence Engine Subsystem.

pub mod api;
pub mod canary;
pub mod classifier;
pub mod correlator;
pub mod integrity;
pub mod models;
pub mod reports;
pub mod storage;

pub use api::{build_monitoring_router, serve_monitoring_api, ApiState, MonitoringApiConfig};
pub use canary::CanaryDetector;
pub use classifier::{RuleBasedClassifier, ScoringWeights};
pub use correlator::TimelineCorrelator;
pub use integrity::IntegrityMonitor;
pub use models::{
    CanaryEvent, EventSeverity, EventSource, EventType, FalsePositiveStatus, IntegrityChangeKind,
    IntegrityFileDiff, IntegrityRecord, IntelligenceReport, MonitoringEvent, SecurityFinding,
    SessionTimeline, TimelineEntry,
};
pub use reports::{
    generate_all_monitoring_reports, render_intelligence_markdown, GeneratedMonitoringReports,
};
pub use storage::{EventStore, SqliteEventStore};
