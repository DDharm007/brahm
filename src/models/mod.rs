//! Domain data models for BRAHM Stage 01.
//!
//! Strongly-typed models for files, directories, languages, classifications,
//! framework detections, and comprehensive repository reports.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

/// Type of filesystem entry discovered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileType {
    File,
    Directory,
    Symlink,
    Other,
}

impl fmt::Display for FileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileType::File => write!(f, "File"),
            FileType::Directory => write!(f, "Directory"),
            FileType::Symlink => write!(f, "Symlink"),
            FileType::Other => write!(f, "Other"),
        }
    }
}

/// Categorical classification of repository files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileClassification {
    SourceCode,
    Configuration,
    Documentation,
    Tests,
    Assets,
    Database,
    Infrastructure,
    Deployment,
    BuildSystem,
    Scripts,
    Generated,
    Dependencies,
    Unknown,
}

impl fmt::Display for FileClassification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceCode => write!(f, "Source Code"),
            Self::Configuration => write!(f, "Configuration"),
            Self::Documentation => write!(f, "Documentation"),
            Self::Tests => write!(f, "Tests"),
            Self::Assets => write!(f, "Assets"),
            Self::Database => write!(f, "Database"),
            Self::Infrastructure => write!(f, "Infrastructure"),
            Self::Deployment => write!(f, "Deployment"),
            Self::BuildSystem => write!(f, "Build System"),
            Self::Scripts => write!(f, "Scripts"),
            Self::Generated => write!(f, "Generated"),
            Self::Dependencies => write!(f, "Dependencies"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Confidence level for framework and pattern detections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceLevel {
    ConfirmedByManifest,
    StronglyIndicated,
    Possible,
    Unknown,
}

impl fmt::Display for ConfidenceLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConfirmedByManifest => write!(f, "Confirmed by Manifest"),
            Self::StronglyIndicated => write!(f, "Strongly Indicated"),
            Self::Possible => write!(f, "Possible"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Metadata and classification for an individual file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileEntry {
    pub relative_path: String,
    pub filename: String,
    pub extension: Option<String>,
    pub size_bytes: u64,
    pub file_type: FileType,
    pub is_binary: bool,
    pub is_hidden: bool,
    pub modified_at: Option<DateTime<Utc>>,
    pub language: Option<String>,
    pub classification: FileClassification,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash_blake3: Option<String>,
}

/// Metadata and statistics for a repository directory.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DirectoryEntry {
    pub relative_path: String,
    pub depth: usize,
    pub file_count: usize,
    pub child_dir_count: usize,
}

/// Detected framework with evidence and confidence ranking.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FrameworkIndicator {
    pub name: String,
    pub confidence: ConfidenceLevel,
    pub evidence: Vec<String>,
}

/// Statistical breakdown of a programming language in the repository.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LanguageStat {
    pub language: String,
    pub file_count: usize,
    pub total_bytes: u64,
    pub percentage_files: f64,
    pub percentage_bytes: f64,
}

/// Warning generated during scanning or analysis.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScanWarning {
    pub path: String,
    pub message: String,
    pub severity: String,
}

/// Basic repository identification metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepositoryMetadata {
    pub name: String,
    pub root_path: String,
    pub canonical_path: String,
}

/// Comprehensive aggregated statistics of the analyzed repository.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RepositoryStatistics {
    pub total_files: usize,
    pub total_directories: usize,
    pub total_bytes: u64,
    pub formatted_size: String,
    pub language_distribution: Vec<LanguageStat>,
    pub classification_counts: BTreeMap<String, usize>,
    pub extension_counts: BTreeMap<String, usize>,
    pub configuration_count: usize,
    pub manifest_count: usize,
    pub unknown_file_count: usize,
    pub excluded_paths_count: usize,
    pub warnings_count: usize,
}

/// Metadata describing the scan run.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScanMetadata {
    pub brahm_version: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub duration_ms: u128,
}

/// Complete top-level structured report produced by BRAHM Repository Analyzer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RepositoryReport {
    pub schema_version: String,
    pub brahm_version: String,
    pub repository: RepositoryMetadata,
    pub scan_metadata: ScanMetadata,
    pub statistics: RepositoryStatistics,
    pub languages: Vec<LanguageStat>,
    pub frameworks: Vec<FrameworkIndicator>,
    pub configurations: Vec<String>,
    pub files: Vec<FileEntry>,
    pub directories: Vec<DirectoryEntry>,
    pub excluded_paths: Vec<String>,
    pub warnings: Vec<ScanWarning>,
}
