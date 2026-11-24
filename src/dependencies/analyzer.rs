//! Dependency analysis coordinator and graph builder.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use tracing::{debug, warn};

use super::cycles::CycleDetector;
use super::languages::c_cpp::extract_c_cpp_imports;
use super::languages::go::extract_go_imports;
use super::languages::java::extract_java_imports;
use super::languages::javascript::extract_javascript_imports;
use super::languages::python::extract_python_imports;
use super::languages::rust::extract_rust_imports;
use super::resolver::{PathResolver, ResolutionResult};
use super::types::{DependencyCycle, ExportStatement, ImportStatement};
use crate::parser::{ParsedSource, SupportedLanguage};

/// Statistical summary of repository dependencies.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyStats {
    /// Total number of import statements detected.
    pub total_imports: usize,
    /// Number of imports resolving to internal repository files.
    pub internal_imports: usize,
    /// Number of imports resolving to third-party or system packages.
    pub external_imports: usize,
    /// Number of unique external packages imported.
    pub unique_external_packages: usize,
    /// Total number of export declarations detected.
    pub total_exports: usize,
    /// Number of detected circular dependency cycles.
    pub circular_dependency_count: usize,
}

/// Comprehensive outcome of dependency and import/export analysis for a repository.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependencyAnalysisResult {
    /// All detected import statements with resolved targets.
    pub imports: Vec<ImportStatement>,

    /// All detected export statements.
    pub exports: Vec<ExportStatement>,

    /// Directed internal file dependencies: source file -> list of imported files.
    pub internal_dependencies: HashMap<PathBuf, Vec<PathBuf>>,

    /// External package dependencies: source file -> list of imported package names.
    pub external_dependencies: HashMap<PathBuf, Vec<String>>,

    /// All unique external package names referenced across the repository.
    pub unique_external_packages: HashSet<String>,

    /// Detected circular dependency cycles.
    pub cycles: Vec<DependencyCycle>,

    /// Aggregate metrics.
    pub stats: DependencyStats,
}

/// Repository dependency analyzer coordinating import extraction, path resolution, and cycle detection.
#[derive(Debug, Clone)]
pub struct DependencyAnalyzer {
    resolver: PathResolver,
}

impl DependencyAnalyzer {
    /// Create a new `DependencyAnalyzer` rooted at the given repository path.
    pub fn new(repo_root: impl Into<PathBuf>) -> Self {
        Self {
            resolver: PathResolver::new(repo_root),
        }
    }

    /// Pre-populate known file paths in the resolver.
    pub fn with_known_files(mut self, files: impl IntoIterator<Item = PathBuf>) -> Self {
        self.resolver = self.resolver.with_known_files(files);
        self
    }

    /// Analyze a single parsed source file to extract its raw imports and exports.
    pub fn extract_file_imports(
        &self,
        parsed: &ParsedSource,
    ) -> (Vec<ImportStatement>, Vec<ExportStatement>) {
        match parsed.language {
            SupportedLanguage::Rust => extract_rust_imports(parsed),
            SupportedLanguage::JavaScript
            | SupportedLanguage::TypeScript
            | SupportedLanguage::Tsx => extract_javascript_imports(parsed),
            SupportedLanguage::Python => extract_python_imports(parsed),
            SupportedLanguage::Go => extract_go_imports(parsed),
            SupportedLanguage::Java => extract_java_imports(parsed),
            SupportedLanguage::C | SupportedLanguage::Cpp => extract_c_cpp_imports(parsed),
        }
    }

    /// Perform full repository-wide dependency analysis over a collection of parsed files.
    pub fn analyze_repository(&self, parsed_files: &[ParsedSource]) -> DependencyAnalysisResult {
        let mut all_imports = Vec::new();
        let mut all_exports = Vec::new();
        let mut internal_dependencies: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
        let mut external_dependencies: HashMap<PathBuf, Vec<String>> = HashMap::new();
        let mut unique_external_packages = HashSet::new();

        let mut internal_count = 0;
        let mut external_count = 0;

        for parsed in parsed_files {
            let (imports, exports) = self.extract_file_imports(parsed);
            let source_path = parsed
                .file_path()
                .map(|p| self.resolver.normalize_path(p))
                .unwrap_or_else(|| PathBuf::from("unknown"));

            for mut imp in imports {
                let res =
                    self.resolver
                        .resolve_import(&imp.source_file, &imp.raw_path, parsed.language);

                match res {
                    ResolutionResult::Internal(target_file) => {
                        imp.resolved_target = Some(target_file.clone());
                        imp.is_external = false;
                        internal_count += 1;

                        internal_dependencies
                            .entry(source_path.clone())
                            .or_default()
                            .push(target_file);
                    }
                    ResolutionResult::External(pkg) => {
                        imp.is_external = true;
                        external_count += 1;
                        unique_external_packages.insert(pkg.clone());

                        external_dependencies
                            .entry(source_path.clone())
                            .or_default()
                            .push(pkg);
                    }
                    ResolutionResult::Unresolved(u) => {
                        debug!("Unresolved import '{}' in '{}'", u, source_path.display());
                        if imp.is_relative {
                            internal_count += 1;
                        } else {
                            external_count += 1;
                            unique_external_packages.insert(u);
                        }
                    }
                }

                all_imports.push(imp);
            }

            all_exports.extend(exports);
        }

        // Deduplicate neighbor lists
        for neighbors in internal_dependencies.values_mut() {
            neighbors.sort();
            neighbors.dedup();
        }
        for packages in external_dependencies.values_mut() {
            packages.sort();
            packages.dedup();
        }

        // Detect dependency cycles
        let cycles = CycleDetector::find_cycles(&internal_dependencies);
        if !cycles.is_empty() {
            warn!(
                "Detected {} circular dependency cycle(s) in repository",
                cycles.len()
            );
        }

        let stats = DependencyStats {
            total_imports: all_imports.len(),
            internal_imports: internal_count,
            external_imports: external_count,
            unique_external_packages: unique_external_packages.len(),
            total_exports: all_exports.len(),
            circular_dependency_count: cycles.len(),
        };

        DependencyAnalysisResult {
            imports: all_imports,
            exports: all_exports,
            internal_dependencies,
            external_dependencies,
            unique_external_packages,
            cycles,
            stats,
        }
    }
}
