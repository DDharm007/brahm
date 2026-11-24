//! Centralized error definitions for BRAHM.
//!
//! Provides strongly-typed domain errors utilizing `thiserror`.

use std::path::PathBuf;
use thiserror::Error;

/// Centralized domain error type for BRAHM operations.
#[derive(Error, Debug)]
pub enum BrahmError {
    #[error("I/O error encountered: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Configuration deserialization error: {0}")]
    TomlDeserialization(#[from] toml::de::Error),

    #[error("Configuration serialization error: {0}")]
    TomlSerialization(#[from] toml::ser::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid path: {path} - {reason}")]
    InvalidPath { path: PathBuf, reason: String },

    #[error("Glob pattern error: {0}")]
    GlobError(#[from] globset::Error),

    #[error("Scanning error at '{path}': {message}")]
    ScanError { path: PathBuf, message: String },

    #[error("Analysis error: {0}")]
    AnalysisError(String),

    #[error("Reporting error: {0}")]
    ReportingError(String),

    #[error("Integrity hashing error for '{path}': {message}")]
    HashingError { path: PathBuf, message: String },

    #[error("Parser error: {0}")]
    Parser(#[from] crate::parser::ParserError),
}

/// Standard Result alias for BRAHM operations.
pub type Result<T> = std::result::Result<T, BrahmError>;
