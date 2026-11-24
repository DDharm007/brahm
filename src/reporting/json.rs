//! JSON report generator.

use crate::errors::Result;
use crate::models::RepositoryReport;
use std::fs;
use std::path::Path;

/// Generates formatted JSON report and writes to file if path is specified.
pub fn generate_report(report: &RepositoryReport, output_path: Option<&Path>) -> Result<String> {
    let json_str = serde_json::to_string_pretty(report)?;
    if let Some(path) = output_path {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, &json_str)?;
    }
    Ok(json_str)
}
