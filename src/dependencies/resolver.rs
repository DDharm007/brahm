//! Import path resolution engine.
//!
//! Resolves relative and module import paths to actual repository files across
//! JavaScript/TypeScript, Python, Rust, Go, Java, and C/C++.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::parser::SupportedLanguage;

/// Outcome of attempting to resolve an import path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionResult {
    /// Resolves to an internal file path within the scanned repository.
    Internal(PathBuf),
    /// Resolves to an external dependency, package, or standard library module.
    External(String),
    /// Import could not be definitively resolved to a file or known package.
    Unresolved(String),
}

/// Path resolver that caches known repository file paths for fast in-memory resolution.
#[derive(Debug, Clone)]
pub struct PathResolver {
    repo_root: PathBuf,
    known_files: HashSet<PathBuf>,
}

impl PathResolver {
    /// Create a new resolver with the repository root.
    pub fn new(repo_root: impl Into<PathBuf>) -> Self {
        Self {
            repo_root: repo_root.into(),
            known_files: HashSet::new(),
        }
    }

    /// Populate known files in the repository for fast in-memory matching.
    pub fn with_known_files(mut self, files: impl IntoIterator<Item = PathBuf>) -> Self {
        self.known_files = files.into_iter().collect();
        self
    }

    /// Add a known file to the resolver.
    pub fn add_known_file(&mut self, file: PathBuf) {
        self.known_files.insert(file);
    }

    /// Resolve an import statement relative to its source file and language.
    pub fn resolve_import(
        &self,
        source_file: &Path,
        raw_path: &str,
        language: SupportedLanguage,
    ) -> ResolutionResult {
        let trimmed = raw_path.trim().trim_matches(['\'', '"', ';', '<', '>']);

        match language {
            SupportedLanguage::JavaScript
            | SupportedLanguage::TypeScript
            | SupportedLanguage::Tsx => self.resolve_javascript(source_file, trimmed),

            SupportedLanguage::Python => self.resolve_python(source_file, trimmed),

            SupportedLanguage::Rust => self.resolve_rust(source_file, trimmed),

            SupportedLanguage::Go => self.resolve_go(source_file, trimmed),

            SupportedLanguage::Java => self.resolve_java(source_file, trimmed),

            SupportedLanguage::C | SupportedLanguage::Cpp => {
                self.resolve_c_cpp(source_file, trimmed, raw_path)
            }
        }
    }

    /// Resolve JS / TS / TSX imports (e.g. `./utils`, `../models/user`, `react`).
    fn resolve_javascript(&self, source_file: &Path, raw: &str) -> ResolutionResult {
        if raw.starts_with("./") || raw.starts_with("../") || raw.starts_with('/') {
            let base_dir = source_file.parent().unwrap_or(&self.repo_root);
            let candidate_base = base_dir.join(raw);

            // Candidate extensions and index variations
            let candidates = [
                candidate_base.clone(),
                candidate_base.with_extension("ts"),
                candidate_base.with_extension("tsx"),
                candidate_base.with_extension("js"),
                candidate_base.with_extension("jsx"),
                candidate_base.with_extension("mjs"),
                candidate_base.join("index.ts"),
                candidate_base.join("index.tsx"),
                candidate_base.join("index.js"),
                candidate_base.join("index.jsx"),
            ];

            for candidate in &candidates {
                if self.file_exists(candidate) {
                    return ResolutionResult::Internal(self.normalize_path(candidate));
                }
            }

            ResolutionResult::Unresolved(raw.to_string())
        } else {
            // Package import (e.g. `react`, `@types/node`, `express`)
            let pkg_name = if raw.starts_with('@') {
                // Scoped package e.g. `@org/package`
                raw.split('/').take(2).collect::<Vec<&str>>().join("/")
            } else {
                raw.split('/').next().unwrap_or(raw).to_string()
            };
            ResolutionResult::External(pkg_name)
        }
    }

    /// Resolve Python imports (e.g. `.utils`, `..config`, `requests`, `os`).
    fn resolve_python(&self, source_file: &Path, raw: &str) -> ResolutionResult {
        if raw.starts_with('.') {
            // Relative import
            let base_dir = source_file.parent().unwrap_or(&self.repo_root);
            let dot_count = raw.chars().take_while(|&c| c == '.').count();
            let remainder = &raw[dot_count..];

            let mut target_dir = base_dir.to_path_buf();
            for _ in 1..dot_count {
                if let Some(parent) = target_dir.parent() {
                    target_dir = parent.to_path_buf();
                }
            }

            let candidate_path = if remainder.is_empty() {
                target_dir
            } else {
                let sub_path = remainder.replace('.', "/");
                target_dir.join(sub_path)
            };

            let candidates = [
                candidate_path.with_extension("py"),
                candidate_path.join("__init__.py"),
            ];

            for candidate in &candidates {
                if self.file_exists(candidate) {
                    return ResolutionResult::Internal(self.normalize_path(candidate));
                }
            }

            ResolutionResult::Unresolved(raw.to_string())
        } else {
            // Check if bare module matches a top-level file or directory in repo
            let module_first = raw.split('.').next().unwrap_or(raw);
            let candidates = [
                self.repo_root.join(format!("{}.py", module_first)),
                self.repo_root.join(module_first).join("__init__.py"),
                self.repo_root
                    .join("src")
                    .join(format!("{}.py", module_first)),
                self.repo_root
                    .join("src")
                    .join(module_first)
                    .join("__init__.py"),
            ];

            for candidate in &candidates {
                if self.file_exists(candidate) {
                    return ResolutionResult::Internal(self.normalize_path(candidate));
                }
            }

            ResolutionResult::External(module_first.to_string())
        }
    }

    /// Resolve Rust imports (e.g. `crate::parser`, `super::types`, `tokio`).
    fn resolve_rust(&self, source_file: &Path, raw: &str) -> ResolutionResult {
        if let Some(sub) = raw.strip_prefix("crate::") {
            let relative = sub.replace("::", "/");
            let src_dir = self.find_rust_src_dir(source_file);

            let candidates = [
                src_dir.join(format!("{}.rs", relative)),
                src_dir.join(&relative).join("mod.rs"),
            ];

            for candidate in &candidates {
                if self.file_exists(candidate) {
                    return ResolutionResult::Internal(self.normalize_path(candidate));
                }
            }

            ResolutionResult::Unresolved(raw.to_string())
        } else if let Some(sub) = raw.strip_prefix("super::") {
            let base_dir = source_file.parent().unwrap_or(&self.repo_root);
            let parent_dir = base_dir.parent().unwrap_or(base_dir);
            let relative = sub.replace("::", "/");

            let candidates = [
                parent_dir.join(format!("{}.rs", relative)),
                parent_dir.join(&relative).join("mod.rs"),
            ];

            for candidate in &candidates {
                if self.file_exists(candidate) {
                    return ResolutionResult::Internal(self.normalize_path(candidate));
                }
            }

            ResolutionResult::Unresolved(raw.to_string())
        } else if let Some(sub) = raw.strip_prefix("self::") {
            let base_dir = source_file.parent().unwrap_or(&self.repo_root);
            let relative = sub.replace("::", "/");

            let candidates = [
                base_dir.join(format!("{}.rs", relative)),
                base_dir.join(&relative).join("mod.rs"),
            ];

            for candidate in &candidates {
                if self.file_exists(candidate) {
                    return ResolutionResult::Internal(self.normalize_path(candidate));
                }
            }

            ResolutionResult::Unresolved(raw.to_string())
        } else {
            // External crate or std
            let crate_name = raw.split("::").next().unwrap_or(raw);
            ResolutionResult::External(crate_name.to_string())
        }
    }

    /// Resolve Go imports.
    fn resolve_go(&self, _source_file: &Path, raw: &str) -> ResolutionResult {
        if raw.contains('/') {
            // Check if it's an internal project package or external URL
            ResolutionResult::External(raw.to_string())
        } else {
            // Standard library package (e.g. `fmt`, `os`, `net`)
            ResolutionResult::External(raw.to_string())
        }
    }

    /// Resolve Java package imports (e.g. `com.brahm.core.*`, `java.util.List`).
    fn resolve_java(&self, _source_file: &Path, raw: &str) -> ResolutionResult {
        if raw.starts_with("java.") || raw.starts_with("javax.") {
            ResolutionResult::External("java.stdlib".to_string())
        } else {
            // Check internal Java package directory structure
            let rel_path = raw.replace('.', "/");
            let candidate_dir = self
                .repo_root
                .join("src")
                .join("main")
                .join("java")
                .join(&rel_path);

            if self.file_exists(&candidate_dir.with_extension("java")) {
                return ResolutionResult::Internal(
                    self.normalize_path(&candidate_dir.with_extension("java")),
                );
            }

            let top_pkg = raw.split('.').take(2).collect::<Vec<&str>>().join(".");
            ResolutionResult::External(top_pkg)
        }
    }

    /// Resolve C / C++ includes.
    fn resolve_c_cpp(&self, source_file: &Path, trimmed: &str, raw_path: &str) -> ResolutionResult {
        let is_quoted = raw_path.contains('"');
        if is_quoted {
            // Quoted include `#include "header.h"` is relative to current file or include dir
            let base_dir = source_file.parent().unwrap_or(&self.repo_root);
            let candidates = [
                base_dir.join(trimmed),
                self.repo_root.join("include").join(trimmed),
                self.repo_root.join("src").join(trimmed),
                self.repo_root.join(trimmed),
            ];

            for candidate in &candidates {
                if self.file_exists(candidate) {
                    return ResolutionResult::Internal(self.normalize_path(candidate));
                }
            }

            ResolutionResult::Unresolved(trimmed.to_string())
        } else {
            // Angle-bracket `#include <stdio.h>` is external/system
            ResolutionResult::External(trimmed.to_string())
        }
    }

    /// Helper to find the nearest `src` directory for Rust projects.
    fn find_rust_src_dir(&self, file_path: &Path) -> PathBuf {
        let mut curr = file_path.parent();
        while let Some(dir) = curr {
            if dir.ends_with("src") {
                return dir.to_path_buf();
            }
            if dir.join("Cargo.toml").exists() {
                return dir.join("src");
            }
            curr = dir.parent();
        }
        self.repo_root.join("src")
    }

    /// Verify whether a candidate path exists either in the known files index or on disk.
    fn file_exists(&self, path: &Path) -> bool {
        let norm = self.normalize_path(path);
        if self.known_files.contains(&norm) {
            return true;
        }
        path.is_file()
    }

    /// Normalize paths for consistent cross-platform representation.
    pub fn normalize_path(&self, path: &Path) -> PathBuf {
        if let Ok(canonical) = path.canonicalize() {
            canonical
        } else {
            path.to_path_buf()
        }
    }
}
