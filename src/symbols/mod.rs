//! BRAHM Symbol Extraction Subsystem.
//!
//! Extracts functions, methods, classes, interfaces, structs, enums, traits, modules,
//! type aliases, and constants with deterministic identifiers, parameter metadata,
//! visibility levels, and source spans across all supported languages.

pub mod extractor;
pub mod id;
pub mod index;
pub mod languages;
pub mod types;

// Re-exports for convenient top-level usage
pub use extractor::{extract_and_index, DefaultSymbolExtractor, SymbolExtractor};
pub use id::generate_symbol_id;
pub use index::{SymbolIndex, SymbolSummary};
pub use types::{ParameterInfo, Symbol, SymbolKind, Visibility};
