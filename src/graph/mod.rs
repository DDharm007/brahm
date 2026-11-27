//! Architecture and Dependency Graph subsystem powered by petgraph.
//!
//! Provides deterministic representation of software components (files, modules, symbols, routes, packages)
//! and their structural relationships (imports, containment, calls, routing).
//!
//! Includes cycle detection, topological sorting, blast radius analysis, and JSON serialization.

pub mod builder;
pub mod engine;
pub mod export;
pub mod types;

// Convenient re-exports
pub use builder::{normalize_file_id, GraphBuilder};
pub use engine::{ArchitectureGraph, BlastRadiusReport};
pub use export::{CentralNodeExport, GraphEdgeExport, GraphExport, GraphNodeExport, GraphSummary};
pub use types::{EdgeKind, GraphEdge, GraphNode, NodeKind};
