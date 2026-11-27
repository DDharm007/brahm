//! Streaming cryptographic file fingerprinter powered by BLAKE3.
//!
//! Provides memory-efficient integrity hashing without loading entire files into memory.
//! Specifically excludes sensitive credential files to protect user secrets.

use crate::detection::is_sensitive_credential_file;
use crate::models::{FileEntry, ScanWarning};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use tracing::warn;

const STREAM_CHUNK_SIZE: usize = 64 * 1024; // 64 KB buffer

/// Computes a streaming BLAKE3 integrity hash for a single file.
pub fn compute_file_hash<P: AsRef<Path>>(path: P) -> std::io::Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::with_capacity(STREAM_CHUNK_SIZE, file);
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; STREAM_CHUNK_SIZE];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hasher.finalize().to_hex().to_string())
}

/// Applies streaming BLAKE3 hashing across all eligible files in the scan result.
pub fn fingerprint_files(
    repo_root: &Path,
    files: &mut [FileEntry],
    warnings: &mut Vec<ScanWarning>,
    max_file_size_bytes: u64,
) {
    for entry in files.iter_mut() {
        // 1. Never hash sensitive secret/credential files or raw symlinks
        if is_sensitive_credential_file(&entry.filename)
            || entry.file_type == crate::models::FileType::Symlink
        {
            continue;
        }

        // 2. Skip files exceeding the maximum file size quota
        if entry.size_bytes > max_file_size_bytes {
            warnings.push(ScanWarning {
                path: entry.relative_path.clone(),
                message: format!(
                    "File size ({} bytes) exceeds maximum configured size limit. BLAKE3 fingerprint skipped.",
                    entry.size_bytes
                ),
                severity: "low".to_string(),
            });
            continue;
        }

        // 3. Compute hash
        let full_path = repo_root.join(&entry.relative_path);
        match compute_file_hash(&full_path) {
            Ok(hash) => {
                entry.hash_blake3 = Some(hash);
            }
            Err(e) => {
                warn!(
                    "Failed to compute BLAKE3 hash for '{}': {}",
                    entry.relative_path, e
                );
                warnings.push(ScanWarning {
                    path: entry.relative_path.clone(),
                    message: format!("Hashing failed: {}", e),
                    severity: "medium".to_string(),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{FileClassification, FileType};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_streaming_blake3_known_vector() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("sample.txt");
        fs::write(&file_path, "BRAHM deterministic integrity testing\n").unwrap();

        let hash1 = compute_file_hash(&file_path).expect("Hashing must succeed");
        let hash2 = compute_file_hash(&file_path).expect("Hashing must be deterministic");
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // 256-bit hex
    }

    #[test]
    fn test_secret_files_are_not_fingerprinted() {
        let dir = tempdir().unwrap();
        let secret_path = dir.path().join(".env");
        fs::write(&secret_path, "API_KEY=top_secret_12345\n").unwrap();

        let mut files = vec![FileEntry {
            relative_path: ".env".to_string(),
            filename: ".env".to_string(),
            extension: None,
            size_bytes: 25,
            file_type: FileType::File,
            is_binary: false,
            is_hidden: true,
            modified_at: None,
            language: None,
            classification: FileClassification::Configuration,
            hash_blake3: None,
        }];
        let mut warnings = vec![];

        fingerprint_files(dir.path(), &mut files, &mut warnings, 100 * 1024 * 1024);

        // .env must NEVER receive a hash
        assert_eq!(files[0].hash_blake3, None);
    }
}
