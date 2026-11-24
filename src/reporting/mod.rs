//! Reporting subsystem for BRAHM.
//!
//! Generates terminal, JSON, and Markdown formatted intelligence reports.

pub mod json;
pub mod markdown;
pub mod terminal;

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
