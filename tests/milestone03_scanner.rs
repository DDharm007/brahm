//! Milestone 03 Recursive Scanner Integration Tests

use brahm::config::BrahmConfig;
use brahm::models::FileType;
use brahm::scanner::{detect_binary_content, Scanner};
use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_recursive_scanning_nested_hierarchy() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Create nested structure:
    // root/
    //   file_root.txt
    //   subdir1/
    //     file_sub1.rs
    //     nested/
    //       file_deep.json
    //   empty_dir/
    fs::create_dir_all(root.join("subdir1").join("nested")).unwrap();
    fs::create_dir_all(root.join("empty_dir")).unwrap();

    fs::write(root.join("file_root.txt"), "Root content").unwrap();
    fs::write(root.join("subdir1").join("file_sub1.rs"), "fn main() {}").unwrap();
    fs::write(
        root.join("subdir1").join("nested").join("file_deep.json"),
        "{}",
    )
    .unwrap();

    let config = BrahmConfig::default();
    let scanner = Scanner::new(root, config);
    let output = scanner.scan();

    assert_eq!(output.files.len(), 3);
    assert!(output
        .files
        .iter()
        .any(|f| f.relative_path == "file_root.txt"));
    assert!(output
        .files
        .iter()
        .any(|f| f.relative_path == "subdir1/file_sub1.rs"));
    assert!(output
        .files
        .iter()
        .any(|f| f.relative_path == "subdir1/nested/file_deep.json"));

    // Verify empty directory was detected
    assert!(output
        .directories
        .iter()
        .any(|d| d.relative_path == "empty_dir" && d.file_count == 0 && d.child_dir_count == 0));
}

#[test]
fn test_unicode_and_special_paths() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let unicode_dir = root.join("кириллица_folder");
    fs::create_dir_all(&unicode_dir).unwrap();

    let unicode_file = unicode_dir.join("файл_тест_🚀.rs");
    fs::write(&unicode_file, "// Unicode code content\n").unwrap();

    let unusual_file = root.join("file with spaces and #@! symbols.txt");
    fs::write(&unusual_file, "text").unwrap();

    let config = BrahmConfig::default();
    let scanner = Scanner::new(root, config);
    let output = scanner.scan();

    assert_eq!(output.files.len(), 2);
    assert!(output.files.iter().any(|f| f.filename.contains("🚀")));
    assert!(output
        .files
        .iter()
        .any(|f| f.filename.contains("file with spaces and #@! symbols.txt")));
}

#[test]
fn test_binary_file_detection() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let text_path = root.join("plain_text.txt");
    fs::write(
        &text_path,
        "Standard human readable ASCII and UTF-8 text content.",
    )
    .unwrap();

    let binary_path = root.join("binary_blob.bin");
    let mut bin_file = File::create(&binary_path).unwrap();
    let binary_data = [0x7f, 0x45, 0x4c, 0x46, 0x02, 0x01, 0x01, 0x00, 0x00, 0x00]; // ELF-like with nulls
    bin_file.write_all(&binary_data).unwrap();

    assert!(!detect_binary_content(&text_path).unwrap());
    assert!(detect_binary_content(&binary_path).unwrap());

    let config = BrahmConfig::default();
    let scanner = Scanner::new(root, config);
    let output = scanner.scan();

    let text_entry = output
        .files
        .iter()
        .find(|f| f.filename == "plain_text.txt")
        .unwrap();
    assert!(!text_entry.is_binary);

    let bin_entry = output
        .files
        .iter()
        .find(|f| f.filename == "binary_blob.bin")
        .unwrap();
    assert!(bin_entry.is_binary);
    assert_eq!(bin_entry.file_type, FileType::File);
}

#[test]
fn test_hidden_file_detection() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    fs::write(root.join(".env.example"), "KEY=VALUE").unwrap();
    fs::write(root.join("regular.txt"), "Regular").unwrap();

    let config = BrahmConfig::default();
    let scanner = Scanner::new(root, config);
    let output = scanner.scan();

    let hidden = output
        .files
        .iter()
        .find(|f| f.filename == ".env.example")
        .unwrap();
    assert!(hidden.is_hidden);

    let regular = output
        .files
        .iter()
        .find(|f| f.filename == "regular.txt")
        .unwrap();
    assert!(!regular.is_hidden);
}

#[test]
fn test_exclusion_filtering() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Create excluded dirs
    fs::create_dir_all(root.join("node_modules").join("nested_lib")).unwrap();
    fs::write(
        root.join("node_modules").join("nested_lib").join("dep.js"),
        "console.log('dep');",
    )
    .unwrap();

    fs::create_dir_all(root.join("target").join("debug")).unwrap();
    fs::write(root.join("target").join("debug").join("app.exe"), "ELF").unwrap();

    // Create excluded file
    fs::write(root.join(".DS_Store"), "macOS metadata").unwrap();

    // Create excluded extension
    fs::write(root.join("cache.tmp"), "temporary").unwrap();

    // Create valid file
    fs::write(root.join("src_valid.rs"), "fn valid() {}").unwrap();

    let config = BrahmConfig::default();
    let scanner = Scanner::new(root, config);
    let output = scanner.scan();

    // Only src_valid.rs should be present in files
    assert_eq!(output.files.len(), 1);
    assert_eq!(output.files[0].filename, "src_valid.rs");

    // Excluded paths must contain node_modules, target, .DS_Store, cache.tmp
    assert!(output
        .excluded_paths
        .iter()
        .any(|p| p.contains("node_modules")));
    assert!(output.excluded_paths.iter().any(|p| p.contains("target")));
    assert!(output
        .excluded_paths
        .iter()
        .any(|p| p.contains(".DS_Store")));
    assert!(output
        .excluded_paths
        .iter()
        .any(|p| p.contains("cache.tmp")));
}

#[test]
fn test_maximum_file_size_handling() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let file_path = root.join("large_file.dat");
    // Write 2 MB file
    let two_mb = vec![0x41u8; 2 * 1024 * 1024];
    fs::write(&file_path, &two_mb).unwrap();

    let mut config = BrahmConfig::default();
    // Configure max file size to 1 MB
    config.scanner.max_file_size_mb = 1;

    let scanner = Scanner::new(root, config);
    let output = scanner.scan();

    assert_eq!(output.files.len(), 1);
    let entry = &output.files[0];
    assert_eq!(entry.size_bytes, 2 * 1024 * 1024);
    // Files exceeding quota are safely classified as binary without buffer overflow
    assert!(entry.is_binary);
}
