//! Concrete Tree-sitter parsing engine implementation.

use std::fs;
use std::path::Path;
use tracing::{debug, warn};
use tree_sitter::Parser;

use super::config::ParserConfig;
use super::error::ParserError;
use super::language::SupportedLanguage;
use super::parsed::ParsedSource;

/// Trait defining the syntax parsing contract for BRAHM.
pub trait LanguageParser: Send + Sync {
    /// Parse raw source code string using the specified language and configuration.
    fn parse_source(
        &self,
        source: &str,
        lang: SupportedLanguage,
        config: &ParserConfig,
    ) -> Result<ParsedSource, ParserError>;

    /// Read and parse a file from disk with optional explicit language override.
    fn parse_file(
        &self,
        path: &Path,
        lang: Option<SupportedLanguage>,
        config: &ParserConfig,
    ) -> Result<ParsedSource, ParserError>;
}

/// Core Tree-sitter based syntax analysis engine.
#[derive(Debug, Default, Clone)]
pub struct TreeSitterEngine;

impl TreeSitterEngine {
    /// Create a new `TreeSitterEngine`.
    pub fn new() -> Self {
        Self
    }

    /// Internal helper to configure a parser instance with a language and options.
    fn prepare_parser(
        lang: SupportedLanguage,
        config: &ParserConfig,
    ) -> Result<Parser, ParserError> {
        let mut parser = Parser::new();
        let ts_lang = lang.tree_sitter_language();

        if let Err(e) = parser.set_language(&ts_lang) {
            return Err(ParserError::LanguageConfigFailed {
                language: lang.name().to_string(),
                reason: format!("{:?}", e),
            });
        }

        #[allow(deprecated)]
        if let Some(timeout) = config.timeout_micros {
            parser.set_timeout_micros(timeout);
        }

        Ok(parser)
    }

    /// Read file content safely, verifying file size and encoding.
    pub fn read_file_safely(path: &Path, config: &ParserConfig) -> Result<String, ParserError> {
        let metadata = fs::metadata(path).map_err(|e| ParserError::IoError {
            path: path.to_path_buf(),
            source: e,
        })?;

        let size = metadata.len() as usize;
        if size > config.max_file_size {
            return Err(ParserError::FileTooLarge {
                path: path.to_path_buf(),
                size,
                max_size: config.max_file_size,
            });
        }

        let raw_bytes = fs::read(path).map_err(|e| ParserError::IoError {
            path: path.to_path_buf(),
            source: e,
        })?;

        match String::from_utf8(raw_bytes) {
            Ok(valid_utf8) => Ok(valid_utf8),
            Err(e) => {
                debug!(
                    "File '{}' contains invalid UTF-8; attempting lossy conversion",
                    path.display()
                );
                let lossy = String::from_utf8_lossy(e.as_bytes()).into_owned();
                Ok(lossy)
            }
        }
    }
}

impl LanguageParser for TreeSitterEngine {
    fn parse_source(
        &self,
        source: &str,
        lang: SupportedLanguage,
        config: &ParserConfig,
    ) -> Result<ParsedSource, ParserError> {
        if source.len() > config.max_file_size {
            return Err(ParserError::FileTooLarge {
                path: Path::new("<memory>").to_path_buf(),
                size: source.len(),
                max_size: config.max_file_size,
            });
        }

        let mut parser = Self::prepare_parser(lang, config)?;

        let tree = parser
            .parse(source, None)
            .ok_or(ParserError::NoAstReturned)?;

        let has_errors = tree.root_node().has_error();
        if has_errors && !config.allow_partial_parse {
            return Err(ParserError::ParseFailed {
                path: None,
                reason: "Syntax contains errors and allow_partial_parse is disabled".to_string(),
            });
        }

        let preserved_source = if config.preserve_source {
            source.to_string()
        } else {
            String::new()
        };

        Ok(ParsedSource::new(None, lang, preserved_source, tree))
    }

    fn parse_file(
        &self,
        path: &Path,
        lang: Option<SupportedLanguage>,
        config: &ParserConfig,
    ) -> Result<ParsedSource, ParserError> {
        let language = match lang {
            Some(l) => l,
            None => {
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or_default();
                SupportedLanguage::from_extension(ext).ok_or_else(|| {
                    ParserError::UnsupportedLanguage(format!(
                        "No supported language mapping for file '{}' (ext: '{}')",
                        path.display(),
                        ext
                    ))
                })?
            }
        };

        let content = Self::read_file_safely(path, config)?;
        let mut parser = Self::prepare_parser(language, config)?;

        let tree = parser
            .parse(&content, None)
            .ok_or_else(|| ParserError::TimeoutOrCancelled {
                path: path.to_path_buf(),
            })?;

        let has_errors = tree.root_node().has_error();
        if has_errors {
            if !config.allow_partial_parse {
                return Err(ParserError::ParseFailed {
                    path: Some(path.to_path_buf()),
                    reason: "File contains syntax errors and partial parsing is disabled"
                        .to_string(),
                });
            }
            warn!(
                "Syntax errors detected while parsing '{}' ({})",
                path.display(),
                language
            );
        }

        Ok(ParsedSource::new(
            Some(path.to_path_buf()),
            language,
            content,
            tree,
        ))
    }
}
