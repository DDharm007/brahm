//! File classification subsystem.
//!
//! Categorizes repository files into architectural roles based on paths,
//! filenames, and extensions.

pub mod categorizer;

pub use categorizer::classify_file;
