//! Statistics calculation and distribution computation.

use crate::models::{FileClassification, FileEntry, LanguageStat, RepositoryStatistics};
use humansize::{format_size, DECIMAL};
use std::collections::{BTreeMap, HashMap};

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
    let mut unknown_count = 0;

    for file in files {
        // Classification counts
        let class_str = file.classification.to_string();
        *class_counts.entry(class_str).or_insert(0) += 1;

        if file.classification == FileClassification::Configuration {
            config_count += 1;
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
        manifest_count: 0,
        unknown_file_count: unknown_count,
        excluded_paths_count,
        warnings_count,
    }
}
