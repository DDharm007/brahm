//! BRAHM AST Parsing Subsystem.
//!
//! Provides deterministic, multi-language source code parsing powered by Tree-sitter.
//! Supports Rust, JavaScript, TypeScript, TSX, Python, Go, Java, C, and C++.
//!
//! Designed for high performance, memory safety, thread safety, and resilience against syntax errors.

pub mod config;
pub mod engine;
pub mod error;
pub mod language;
pub mod parsed;
pub mod registry;
pub mod span;

// Re-exports for convenient top-level usage
pub use config::{ParserConfig, DEFAULT_MAX_PARSE_FILE_SIZE};
pub use engine::{LanguageParser, TreeSitterEngine};
pub use error::ParserError;
pub use language::SupportedLanguage;
pub use parsed::ParsedSource;
pub use registry::ParserRegistry;
pub use span::SourceSpan;
