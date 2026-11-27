//! Stage 02 JSON artifact generators:
//! - BRAHM_ARCHITECTURE.json
//! - BRAHM_DEPENDENCY_GRAPH.json
//! - BRAHM_SYMBOL_INDEX.json

use chrono::Utc;
use std::fs;
use std::path::Path;

use crate::architecture::{ArchitectureReport, SymbolIndexExport};
use crate::errors::Result;
use crate::graph::GraphExport;
use crate::symbols::SymbolIndex;

/// Generate and optionally write `BRAHM_ARCHITECTURE.json`.
pub fn generate_architecture_json(
    report: &ArchitectureReport,
    output_path: Option<&Path>,
) -> Result<String> {
    let json_str = serde_json::to_string_pretty(report)?;
    write_if_path(output_path, &json_str)?;
    Ok(json_str)
}

/// Generate and optionally write `BRAHM_DEPENDENCY_GRAPH.json`.
pub fn generate_dependency_graph_json(
    graph_export: &GraphExport,
    output_path: Option<&Path>,
) -> Result<String> {
    let json_str = serde_json::to_string_pretty(graph_export)?;
    write_if_path(output_path, &json_str)?;
    Ok(json_str)
}

/// Generate and optionally write `BRAHM_SYMBOL_INDEX.json`.
pub fn generate_symbol_index_json(
    symbol_index: &SymbolIndex,
    output_path: Option<&Path>,
) -> Result<String> {
    let export = SymbolIndexExport {
        version: "1.0".to_string(),
        generated_at: Utc::now(),
        summary: symbol_index.summary(),
        symbols: symbol_index.all().to_vec(),
    };

    let json_str = serde_json::to_string_pretty(&export)?;
    write_if_path(output_path, &json_str)?;
    Ok(json_str)
}

fn write_if_path(output_path: Option<&Path>, contents: &str) -> Result<()> {
    if let Some(path) = output_path {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, contents)?;
    }
    Ok(())
}
