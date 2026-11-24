//! Recursive repository scanning subsystem.
//!
//! Provides deterministic traversal of directory trees, safe symbolic link handling,
//! permission resilience, and metadata extraction.

pub mod engine;

pub use engine::{detect_binary_content, RawScanOutput, Scanner};
