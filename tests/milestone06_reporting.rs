//! Milestone 06 Reporting System Integration Tests

use brahm::analysis::run_repository_analysis;
use brahm::cli::OutputFormat;
use brahm::config::BrahmConfig;
use brahm::reporting::{
    dispatch_reports, generate_json_report, generate_markdown_report, print_terminal_report,
};
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_json_report_structure_and_schema() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src").join("main.rs"), "fn main() {}\n").unwrap();

    let config = BrahmConfig::default();
    let report = run_repository_analysis(root, config).unwrap();

    let json_str = generate_json_report(&report, None).expect("JSON generation must succeed");
    let json_val: Value = serde_json::from_str(&json_str).expect("Valid JSON");

    // Verify root schema fields
    assert_eq!(json_val["schema_version"], "1.0");
    assert_eq!(json_val["brahm_version"], env!("CARGO_PKG_VERSION"));
    assert!(json_val["repository"].is_object());
    assert!(json_val["statistics"].is_object());
    assert!(json_val["languages"].is_array());
    assert!(json_val["frameworks"].is_array());
    assert!(json_val["files"].is_array());
    assert!(json_val["directories"].is_array());
    assert!(json_val["configurations"].is_array());
    assert!(json_val["excluded_paths"].is_array());
    assert!(json_val["warnings"].is_array());
    assert!(json_val["scan_metadata"].is_object());

    // Write to file check
    let output_file = dir.path().join("out.json");
    generate_json_report(&report, Some(&output_file)).unwrap();
    assert!(output_file.is_file());
}

#[test]
fn test_markdown_report_all_11_sections() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    fs::write(root.join("package.json"), "{\"name\": \"test-app\"}\n").unwrap();
    fs::write(root.join("README.md"), "# Test App\n").unwrap();

    let config = BrahmConfig::default();
    let report = run_repository_analysis(root, config).unwrap();

    let md_str = generate_markdown_report(&report, None).expect("Markdown generation must succeed");

    // Verify all 11 required sections exist in the markdown
    assert!(md_str.contains("## 1. Executive Overview"));
    assert!(md_str.contains("## 2. Repository Structure"));
    assert!(md_str.contains("## 3. Language Distribution"));
    assert!(md_str.contains("## 4. Framework Detection"));
    assert!(md_str.contains("## 5. File Classification"));
    assert!(md_str.contains("## 6. Important Configurations"));
    assert!(md_str.contains("## 7. Repository Statistics"));
    assert!(md_str.contains("## 8. Excluded Paths"));
    assert!(md_str.contains("## 9. Scan Warnings"));
    assert!(md_str.contains("## 10. Technical Limitations"));
    assert!(md_str.contains("## 11. Scan Metadata"));

    // Write to file check
    let output_file = dir.path().join("out.md");
    generate_markdown_report(&report, Some(&output_file)).unwrap();
    assert!(output_file.is_file());
}

#[test]
fn test_terminal_report_output_does_not_panic() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::write(root.join("main.rs"), "fn main() {}\n").unwrap();

    let config = BrahmConfig::default();
    let report = run_repository_analysis(root, config).unwrap();

    // Terminal print should cleanly format and output
    print_terminal_report(&report);
}

#[test]
fn test_dispatch_reports_all_formats() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let out_dir = dir.path().join("reports_output");

    fs::write(root.join("main.py"), "print('hello')\n").unwrap();

    let config = BrahmConfig::default();
    let report = run_repository_analysis(root, config).unwrap();

    dispatch_reports(&report, OutputFormat::All, Some(&out_dir)).unwrap();

    assert!(out_dir.join("BRAHM_REPOSITORY_REPORT.json").is_file());
    assert!(out_dir.join("BRAHM_REPOSITORY_REPORT.md").is_file());
}
