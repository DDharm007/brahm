//! Reporting subsystem for BRAHM.
//!
//! Generates terminal, JSON, and Markdown formatted intelligence reports.

pub mod architecture_json;
pub mod architecture_md;
pub mod json;
pub mod markdown;
pub mod terminal;

use crate::architecture::ArchitectureAnalysisBundle;
use crate::cli::OutputFormat;
use crate::errors::Result;
use crate::models::RepositoryReport;
use std::path::{Path, PathBuf};
use tracing::info;

/// Renders report to terminal output.
pub fn print_terminal_report(report: &RepositoryReport) {
    terminal::print_report(report);
}

/// Generates JSON report and optionally writes to file.
pub fn generate_json_report(
    report: &RepositoryReport,
    output_path: Option<&Path>,
) -> Result<String> {
    json::generate_report(report, output_path)
}

/// Generates Markdown report and optionally writes to file.
pub fn generate_markdown_report(
    report: &RepositoryReport,
    output_path: Option<&Path>,
) -> Result<String> {
    markdown::generate_report(report, output_path)
}

/// Dispatches report generation based on the requested output format.
pub fn dispatch_reports(
    report: &RepositoryReport,
    format: OutputFormat,
    output_dir: Option<&Path>,
) -> Result<()> {
    let out_dir: PathBuf = output_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    match format {
        OutputFormat::Terminal => {
            print_terminal_report(report);
        }
        OutputFormat::Json => {
            let json_path = out_dir.join("BRAHM_REPOSITORY_REPORT.json");
            generate_json_report(report, Some(&json_path))?;
            info!("JSON report written to: {}", json_path.display());
            println!("Report generated: {}", json_path.display());
        }
        OutputFormat::Markdown => {
            let md_path = out_dir.join("BRAHM_REPOSITORY_REPORT.md");
            generate_markdown_report(report, Some(&md_path))?;
            info!("Markdown report written to: {}", md_path.display());
            println!("Report generated: {}", md_path.display());
        }
        OutputFormat::All => {
            print_terminal_report(report);

            let json_path = out_dir.join("BRAHM_REPOSITORY_REPORT.json");
            generate_json_report(report, Some(&json_path))?;
            println!("JSON report generated: {}", json_path.display());

            let md_path = out_dir.join("BRAHM_REPOSITORY_REPORT.md");
            generate_markdown_report(report, Some(&md_path))?;
            println!("Markdown report generated: {}", md_path.display());
        }
    }

    Ok(())
}

/// Dispatches Stage 02 architectural intelligence report generation (`BRAHM_ARCHITECTURE.*`,
/// `BRAHM_DEPENDENCY_GRAPH.json`, and `BRAHM_SYMBOL_INDEX.json`).
pub fn dispatch_architecture_reports(
    bundle: &ArchitectureAnalysisBundle,
    format: OutputFormat,
    output_dir: Option<&Path>,
) -> Result<()> {
    let out_dir: PathBuf = output_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    match format {
        OutputFormat::Terminal => {
            terminal::print_architecture_report(&bundle.report);
        }
        OutputFormat::Json => {
            write_stage02_json_artifacts(bundle, &out_dir)?;
        }
        OutputFormat::Markdown => {
            let md_path = out_dir.join("BRAHM_ARCHITECTURE.md");
            architecture_md::generate_architecture_markdown(&bundle.report, Some(&md_path))?;
            info!("Architecture Markdown written to: {}", md_path.display());
            println!("Architecture Markdown generated: {}", md_path.display());
        }
        OutputFormat::All => {
            terminal::print_architecture_report(&bundle.report);
            write_stage02_json_artifacts(bundle, &out_dir)?;

            let md_path = out_dir.join("BRAHM_ARCHITECTURE.md");
            architecture_md::generate_architecture_markdown(&bundle.report, Some(&md_path))?;
            println!("Architecture Markdown generated: {}", md_path.display());
        }
    }

    Ok(())
}

fn write_stage02_json_artifacts(bundle: &ArchitectureAnalysisBundle, out_dir: &Path) -> Result<()> {
    let arch_path = out_dir.join("BRAHM_ARCHITECTURE.json");
    architecture_json::generate_architecture_json(&bundle.report, Some(&arch_path))?;
    println!("Architecture JSON generated: {}", arch_path.display());

    let graph_path = out_dir.join("BRAHM_DEPENDENCY_GRAPH.json");
    architecture_json::generate_dependency_graph_json(&bundle.graph_export, Some(&graph_path))?;
    println!("Dependency Graph JSON generated: {}", graph_path.display());

    let sym_path = out_dir.join("BRAHM_SYMBOL_INDEX.json");
    architecture_json::generate_symbol_index_json(&bundle.symbol_index, Some(&sym_path))?;
    println!("Symbol Index JSON generated: {}", sym_path.display());

    Ok(())
}
