//! Architecture Intelligence & Synthesis Engine (Stage 02 Coordinator).
//!
//! Connects AST parsing, symbol extraction, dependency analysis, structural landmark detection,
//! and graph metrics into comprehensive architectural blueprints (`BRAHM_ARCHITECTURE.*`).

pub mod engine;
pub mod types;

pub use engine::ArchitectureEngine;
pub use types::{
    ArchitecturalLayer, ArchitecturalPatternSummary, ArchitectureAnalysisBundle,
    ArchitectureMetrics, ArchitectureReport, ComponentHotspot, DependencyIntelligence,
    FactVsInference, SymbolIndexExport, UnresolvedImport,
};
