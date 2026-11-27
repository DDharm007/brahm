//! Core filesystem traversal, safe symbolic link handling, and file inspection engine.

use crate::classification::classify_file;
use crate::config::BrahmConfig;
use crate::detection::detect_language;
use crate::models::{DirectoryEntry, FileEntry, FileType, ScanWarning};

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use tracing::{debug, trace, warn};
use walkdir::{DirEntry, WalkDir};

/// Core scanner engine.
pub struct Scanner {
    pub root: PathBuf,
    pub canonical_root: PathBuf,
    pub config: BrahmConfig,
}

/// Raw output collected during scanning traversal.
#[derive(Debug, Default, Clone)]
pub struct RawScanOutput {
    pub files: Vec<FileEntry>,
    pub directories: Vec<DirectoryEntry>,
    pub excluded_paths: Vec<String>,
    pub warnings: Vec<ScanWarning>,
}

impl Scanner {
    /// Constructs a new Scanner instance.
    pub fn new<P: AsRef<Path>>(root: P, config: BrahmConfig) -> Self {
        let root_buf = root.as_ref().to_path_buf();
        let canonical_root = fs::canonicalize(&root_buf).unwrap_or_else(|_| root_buf.clone());
        Self {
            root: root_buf,
            canonical_root,
            config,
        }
    }

    /// Recursively scans the repository according to configuration constraints.
    pub fn scan(&self) -> RawScanOutput {
        let mut output = RawScanOutput::default();

        let max_file_size_bytes = self.config.scanner.max_file_size_mb * 1024 * 1024;
        let excluded_dirs = &self.config.scanner.exclusions.directories;
        let excluded_files = &self.config.scanner.exclusions.files;
        let excluded_exts = &self.config.scanner.exclusions.extensions;

        // Tracks directory children for directory statistics (file_count, child_dir_count)
        let mut dir_file_counts: HashMap<PathBuf, usize> = HashMap::new();
        let mut dir_child_counts: HashMap<PathBuf, usize> = HashMap::new();
        let mut discovered_dirs: Vec<(PathBuf, usize)> = Vec::new();

        let walker = WalkDir::new(&self.root)
            .follow_links(self.config.scanner.follow_symlinks)
            .sort_by_file_name();

        let mut it = walker.into_iter();
        while let Some(entry_res) = it.next() {
            let entry = match entry_res {
                Ok(e) => e,
                Err(err) => {
                    let path_str = err
                        .path()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "<unknown>".to_string());
                    warn!("Filesystem traversal error at '{}': {}", path_str, err);
                    output.warnings.push(ScanWarning {
                        path: path_str,
                        message: format!("Filesystem traversal error: {}", err),
                        severity: "medium".to_string(),
                    });
                    continue;
                }
            };

            let path = entry.path();
            let is_root = path == self.root;

            // Handle exclusions for directories
            if entry.file_type().is_dir() && !is_root {
                let dir_name = entry.file_name().to_string_lossy();
                if excluded_dirs
                    .iter()
                    .any(|ex| ex.eq_ignore_ascii_case(&dir_name))
                {
                    let rel_path = self.relative_path_string(path);
                    debug!("Excluding directory from scan: {}", rel_path);
                    output.excluded_paths.push(rel_path);
                    it.skip_current_dir();

                    continue;
                }
            }

            // Check symlink safety strictly: never read/follow symlinks when follow_symlinks=false,
            // and confine followed symlinks strictly inside canonical_root.
            if entry.path_is_symlink() {
                if !self.config.scanner.follow_symlinks {
                    let rel_path = self.relative_path_string(path);
                    trace!(
                        "Skipping symlink (follow_symlinks=false): {}",
                        path.display()
                    );
                    output.excluded_paths.push(rel_path);
                    if entry.file_type().is_dir() {
                        it.skip_current_dir();
                    }
                    continue;
                }

                match fs::canonicalize(path) {
                    Ok(target) if target.starts_with(&self.canonical_root) => {}
                    Ok(target) => {
                        warn!(
                            "Symlink '{}' targets outside repository boundary '{}'. Traversal blocked.",
                            path.display(),
                            target.display()
                        );
                        output.warnings.push(ScanWarning {
                            path: self.relative_path_string(path),
                            message: format!(
                                "Symlink target '{}' escapes repository root. Traversal forbidden.",
                                target.display()
                            ),
                            severity: "high".to_string(),
                        });
                        if entry.file_type().is_dir() {
                            it.skip_current_dir();
                        }
                        continue;
                    }
                    Err(e) => {
                        output.warnings.push(ScanWarning {
                            path: self.relative_path_string(path),
                            message: format!("Unresolvable symlink skipped: {}", e),
                            severity: "medium".to_string(),
                        });
                        if entry.file_type().is_dir() {
                            it.skip_current_dir();
                        }
                        continue;
                    }
                }
            }

            // Record directory entry
            if entry.file_type().is_dir() {
                let depth = entry.depth();
                discovered_dirs.push((path.to_path_buf(), depth));

                // Update parent directory child count
                if let Some(parent) = path.parent() {
                    *dir_child_counts.entry(parent.to_path_buf()).or_insert(0) += 1;
                }
                continue;
            }

            // Process file entry
            if entry.file_type().is_file() || entry.path_is_symlink() {
                let filename = entry.file_name().to_string_lossy().to_string();

                // Check file exclusions
                if excluded_files
                    .iter()
                    .any(|ex| ex.eq_ignore_ascii_case(&filename))
                {
                    let rel_path = self.relative_path_string(path);
                    output.excluded_paths.push(rel_path);
                    continue;
                }

                let extension = path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|s| s.to_ascii_lowercase());

                // Check extension exclusions
                if let Some(ref ext) = extension {
                    if excluded_exts.iter().any(|ex| ex.eq_ignore_ascii_case(ext)) {
                        let rel_path = self.relative_path_string(path);
                        output.excluded_paths.push(rel_path);
                        continue;
                    }
                }

                // Update parent directory file count
                if let Some(parent) = path.parent() {
                    *dir_file_counts.entry(parent.to_path_buf()).or_insert(0) += 1;
                }

                // Collect file metadata
                match self.inspect_file(&entry, max_file_size_bytes) {
                    Ok(file_entry) => {
                        output.files.push(file_entry);
                    }
                    Err(warn_msg) => {
                        output.warnings.push(ScanWarning {
                            path: self.relative_path_string(path),
                            message: warn_msg,
                            severity: "medium".to_string(),
                        });
                    }
                }
            }
        }

        // Aggregate directory metrics
        for (dir_path, depth) in discovered_dirs {
            let rel_path = self.relative_path_string(&dir_path);
            let file_count = dir_file_counts.get(&dir_path).copied().unwrap_or(0);
            let child_dir_count = dir_child_counts.get(&dir_path).copied().unwrap_or(0);

            output.directories.push(DirectoryEntry {
                relative_path: if rel_path.is_empty() {
                    ".".to_string()
                } else {
                    rel_path
                },
                depth,
                file_count,
                child_dir_count,
            });
        }

        output
    }

    /// Normalizes a path relative to the repository root.
    pub fn relative_path_string<P: AsRef<Path>>(&self, path: P) -> String {
        let p = path.as_ref();
        if let Ok(rel) = p.strip_prefix(&self.root) {
            let s = rel.to_string_lossy().replace('\\', "/");
            if s.is_empty() {
                ".".to_string()
            } else {
                s
            }
        } else {
            p.to_string_lossy().replace('\\', "/")
        }
    }

    /// Inspects an individual file entry to extract filesystem metadata.
    fn inspect_file(&self, entry: &DirEntry, max_size_bytes: u64) -> Result<FileEntry, String> {
        let path = entry.path();
        let filename = entry.file_name().to_string_lossy().to_string();
        let relative_path = self.relative_path_string(path);

        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(e) => return Err(format!("Failed to retrieve file metadata: {}", e)),
        };

        let size_bytes = metadata.len();
        let file_type = if entry.path_is_symlink() {
            FileType::Symlink
        } else if metadata.is_file() {
            FileType::File
        } else {
            FileType::Other
        };

        let modified_at = metadata.modified().ok().map(DateTime::<Utc>::from);

        let is_hidden = is_hidden_file(entry);

        // Detect if binary via first 8192 bytes
        let is_binary = if size_bytes == 0 {
            false
        } else if size_bytes > max_size_bytes {
            // Treat files exceeding maximum size quota as binary by default to prevent buffer allocation
            true
        } else {
            detect_binary_content(path).unwrap_or(false)
        };

        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_string());

        let language = detect_language(&filename, extension.as_deref());
        let classification = classify_file(&relative_path, &filename, extension.as_deref());

        Ok(FileEntry {
            relative_path,
            filename,
            extension,
            size_bytes,
            file_type,
            is_binary,
            is_hidden,
            modified_at,
            language,
            classification,
            hash_blake3: None, // Will be populated by analysis engine (Milestone 05)
        })
    }
}

/// Detects if a file is hidden (starts with '.' or has Windows hidden attribute).
fn is_hidden_file(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();
    let dot_hidden = name.starts_with('.') && name != "." && name != "..";

    // Windows FILE_ATTRIBUTE_HIDDEN check (compiled out on other platforms).
    #[cfg(windows)]
    if !dot_hidden {
        if let Ok(meta) = entry.metadata() {
            use std::os::windows::fs::MetadataExt;
            const FILE_ATTRIBUTE_HIDDEN: u32 = 0x0000_0002;
            if (meta.file_attributes() & FILE_ATTRIBUTE_HIDDEN) != 0 {
                return true;
            }
        }
    }

    dot_hidden
}

/// Inspects the first 8 KB of a file to reliably determine if it contains binary data.
pub fn detect_binary_content<P: AsRef<Path>>(path: P) -> std::io::Result<bool> {
    let mut file = File::open(path)?;
    let mut buffer = [0u8; 8192];
    let bytes_read = file.read(&mut buffer)?;

    if bytes_read == 0 {
        return Ok(false);
    }

    let sample = &buffer[..bytes_read];

    // Check for null bytes (definitive binary indicator)
    if sample.contains(&0) {
        return Ok(true);
    }

    // Check non-printable control characters (excluding standard whitespace \t, \r, \n, \x0c)
    let non_text = sample
        .iter()
        .filter(|&&b| b < 32 && b != b'\t' && b != b'\r' && b != b'\n' && b != 12)
        .count();

    // If more than 10% non-text characters in the sample, classify as binary
    let ratio = non_text as f64 / bytes_read as f64;
    Ok(ratio > 0.10)
}
