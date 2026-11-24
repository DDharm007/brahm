//! Strongly typed error definitions for parsing operations.

use std::path::PathBuf;
use thiserror::Error;

/// Errors arising during source-code parsing and AST traversal.
#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Language '{0}' is not supported for AST parsing")]
    UnsupportedLanguage(String),

    #[error("Tree-sitter language initialization failed for '{language}': {reason}")]
    LanguageConfigFailed { language: String, reason: String },

    #[error("File exceeds maximum parsing size limit ({size} bytes > {max_size} bytes): {path}")]
    FileTooLarge {
        path: PathBuf,
        size: usize,
        max_size: usize,
    },

    #[error("Failed to read source file '{path}': {source}")]
    IoError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Encoding error in source file '{path}': {reason}")]
    EncodingError { path: PathBuf, reason: String },

    #[error("Parsing timed out or was cancelled for '{path}'")]
    TimeoutOrCancelled { path: PathBuf },

    #[error("Parse failed for '{path:?}': {reason}")]
    ParseFailed {
        path: Option<PathBuf>,
        reason: String,
    },

    #[error("Parser produced no AST output")]
    NoAstReturned,
}
