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
    println!("SCAN STATUS:        SUCCESS (Deterministic Analysis Complete)");
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
        println!("DETECTED FRAMEWORKS & ARCHITECTURE INDICATORS:");
        for fw in &report.frameworks {
            println!("  - {:<18} [{}]", fw.name, fw.confidence);
            for ev in &fw.evidence {
                println!("      evidence: {}", ev);
            }
        }
        println!();
    }

    if !report.languages.is_empty() {
        println!("PROGRAMMING LANGUAGE DISTRIBUTION:");
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
        println!("FILE CLASSIFICATION BREAKDOWN:");
        for (class_name, count) in &report.statistics.classification_counts {
            println!("  - {:<20} {:>6} files", class_name, count);
        }
        println!();
    }

    if !report.configurations.is_empty() {
        println!("IMPORTANT CONFIGURATIONS & MANIFESTS:");
        for cfg in &report.configurations {
            println!("  - {}", cfg);
        }
        println!();
    }

    if !report.excluded_paths.is_empty() {
        println!("EXCLUDED PATHS INVENTORY (first 10):");
        for ex in report.excluded_paths.iter().take(10) {
            println!("  - [excluded] {}", ex);
        }
        if report.excluded_paths.len() > 10 {
            println!(
                "    ... and {} more excluded paths",
                report.excluded_paths.len() - 10
            );
        }
        println!();
    }

    if !report.warnings.is_empty() {
        println!("SCAN WARNINGS & DIAGNOSTICS:");
        for w in &report.warnings {
            println!("  [{}] {}: {}", w.severity, w.path, w.message);
        }
        println!();
    }
    println!("================================================================================");
}

/// Prints Stage 02 Code Intelligence & Architecture summary to stdout.
pub fn print_architecture_report(report: &crate::architecture::ArchitectureReport) {
    println!();
    println!("================================================================================");
    println!("  BRAHM — Stage 02: Code Intelligence & Architecture Mapping");
    println!("================================================================================");
    println!(
        "  Architecture:     {} (Confidence: {})",
        report.pattern.primary_pattern, report.pattern.confidence
    );
    println!("  Analysis Time:    {} ms", report.analysis_duration_ms);
    println!(
        "  AST Parsed:       {} / {} files",
        report.metrics.files_parsed_ast, report.metrics.total_files_scanned
    );
    println!("  Total Symbols:    {}", report.metrics.total_symbols);
    println!("  Entry Points:     {}", report.metrics.entry_point_count);
    println!("  API Routes:       {}", report.metrics.api_route_count);
    println!("  Data Models:      {}", report.metrics.data_model_count);
    println!(
        "  Import Edges:     {} ({} internal, {} external)",
        report.metrics.total_imports,
        report.metrics.internal_imports,
        report.metrics.external_imports
    );
    println!(
        "  Dep Cycles:       {}",
        report.metrics.circular_dependency_count
    );
    println!(
        "  Graph Size:       {} nodes, {} edges (DAG: {})",
        report.metrics.graph_nodes, report.metrics.graph_edges, report.metrics.graph_is_dag
    );
    if !report.api_routes.is_empty() {
        println!();
        println!("API SURFACE (first 10):");
        for route in report.api_routes.iter().take(10) {
            println!(
                "  - {:<7} {:<30} [{}]",
                route.method, route.path_pattern, route.framework
            );
        }
    }
    if !report.hotspots.is_empty() {
        println!();
        println!("TOP ARCHITECTURE HOTSPOTS:");
        for hs in report.hotspots.iter().take(5) {
            println!(
                "  - {:<28} fan-in: {:<3} blast-radius: {}",
                hs.name, hs.fan_in, hs.blast_radius_count
            );
        }
    }
    println!("================================================================================");
}
