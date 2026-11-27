//! High-level application architecture and structure analyzer.

use serde::{Deserialize, Serialize};

use super::datamodels::DataModelDetector;
use super::entrypoints::EntryPointDetector;
use super::routes::ApiRouteDetector;
use super::types::{ApiRoute, DataModel, EntryPoint};
use crate::parser::ParsedSource;
use crate::symbols::SymbolIndex;

/// Summary metrics of detected architectural landmarks.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructureSummary {
    /// Number of detected application entry points.
    pub entry_point_count: usize,
    /// Total number of detected HTTP API routes.
    pub route_count: usize,
    /// Total number of detected database models / migrations.
    pub data_model_count: usize,
}

/// Comprehensive architectural landmark inventory for a repository.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApplicationStructureResult {
    /// All detected application entry points.
    pub entrypoints: Vec<EntryPoint>,
    /// All detected HTTP API routes.
    pub routes: Vec<ApiRoute>,
    /// All detected data models and migration files.
    pub data_models: Vec<DataModel>,
    /// Summary metrics.
    pub summary: StructureSummary,
}

/// Analyzer coordinating detection of entry points, API routes, and database models.
#[derive(Debug, Default, Clone)]
pub struct StructureAnalyzer {
    entrypoint_detector: EntryPointDetector,
    route_detector: ApiRouteDetector,
    model_detector: DataModelDetector,
}

impl StructureAnalyzer {
    pub fn new() -> Self {
        Self {
            entrypoint_detector: EntryPointDetector::new(),
            route_detector: ApiRouteDetector::new(),
            model_detector: DataModelDetector::new(),
        }
    }

    /// Analyze a collection of parsed source files and their symbol indices.
    pub fn analyze(
        &self,
        parsed_files: &[ParsedSource],
        symbol_index: &SymbolIndex,
    ) -> ApplicationStructureResult {
        let mut entrypoints = Vec::new();
        let mut routes = Vec::new();
        let mut data_models = Vec::new();

        for parsed in parsed_files {
            // Detect entry points
            let file_entries = self
                .entrypoint_detector
                .detect_file_entrypoints(parsed, symbol_index);
            entrypoints.extend(file_entries);

            // Detect API routes
            let file_routes = self.route_detector.detect_routes(parsed);
            routes.extend(file_routes);

            // Detect database models
            let file_models = self.model_detector.detect_models(parsed);
            data_models.extend(file_models);
        }

        let summary = StructureSummary {
            entry_point_count: entrypoints.len(),
            route_count: routes.len(),
            data_model_count: data_models.len(),
        };

        ApplicationStructureResult {
            entrypoints,
            routes,
            data_models,
            summary,
        }
    }
}
