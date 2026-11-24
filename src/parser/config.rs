//! Configuration options for AST parsing.

use serde::{Deserialize, Serialize};

/// Default maximum file size permitted for AST parsing (2 MB).
pub const DEFAULT_MAX_PARSE_FILE_SIZE: usize = 2 * 1024 * 1024;

/// Configuration governing parsing behavior and safety limits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParserConfig {
    /// Maximum file size in bytes to attempt parsing.
    /// Files exceeding this limit are skipped with a `ParserError::FileTooLarge`
    /// to avoid runaway memory consumption or DOS on minified/bundled code.
    pub max_file_size: usize,

    /// Optional timeout in microseconds per parse operation.
    pub timeout_micros: Option<u64>,

    /// Whether to allow parsing files with syntax errors (tree-sitter produces partial error nodes).
    pub allow_partial_parse: bool,

    /// Whether to retain the full source code in `ParsedSource` for extracting node text.
    pub preserve_source: bool,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            max_file_size: DEFAULT_MAX_PARSE_FILE_SIZE,
            timeout_micros: Some(5_000_000), // 5 seconds default timeout
            allow_partial_parse: true,
            preserve_source: true,
        }
    }
}

impl ParserConfig {
    /// Create a new `ParserConfig` with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set custom maximum file size in bytes.
    pub fn with_max_file_size(mut self, size: usize) -> Self {
        self.max_file_size = size;
        self
    }

    /// Set timeout in microseconds.
    pub fn with_timeout_micros(mut self, timeout: Option<u64>) -> Self {
        self.timeout_micros = timeout;
        self
    }

    /// Set partial parse allowance.
    pub fn with_allow_partial(mut self, allow: bool) -> Self {
        self.allow_partial_parse = allow;
        self
    }
}
