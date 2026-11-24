//! Markdown report generator.

use crate::errors::Result;
use crate::models::RepositoryReport;
use std::fs;
use std::path::Path;

/// Generates formatted Markdown report and writes to file if path is specified.
pub fn generate_report(report: &RepositoryReport, output_path: Option<&Path>) -> Result<String> {
    let mut md = String::new();

    md.push_str("# BRAHM Repository Intelligence Report\n\n");
    md.push_str("> Generated deterministically by **BRAHM (Blind Reverse-engineering & Adversarial Honeypot Mechanism)**\n\n");

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
        "| **Total Size** | `{}` |\n",
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

    md.push_str("## 2. Detected Frameworks & Architecture Indicators\n\n");
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

    md.push_str("## 3. Programming Language Distribution\n\n");
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

    md.push_str("## 4. File Classification Breakdown\n\n");
    md.push_str("| Classification Category | File Count |\n");
    md.push_str("|---|---|\n");
    for (category, count) in &report.statistics.classification_counts {
        md.push_str(&format!("| **{}** | `{}` |\n", category, count));
    }
    md.push('\n');

    md.push_str("## 5. Discovered Configuration & Manifest Files\n\n");
    if report.configurations.is_empty() {
        md.push_str("_No standard manifest or configuration files identified._\n\n");
    } else {
        for config_file in &report.configurations {
            md.push_str(&format!("- `{}`\n", config_file));
        }
        md.push('\n');
    }

    if !report.warnings.is_empty() {
        md.push_str("## 6. Scan Warnings & Diagnostics\n\n");
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

    md.push_str("## 7. Technical Limitations & Operational Context\n\n");
    md.push_str("- Deterministic Stage 01 analysis without AI runtime or AST inference.\n");
    md.push_str(
        "- File hashing represents integrity fingerprints, not dynamic trust evaluation.\n",
    );
    md.push_str("- Secret values are strictly masked and never stored in reports.\n\n");

    md.push_str("---\n");
    md.push_str(&format!(
        "_BRAHM Version: `{}` | Scan Timestamp: `{}`_\n",
        report.brahm_version, report.scan_metadata.completed_at
    ));

    if let Some(path) = output_path {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, &md)?;
    }

    Ok(md)
}
