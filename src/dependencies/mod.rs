//! BRAHM Import, Export & Dependency Analysis Subsystem.
//!
//! Statically maps relationships between files, modules, and external packages.
//! Extracts imports and exports across 9 programming languages, resolves relative import paths,
//! distinguishes internal vs. external dependencies, and detects circular dependency cycles.

pub mod analyzer;
pub mod cycles;
pub mod languages;
pub mod resolver;
pub mod types;

// Re-exports for convenient top-level usage
pub use analyzer::{DependencyAnalysisResult, DependencyAnalyzer, DependencyStats};
pub use cycles::CycleDetector;
pub use resolver::{PathResolver, ResolutionResult};
pub use types::{DependencyCycle, ExportStatement, ImportKind, ImportSpecifier, ImportStatement};
