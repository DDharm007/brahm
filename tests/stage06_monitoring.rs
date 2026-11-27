//! Stage 06 Monitoring & Intelligence Engine Comprehensive Test Suite.

use brahm::cli::{
    Cli, Commands, EventsCommands, IntegrityCommands, MonitorCommands, ReportsCommands,
    SessionsTimelineCommands,
};
use brahm::deception::decoy::{DecoyEngine, DecoyGenerationProfile};
use brahm::deception::{
    ControlledInspectionEnvironment, DeceptionPolicy, DeceptionPolicyProfile,
    InspectionSessionManager, ProductionIntegrityGuard, WorkspaceAccessRecord,
};
use brahm::monitoring::{
    build_monitoring_router, generate_all_monitoring_reports, ApiState, CanaryDetector,
    EventSeverity, EventSource, EventStore, EventType, FalsePositiveStatus, IntegrityChangeKind,
    IntegrityMonitor, IntegrityRecord, MonitoringApiConfig, MonitoringEvent, RuleBasedClassifier,
    ScoringWeights, SqliteEventStore, TimelineCorrelator,
};
use chrono::Utc;
use clap::Parser;
use std::collections::{BTreeMap, HashSet, VecDeque};
use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
use tempfile::tempdir;

#[test]
fn test_01_sqlite_event_store_migrations_and_queries() {
    let db_dir = tempdir().unwrap();
    let db_path = db_dir.path().join("monitoring.db");
    let store = SqliteEventStore::open(&db_path).expect("SQLite DB must open & migrate");
    assert!(db_path.is_file());

    let mut meta = BTreeMap::new();
    meta.insert(
        "requested_path".to_string(),
        "canary/BRAHM_CANARY_ANCHOR.md".to_string(),
    );

    let e1 = MonitoringEvent {
        event_id: "evt_001".to_string(),
        event_type: EventType::CanaryAccessed,
        repository_id: "repo_abc".to_string(),
        session_id: Some("sess_123".to_string()),
        artifact_id: Some("decoy_999".to_string()),
        timestamp: Utc::now(),
        source: EventSource::CanaryDetector,
        severity: EventSeverity::Medium,
        correlation_id: "corr_1".to_string(),
        metadata: meta,
    };

    store.record_event(&e1).unwrap();
    assert_eq!(store.count_events().unwrap(), 1);

    let loaded = store.get_event("evt_001").unwrap().unwrap();
    assert_eq!(loaded.event_type, EventType::CanaryAccessed);
    assert_eq!(loaded.session_id.as_deref(), Some("sess_123"));

    let list = store
        .list_events(Some("repo_abc"), Some("sess_123"), 50)
        .unwrap();
    assert_eq!(list.len(), 1);
}

#[test]
fn test_02_canary_detection_dedup_and_false_positive_classification() {
    let repo_dir = tempdir().unwrap();
    fs::write(repo_dir.path().join("lib.rs"), "pub fn core() {}\n").unwrap();

    let stage1 = brahm::analysis::run_repository_analysis(
        repo_dir.path(),
        brahm::config::BrahmConfig::default(),
    )
    .unwrap();
    let arch = brahm::architecture::ArchitectureEngine::new()
        .analyze(repo_dir.path(), &stage1)
        .unwrap();

    let decoy_dir = tempdir().unwrap();
    let decoy_res = DecoyEngine::new()
        .generate_and_stage(
            repo_dir.path(),
            &arch.report,
            DecoyGenerationProfile::Conservative,
            decoy_dir.path(),
        )
        .unwrap();

    let canary_art = decoy_res
        .registry
        .artifacts()
        .iter()
        .find(|a| a.canary_token.is_some())
        .unwrap();
    let token = canary_art.canary_token.as_ref().unwrap();

    let mut detector = CanaryDetector::new(10);
    let now = Utc::now();

    let rec1 = WorkspaceAccessRecord {
        session_id: "sess_1".to_string(),
        timestamp: now,
        requested_path: canary_art.synthetic_relative_path.clone(),
        allowed: true,
        matched_artifact_id: Some(canary_art.artifact_id.clone()),
        matched_canary_token: Some(token.clone()),
        reason: "read".to_string(),
    };
    let rec2 = rec1.clone();

    let (canaries, events) = detector.process_sandbox_accesses(
        &decoy_res.registry.data().repository_id,
        &decoy_res.registry,
        &[rec1, rec2],
        "corr_canary",
    );

    assert_eq!(canaries.len(), 2);
    assert_eq!(events.len(), 2);
    assert!(!canaries[0].is_duplicate);
    assert!(canaries[1].is_duplicate);
    assert_eq!(
        canaries[0].classification,
        FalsePositiveStatus::ConfirmedSignal
    );

    // Internal benign validator scan -> FalsePositiveStatus::BenignValidatorScan
    let (benign, _) = detector
        .detect_marker_token(
            &decoy_res.registry.data().repository_id,
            Some("sess_val"),
            token,
            "validator",
            &decoy_res.registry,
            true,
            "corr_val",
            now,
        )
        .unwrap();
    assert_eq!(
        benign.classification,
        FalsePositiveStatus::BenignValidatorScan
    );

    // Unregistered marker ignored safely
    assert!(detector
        .detect_marker_token(
            "repo_x",
            None,
            "BRAHM-CANARY-UNREGISTERED",
            "none",
            &decoy_res.registry,
            false,
            "corr_none",
            now,
        )
        .is_none());
}

#[test]
fn test_03_rule_based_classifier_and_uncertainty_disclaimer() {
    let classifier = RuleBasedClassifier::new(ScoringWeights::default());

    // 1. Single informational canary access does NOT overclaim malicious intent
    let mut meta = BTreeMap::new();
    meta.insert("duplicate".to_string(), "false".to_string());
    meta.insert("classification".to_string(), "ConfirmedSignal".to_string());

    let single = MonitoringEvent {
        event_id: "e1".to_string(),
        event_type: EventType::CanaryAccessed,
        repository_id: "repo".to_string(),
        session_id: Some("s1".to_string()),
        artifact_id: Some("d1".to_string()),
        timestamp: Utc::now(),
        source: EventSource::CanaryDetector,
        severity: EventSeverity::Medium,
        correlation_id: "c1".to_string(),
        metadata: meta.clone(),
    };
    let (f_single, score_single) = classifier.classify(std::slice::from_ref(&single));
    assert!(f_single.is_empty());
    assert_eq!(score_single, 0);

    // 2. Repeated canary + boundary traversal + modified decoy -> triggers documented rules with disclaimers
    let mut e2 = single.clone();
    e2.event_id = "e2".to_string();

    let e_boundary = MonitoringEvent {
        event_id: "e3".to_string(),
        event_type: EventType::ValidationFailed,
        repository_id: "repo".to_string(),
        session_id: Some("s1".to_string()),
        artifact_id: None,
        timestamp: Utc::now(),
        source: EventSource::InspectionSandbox,
        severity: EventSeverity::High,
        correlation_id: "c1".to_string(),
        metadata: BTreeMap::new(),
    };

    let (findings, score) = classifier.classify(&[single, e2, e_boundary]);
    assert_eq!(findings.len(), 2);
    assert!(score >= 60);
    for f in findings {
        assert!(f.attribution_disclaimer.contains("uncertain"));
    }
}

#[test]
fn test_04_integrity_monitor_baseline_diff_and_decoy_tamper() {
    let repo_dir = tempdir().unwrap();
    fs::write(repo_dir.path().join("a.rs"), "pub fn a() {}\n").unwrap();
    fs::write(repo_dir.path().join("b.rs"), "pub fn b() {}\n").unwrap();

    let stage1 = brahm::analysis::run_repository_analysis(
        repo_dir.path(),
        brahm::config::BrahmConfig::default(),
    )
    .unwrap();
    let arch = brahm::architecture::ArchitectureEngine::new()
        .analyze(repo_dir.path(), &stage1)
        .unwrap();

    let decoy_dir = tempdir().unwrap();
    let decoy_res = DecoyEngine::new()
        .generate_and_stage(
            repo_dir.path(),
            &arch.report,
            DecoyGenerationProfile::Conservative,
            decoy_dir.path(),
        )
        .unwrap();

    let guard = ProductionIntegrityGuard::new();
    let baseline = guard.capture_genuine_baseline(repo_dir.path()).unwrap();

    // Mutate repository & staged decoy cleanly for test detection
    fs::write(repo_dir.path().join("a.rs"), "pub fn a_modified() {}\n").unwrap();
    fs::remove_file(repo_dir.path().join("b.rs")).unwrap();
    fs::write(repo_dir.path().join("c_added.rs"), "pub fn c() {}\n").unwrap();

    let decoy_file = decoy_res
        .decoy_layer_root
        .join(&decoy_res.registry.artifacts()[0].synthetic_relative_path);
    fs::write(decoy_file, "tampered synthetic artifact").unwrap();

    let monitor = IntegrityMonitor::new();
    let expected = HashSet::new();
    let (record, events) = monitor
        .check_integrity(
            &decoy_res.registry.data().repository_id,
            repo_dir.path(),
            &baseline,
            Some((&decoy_res.registry, &decoy_res.decoy_layer_root)),
            &expected,
            "corr_int",
        )
        .unwrap();

    assert!(!record.baseline_matches);
    assert!(!record.decoy_integrity_valid);
    assert_eq!(record.unexpected_change_count, 4);
    assert!(record
        .diffs
        .iter()
        .any(|d| d.change_kind == IntegrityChangeKind::Modified));
    assert!(record
        .diffs
        .iter()
        .any(|d| d.change_kind == IntegrityChangeKind::Removed));
    assert!(record
        .diffs
        .iter()
        .any(|d| d.change_kind == IntegrityChangeKind::Added));
    assert!(record
        .diffs
        .iter()
        .any(|d| d.change_kind == IntegrityChangeKind::DecoyModified));
    assert_eq!(events.len(), 4);
}

#[tokio::test]
async fn test_05_authenticated_axum_api_security_and_rate_limiting() {
    let state_dir = tempdir().unwrap();

    // Reject non-loopback without explicit allow_remote
    let unsafe_cfg = MonitoringApiConfig {
        bind_addr: "0.0.0.0:8787".parse().unwrap(),
        allow_remote: false,
        auth_token: "very_secure_test_token_123456".to_string(),
        rate_limit_per_minute: 60,
        state_dir: state_dir.path().to_path_buf(),
    };
    assert!(unsafe_cfg.validate().is_err());

    let store = SqliteEventStore::open_in_memory().unwrap();
    let token = "very_secure_test_token_123456".to_string();
    let cfg = MonitoringApiConfig {
        bind_addr: "127.0.0.1:0".parse().unwrap(),
        allow_remote: false,
        auth_token: token.clone(),
        rate_limit_per_minute: 3,
        state_dir: state_dir.path().to_path_buf(),
    };
    cfg.validate().unwrap();

    let state = ApiState {
        store,
        config: cfg,
        request_timestamps: Arc::new(Mutex::new(VecDeque::new())),
    };
    let app = build_monitoring_router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    let server_handle = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    // Helper performing synchronous HTTP/1.1 GET on loopback
    let send_get = move |path: &str, bearer: Option<&str>| -> String {
        let mut stream = TcpStream::connect(addr).unwrap();
        let auth_line = match bearer {
            Some(t) => format!("Authorization: Bearer {}\r\n", t),
            None => String::new(),
        };
        let req = format!(
            "GET {} HTTP/1.1\r\nHost: 127.0.0.1\r\n{}Connection: close\r\n\r\n",
            path, auth_line
        );
        stream.write_all(req.as_bytes()).unwrap();
        let mut resp = String::new();
        stream.read_to_string(&mut resp).unwrap();
        resp
    };

    // 1. Health endpoint succeeds
    let r_health = tokio::task::spawn_blocking(move || send_get("/api/v1/health", None))
        .await
        .unwrap();
    assert!(r_health.starts_with("HTTP/1.1 200"));

    // 2. Unauthenticated /api/v1/events returns 401 Unauthorized
    let r_unauth = tokio::task::spawn_blocking(move || send_get("/api/v1/events", None))
        .await
        .unwrap();
    assert!(r_unauth.starts_with("HTTP/1.1 401"));

    // 3. Authenticated invalid query returns 400 Bad Request (and counts toward rate limit 1/3)
    let t1 = token.clone();
    let r_bad_q =
        tokio::task::spawn_blocking(move || send_get("/api/v1/events?limit=99999", Some(&t1)))
            .await
            .unwrap();
    assert!(r_bad_q.starts_with("HTTP/1.1 400"));

    // 4. Valid authenticated /api/v1/events returns 200 (2/3, 3/3) and 4th hits 429 Too Many Requests
    let t2 = token.clone();
    let r_ok1 = tokio::task::spawn_blocking(move || send_get("/api/v1/events?limit=10", Some(&t2)))
        .await
        .unwrap();
    assert!(r_ok1.starts_with("HTTP/1.1 200"));

    let t3 = token.clone();
    let r_ok2 = tokio::task::spawn_blocking(move || send_get("/api/v1/repositories", Some(&t3)))
        .await
        .unwrap();
    assert!(r_ok2.starts_with("HTTP/1.1 200"));

    let t4 = token;
    let r_limited = tokio::task::spawn_blocking(move || send_get("/api/v1/decoys", Some(&t4)))
        .await
        .unwrap();
    assert!(r_limited.starts_with("HTTP/1.1 429"));

    server_handle.abort();
}

#[test]
fn test_06_intelligence_report_all_5_deliverables_and_cli_parsing() {
    let repo_dir = tempdir().unwrap();
    fs::write(repo_dir.path().join("main.py"), "print('ok')\n").unwrap();
    let stage1 = brahm::analysis::run_repository_analysis(
        repo_dir.path(),
        brahm::config::BrahmConfig::default(),
    )
    .unwrap();
    let arch = brahm::architecture::ArchitectureEngine::new()
        .analyze(repo_dir.path(), &stage1)
        .unwrap();

    let state_dir = tempdir().unwrap();
    let decoy_res = DecoyEngine::new()
        .generate_and_stage(
            repo_dir.path(),
            &arch.report,
            DecoyGenerationProfile::Conservative,
            &state_dir.path().join("decoys"),
        )
        .unwrap();

    let sess_mgr = InspectionSessionManager::new(state_dir.path().join("sessions"));
    let prepared = sess_mgr
        .create_and_prepare(
            repo_dir.path(),
            &decoy_res.registry,
            &decoy_res.decoy_layer_root,
            &DeceptionPolicy::for_profile(DeceptionPolicyProfile::Conservative),
            3600,
        )
        .unwrap();

    let active = sess_mgr
        .activate_session(&prepared.session.session_id)
        .unwrap();
    let mut sandbox =
        ControlledInspectionEnvironment::open(active, decoy_res.registry.clone()).unwrap();
    let _ = sandbox
        .read_workspace_file("canary/BRAHM_CANARY_ANCHOR.md")
        .unwrap();
    let _ = sandbox.read_workspace_file("../escape.txt");

    let mut detector = CanaryDetector::new(5);
    let (_, events) = detector.process_sandbox_accesses(
        &decoy_res.registry.data().repository_id,
        &decoy_res.registry,
        sandbox.access_log(),
        "corr_report",
    );

    let timelines = TimelineCorrelator::new()
        .correlate_all_sessions(&decoy_res.registry.data().repository_id, &events);
    assert_eq!(timelines.len(), 1);
    assert_eq!(timelines[0].canary_access_count, 1);

    let int_record = IntegrityRecord {
        record_id: "int_1".to_string(),
        repository_id: decoy_res.registry.data().repository_id.clone(),
        checked_at: Utc::now(),
        baseline_matches: true,
        decoy_integrity_valid: true,
        unexpected_change_count: 0,
        diffs: Vec::new(),
        remediation_guidance: "None required".to_string(),
    };

    let reports = generate_all_monitoring_reports(
        &decoy_res.registry.data().repository_id,
        &arch.report.repository.name,
        &events,
        int_record,
        1,
        &RuleBasedClassifier::default(),
        state_dir.path(),
    )
    .unwrap();

    assert!(reports.monitoring_report_path.is_file());
    assert!(reports.events_path.is_file());
    assert!(reports.timeline_path.is_file());
    assert!(reports.integrity_path.is_file());
    assert!(reports.markdown_path.is_file());

    let md = fs::read_to_string(&reports.markdown_path).unwrap();
    assert!(md.contains("## 1. Repository Identity"));
    assert!(md.contains("## 10. Known Limitations"));
    assert!(md.contains("## 11. Recommended Defensive Actions"));

    // Verify Stage 06 CLI commands parse accurately
    assert!(matches!(
        Cli::try_parse_from(["brahm", "monitor", "status"])
            .unwrap()
            .command,
        Commands::Monitor {
            action: MonitorCommands::Status { .. }
        }
    ));
    assert!(matches!(
        Cli::try_parse_from(["brahm", "events", "list"])
            .unwrap()
            .command,
        Commands::Events {
            action: EventsCommands::List { .. }
        }
    ));
    assert!(matches!(
        Cli::try_parse_from(["brahm", "sessions", "timeline", "sess_1"])
            .unwrap()
            .command,
        Commands::Sessions {
            action: SessionsTimelineCommands::Timeline { .. }
        }
    ));
    assert!(matches!(
        Cli::try_parse_from(["brahm", "integrity", "check"])
            .unwrap()
            .command,
        Commands::Integrity {
            action: IntegrityCommands::Check { .. }
        }
    ));
    assert!(matches!(
        Cli::try_parse_from(["brahm", "reports", "generate"])
            .unwrap()
            .command,
        Commands::Reports {
            action: ReportsCommands::Generate { .. }
        }
    ));
}
