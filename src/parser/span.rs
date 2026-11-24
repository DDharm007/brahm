//! Source span and coordinate representations for parsed ASTs.

use serde::{Deserialize, Serialize};

/// Precise source code span identifying a region in a file.
///
/// Lines and columns are 1-based for consistent developer-facing representation.
/// Byte offsets are 0-based.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceSpan {
    /// 1-based starting line number.
    pub start_line: usize,
    /// 1-based starting column number.
    pub start_col: usize,
    /// 1-based ending line number.
    pub end_line: usize,
    /// 1-based ending column number.
    pub end_col: usize,
    /// 0-based starting byte offset.
    pub start_byte: usize,
    /// 0-based ending byte offset (exclusive).
    pub end_byte: usize,
}

impl SourceSpan {
    /// Create a new SourceSpan from explicit coordinates.
    pub fn new(
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
        start_byte: usize,
        end_byte: usize,
    ) -> Self {
        Self {
            start_line,
            start_col,
            end_line,
            end_col,
            start_byte,
            end_byte,
        }
    }

    /// Construct a `SourceSpan` from a Tree-sitter node.
    pub fn from_node(node: &tree_sitter::Node) -> Self {
        let start_point = node.start_position();
        let end_point = node.end_position();

        Self {
            start_line: start_point.row + 1,
            start_col: start_point.column + 1,
            end_line: end_point.row + 1,
            end_col: end_point.column + 1,
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
        }
    }

    /// Number of lines spanned.
    pub fn line_count(&self) -> usize {
        (self.end_line + 1).saturating_sub(self.start_line)
    }

    /// Byte length of the span.
    pub fn byte_len(&self) -> usize {
        self.end_byte.saturating_sub(self.start_byte)
    }
}

impl std::fmt::Display for SourceSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}-{}:{}",
            self.start_line, self.start_col, self.end_line, self.end_col
        )
    }
}
