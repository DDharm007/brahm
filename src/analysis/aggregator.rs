//! Statistics calculation and repository analysis pipeline.

use crate::analysis::hasher::fingerprint_files;
use crate::config::BrahmConfig;
use crate::detection::{detect_frameworks, is_configuration_or_manifest, is_manifest};
use crate::errors::Result;
use crate::models::{
    FileClassification, FileEntry, LanguageStat, RepositoryMetadata, RepositoryReport,
    RepositoryStatistics, ScanMetadata,
};
use crate::scanner::Scanner;
use chrono::Utc;
use humansize::{format_size, DECIMAL};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;
use std::time::Instant;

/// Orchestrates full repository analysis: scanning, detection, hashing, and statistical aggregation.
pub fn run_repository_analysis(repo_root: &Path, config: BrahmConfig) -> Result<RepositoryReport> {
    let start_instant = Instant::now();
    let started_at = Utc::now();

    let canonical_root = fs::canonicalize(repo_root).unwrap_or_else(|_| repo_root.to_path_buf());
    let repo_name = repo_root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "repository".to_string());

    // 1. Filesystem Traversal
    let scanner = Scanner::new(repo_root, config.clone());
    let mut raw_output = scanner.scan();

    // 2. Cryptographic Fingerprinting (optional, streaming BLAKE3)
    let max_file_size_bytes = config.scanner.max_file_size_mb * 1024 * 1024;
    if config.scanner.calculate_hashes {
        fingerprint_files(
            repo_root,
            &mut raw_output.files,
            &mut raw_output.warnings,
            max_file_size_bytes,
        );
    }

    // 3. Framework & Ecosystem Detection
    let frameworks = detect_frameworks(repo_root, &raw_output.files);

    // 4. Discover Manifests and Configurations
    let mut configurations: Vec<String> = raw_output
        .files
        .iter()
        .filter(|f| is_configuration_or_manifest(&f.filename))
        .map(|f| f.relative_path.clone())
        .collect();
    configurations.sort();

    // 5. Statistical Aggregation
    let statistics = analyze_repository(
        &raw_output.files,
        raw_output.directories.len(),
        raw_output.excluded_paths.len(),
        raw_output.warnings.len(),
    );

    let completed_at = Utc::now();
    let duration_ms = start_instant.elapsed().as_millis();

    let scan_metadata = ScanMetadata {
        brahm_version: env!("CARGO_PKG_VERSION").to_string(),
        started_at,
        completed_at,
        duration_ms,
    };

    let repository = RepositoryMetadata {
        name: repo_name,
        root_path: repo_root.to_string_lossy().replace('\\', "/"),
        canonical_path: canonical_root.to_string_lossy().replace('\\', "/"),
    };

    let languages = statistics.language_distribution.clone();

    Ok(RepositoryReport {
        schema_version: "1.0".to_string(),
        brahm_version: env!("CARGO_PKG_VERSION").to_string(),
        repository,
        scan_metadata,
        statistics,
        languages,
        frameworks,
        configurations,
        files: raw_output.files,
        directories: raw_output.directories,
        excluded_paths: raw_output.excluded_paths,
        warnings: raw_output.warnings,
    })
}

/// Aggregates scan data into structured repository statistics.
pub fn analyze_repository(
    files: &[FileEntry],
    total_directories: usize,
    excluded_paths_count: usize,
    warnings_count: usize,
) -> RepositoryStatistics {
    let total_files = files.len();
    let total_bytes: u64 = files.iter().map(|f| f.size_bytes).sum();
    let formatted_size = format_size(total_bytes, DECIMAL);

    let mut lang_bytes: HashMap<String, u64> = HashMap::new();
    let mut lang_counts: HashMap<String, usize> = HashMap::new();
    let mut class_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut ext_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut config_count = 0;
    let mut manifest_count = 0;
    let mut unknown_count = 0;

    for file in files {
        // Classification counts
        let class_str = file.classification.to_string();
        *class_counts.entry(class_str).or_insert(0) += 1;

        if file.classification == FileClassification::Configuration {
            config_count += 1;
        }
        if is_manifest(&file.filename) {
            manifest_count += 1;
        }
        if file.classification == FileClassification::Unknown {
            unknown_count += 1;
        }

        // Extension counts
        let ext = file
            .extension
            .as_deref()
            .unwrap_or("(no extension)")
            .to_ascii_lowercase();
        *ext_counts.entry(ext).or_insert(0) += 1;

        // Language distribution
        if let Some(ref lang) = file.language {
            *lang_counts.entry(lang.clone()).or_insert(0) += 1;
            *lang_bytes.entry(lang.clone()).or_insert(0) += file.size_bytes;
        }
    }

    let mut language_distribution: Vec<LanguageStat> = lang_counts
        .into_iter()
        .map(|(lang, count)| {
            let bytes = lang_bytes.get(&lang).copied().unwrap_or(0);
            let pct_files = if total_files > 0 {
                (count as f64 / total_files as f64) * 100.0
            } else {
                0.0
            };
            let pct_bytes = if total_bytes > 0 {
                (bytes as f64 / total_bytes as f64) * 100.0
            } else {
                0.0
            };
            LanguageStat {
                language: lang,
                file_count: count,
                total_bytes: bytes,
                percentage_files: (pct_files * 10.0).round() / 10.0,
                percentage_bytes: (pct_bytes * 10.0).round() / 10.0,
            }
        })
        .collect();

    // Sort languages descending by bytes
    language_distribution.sort_by_key(|a| std::cmp::Reverse(a.total_bytes));

    RepositoryStatistics {
        total_files,
        total_directories,
        total_bytes,
        formatted_size,
        language_distribution,
        classification_counts: class_counts,
        extension_counts: ext_counts,
        configuration_count: config_count,
        manifest_count,
        unknown_file_count: unknown_count,
        excluded_paths_count,
        warnings_count,
    }
}
