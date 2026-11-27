//! BRAHM Application Structure & Architectural Landmark Detection Subsystem.
//!
//! Statically identifies entry points (executables, servers, CLI, library roots),
//! HTTP API endpoints across major web frameworks (Express, Next.js, FastAPI, Flask, Django, Gin, Actix/Axum, Spring Boot),
//! and persistent data models (ORM entities, SQL migrations).

pub mod analyzer;
pub mod datamodels;
pub mod entrypoints;
pub mod routes;
pub mod types;

// Re-exports for convenient top-level usage
pub use analyzer::{ApplicationStructureResult, StructureAnalyzer, StructureSummary};
pub use datamodels::DataModelDetector;
pub use entrypoints::EntryPointDetector;
pub use routes::ApiRouteDetector;
pub use types::{ApiRoute, DataModel, DataModelKind, EntryPoint, EntryPointKind, HttpMethod};
