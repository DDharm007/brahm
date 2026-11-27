//! Milestone 07 Stage 02 Comprehensive Validation, Accuracy & Security Suite.

use brahm::analysis::run_repository_analysis;
use brahm::architecture::ArchitectureEngine;
use brahm::cli::OutputFormat;
use brahm::config::BrahmConfig;
use brahm::reporting::dispatch_architecture_reports;
use brahm::structure::types::HttpMethod;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_01_fixture_d_accuracy_and_recall() {
    let fixture_path = Path::new("fixtures/fixture_d_mixed");
    let stage1 = run_repository_analysis(fixture_path, BrahmConfig::default()).unwrap();
    let bundle = ArchitectureEngine::new()
        .analyze(fixture_path, &stage1)
        .expect("Stage 02 analysis must succeed on Fixture D");

    // 1. Verify syntax error recovery without aborting pipeline
    assert!(
        bundle.report.metrics.files_with_syntax_errors >= 1,
        "Broken syntax file must be counted under syntax error recovery"
    );
    assert!(bundle
        .report
        .warnings
        .iter()
        .any(|w| w.contains("syntax_error.rs")));

    // 2. Symbol Recall Check against known expected ground-truth symbols
    let expected_symbols = [
        "health_check",
        "create_audit",
        "AuditLog",
        "gatewayStatus",
        "BillingController",
        "getBillingSummary",
        "fnA",
        "fnB",
        "Invoice",
        "process_batch",
        "valid_before_error",
        "compute_checksum",
    ];
    let extracted_names: Vec<&str> = bundle
        .symbol_index
        .all()
        .iter()
        .map(|s| s.name.as_str())
        .collect();

    let mut matched_count = 0;
    for exp in &expected_symbols {
        if extracted_names.contains(exp) {
            matched_count += 1;
        }
    }
    let recall = matched_count as f64 / expected_symbols.len() as f64;
    assert!(
        recall >= 0.90,
        "Symbol extraction recall must be >= 90%, got {:.1}% (extracted: {:?})",
        recall * 100.0,
        extracted_names
    );

    // 3. API Routes Recall across 3 backend languages (FastAPI, Gin, Spring Boot)
    let routes = &bundle.report.api_routes;
    assert!(
        routes
            .iter()
            .any(|r| r.method == HttpMethod::Get && r.path_pattern == "/api/v1/health"),
        "Missing FastAPI GET /api/v1/health"
    );
    assert!(
        routes
            .iter()
            .any(|r| r.method == HttpMethod::Post && r.path_pattern == "/api/v1/audit"),
        "Missing FastAPI POST /api/v1/audit"
    );
    assert!(
        routes
            .iter()
            .any(|r| r.method == HttpMethod::Get && r.path_pattern == "/gateway/status"),
        "Missing Go Gin GET /gateway/status"
    );
    assert!(
        routes
            .iter()
            .any(|r| r.method == HttpMethod::Get && r.path_pattern == "/api/v1/billing/summary"),
        "Missing Spring Boot GET /api/v1/billing/summary"
    );

    // 4. ORM & Data Models Recall
    let model_names: Vec<&str> = bundle
        .report
        .data_models
        .iter()
        .map(|m| m.name.as_str())
        .collect();
    assert!(
        model_names.contains(&"AuditLog"),
        "Missing SQLAlchemy model AuditLog"
    );
    assert!(
        model_names.contains(&"Invoice"),
        "Missing TypeORM model Invoice"
    );

    // 5. Intentional Circular Dependency Verification (a.ts <-> b.ts)
    assert!(
        bundle.report.metrics.circular_dependency_count >= 1,
        "Must detect circular dependency between a.ts and b.ts"
    );
    assert!(!bundle.report.metrics.graph_is_dag);
}

#[test]
fn test_02_strict_readonly_stage02() {
    let repo_dir = tempdir().unwrap();
    let out_dir = tempdir().unwrap();

    let rs_file = repo_dir.path().join("lib.rs");
    fs::write(&rs_file, "pub fn verify_immutable() -> i32 { 1 }\n").unwrap();

    let meta_before = fs::metadata(&rs_file).unwrap();
    let mtime_before = meta_before.modified().unwrap();
    let len_before = meta_before.len();

    let stage1 = run_repository_analysis(repo_dir.path(), BrahmConfig::default()).unwrap();
    let bundle = ArchitectureEngine::new()
        .analyze(repo_dir.path(), &stage1)
        .unwrap();
    dispatch_architecture_reports(&bundle, OutputFormat::All, Some(out_dir.path())).unwrap();

    let meta_after = fs::metadata(&rs_file).unwrap();
    assert_eq!(meta_after.len(), len_before);
    assert_eq!(meta_after.modified().unwrap(), mtime_before);

    // Ensure no BRAHM artifacts were created inside the target repo directory
    let entries: Vec<_> = fs::read_dir(repo_dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].to_str().unwrap(), "lib.rs");
}

#[test]
fn test_03_secret_protection_stage02() {
    let repo_dir = tempdir().unwrap();
    let out_dir = tempdir().unwrap();

    fs::write(
        repo_dir.path().join(".env"),
        "SUPER_SECRET_DB_PASSWORD=do_not_leak_in_brahm_reports_99999\n",
    )
    .unwrap();
    fs::write(
        repo_dir.path().join("private_key.pem"),
        "-----BEGIN PRIVATE KEY-----\nSECRET_KEY_MATERIAL_DO_NOT_LEAK\n-----END PRIVATE KEY-----\n",
    )
    .unwrap();
    fs::write(repo_dir.path().join("main.rs"), "pub fn main() {}\n").unwrap();

    let stage1 = run_repository_analysis(repo_dir.path(), BrahmConfig::default()).unwrap();
    let bundle = ArchitectureEngine::new()
        .analyze(repo_dir.path(), &stage1)
        .unwrap();
    dispatch_architecture_reports(&bundle, OutputFormat::All, Some(out_dir.path())).unwrap();

    for artifact in [
        "BRAHM_ARCHITECTURE.json",
        "BRAHM_ARCHITECTURE.md",
        "BRAHM_DEPENDENCY_GRAPH.json",
        "BRAHM_SYMBOL_INDEX.json",
    ] {
        let content = fs::read_to_string(out_dir.path().join(artifact)).unwrap();
        assert!(
            !content.contains("do_not_leak_in_brahm_reports_99999"),
            "Secret leaked into {}",
            artifact
        );
        assert!(
            !content.contains("SUPER_SECRET_DB_PASSWORD"),
            "Secret key leaked into {}",
            artifact
        );
        assert!(
            !content.contains("SECRET_KEY_MATERIAL_DO_NOT_LEAK"),
            "PEM key leaked into {}",
            artifact
        );
    }
}

#[test]
fn test_04_deterministic_execution() {
    let fixture_path = Path::new("fixtures/fixture_d_mixed");
    let stage1 = run_repository_analysis(fixture_path, BrahmConfig::default()).unwrap();
    let engine = ArchitectureEngine::new();

    let run1 = engine.analyze(fixture_path, &stage1).unwrap();
    let run2 = engine.analyze(fixture_path, &stage1).unwrap();

    let mut ids1: Vec<String> = run1
        .symbol_index
        .all()
        .iter()
        .map(|s| s.id.clone())
        .collect();
    let mut ids2: Vec<String> = run2
        .symbol_index
        .all()
        .iter()
        .map(|s| s.id.clone())
        .collect();
    ids1.sort();
    ids2.sort();

    assert_eq!(ids1, ids2, "Symbol IDs must be strictly deterministic");
    assert_eq!(run1.report.metrics, run2.report.metrics);
    assert_eq!(run1.report.pattern, run2.report.pattern);
    assert_eq!(run1.graph.node_count(), run2.graph.node_count());
    assert_eq!(run1.graph.edge_count(), run2.graph.edge_count());
}

#[test]
fn test_05_cross_fixture_stage02_performance() {
    for fixture in [
        "fixtures/fixture_a_react",
        "fixtures/fixture_b_fullstack",
        "fixtures/fixture_c_multilingual",
        "fixtures/fixture_d_mixed",
    ] {
        let p = Path::new(fixture);
        let stage1 = run_repository_analysis(p, BrahmConfig::default()).unwrap();
        let bundle = ArchitectureEngine::new()
            .analyze(p, &stage1)
            .unwrap_or_else(|e| panic!("Failed Stage 02 on {}: {}", fixture, e));

        assert!(
            bundle.report.analysis_duration_ms < 2000,
            "Stage 02 took unexpectedly long on {}: {} ms",
            fixture,
            bundle.report.analysis_duration_ms
        );
        assert!(bundle.graph.node_count() > 0);
    }
}
