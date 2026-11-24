//! Representation of parsed source code and its concrete syntax tree.

use std::path::{Path, PathBuf};
use tree_sitter::{Node, Tree};

use super::language::SupportedLanguage;
use super::span::SourceSpan;

/// Result of parsing a source file or text buffer into a Tree-sitter AST.
pub struct ParsedSource {
    /// Optional file path if loaded from disk.
    pub path: Option<PathBuf>,
    /// Language identified or selected for this source.
    pub language: SupportedLanguage,
    /// Preserved source code content.
    pub source: String,
    /// Tree-sitter parsed concrete syntax tree.
    pub tree: Tree,
    /// Indicates whether syntax errors (ERROR or MISSING nodes) were detected.
    pub has_syntax_errors: bool,
}

impl ParsedSource {
    /// Create a new `ParsedSource` instance.
    pub fn new(
        path: Option<PathBuf>,
        language: SupportedLanguage,
        source: String,
        tree: Tree,
    ) -> Self {
        let has_syntax_errors = tree.root_node().has_error();
        Self {
            path,
            language,
            source,
            tree,
            has_syntax_errors,
        }
    }

    /// Access the root node of the syntax tree.
    pub fn root_node(&self) -> Node<'_> {
        self.tree.root_node()
    }

    /// Extract the UTF-8 text slice corresponding to a Tree-sitter node.
    pub fn node_text<'a>(&'a self, node: &Node) -> &'a str {
        let start = node.start_byte();
        let end = node.end_byte();
        if end <= self.source.len() && start <= end {
            &self.source[start..end]
        } else {
            ""
        }
    }

    /// Compute the coordinate `SourceSpan` of a node.
    pub fn node_span(&self, node: &Node) -> SourceSpan {
        SourceSpan::from_node(node)
    }

    /// Path reference if available, or placeholder string.
    pub fn path_display(&self) -> String {
        self.path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "<memory>".to_string())
    }

    /// Associated path slice if present.
    pub fn file_path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Find all immediate or nested descendant nodes matching a given syntax kind.
    pub fn find_nodes_by_kind(&self, kind: &str) -> Vec<Node<'_>> {
        let mut results = Vec::new();
        let mut stack = vec![self.tree.root_node()];

        while let Some(current) = stack.pop() {
            if current.kind() == kind {
                results.push(current);
            }
            let count = current.child_count();
            for i in (0..count).rev() {
                if let Some(child) = current.child(i) {
                    stack.push(child);
                }
            }
        }

        results
    }

    /// Collect spans of all syntax error nodes in the AST.
    pub fn collect_error_spans(&self) -> Vec<SourceSpan> {
        let mut errors = Vec::new();
        if !self.has_syntax_errors {
            return errors;
        }

        let mut stack = vec![self.tree.root_node()];
        while let Some(node) = stack.pop() {
            if node.is_error() || node.is_missing() {
                errors.push(SourceSpan::from_node(&node));
            }
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.has_error() {
                    stack.push(child);
                }
            }
        }

        errors
    }

    /// Total line count of the source file.
    pub fn line_count(&self) -> usize {
        if self.source.is_empty() {
            return 0;
        }
        self.source.lines().count()
    }

    /// Byte length of the source.
    pub fn byte_len(&self) -> usize {
        self.source.len()
    }
}

impl std::fmt::Debug for ParsedSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParsedSource")
            .field("path", &self.path)
            .field("language", &self.language)
            .field("source_len", &self.source.len())
            .field("has_syntax_errors", &self.has_syntax_errors)
            .finish()
    }
}
