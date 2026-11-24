//! Core filesystem traversal and file metadata collector.

use crate::config::BrahmConfig;
use crate::models::{DirectoryEntry, FileEntry, ScanWarning};
use std::path::{Path, PathBuf};

/// Scanner engine responsible for safe recursive traversal.
pub struct Scanner {
    pub root: PathBuf,
    pub config: BrahmConfig,
}

/// Raw scan result collected during traversal.
#[derive(Debug, Default)]
pub struct RawScanOutput {
    pub files: Vec<FileEntry>,
    pub directories: Vec<DirectoryEntry>,
    pub excluded_paths: Vec<String>,
    pub warnings: Vec<ScanWarning>,
}

impl Scanner {
    /// Creates a new Scanner for the specified repository root.
    pub fn new<P: AsRef<Path>>(root: P, config: BrahmConfig) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            config,
        }
    }
}
