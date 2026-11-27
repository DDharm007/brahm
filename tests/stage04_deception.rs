//! Stage 04 Repository Deception Engine Comprehensive Acceptance & Isolation Test Suite.

use brahm::analysis::run_repository_analysis;
use brahm::architecture::ArchitectureEngine;
use brahm::cli::{Cli, Commands, DeceptionCommands, PolicyCommands, SessionCommands};
use brahm::config::BrahmConfig;
use brahm::deception::decoy::{DecoyCategory, DecoyEngine, DecoyGenerationProfile};
use brahm::deception::{
    ControlledInspectionEnvironment, DeceptionPolicy, DeceptionPolicyEngine,
    DeceptionPolicyProfile, InspectionManifest, InspectionSession, InspectionSessionManager,
    ProductionIntegrityGuard, ProductionIntegrityReport, SessionStatus,
};
use clap::Parser;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_01_deception_policy_engine_and_security_enforcement() {
    let engine = DeceptionPolicyEngine::new();
    let presets = engine.list_presets();
    assert_eq!(presets.len(), 4);
    assert!(presets
        .iter()
        .any(|p| p.profile == DeceptionPolicyProfile::BlindShield && p.redact_implementation));

    // Unsafe policy overrides must fail validation immediately
    let mut bad_secret_pol = DeceptionPolicy::for_profile(DeceptionPolicyProfile::Standard);
    bad_secret_pol.allow_secret_exposure = true;
    assert!(bad_secret_pol.validate().is_err());

    let mut bad_prod_pol = DeceptionPolicy::for_profile(DeceptionPolicyProfile::Research);
    bad_prod_pol.allow_production_deployment = true;
    assert!(bad_prod_pol.validate().is_err());

    // Evaluate against generated decoys across profiles
    let fixture_b = Path::new("fixtures/fixture_b_fullstack");
    let stage1 = run_repository_analysis(fixture_b, BrahmConfig::default()).unwrap();
    let arch = ArchitectureEngine::new()
        .analyze(fixture_b, &stage1)
        .unwrap();

    let stage_dir = tempdir().unwrap();
    let decoy_res = DecoyEngine::new()
        .generate_and_stage(
            fixture_b,
            &arch.report,
            DecoyGenerationProfile::Research,
            stage_dir.path(),
        )
        .unwrap();

    // Conservative allows only docs + canary
    let cons_eval = engine
        .evaluate(
            &DeceptionPolicy::for_profile(DeceptionPolicyProfile::Conservative),
            decoy_res.registry.artifacts(),
        )
        .unwrap();
    assert!(!cons_eval.included_ids.is_empty());
    assert!(cons_eval
        .decisions
        .iter()
        .filter(|d| d.included)
        .all(|d| matches!(
            d.category,
            DecoyCategory::SyntheticDocumentation | DecoyCategory::CanaryArtifact
        )));

    // Research allows broader mixture including synthetic dependencies
    let res_eval = engine
        .evaluate(
            &DeceptionPolicy::for_profile(DeceptionPolicyProfile::Research),
            decoy_res.registry.artifacts(),
        )
        .unwrap();
    assert!(res_eval.included_ids.len() > cons_eval.included_ids.len());
}

#[test]
fn test_02_mirror_layer_isolation_and_secret_stripping() {
    let repo_dir = tempdir().unwrap();
    fs::write(
        repo_dir.path().join("app.js"),
        "const express = require('express');\n",
    )
    .unwrap();
    fs::write(
        repo_dir.path().join(".env"),
        "SECRET_API_KEY=must_never_enter_inspection_mirror\n",
    )
    .unwrap();
    fs::write(
        repo_dir.path().join("server.key"),
        "PRIVATE_KEY_MUST_NOT_MIRROR\n",
    )
    .unwrap();

    let stage1 = run_repository_analysis(repo_dir.path(), BrahmConfig::default()).unwrap();
    let arch = ArchitectureEngine::new()
        .analyze(repo_dir.path(), &stage1)
        .unwrap();

    let decoy_dir = tempdir().unwrap();
    let decoy_res = DecoyEngine::new()
        .generate_and_stage(
            repo_dir.path(),
            &arch.report,
            DecoyGenerationProfile::Standard,
            decoy_dir.path(),
        )
        .unwrap();

    let sessions_dir = tempdir().unwrap();
    let mgr = InspectionSessionManager::new(sessions_dir.path());
    let policy = DeceptionPolicy::for_profile(DeceptionPolicyProfile::Standard);

    let outcome = mgr
        .create_and_prepare(
            repo_dir.path(),
            &decoy_res.registry,
            &decoy_res.decoy_layer_root,
            &policy,
            3600,
        )
        .expect("Mirror session creation must succeed");

    // 1. Original repo has NO decoy artifacts or extra files
    assert!(!repo_dir.path().join("canary").exists());
    assert!(!repo_dir.path().join("src").exists());

    // 2. Inspection mirror workspace MUST have genuine app.js and synthetic decoys
    assert!(outcome.session.workspace_path.join("app.js").is_file());
    assert!(!outcome.session.registered_artifacts.is_empty());

    // 3. Inspection mirror workspace MUST NOT contain .env or server.key
    assert!(!outcome.session.workspace_path.join(".env").exists());
    assert!(!outcome.session.workspace_path.join("server.key").exists());
    assert_eq!(outcome.mirror.manifest.stripped_secret_count, 2);

    // 4. All 4 Stage 04 deliverable JSON artifacts must exist & parse cleanly
    for artifact in [
        "BRAHM_DECEPTION_POLICY.json",
        "BRAHM_INSPECTION_MANIFEST.json",
        "BRAHM_SESSION_REPORT.json",
        "BRAHM_INTEGRITY_REPORT.json",
    ] {
        assert!(
            outcome.session.state_dir.join(artifact).is_file(),
            "Missing Stage 04 deliverable {}",
            artifact
        );
    }

    let _: DeceptionPolicy = serde_json::from_str(
        &fs::read_to_string(
            outcome
                .session
                .state_dir
                .join("BRAHM_DECEPTION_POLICY.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let _: InspectionManifest = serde_json::from_str(
        &fs::read_to_string(
            outcome
                .session
                .state_dir
                .join("BRAHM_INSPECTION_MANIFEST.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let _: InspectionSession = serde_json::from_str(
        &fs::read_to_string(outcome.session.state_dir.join("BRAHM_SESSION_REPORT.json")).unwrap(),
    )
    .unwrap();
    let _: ProductionIntegrityReport = serde_json::from_str(
        &fs::read_to_string(
            outcome
                .session
                .state_dir
                .join("BRAHM_INTEGRITY_REPORT.json"),
        )
        .unwrap(),
    )
    .unwrap();
}

#[test]
fn test_03_production_integrity_guard_blocks_contamination() {
    let repo_dir = tempdir().unwrap();
    fs::write(repo_dir.path().join("lib.rs"), "pub fn prod() {}\n").unwrap();
    fs::write(
        repo_dir.path().join("Cargo.toml"),
        "[package]\nname=\"app\"\nversion=\"0.1.0\"\n",
    )
    .unwrap();

    let stage1 = run_repository_analysis(repo_dir.path(), BrahmConfig::default()).unwrap();
    let arch = ArchitectureEngine::new()
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
    let clean_report = guard
        .verify_production_isolation(repo_dir.path(), &decoy_res.manifest)
        .unwrap();
    assert!(clean_report.production_isolated);

    // Simulate accidental leakage of a decoy file directly into the genuine repo
    let leaked_path = repo_dir
        .path()
        .join(&decoy_res.manifest.entries[0].relative_path);
    fs::create_dir_all(leaked_path.parent().unwrap()).unwrap();
    fs::copy(
        decoy_res
            .decoy_layer_root
            .join(&decoy_res.manifest.entries[0].relative_path),
        &leaked_path,
    )
    .unwrap();

    let dirty_report = guard
        .verify_production_isolation(repo_dir.path(), &decoy_res.manifest)
        .unwrap();
    assert!(!dirty_report.production_isolated);
    assert!(dirty_report.failures_count >= 1);

    // Session creation MUST block if genuine repo fails integrity check
    let sessions_dir = tempdir().unwrap();
    let mgr = InspectionSessionManager::new(sessions_dir.path());
    assert!(mgr
        .create_and_prepare(
            repo_dir.path(),
            &decoy_res.registry,
            &decoy_res.decoy_layer_root,
            &DeceptionPolicy::for_profile(DeceptionPolicyProfile::Conservative),
            3600,
        )
        .is_err());
}

#[test]
fn test_04_session_lifecycle_concurrency_and_safe_cleanup() {
    let genuine = Path::new("fixtures/fixture_a_react");
    let stage1 = run_repository_analysis(genuine, BrahmConfig::default()).unwrap();
    let arch = ArchitectureEngine::new().analyze(genuine, &stage1).unwrap();

    let decoy_dir = tempdir().unwrap();
    let decoy_res = DecoyEngine::new()
        .generate_and_stage(
            genuine,
            &arch.report,
            DecoyGenerationProfile::Standard,
            decoy_dir.path(),
        )
        .unwrap();

    let sessions_dir = tempdir().unwrap();
    let mgr = InspectionSessionManager::new(sessions_dir.path());
    let policy = DeceptionPolicy::for_profile(DeceptionPolicyProfile::Standard);

    // Create two concurrent isolated sessions
    let o1 = mgr
        .create_and_prepare(
            genuine,
            &decoy_res.registry,
            &decoy_res.decoy_layer_root,
            &policy,
            3600,
        )
        .unwrap();
    let o2 = mgr
        .create_and_prepare(
            genuine,
            &decoy_res.registry,
            &decoy_res.decoy_layer_root,
            &policy,
            3600,
        )
        .unwrap();

    assert_ne!(o1.session.session_id, o2.session.session_id);
    assert_ne!(o1.session.workspace_path, o2.session.workspace_path);

    // Lifecycle transitions on session 1
    let active = mgr.activate_session(&o1.session.session_id).unwrap();
    assert_eq!(active.status, SessionStatus::Active);

    let suspended = mgr.suspend_session(&o1.session.session_id).unwrap();
    assert_eq!(suspended.status, SessionStatus::Suspended);

    let stopped = mgr.stop_session(&o1.session.session_id).unwrap();
    assert_eq!(stopped.status, SessionStatus::Completed);

    // Cleanup session 1 MUST preserve session 2 workspace cleanly
    let cleaned = mgr.cleanup_session(&o1.session.session_id).unwrap();
    assert_eq!(cleaned.status, SessionStatus::Cleaned);
    assert!(!o1.session.workspace_path.exists());
    assert!(o2.session.workspace_path.is_dir());

    let listed = mgr.list_sessions().unwrap();
    assert_eq!(listed.len(), 2);
}

#[test]
fn test_05_controlled_sandbox_isolation_and_canary_telemetry() {
    let genuine = Path::new("fixtures/fixture_b_fullstack");
    let stage1 = run_repository_analysis(genuine, BrahmConfig::default()).unwrap();
    let arch = ArchitectureEngine::new().analyze(genuine, &stage1).unwrap();

    let decoy_dir = tempdir().unwrap();
    let decoy_res = DecoyEngine::new()
        .generate_and_stage(
            genuine,
            &arch.report,
            DecoyGenerationProfile::Standard,
            decoy_dir.path(),
        )
        .unwrap();

    let sessions_dir = tempdir().unwrap();
    let mgr = InspectionSessionManager::new(sessions_dir.path());
    let policy = DeceptionPolicy::for_profile(DeceptionPolicyProfile::Standard);
    let outcome = mgr
        .create_and_prepare(
            genuine,
            &decoy_res.registry,
            &decoy_res.decoy_layer_root,
            &policy,
            3600,
        )
        .unwrap();

    let active_sess = mgr.activate_session(&outcome.session.session_id).unwrap();
    let mut sandbox =
        ControlledInspectionEnvironment::open(active_sess, decoy_res.registry).unwrap();
    assert!(!sandbox.outbound_network_allowed());

    // Directory traversal must be rejected and logged
    assert!(sandbox
        .read_workspace_file("../outside_secret.txt")
        .is_err());

    // Reading canary anchor inside workspace must succeed AND log canary token match
    let canary_rel = "canary/BRAHM_CANARY_ANCHOR.md";
    let content = sandbox.read_workspace_file(canary_rel).unwrap();
    assert!(content.contains("BRAHM-CANARY-"));

    let log = sandbox.access_log();
    assert_eq!(log.len(), 2);
    assert!(!log[0].allowed);
    assert!(log[1].allowed);
    assert!(log[1].matched_canary_token.is_some());
}

#[test]
fn test_06_cli_deception_commands_parsing() {
    let c1 = Cli::try_parse_from([
        "brahm",
        "deception",
        "prepare",
        "./repo",
        "--profile",
        "conservative",
    ])
    .unwrap();
    assert!(matches!(
        c1.command,
        Commands::Deception {
            action: DeceptionCommands::Prepare { .. }
        }
    ));

    let c2 = Cli::try_parse_from(["brahm", "deception", "policy", "list"]).unwrap();
    assert!(matches!(
        c2.command,
        Commands::Deception {
            action: DeceptionCommands::Policy {
                action: PolicyCommands::List
            }
        }
    ));

    let c3 = Cli::try_parse_from(["brahm", "deception", "session", "inspect", "sess_123"]).unwrap();
    assert!(matches!(
        c3.command,
        Commands::Deception {
            action: DeceptionCommands::Session {
                action: SessionCommands::Inspect { .. }
            }
        }
    ));
}
