//! High-level symbol extraction coordinator.

use tracing::debug;

use super::index::SymbolIndex;
use super::languages::c_cpp::extract_c_cpp_symbols;
use super::languages::go::extract_go_symbols;
use super::languages::java::extract_java_symbols;
use super::languages::javascript::extract_javascript_symbols;
use super::languages::python::extract_python_symbols;
use super::languages::rust::extract_rust_symbols;
use super::types::Symbol;
use crate::parser::{ParsedSource, SupportedLanguage};

/// Trait defining standard symbol extraction behavior.
pub trait SymbolExtractor: Send + Sync {
    /// Extract all code symbols from a parsed source file.
    fn extract_symbols(&self, parsed: &ParsedSource) -> Vec<Symbol>;
}

/// Default multi-language static symbol extractor.
#[derive(Debug, Default, Clone)]
pub struct DefaultSymbolExtractor;

impl DefaultSymbolExtractor {
    pub fn new() -> Self {
        Self
    }
}

impl SymbolExtractor for DefaultSymbolExtractor {
    fn extract_symbols(&self, parsed: &ParsedSource) -> Vec<Symbol> {
        let symbols = match parsed.language {
            SupportedLanguage::Rust => extract_rust_symbols(parsed),
            SupportedLanguage::JavaScript
            | SupportedLanguage::TypeScript
            | SupportedLanguage::Tsx => extract_javascript_symbols(parsed),
            SupportedLanguage::Python => extract_python_symbols(parsed),
            SupportedLanguage::Go => extract_go_symbols(parsed),
            SupportedLanguage::Java => extract_java_symbols(parsed),
            SupportedLanguage::C | SupportedLanguage::Cpp => extract_c_cpp_symbols(parsed),
        };

        debug!(
            "Extracted {} symbols from '{}' ({})",
            symbols.len(),
            parsed.path_display(),
            parsed.language
        );

        symbols
    }
}

/// Convenience function to extract symbols from a `ParsedSource` and return a `SymbolIndex`.
pub fn extract_and_index(parsed: &ParsedSource) -> SymbolIndex {
    let extractor = DefaultSymbolExtractor::new();
    let symbols = extractor.extract_symbols(parsed);
    SymbolIndex::from_symbols(symbols)
}
