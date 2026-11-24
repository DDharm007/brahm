//! Milestone 05 Configuration, Exclusions & Analysis Engine Integration Tests

use brahm::analysis::run_repository_analysis;
use brahm::config::BrahmConfig;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_end_to_end_repository_analysis() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Create a multi-language structure
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::create_dir_all(root.join("scripts")).unwrap();

    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(
        root.join("src").join("main.rs"),
        "fn main() { println!(\"Hi\"); }\n",
    )
    .unwrap();
    fs::write(
        root.join("src").join("lib.rs"),
        "pub fn add(a: i32, b: i32) -> i32 { a + b }\n",
    )
    .unwrap();
    fs::write(
        root.join("docs").join("README.md"),
        "# Demo Project Documentation\n",
    )
    .unwrap();
    fs::write(
        root.join("scripts").join("build.sh"),
        "#!/bin/bash\necho building\n",
    )
    .unwrap();
    fs::write(
        root.join(".env.example"),
        "DATABASE_URL=postgres://localhost:5432\n",
    )
    .unwrap();
    fs::write(
        root.join(".env"),
        "DATABASE_URL=postgres://user:super_secret_pw@prod\n",
    )
    .unwrap();

    let mut config = BrahmConfig::default();
    config.scanner.calculate_hashes = true;

    let report = run_repository_analysis(root, config).expect("Analysis must succeed");

    // Metrics verification
    assert_eq!(report.statistics.total_files, 7);
    assert!(report.statistics.total_directories >= 3);
    assert!(report.statistics.total_bytes > 0);

    // Language verification
    assert!(report
        .languages
        .iter()
        .any(|l| l.language == "Rust" && l.file_count == 2));
    assert!(report
        .languages
        .iter()
        .any(|l| l.language == "Markdown" && l.file_count == 1));
    assert!(report
        .languages
        .iter()
        .any(|l| l.language == "Shell" && l.file_count == 1));

    // Manifest and config verification
    assert!(report
        .configurations
        .iter()
        .any(|c| c.contains("Cargo.toml")));
    assert!(report
        .configurations
        .iter()
        .any(|c| c.contains(".env.example")));
    assert_eq!(report.statistics.manifest_count, 1);

    // BLAKE3 Hashing verification
    let main_rs = report
        .files
        .iter()
        .find(|f| f.filename == "main.rs")
        .unwrap();
    assert!(main_rs.hash_blake3.is_some());
    assert_eq!(main_rs.hash_blake3.as_ref().unwrap().len(), 64);

    // Sensitive file protection: .env MUST NOT have a hash
    let env_file = report.files.iter().find(|f| f.filename == ".env").unwrap();
    assert_eq!(env_file.hash_blake3, None);

    // Scan metadata verification
    assert!(report.scan_metadata.duration_ms < 5000);
    assert_eq!(
        report.scan_metadata.brahm_version,
        env!("CARGO_PKG_VERSION")
    );
}

#[test]
fn test_custom_user_exclusion_rules() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    fs::create_dir_all(root.join("custom_ignored")).unwrap();
    fs::write(root.join("custom_ignored").join("data.txt"), "ignored").unwrap();
    fs::write(root.join("keep.txt"), "keep this").unwrap();

    let mut config = BrahmConfig::default();
    config
        .scanner
        .exclusions
        .directories
        .push("custom_ignored".to_string());

    let report = run_repository_analysis(root, config).expect("Analysis must succeed");

    assert_eq!(report.statistics.total_files, 1);
    assert_eq!(report.files[0].filename, "keep.txt");
    assert!(report
        .excluded_paths
        .iter()
        .any(|p| p.contains("custom_ignored")));
}
