//! BRAHM — Blind Reverse-engineering & Adversarial Honeypot Mechanism
//!
//! Stage 01: Repository Analyzer Core Library
//! Stage 02: Code Intelligence & Architecture Mapping Engine
//!
//! Provides deterministic repository intelligence, recursive scanning,
//! programming language detection, file classification, framework detection,
//! integrity hashing, multi-language AST parsing, symbol extraction,
//! dependency analysis, and architecture graph generation without AI runtime dependencies.

pub mod analysis;
pub mod architecture;
pub mod classification;
pub mod cli;
pub mod config;
pub mod deception;
pub mod dependencies;
pub mod detection;
pub mod errors;
pub mod graph;
pub mod models;
pub mod parser;
pub mod reporting;
pub mod scanner;
pub mod structure;
pub mod symbols;

pub use errors::{BrahmError, Result};
