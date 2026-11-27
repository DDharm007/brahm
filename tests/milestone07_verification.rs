//! Milestone 07 Complete Stage 01 Verification & Security Test Suite
//!
//! Validates the 20 engineering requirements across real fixture repositories
//! and asserts strict read-only security guarantees.

use brahm::analysis::run_repository_analysis;
use brahm::cli::OutputFormat;
use brahm::config::BrahmConfig;
use brahm::detection::is_sensitive_credential_file;
use brahm::models::{ConfidenceLevel, FileClassification, FileType};
use brahm::reporting::dispatch_reports;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_01_fixture_a_react_project() {
    let fixture_path = Path::new("fixtures/fixture_a_react");
    let config = BrahmConfig::default();
    let report =
        run_repository_analysis(fixture_path, config).expect("Fixture A analysis must succeed");

    assert!(report.statistics.total_files >= 5);
    // Framework detection
    assert!(report
        .frameworks
        .iter()
        .any(|f| f.name == "React" && f.confidence == ConfidenceLevel::ConfirmedByManifest));
    assert!(report
        .frameworks
        .iter()
        .any(|f| f.name == "Vite" && f.confidence == ConfidenceLevel::ConfirmedByManifest));
    // Languages
    assert!(report.languages.iter().any(|l| l.language == "TypeScript"));
    assert!(report.languages.iter().any(|l| l.language == "JavaScript"));
    assert!(report.languages.iter().any(|l| l.language == "CSS"));
    assert!(report.languages.iter().any(|l| l.language == "Markdown"));
}

#[test]
fn test_02_fixture_b_fullstack_project() {
    let fixture_path = Path::new("fixtures/fixture_b_fullstack");
    let config = BrahmConfig::default();
    let report =
        run_repository_analysis(fixture_path, config).expect("Fixture B analysis must succeed");

    // Framework detection
    assert!(report.frameworks.iter().any(|f| f.name == "Next.js"));
    assert!(report.frameworks.iter().any(|f| f.name == "React"));
    assert!(report.frameworks.iter().any(|f| f.name == "Node.js"));

    // Infrastructure & database classification
    assert!(report
        .files
        .iter()
        .any(|f| f.classification == FileClassification::Infrastructure
            && f.filename == "Dockerfile"));
    assert!(report
        .files
        .iter()
        .any(|f| f.classification == FileClassification::Infrastructure
            && f.filename == "docker-compose.yml"));
    assert!(report
        .files
        .iter()
        .any(|f| f.classification == FileClassification::Database && f.filename.ends_with(".sql")));

    // Secret protection: .env must NEVER have a hash
    let env_entry = report
        .files
        .iter()
        .find(|f| f.filename == ".env")
        .expect("fixture_b_fullstack/.env must be present in report.files; check that the file is committed (not gitignored) so CI can find it");
    assert_eq!(env_entry.hash_blake3, None);
    assert!(is_sensitive_credential_file(&env_entry.filename));
}

#[test]
fn test_03_fixture_c_multilingual_project() {
    let fixture_path = Path::new("fixtures/fixture_c_multilingual");
    let config = BrahmConfig::default();
    let report =
        run_repository_analysis(fixture_path, config).expect("Fixture C analysis must succeed");

    // Multi-language verification
    let langs: Vec<&str> = report
        .languages
        .iter()
        .map(|l| l.language.as_str())
        .collect();
    assert!(langs.contains(&"Rust"));
    assert!(langs.contains(&"Python"));
    assert!(langs.contains(&"Go"));
    assert!(langs.contains(&"JavaScript"));
    assert!(langs.contains(&"TypeScript"));
    assert!(langs.contains(&"SQL"));
    assert!(langs.contains(&"Shell"));
    assert!(langs.contains(&"YAML"));
    assert!(langs.contains(&"JSON"));
    assert!(langs.contains(&"Markdown"));

    // Frameworks
    assert!(report.frameworks.iter().any(|f| f.name == "Rust"));
    assert!(report.frameworks.iter().any(|f| f.name == "Go"));
    assert!(report.frameworks.iter().any(|f| f.name == "Flask"));
}

#[test]
fn test_04_repository_read_only_guarantee() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let sample_file = root.join("critical_source.rs");
    fs::write(&sample_file, "pub fn genuine_logic() -> bool { true }\n").unwrap();
    let initial_metadata = fs::metadata(&sample_file).unwrap();
    let initial_mod_time = initial_metadata.modified().unwrap();
    let initial_len = initial_metadata.len();

    let config = BrahmConfig::default();
    let _ = run_repository_analysis(root, config).unwrap();

    let after_metadata = fs::metadata(&sample_file).unwrap();
    assert_eq!(after_metadata.len(), initial_len);
    assert_eq!(after_metadata.modified().unwrap(), initial_mod_time);

    // Assert no extraneous files were created inside the scanned repository
    let entries: Vec<_> = fs::read_dir(root)
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].to_str().unwrap(), "critical_source.rs");
}

#[test]
fn test_05_unknown_extensions_and_binary_resilience() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    fs::write(
        root.join("unknown_format.xyz999"),
        "custom unstructured data\n",
    )
    .unwrap();
    fs::write(root.join("firmware.rom"), [0x00u8, 0x11, 0x22, 0x00, 0xFF]).unwrap();

    let config = BrahmConfig::default();
    let report = run_repository_analysis(root, config).unwrap();

    let unknown_entry = report
        .files
        .iter()
        .find(|f| f.filename == "unknown_format.xyz999")
        .unwrap();
    assert_eq!(unknown_entry.language, None);
    assert_eq!(unknown_entry.classification, FileClassification::Unknown);

    let bin_entry = report
        .files
        .iter()
        .find(|f| f.filename == "firmware.rom")
        .unwrap();
    assert!(bin_entry.is_binary);
    assert_eq!(bin_entry.file_type, FileType::File);
}

#[test]
fn test_06_reports_generation_all_formats_consistency() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let report_dir = dir.path().join("out_reports");

    fs::write(root.join("main.rs"), "fn main() {}\n").unwrap();

    let config = BrahmConfig::default();
    let report = run_repository_analysis(root, config).unwrap();

    dispatch_reports(&report, OutputFormat::All, Some(&report_dir)).unwrap();

    let json_file = report_dir.join("BRAHM_REPOSITORY_REPORT.json");
    let md_file = report_dir.join("BRAHM_REPOSITORY_REPORT.md");

    assert!(json_file.is_file());
    assert!(md_file.is_file());

    let json_str = fs::read_to_string(&json_file).unwrap();
    let md_str = fs::read_to_string(&md_file).unwrap();

    // Verify consistency between JSON and Markdown reports
    assert!(json_str.contains(&report.repository.name));
    assert!(md_str.contains(&report.repository.name));
    assert!(json_str.contains(&format!(
        "\"total_files\": {}",
        report.statistics.total_files
    )));
    assert!(md_str.contains(&format!(
        "| **Total Scanned Files** | `{}` |",
        report.statistics.total_files
    )));
}
