//! BRAHM — Blind Reverse-engineering & Adversarial Honeypot Mechanism
//!
//! Stage 01: Repository Analyzer Core Library
//!
//! Provides deterministic repository intelligence, recursive scanning,
//! programming language detection, file classification, framework detection,
//! integrity hashing, and multi-format reporting without AI runtime dependencies.

pub mod analysis;
pub mod classification;
pub mod cli;
pub mod config;
pub mod detection;
pub mod errors;
pub mod models;
pub mod reporting;
pub mod scanner;

pub use errors::{BrahmError, Result};
