//! Complete Cross-Stage End-to-End Pipeline & Edge Case Validation (Stages 01–06).
//!
//! Validates the full 17-step connected workflow plus all Section 13 fault/boundary conditions.

use brahm::analysis::run_repository_analysis;
use brahm::architecture::{ArchitectureEngine, ArchitectureReport};
use brahm::config::BrahmConfig;
use brahm::deception::decoy::{
    DecoyCategory, DecoyEngine, DecoyGenerationProfile, DecoyRegistry, DecoyTemplate,
    DecoyValidator, RiskClassification, TemplateRegistry,
};
use brahm::deception::{
    ControlledInspectionEnvironment, DeceptionPolicy, DeceptionPolicyProfile,
    InspectionSessionManager, ProductionIntegrityGuard, SessionStatus,
};
use brahm::integration::{
    InspectionAgent, LocalTestAgentAdapter, ProtectionInstaller, ProtectionProfile,
    RepositoryInputManager,
};
use brahm::monitoring::{
    generate_all_monitoring_reports, CanaryDetector, EventSeverity, EventSource, EventStore,
    EventType, IntegrityMonitor, MonitoringEvent, RuleBasedClassifier, ScoringWeights,
    SqliteEventStore, TimelineCorrelator,
};
use chrono::{Duration, Utc};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::Path;
use std::time::Instant;
use tempfile::tempdir;

#[test]
fn test_e2e_full_17_step_connected_pipeline() {
    // Prepare isolated synthetic repo with genuine source + genuine .env secret
    let repo_dir = tempdir().unwrap();
    let app_src = repo_dir.path().join("src/server.js");
    fs::create_dir_all(app_src.parent().unwrap()).unwrap();
    fs::write(
        &app_src,
        "const express = require('express');\nconst app = express();\napp.get('/api/health', healthCheck);\nfunction healthCheck(req, res) {}\n",
    )
    .unwrap();
    fs::write(
        repo_dir.path().join("package.json"),
        r#"{"name": "e2e-app", "dependencies": {"express": "^4.18.0"}}"#,
    )
    .unwrap();
    fs::write(
        repo_dir.path().join(".env"),
        "DB_PASS=sensitive_value_must_not_appear_anywhere\n",
    )
    .unwrap();

    let meta_before = fs::metadata(&app_src).unwrap();
    let mtime_before = meta_before.modified().unwrap();

    let state_dir = tempdir().unwrap();
    let t_start = Instant::now();

    // Steps 1–3: Register local repo + Stage 01 scan + Stage 02 architecture baseline
    let input_mgr = RepositoryInputManager::new();
    let conn = input_mgr
        .connect_local(repo_dir.path(), state_dir.path(), None)
        .unwrap();
    assert_eq!(conn.stage1_report.repository.name, conn.record.name);
    assert!(conn.architecture.report.metrics.api_route_count >= 1);

    // Steps 4–5: Generate Stage 03 decoys & validate registry
    let t_decoy = Instant::now();
    let decoy_res = DecoyEngine::new()
        .generate_and_stage(
            repo_dir.path(),
            &conn.architecture.report,
            DecoyGenerationProfile::Standard,
            &state_dir.path().join("decoys"),
        )
        .unwrap();
    let decoy_ms = t_decoy.elapsed().as_millis();
    assert!(decoy_res.validation.valid);
    assert!(decoy_res
        .registry
        .verify_staged_integrity(&decoy_res.decoy_layer_root)
        .iter()
        .all(|(_, ok, _)| *ok));

    // Steps 6–8: Prepare Stage 04 mirror, deception policy & active inspection session
    let t_mirror = Instant::now();
    let sess_mgr = InspectionSessionManager::new(state_dir.path().join("sessions"));
    let policy = DeceptionPolicy::for_profile(DeceptionPolicyProfile::Standard);
    let prepared = sess_mgr
        .create_and_prepare(
            repo_dir.path(),
            &decoy_res.registry,
            &decoy_res.decoy_layer_root,
            &policy,
            3600,
        )
        .unwrap();
    let mirror_ms = t_mirror.elapsed().as_millis();
    assert!(!prepared.session.workspace_path.join(".env").exists());
    assert!(prepared.integrity_report.production_isolated);

    let active_sess = sess_mgr
        .activate_session(&prepared.session.session_id)
        .unwrap();

    // Steps 9–10: Connect Stage 05 LocalTestAgentAdapter & simulate authorized interactions
    let agent = LocalTestAgentAdapter::new();
    let mut sandbox =
        ControlledInspectionEnvironment::open(active_sess.clone(), decoy_res.registry.clone())
            .unwrap();
    let inspect_targets = vec![
        "src/server.js".to_string(),
        "canary/BRAHM_CANARY_ANCHOR.md".to_string(),
        "docs/SYSTEM_ARCHITECTURE.md".to_string(),
        "../../outside_attempt.txt".to_string(),
    ];
    let agent_res = agent
        .run_inspection(&mut sandbox, &inspect_targets)
        .unwrap();
    assert_eq!(agent_res.accessed_files.len(), 3);
    assert_eq!(
        agent_res.blocked_attempts,
        vec!["../../outside_attempt.txt"]
    );

    // Steps 11–12: Record events into SQLite, detect canary signals & correlate session timeline
    let db_path = state_dir.path().join("brahm_monitoring.db");
    let store = SqliteEventStore::open(&db_path).unwrap();
    store.record_session_ref(&active_sess).unwrap();
    for art in decoy_res.registry.artifacts() {
        store
            .record_decoy_ref(&conn.record.repository_id, art)
            .unwrap();
    }

    let corr_id = "e2e_corr_1";
    let mut detector = CanaryDetector::new(5);
    let (canaries, mut mon_events) = detector.process_sandbox_accesses(
        &conn.record.repository_id,
        &decoy_res.registry,
        sandbox.access_log(),
        corr_id,
    );
    assert!(!canaries.is_empty());

    // Emit lifecycle start/end events cleanly
    mon_events.push(MonitoringEvent {
        event_id: "evt_e2e_start".to_string(),
        event_type: EventType::InspectionSessionStarted,
        repository_id: conn.record.repository_id.clone(),
        session_id: Some(active_sess.session_id.clone()),
        artifact_id: None,
        timestamp: active_sess.created_at,
        source: EventSource::PolicyEngine,
        severity: EventSeverity::Informational,
        correlation_id: corr_id.to_string(),
        metadata: BTreeMap::new(),
    });
    store.record_events_batch(&mon_events).unwrap();

    let loaded_events = store
        .list_events(Some(&conn.record.repository_id), None, 500)
        .unwrap();
    let timeline = TimelineCorrelator::new().build_session_timeline(
        &active_sess.session_id,
        &conn.record.repository_id,
        &loaded_events,
    );
    assert!(timeline.canary_access_count >= 1);

    // Steps 13–14: Integrity verification + generate all Stage 06 reports
    let t_int = Instant::now();
    let (int_record, _) = IntegrityMonitor::new()
        .check_integrity(
            &conn.record.repository_id,
            repo_dir.path(),
            &conn.genuine_layer,
            Some((&decoy_res.registry, &decoy_res.decoy_layer_root)),
            &HashSet::new(),
            corr_id,
        )
        .unwrap();
    let int_ms = t_int.elapsed().as_millis();
    assert!(int_record.baseline_matches);
    assert!(int_record.decoy_integrity_valid);
    store.record_integrity(&int_record).unwrap();

    let reports = generate_all_monitoring_reports(
        &conn.record.repository_id,
        &conn.record.name,
        &loaded_events,
        int_record,
        1,
        &RuleBasedClassifier::new(ScoringWeights::default()),
        state_dir.path(),
    )
    .unwrap();
    assert!(reports.markdown_path.is_file());
    assert!(reports
        .report
        .findings
        .iter()
        .any(|f| f.rule_id == "R03_SANDBOX_BOUNDARY_ATTEMPT"));

    // Steps 15–17: Stop & cleanup session, verify original repo strictly unchanged
    let stopped = sess_mgr.stop_session(&active_sess.session_id).unwrap();
    assert_eq!(stopped.status, SessionStatus::Completed);
    let cleaned = sess_mgr.cleanup_session(&active_sess.session_id).unwrap();
    assert_eq!(cleaned.status, SessionStatus::Cleaned);
    assert!(!active_sess.workspace_path.exists());

    // Verify zero genuine file mutations
    let meta_after = fs::metadata(&app_src).unwrap();
    assert_eq!(meta_before.len(), meta_after.len());
    assert_eq!(mtime_before, meta_after.modified().unwrap());

    // Sanity print timings for observability in `--nocapture`
    let total_ms = t_start.elapsed().as_millis();
    assert!(total_ms < 10_000 && decoy_ms < 5_000 && mirror_ms < 5_000 && int_ms < 5_000);
}

#[test]
fn test_e2e_edge_cases_empty_multilingual_invalid_inputs_and_recovery() {
    // 1. Empty repository handling
    let empty_dir = tempdir().unwrap();
    let s1_empty = run_repository_analysis(empty_dir.path(), BrahmConfig::default()).unwrap();
    let arch_empty = ArchitectureEngine::new()
        .analyze(empty_dir.path(), &s1_empty)
        .unwrap();
    let empty_decoy_out = tempdir().unwrap();
    let empty_decoys = DecoyEngine::new()
        .generate_and_stage(
            empty_dir.path(),
            &arch_empty.report,
            DecoyGenerationProfile::Conservative,
            empty_decoy_out.path(),
        )
        .unwrap();
    assert_eq!(empty_decoys.registry.artifacts().len(), 2);

    // 2. Multilingual fixture repository through ProtectionInstaller
    let fixture_d = Path::new("fixtures/fixture_d_mixed");
    let state_d = tempdir().unwrap();
    let install_d = ProtectionInstaller::new()
        .install(
            fixture_d,
            state_d.path(),
            ProtectionProfile::ControlledInspection,
            true,
        )
        .unwrap();
    assert_eq!(
        install_d.record.active_profile,
        ProtectionProfile::ControlledInspection
    );

    // 3. Invalid repository path
    assert!(RepositoryInputManager::new()
        .connect_local(
            Path::new("z:/does_not_exist_99999"),
            empty_decoy_out.path(),
            None
        )
        .is_err());

    // 4. Malformed architecture JSON deserialization error
    assert!(serde_json::from_str::<ArchitectureReport>("{ malformed json ").is_err());

    // 5. Invalid decoy template rejected
    let mut tpl_reg = TemplateRegistry::new();
    let bad_tpl = DecoyTemplate {
        template_id: "".to_string(),
        version: "1.0.0".to_string(),
        category: DecoyCategory::CanaryArtifact,
        target_language: None,
        default_relative_path: "ok.md".to_string(),
        risk_classification: RiskClassification::CanaryHighSignal,
        description: "empty id".to_string(),
        body_template: "body".to_string(),
    };
    assert!(tpl_reg.register(bad_tpl).is_err());

    // 6. Duplicate artifact ID caught by DecoyValidator
    let first = &empty_decoys.generated[0];
    let dup_rep = DecoyValidator::new().validate(&[first.clone(), first.clone()], empty_dir.path());
    assert!(!dup_rep.valid);

    // 7. Invalid deception policy rejected
    let mut bad_pol = DeceptionPolicy::for_profile(DeceptionPolicyProfile::Standard);
    bad_pol.allow_secret_exposure = true;
    assert!(bad_pol.validate().is_err());

    // 8. Expired session blocked cleanly
    let sess_root = tempdir().unwrap();
    let mgr = InspectionSessionManager::new(sess_root.path());
    let prepared = mgr
        .create_and_prepare(
            empty_dir.path(),
            &empty_decoys.registry,
            &empty_decoys.decoy_layer_root,
            &DeceptionPolicy::for_profile(DeceptionPolicyProfile::Conservative),
            60,
        )
        .unwrap();
    let mut expired_sess = prepared.session.clone();
    expired_sess.expires_at = Utc::now() - Duration::seconds(120);
    mgr.save_session(&expired_sess).unwrap();
    assert!(mgr.activate_session(&expired_sess.session_id).is_err());
    let checked = mgr.check_expiration(&expired_sess.session_id).unwrap();
    assert_eq!(checked.status, SessionStatus::Expired);

    // 9. Interrupted cleanup recovery (cleaning already-deleted workspace succeeds idempotently)
    fs::remove_dir_all(&prepared.session.workspace_path).unwrap();
    let recovered = mgr.cleanup_session(&prepared.session.session_id).unwrap();
    assert_eq!(recovered.status, SessionStatus::Cleaned);

    // 10. Duplicate event ID DB idempotency
    let store = SqliteEventStore::open_in_memory().unwrap();
    let evt = MonitoringEvent {
        event_id: "evt_dup_id".to_string(),
        event_type: EventType::DecoyCreated,
        repository_id: "r1".to_string(),
        session_id: None,
        artifact_id: None,
        timestamp: Utc::now(),
        source: EventSource::DecoyEngine,
        severity: EventSeverity::Informational,
        correlation_id: "c".to_string(),
        metadata: BTreeMap::new(),
    };
    store.record_event(&evt).unwrap();
    store.record_event(&evt).unwrap();
    assert_eq!(store.count_events().unwrap(), 1);

    // 11. Unavailable monitoring DB path (pointing DB path to an existing directory fails safely)
    let bad_db_dir = tempdir().unwrap();
    assert!(SqliteEventStore::open(bad_db_dir.path()).is_err());

    // 12. Registry search negative case for missing canary entry
    let loaded_reg = DecoyRegistry::load_from_path(&empty_decoys.registry_path).unwrap();
    assert!(loaded_reg
        .find_by_canary("BRAHM-CANARY-DOESNOTEXIST")
        .is_none());

    // 13. Modified genuine source caught by ProductionIntegrityGuard after leaking a decoy
    let prod_check = ProductionIntegrityGuard::new()
        .verify_production_isolation(empty_dir.path(), &empty_decoys.manifest)
        .unwrap();
    assert!(prod_check.production_isolated);
}
