//! Domain models and types for import, export, and dependency analysis.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::parser::SourceSpan;

/// Category of import statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportKind {
    /// Explicit named symbols (e.g. `import { a, b } from 'c'`).
    Named,
    /// Default import (e.g. `import a from 'b'`).
    Default,
    /// Wildcard / glob import (e.g. `import * as a from 'b'`, `use crate::*`).
    Wildcard,
    /// Side-effect only import (e.g. `import './style.css'`).
    SideEffect,
    /// Dynamic runtime import (e.g. `require('a')`, `import('a')`).
    Dynamic,
    /// Re-export statement (e.g. `export { a } from 'b'`).
    ReExport,
}

impl ImportKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Named => "named",
            Self::Default => "default",
            Self::Wildcard => "wildcard",
            Self::SideEffect => "side_effect",
            Self::Dynamic => "dynamic",
            Self::ReExport => "re_export",
        }
    }
}

/// A specific imported or aliased symbol within an import statement.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ImportSpecifier {
    /// Original imported identifier name (or `*` for wildcard).
    pub name: String,
    /// Local alias if renamed (e.g. `import { foo as bar }`).
    pub alias: Option<String>,
}

impl ImportSpecifier {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            alias: None,
        }
    }

    pub fn with_alias(name: impl Into<String>, alias: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            alias: Some(alias.into()),
        }
    }

    /// Effective local name in scope.
    pub fn local_name(&self) -> &str {
        self.alias.as_deref().unwrap_or(&self.name)
    }
}

/// A statically detected import or dependency declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportStatement {
    /// Raw path string exactly as declared in the source code (e.g. `"./utils"`, `"react"`).
    pub raw_path: String,

    /// Source file containing this import.
    pub source_file: PathBuf,

    /// Coordinate span in the source file.
    pub span: SourceSpan,

    /// Import syntax classification.
    pub kind: ImportKind,

    /// Individual imported symbols if named.
    pub specifiers: Vec<ImportSpecifier>,

    /// Whether the import path syntactically refers to a relative local path.
    pub is_relative: bool,

    /// Resolved internal target file path if located within the repository.
    pub resolved_target: Option<PathBuf>,

    /// Whether this import resolves to an external library, package, or standard library.
    pub is_external: bool,
}

/// A statically detected export statement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportStatement {
    /// Source file containing this export.
    pub source_file: PathBuf,

    /// Coordinate span in the source file.
    pub span: SourceSpan,

    /// Name of the exported symbol if identifiable.
    pub symbol_name: Option<String>,

    /// Whether this is a default export (`export default ...`).
    pub is_default: bool,

    /// Source module path if this is a re-export (`export { a } from './b'`).
    pub reexport_from: Option<String>,
}

/// Represents a detected circular dependency cycle between repository files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyCycle {
    /// Sequence of file paths forming the circular loop (e.g. `[A, B, C, A]`).
    pub cycle: Vec<PathBuf>,
}

impl DependencyCycle {
    pub fn new(cycle: Vec<PathBuf>) -> Self {
        Self { cycle }
    }

    /// Number of distinct files involved in the cycle.
    pub fn length(&self) -> usize {
        if self.cycle.len() <= 1 {
            self.cycle.len()
        } else {
            self.cycle.len() - 1
        }
    }

    /// Format cycle as a human-readable arrow sequence: `a.ts -> b.ts -> a.ts`.
    pub fn display_path(&self, repo_root: Option<&Path>) -> String {
        let parts: Vec<String> = self
            .cycle
            .iter()
            .map(|p| {
                if let Some(root) = repo_root {
                    p.strip_prefix(root)
                        .unwrap_or(p)
                        .to_string_lossy()
                        .to_string()
                } else {
                    p.to_string_lossy().to_string()
                }
            })
            .collect();

        parts.join(" -> ")
    }
}
