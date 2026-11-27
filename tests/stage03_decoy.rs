//! Stage 03 Decoy Generation Engine Comprehensive Test Suite.

use brahm::analysis::run_repository_analysis;
use brahm::architecture::ArchitectureEngine;
use brahm::cli::{Cli, Commands, DecoyCommands};
use brahm::config::BrahmConfig;
use brahm::deception::decoy::{
    DecoyCategory, DecoyEngine, DecoyGenerationProfile, DecoyGenerator, DecoyInstallState,
    DecoyManifest, DecoyRegistry, DecoyTemplate, DecoyValidator, GeneratedDecoy,
    RiskClassification, TemplateRegistry,
};
use clap::Parser;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_01_template_registry_and_security_validation() {
    let mut reg = TemplateRegistry::with_builtin_templates();
    assert!(reg.list().len() >= 10);

    // Unsafe path traversal must be rejected
    let bad_path_tpl = DecoyTemplate {
        template_id: "bad_path".to_string(),
        version: "1.0.0".to_string(),
        category: DecoyCategory::SyntheticSource,
        target_language: None,
        default_relative_path: "../outside.txt".to_string(),
        risk_classification: RiskClassification::InertLow,
        description: "unsafe".to_string(),
        body_template: "ok".to_string(),
    };
    assert!(reg.register(bad_path_tpl).is_err());

    // Credential pattern inside template must be rejected
    let bad_secret_tpl = DecoyTemplate {
        template_id: "bad_secret".to_string(),
        version: "1.0.0".to_string(),
        category: DecoyCategory::CanaryArtifact,
        target_language: None,
        default_relative_path: "canary/test.txt".to_string(),
        risk_classification: RiskClassification::CanaryHighSignal,
        description: "unsafe secret".to_string(),
        body_template: "AWS_ACCESS_KEY=AKIA1234567890123456".to_string(),
    };
    assert!(reg.register(bad_secret_tpl).is_err());
}

#[test]
fn test_02_architecture_aware_generation_across_profiles() {
    let fixture_b = Path::new("fixtures/fixture_b_fullstack");
    let stage1 = run_repository_analysis(fixture_b, BrahmConfig::default()).unwrap();
    let arch = ArchitectureEngine::new()
        .analyze(fixture_b, &stage1)
        .unwrap();

    let generator = DecoyGenerator::new();

    // Disabled -> 0 artifacts
    let disabled = generator
        .generate_for_architecture(fixture_b, &arch.report, DecoyGenerationProfile::Disabled)
        .unwrap();
    assert!(disabled.is_empty());

    // Conservative -> minimal docs + canary
    let conservative = generator
        .generate_for_architecture(
            fixture_b,
            &arch.report,
            DecoyGenerationProfile::Conservative,
        )
        .unwrap();
    assert_eq!(conservative.len(), 2);

    // Standard -> includes JS/TS + API configs + SQL schema, NOT Go or Rust (absent from Fixture B)
    let standard = generator
        .generate_for_architecture(fixture_b, &arch.report, DecoyGenerationProfile::Standard)
        .unwrap();
    let paths: Vec<&str> = standard
        .iter()
        .map(|d| d.artifact.synthetic_relative_path.as_str())
        .collect();
    assert!(paths.contains(&"src/syntheticRouterStub.js"));
    assert!(paths.contains(&"src/syntheticTelemetryBridge.ts"));
    assert!(paths.contains(&"config/api.decoy.json"));
    assert!(!paths.contains(&"internal/synthetic_trace.go"));
    assert!(!paths.contains(&"src/synthetic_dispatch_adapter.rs"));

    // Research -> expanded suite including secondary canary token
    let research = generator
        .generate_for_architecture(fixture_b, &arch.report, DecoyGenerationProfile::Research)
        .unwrap();
    assert!(research.len() > standard.len());
    assert!(research
        .iter()
        .any(|d| d.artifact.template_id == "tpl_canary_trace_json"));
}

#[test]
fn test_03_decoy_validation_rejects_secrets_duplicates_and_manifests() {
    let repo = tempdir().unwrap();
    let generator = DecoyGenerator::new();
    let validator = DecoyValidator::new();

    let fixture_a = Path::new("fixtures/fixture_a_react");
    let s1 = run_repository_analysis(fixture_a, BrahmConfig::default()).unwrap();
    let arch = ArchitectureEngine::new().analyze(fixture_a, &s1).unwrap();
    let mut decoys = generator
        .generate_for_architecture(
            fixture_a,
            &arch.report,
            DecoyGenerationProfile::Conservative,
        )
        .unwrap();

    // Clean pass
    let ok_rep = validator.validate(&decoys, repo.path());
    assert!(ok_rep.valid);

    // 1. Secret leak violation
    let mut leaked = decoys[0].clone();
    leaked.content = "stripe_key = sk_live_00000000000000000".to_string();
    let bad_secret = validator.validate(&[leaked], repo.path());
    assert!(!bad_secret.valid);

    // 2. Production manifest overlap violation
    let mut manifest_collision = decoys[0].clone();
    manifest_collision.artifact.synthetic_relative_path = "Cargo.toml".to_string();
    let bad_manifest = validator.validate(&[manifest_collision], repo.path());
    assert!(!bad_manifest.valid);

    // 3. Duplicate ID violation
    let dup = decoys[0].clone();
    decoys.push(dup);
    let bad_dup = validator.validate(&decoys, repo.path());
    assert!(!bad_dup.valid);

    // 4. Executable / unsafe code pattern violation
    let mut exec_bad = GeneratedDecoy {
        artifact: decoys[0].artifact.clone(),
        content: "unsafe { std::process::Command::new(\"sh\"); }".to_string(),
    };
    exec_bad.artifact.synthetic_relative_path = "src/bad.rs".to_string();
    let bad_exec = validator.validate(&[exec_bad], repo.path());
    assert!(!bad_exec.valid);
}

#[test]
fn test_04_registry_persistence_search_deactivation_and_integrity() {
    let genuine = Path::new("fixtures/fixture_d_mixed");
    let stage1 = run_repository_analysis(genuine, BrahmConfig::default()).unwrap();
    let arch = ArchitectureEngine::new().analyze(genuine, &stage1).unwrap();

    let stage_out = tempdir().unwrap();
    let result = DecoyEngine::new()
        .generate_and_stage(
            genuine,
            &arch.report,
            DecoyGenerationProfile::Standard,
            stage_out.path(),
        )
        .expect("Staging must succeed");

    assert!(result.registry_path.is_file());
    assert!(result.manifest_path.is_file());
    assert!(result.report_path.is_file());

    // Load manifest & verify
    let manifest: DecoyManifest =
        serde_json::from_str(&fs::read_to_string(&result.manifest_path).unwrap()).unwrap();
    assert!(manifest.production_build_excluded);
    assert_eq!(manifest.total_artifacts, result.registry.artifacts().len());

    // Load persisted registry
    let mut loaded = DecoyRegistry::load_from_path(&result.registry_path).unwrap();
    let checks = loaded.verify_staged_integrity(&result.decoy_layer_root);
    assert!(!checks.is_empty());
    assert!(checks.iter().all(|(_, ok, _)| *ok));

    // Search & canary lookup
    let canaries = loaded.search(None, Some(DecoyCategory::CanaryArtifact));
    assert!(!canaries.is_empty());
    let first_id = canaries[0].artifact_id.clone();
    let first_token = canaries[0].canary_token.clone().unwrap();
    assert!(loaded.find_by_canary(&first_token).is_some());

    // Tamper with staged file -> integrity verification catches mismatch
    let first_path = result
        .decoy_layer_root
        .join(&canaries[0].synthetic_relative_path);
    fs::write(&first_path, "tampered content").unwrap();
    let tampered_checks = loaded.verify_staged_integrity(&result.decoy_layer_root);
    assert!(tampered_checks
        .iter()
        .any(|(id, ok, _)| id == &first_id && !*ok));

    // Deactivate artifact
    loaded.deactivate(&first_id).unwrap();
    assert_eq!(
        loaded.get(&first_id).unwrap().install_state,
        DecoyInstallState::Inactive
    );
}

#[test]
fn test_05_safe_ownership_gated_cleanup_and_dry_run() {
    let genuine = Path::new("fixtures/fixture_a_react");
    let stage1 = run_repository_analysis(genuine, BrahmConfig::default()).unwrap();
    let arch = ArchitectureEngine::new().analyze(genuine, &stage1).unwrap();

    let stage_out = tempdir().unwrap();
    let mut res = DecoyEngine::new()
        .generate_and_stage(
            genuine,
            &arch.report,
            DecoyGenerationProfile::Conservative,
            stage_out.path(),
        )
        .unwrap();

    // Plant an unrelated user file inside staging root -> MUST NEVER be deleted
    let user_file = res.decoy_layer_root.join("user_notes.txt");
    fs::write(&user_file, "important manual notes").unwrap();

    // Dry-run leaves all files intact
    let dry = res
        .registry
        .cleanup_owned_artifacts(&res.decoy_layer_root, true)
        .unwrap();
    assert_eq!(dry.removed_paths.len(), 2);
    assert!(res
        .decoy_layer_root
        .join(&res.registry.artifacts()[0].synthetic_relative_path)
        .exists());

    // Real cleanup removes ONLY verified BRAHM-owned decoys and preserves user_notes.txt
    let exec = res
        .registry
        .cleanup_owned_artifacts(&res.decoy_layer_root, false)
        .unwrap();
    assert_eq!(exec.removed_paths.len(), 2);
    assert!(user_file.is_file());
    assert!(!res
        .decoy_layer_root
        .join(&res.registry.artifacts()[0].synthetic_relative_path)
        .exists());
}

#[test]
fn test_06_cli_decoy_commands_parsing() {
    let cli_gen = Cli::try_parse_from([
        "brahm",
        "decoy",
        "generate",
        "./repo",
        "--profile",
        "research",
    ])
    .unwrap();
    match cli_gen.command {
        Commands::Decoy {
            action: DecoyCommands::Generate { profile, .. },
        } => {
            assert_eq!(format!("{:?}", profile), "Research");
        }
        _ => panic!("Expected decoy generate"),
    }

    let cli_rm = Cli::try_parse_from(["brahm", "decoy", "remove", "--dry-run"]).unwrap();
    match cli_rm.command {
        Commands::Decoy {
            action: DecoyCommands::Remove { dry_run, .. },
        } => {
            assert!(dry_run);
        }
        _ => panic!("Expected decoy remove"),
    }
}
