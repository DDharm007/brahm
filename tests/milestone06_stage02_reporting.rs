//! Integration test suite for Milestone 06: Architecture Intelligence & Reporting.

use brahm::analysis::run_repository_analysis;
use brahm::architecture::{ArchitectureEngine, ArchitectureReport, SymbolIndexExport};
use brahm::cli::{Cli, Commands, OutputFormat};
use brahm::config::BrahmConfig;
use brahm::graph::GraphExport;
use brahm::reporting::dispatch_architecture_reports;
use clap::Parser;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_01_architecture_engine_on_fixture_b_fullstack() {
    let fixture_path = Path::new("fixtures/fixture_b_fullstack");
    let stage1 = run_repository_analysis(fixture_path, BrahmConfig::default()).unwrap();

    let engine = ArchitectureEngine::new();
    let bundle = engine
        .analyze(fixture_path, &stage1)
        .expect("Stage 02 analysis must succeed");

    // Pattern inference
    assert_eq!(
        bundle.report.pattern.primary_pattern,
        "Full-Stack Web Application"
    );
    assert_eq!(bundle.report.pattern.confidence, "High");

    // Landmarks & persistence
    assert!(bundle.report.metrics.api_route_count >= 1);
    assert!(bundle
        .report
        .data_models
        .iter()
        .any(|m| m.name == "001_create_users.sql"));

    // Clean relative path validation (no Windows \\?\ UNC prefixes)
    for sym in bundle.symbol_index.all() {
        let p = sym.file_path.to_string_lossy();
        assert!(
            !p.contains("\\\\?\\"),
            "UNC prefix leaked into symbol path: {}",
            p
        );
    }

    assert!(!bundle.report.confidence_breakdown.detected_facts.is_empty());
    assert!(!bundle
        .report
        .confidence_breakdown
        .inferred_relationships
        .is_empty());
}

#[test]
fn test_02_architecture_engine_on_fixture_c_multilingual() {
    let fixture_path = Path::new("fixtures/fixture_c_multilingual");
    let stage1 = run_repository_analysis(fixture_path, BrahmConfig::default()).unwrap();

    let engine = ArchitectureEngine::new();
    let bundle = engine
        .analyze(fixture_path, &stage1)
        .expect("Stage 02 analysis must succeed on multilingual repo");

    assert!(bundle.report.metrics.files_parsed_ast >= 5);
    assert!(bundle
        .report
        .pattern
        .languages_parsed
        .contains(&"Rust".to_string()));
    assert!(bundle
        .report
        .pattern
        .languages_parsed
        .contains(&"Python".to_string()));
    assert!(bundle
        .report
        .pattern
        .languages_parsed
        .contains(&"Go".to_string()));

    // Check extracted symbols exist across multiple languages
    assert!(bundle.report.metrics.total_symbols >= 3);
    assert!(bundle.report.metrics.graph_nodes > 0);
}

#[test]
fn test_03_dispatch_all_stage02_artifacts() {
    let fixture_path = Path::new("fixtures/fixture_b_fullstack");
    let stage1 = run_repository_analysis(fixture_path, BrahmConfig::default()).unwrap();
    let bundle = ArchitectureEngine::new()
        .analyze(fixture_path, &stage1)
        .unwrap();

    let out_dir = tempdir().unwrap();
    dispatch_architecture_reports(&bundle, OutputFormat::All, Some(out_dir.path()))
        .expect("Report dispatch must succeed");

    // Verify all 4 Stage 02 artifacts exist and parse cleanly
    let arch_json_path = out_dir.path().join("BRAHM_ARCHITECTURE.json");
    let arch_md_path = out_dir.path().join("BRAHM_ARCHITECTURE.md");
    let graph_json_path = out_dir.path().join("BRAHM_DEPENDENCY_GRAPH.json");
    let sym_json_path = out_dir.path().join("BRAHM_SYMBOL_INDEX.json");

    assert!(arch_json_path.is_file());
    assert!(arch_md_path.is_file());
    assert!(graph_json_path.is_file());
    assert!(sym_json_path.is_file());

    let arch_json: ArchitectureReport =
        serde_json::from_str(&fs::read_to_string(&arch_json_path).unwrap()).unwrap();
    assert_eq!(arch_json.schema_version, "2.0");

    let graph_json: GraphExport =
        serde_json::from_str(&fs::read_to_string(&graph_json_path).unwrap()).unwrap();
    assert_eq!(graph_json.version, "1.0");

    let sym_json: SymbolIndexExport =
        serde_json::from_str(&fs::read_to_string(&sym_json_path).unwrap()).unwrap();
    assert_eq!(sym_json.version, "1.0");
}

#[test]
fn test_04_architecture_markdown_sections_and_facts_vs_inferences() {
    let fixture_path = Path::new("fixtures/fixture_a_react");
    let stage1 = run_repository_analysis(fixture_path, BrahmConfig::default()).unwrap();
    let bundle = ArchitectureEngine::new()
        .analyze(fixture_path, &stage1)
        .unwrap();

    let md =
        brahm::reporting::architecture_md::generate_architecture_markdown(&bundle.report, None)
            .unwrap();

    assert!(md.contains("# BRAHM Architectural Blueprint & Code Intelligence Report"));
    assert!(md.contains("## 1. Executive Architecture Overview"));
    assert!(md.contains("## 2. Verification & Epistemological Breakdown"));
    assert!(md.contains("### Verified Static Facts"));
    assert!(md.contains("### Inferred Relationships & Heuristics"));
    assert!(md.contains("### Unresolved References"));
    assert!(md.contains("## 3. Application Entry Points"));
    assert!(md.contains("## 4. API Surface & Endpoint Catalog"));
    assert!(md.contains("## 5. Data Models & Persistence Layer"));
    assert!(md.contains("## 6. Architectural Layers & Component Distribution"));
    assert!(md.contains("## 7. Dependency Topology & Cycles"));
    assert!(md.contains("## 8. Architectural Hotspots"));
    assert!(md.contains("## 9. External Ecosystem Packages"));
}

#[test]
fn test_05_cli_arch_flag_and_subcommand_parsing() {
    let cli_flag = Cli::try_parse_from(["brahm", "scan", "./repo", "--arch"]).unwrap();
    if let Commands::Scan(args) = cli_flag.command {
        assert!(args.arch);
    } else {
        panic!("Expected Scan command");
    }

    let cli_deep_alias = Cli::try_parse_from(["brahm", "scan", "./repo", "--deep"]).unwrap();
    if let Commands::Scan(args) = cli_deep_alias.command {
        assert!(args.arch);
    } else {
        panic!("Expected Scan command");
    }

    let cli_subcmd = Cli::try_parse_from(["brahm", "arch", "./repo"]).unwrap();
    if let Commands::Arch(args) = cli_subcmd.command {
        assert_eq!(args.repository.to_str().unwrap(), "./repo");
    } else {
        panic!("Expected Arch command");
    }
}
