//! Terminal-based report renderer.

use crate::models::RepositoryReport;

/// Prints the structured intelligence report to stdout.
pub fn print_report(report: &RepositoryReport) {
    println!();
    println!("================================================================================");
    println!("  BRAHM — Blind Reverse-engineering & Adversarial Honeypot Mechanism");
    println!("  Stage 01: Repository Intelligence & Architecture Analyzer");
    println!("================================================================================");
    println!();
    println!("REPOSITORY METADATA:");
    println!("  Name:             {}", report.repository.name);
    println!("  Path:             {}", report.repository.root_path);
    println!("  Canonical Path:   {}", report.repository.canonical_path);
    println!();
    println!("SCAN METRICS:");
    println!(
        "  Duration:         {} ms",
        report.scan_metadata.duration_ms
    );
    println!("  Total Files:      {}", report.statistics.total_files);
    println!(
        "  Total Directory:  {}",
        report.statistics.total_directories
    );
    println!("  Scanned Size:     {}", report.statistics.formatted_size);
    println!(
        "  Excluded Paths:   {}",
        report.statistics.excluded_paths_count
    );
    println!("  Scan Warnings:    {}", report.statistics.warnings_count);
    println!();

    if !report.frameworks.is_empty() {
        println!("DETECTED FRAMEWORKS & ECOSYSTEMS:");
        for fw in &report.frameworks {
            println!("  - {:<18} [{}]", fw.name, fw.confidence);
            for ev in &fw.evidence {
                println!("      evidence: {}", ev);
            }
        }
        println!();
    }

    if !report.languages.is_empty() {
        println!("PRIMARY PROGRAMMING LANGUAGES:");
        for lang in &report.languages {
            println!(
                "  - {:<16} {:>6} files ({:>5.1}%)  {:>10} ({:>5.1}%)",
                lang.language,
                lang.file_count,
                lang.percentage_files,
                humansize::format_size(lang.total_bytes, humansize::DECIMAL),
                lang.percentage_bytes
            );
        }
        println!();
    }

    if !report.statistics.classification_counts.is_empty() {
        println!("FILE CLASSIFICATION SUMMARY:");
        for (class_name, count) in &report.statistics.classification_counts {
            println!("  - {:<20} {:>6} files", class_name, count);
        }
        println!();
    }

    if !report.warnings.is_empty() {
        println!("SCAN WARNINGS:");
        for w in &report.warnings {
            println!("  [{}] {}: {}", w.severity, w.path, w.message);
        }
        println!();
    }
    println!("================================================================================");
}
