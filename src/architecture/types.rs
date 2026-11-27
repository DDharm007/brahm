//! Domain models for BRAHM Stage 02 Architecture Intelligence Reports.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::dependencies::types::DependencyCycle;
use crate::graph::{ArchitectureGraph, GraphExport};
use crate::models::RepositoryMetadata;
use crate::parser::ParsedSource;
use crate::structure::types::{ApiRoute, DataModel, EntryPoint};
use crate::symbols::{Symbol, SymbolIndex, SymbolSummary};

/// Top-level machine-readable architectural report (`BRAHM_ARCHITECTURE.json`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchitectureReport {
    /// Report schema version.
    pub schema_version: String,
    /// BRAHM engine version.
    pub brahm_version: String,
    /// Generation timestamp in UTC.
    pub generated_at: DateTime<Utc>,
    /// Stage 02 code intelligence duration in milliseconds.
    pub analysis_duration_ms: u128,
    /// Metadata about the target repository.
    pub repository: RepositoryMetadata,
    /// Inferred overall application architecture pattern.
    pub pattern: ArchitecturalPatternSummary,
    /// Quantitative code intelligence metrics.
    pub metrics: ArchitectureMetrics,
    /// Identified module and directory layers.
    pub layers: Vec<ArchitecturalLayer>,
    /// Detected application entry points.
    pub entry_points: Vec<EntryPoint>,
    /// Detected HTTP / API endpoints.
    pub api_routes: Vec<ApiRoute>,
    /// Detected persistent database models and migrations.
    pub data_models: Vec<DataModel>,
    /// Dependency intelligence, ecosystem packages, cycles, and unresolved imports.
    pub dependencies: DependencyIntelligence,
    /// Centrality and blast-radius hotspots.
    pub hotspots: Vec<ComponentHotspot>,
    /// Clear separation between verified static facts, heuristics/inferences, and unresolved references.
    pub confidence_breakdown: FactVsInference,
    /// Parse errors, skipped oversized files, and diagnostic notes.
    pub warnings: Vec<String>,
}

/// High-level inferred architectural pattern.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchitecturalPatternSummary {
    /// Primary detected pattern (e.g., "REST API Service", "Full-Stack Monorepo").
    pub primary_pattern: String,
    /// Confidence in pattern inference ("High", "Medium", "Low").
    pub confidence: String,
    /// Confirmed frameworks from Stage 01 & 02.
    pub detected_frameworks: Vec<String>,
    /// Source languages parsed via Tree-sitter AST.
    pub languages_parsed: Vec<String>,
    /// Evidence supporting the inferred architectural pattern.
    pub rationale: Vec<String>,
}

/// Quantitative Stage 02 static analysis metrics.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ArchitectureMetrics {
    pub total_files_scanned: usize,
    pub files_parsed_ast: usize,
    pub files_with_syntax_errors: usize,
    pub total_symbols: usize,
    pub symbols_by_kind: HashMap<String, usize>,
    pub total_imports: usize,
    pub internal_imports: usize,
    pub external_imports: usize,
    pub unique_external_packages: usize,
    pub circular_dependency_count: usize,
    pub entry_point_count: usize,
    pub api_route_count: usize,
    pub data_model_count: usize,
    pub graph_nodes: usize,
    pub graph_edges: usize,
    pub graph_is_dag: bool,
    pub graph_density: f64,
}

/// Grouping of source components into a logical architectural layer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchitecturalLayer {
    pub name: String,
    pub role: String,
    pub directory_paths: Vec<String>,
    pub file_count: usize,
    pub symbol_count: usize,
    pub route_count: usize,
}

/// Dependency and cycle findings.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DependencyIntelligence {
    pub external_packages: Vec<String>,
    pub circular_dependencies: Vec<DependencyCycle>,
    pub graph_scc_cycles: Vec<Vec<String>>,
    pub unresolved_internal_imports: Vec<UnresolvedImport>,
}

/// An internal-looking import statement whose target file could not be resolved locally.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnresolvedImport {
    pub source_file: String,
    pub raw_import_path: String,
    pub line: usize,
}

/// Architectural hotspot determined by graph centrality and blast radius.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentHotspot {
    pub node_id: String,
    pub name: String,
    pub kind: String,
    pub file_path: Option<String>,
    pub fan_in: usize,
    pub fan_out: usize,
    pub blast_radius_count: usize,
}

/// Distinction between AST facts, architectural inferences, and unresolved references.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactVsInference {
    pub detected_facts: Vec<String>,
    pub inferred_relationships: Vec<String>,
    pub unresolved_references_count: usize,
    pub unresolved_references_sample: Vec<String>,
}

/// Export schema for `BRAHM_SYMBOL_INDEX.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolIndexExport {
    pub version: String,
    pub generated_at: DateTime<Utc>,
    pub summary: SymbolSummary,
    pub symbols: Vec<Symbol>,
}

/// Complete Stage 02 output bundle returned by the ArchitectureEngine.
pub struct ArchitectureAnalysisBundle {
    pub report: ArchitectureReport,
    pub graph: ArchitectureGraph,
    pub graph_export: GraphExport,
    pub symbol_index: SymbolIndex,
    pub parsed_files: Vec<ParsedSource>,
}
