//! Reporting subsystem for BRAHM.
//!
//! Generates terminal, JSON, and Markdown formatted intelligence reports.

pub mod json;
pub mod markdown;
pub mod terminal;

use crate::errors::Result;
use crate::models::RepositoryReport;
use std::path::Path;

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
