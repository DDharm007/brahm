//! Repository analysis, statistical aggregation, and integrity hashing.

pub mod aggregator;
pub mod hasher;

pub use aggregator::{analyze_repository, run_repository_analysis};
pub use hasher::{compute_file_hash, fingerprint_files};
