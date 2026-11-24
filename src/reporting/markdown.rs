//! Markdown report generator.
//!
//! Generates comprehensive GitHub-flavored Markdown intelligence reports
//! conforming strictly to the Stage 01 specification.

use crate::errors::Result;
use crate::models::RepositoryReport;
use std::fs;
use std::path::Path;

/// Generates formatted Markdown report and writes to file if path is specified.
pub fn generate_report(report: &RepositoryReport, output_path: Option<&Path>) -> Result<String> {
    let mut md = String::new();

    md.push_str("# BRAHM Repository Intelligence Report\n\n");
    md.push_str("> Generated deterministically by **BRAHM (Blind Reverse-engineering & Adversarial Honeypot Mechanism)**\n\n");

    // 1. Executive overview
    md.push_str("## 1. Executive Overview\n\n");
    md.push_str("| Metric | Value |\n");
    md.push_str("|---|---|\n");
    md.push_str(&format!(
        "| **Repository Name** | `{}` |\n",
        report.repository.name
    ));
    md.push_str(&format!(
        "| **Target Path** | `{}` |\n",
        report.repository.root_path
    ));
    md.push_str(&format!(
        "| **Canonical Path** | `{}` |\n",
        report.repository.canonical_path
    ));
    md.push_str(&format!(
        "| **Scan Duration** | `{} ms` |\n",
        report.scan_metadata.duration_ms
    ));
    md.push_str(&format!(
        "| **Total Scanned Files** | `{}` |\n",
        report.statistics.total_files
    ));
    md.push_str(&format!(
        "| **Total Directories** | `{}` |\n",
        report.statistics.total_directories
    ));
    md.push_str(&format!(
        "| **Total Scanned Size** | `{}` |\n",
        report.statistics.formatted_size
    ));
    md.push_str(&format!(
        "| **Excluded Items** | `{}` |\n",
        report.statistics.excluded_paths_count
    ));
    md.push_str(&format!(
        "| **Scan Warnings** | `{}` |\n\n",
        report.statistics.warnings_count
    ));

    // 2. Repository structure
    md.push_str("## 2. Repository Structure\n\n");
    md.push_str("| Directory Depth | Discovered Directories | Total Files Encountered |\n");
    md.push_str("|---|---|---|\n");
    let max_depth = report
        .directories
        .iter()
        .map(|d| d.depth)
        .max()
        .unwrap_or(0);
    md.push_str(&format!(
        "| Maximum Tree Depth: `{}` | `{}` directories | `{}` files |\n\n",
        max_depth, report.statistics.total_directories, report.statistics.total_files
    ));

    // 3. Language distribution
    md.push_str("## 3. Language Distribution\n\n");
    if report.languages.is_empty() {
        md.push_str("_No recognized source code programming languages detected._\n\n");
    } else {
        md.push_str("| Language | Files | % Files | Total Size | % Bytes |\n");
        md.push_str("|---|---|---|---|---|\n");
        for lang in &report.languages {
            md.push_str(&format!(
                "| **{}** | `{}` | `{}%` | `{}` | `{}%` |\n",
                lang.language,
                lang.file_count,
                lang.percentage_files,
                humansize::format_size(lang.total_bytes, humansize::DECIMAL),
                lang.percentage_bytes
            ));
        }
        md.push('\n');
    }

    // 4. Framework detection
    md.push_str("## 4. Framework Detection\n\n");
    if report.frameworks.is_empty() {
        md.push_str("_No specific application frameworks confirmed._\n\n");
    } else {
        md.push_str("| Framework | Confidence Level | Evidence Discovered |\n");
        md.push_str("|---|---|---|\n");
        for fw in &report.frameworks {
            md.push_str(&format!(
                "| **{}** | `{}` | {} |\n",
                fw.name,
                fw.confidence,
                fw.evidence.join("; ")
            ));
        }
        md.push('\n');
    }

    // 5. File classification
    md.push_str("## 5. File Classification\n\n");
    md.push_str("| Classification Category | File Count |\n");
    md.push_str("|---|---|\n");
    for (category, count) in &report.statistics.classification_counts {
        md.push_str(&format!("| **{}** | `{}` |\n", category, count));
    }
    md.push('\n');

    // 6. Important configurations
    md.push_str("## 6. Important Configurations\n\n");
    if report.configurations.is_empty() {
        md.push_str("_No standard manifest or configuration files identified._\n\n");
    } else {
        for config_file in &report.configurations {
            md.push_str(&format!("- `{}`\n", config_file));
        }
        md.push('\n');
    }

    // 7. Repository statistics
    md.push_str("## 7. Repository Statistics\n\n");
    md.push_str(&format!(
        "- **Total Scanned Bytes:** `{}`\n",
        report.statistics.total_bytes
    ));
    md.push_str(&format!(
        "- **Configuration Files Count:** `{}`\n",
        report.statistics.configuration_count
    ));
    md.push_str(&format!(
        "- **Package Manifest Count:** `{}`\n",
        report.statistics.manifest_count
    ));
    md.push_str(&format!(
        "- **Unknown File Format Count:** `{}`\n",
        report.statistics.unknown_file_count
    ));
    md.push_str("\n### File Extension Summary\n\n");
    md.push_str("| Extension | Count |\n");
    md.push_str("|---|---|\n");
    for (ext, count) in &report.statistics.extension_counts {
        md.push_str(&format!("| `{}` | `{}` |\n", ext, count));
    }
    md.push('\n');

    // 8. Excluded paths
    md.push_str("## 8. Excluded Paths\n\n");
    if report.excluded_paths.is_empty() {
        md.push_str("_No paths were excluded during scanning._\n\n");
    } else {
        for ex in &report.excluded_paths {
            md.push_str(&format!("- `[excluded]` `{}`\n", ex));
        }
        md.push('\n');
    }

    // 9. Scan warnings
    md.push_str("## 9. Scan Warnings\n\n");
    if report.warnings.is_empty() {
        md.push_str("_Zero scan warnings recorded. Traversal was clean._\n\n");
    } else {
        md.push_str("| Severity | Path | Diagnostic Message |\n");
        md.push_str("|---|---|---|\n");
        for w in &report.warnings {
            md.push_str(&format!(
                "| `{}` | `{}` | {} |\n",
                w.severity, w.path, w.message
            ));
        }
        md.push('\n');
    }

    // 10. Technical limitations
    md.push_str("## 10. Technical Limitations\n\n");
    md.push_str("- Deterministic Stage 01 analysis operates without runtime AI models or abstract syntax tree (AST) semantic inferencing.\n");
    md.push_str("- File hashing represents cryptographic integrity fingerprints (BLAKE3), not dynamic trust evaluations or behavior validation.\n");
    md.push_str("- Secret values, private keys, and credential stores are masked and intentionally excluded from reports.\n\n");

    // 11. Scan metadata
    md.push_str("## 11. Scan Metadata\n\n");
    md.push_str(&format!(
        "- **BRAHM Version:** `{}`\n",
        report.brahm_version
    ));
    md.push_str(&format!(
        "- **Scan Started At:** `{}`\n",
        report.scan_metadata.started_at
    ));
    md.push_str(&format!(
        "- **Scan Completed At:** `{}`\n",
        report.scan_metadata.completed_at
    ));
    md.push_str(&format!(
        "- **Elapsed Scan Time:** `{} ms`\n\n",
        report.scan_metadata.duration_ms
    ));

    if let Some(path) = output_path {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, &md)?;
    }

    Ok(md)
}
