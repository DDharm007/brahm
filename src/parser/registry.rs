//! Language Parser Registry and high-level dispatch coordinator.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tracing::debug;

use super::config::ParserConfig;
use super::engine::{LanguageParser, TreeSitterEngine};
use super::error::ParserError;
use super::language::SupportedLanguage;
use super::parsed::ParsedSource;

/// Central registry managing language detection, configuration, and AST parser dispatch.
#[derive(Clone)]
pub struct ParserRegistry {
    config: ParserConfig,
    engine: Arc<dyn LanguageParser>,
    extension_map: HashMap<String, SupportedLanguage>,
    name_map: HashMap<String, SupportedLanguage>,
}

impl Default for ParserRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ParserRegistry {
    /// Create a new `ParserRegistry` with default configuration and standard language mappings.
    pub fn new() -> Self {
        Self::with_config(ParserConfig::default())
    }

    /// Create a new `ParserRegistry` with custom parsing configuration.
    pub fn with_config(config: ParserConfig) -> Self {
        let mut extension_map = HashMap::new();
        let mut name_map = HashMap::new();

        for lang in SupportedLanguage::all() {
            for ext in lang.extensions() {
                extension_map.insert(ext.to_ascii_lowercase(), *lang);
            }
            name_map.insert(lang.name().to_ascii_lowercase(), *lang);
            name_map.insert(lang.display_name().to_ascii_lowercase(), *lang);
        }

        // Additional common aliases from Stage 01 scanner
        name_map.insert("c++".to_string(), SupportedLanguage::Cpp);
        name_map.insert("csharp".to_string(), SupportedLanguage::C); // fallback or ignore
        name_map.insert("golang".to_string(), SupportedLanguage::Go);

        Self {
            config,
            engine: Arc::new(TreeSitterEngine::new()),
            extension_map,
            name_map,
        }
    }

    /// Access the underlying configuration.
    pub fn config(&self) -> &ParserConfig {
        &self.config
    }

    /// Check if a given file extension is supported for AST parsing.
    pub fn is_supported_extension(&self, ext: &str) -> bool {
        let clean = ext.trim_start_matches('.').to_ascii_lowercase();
        self.extension_map.contains_key(&clean)
    }

    /// Check if a file path has an extension supported for AST parsing.
    pub fn is_supported_path(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|ext| self.is_supported_extension(ext))
            .unwrap_or(false)
    }

    /// Resolve a `SupportedLanguage` from file path and optional Stage 01 detected language name.
    pub fn detect_language(
        &self,
        path: &Path,
        detected_language: Option<&str>,
    ) -> Option<SupportedLanguage> {
        // Priority 1: Check explicit extension on path
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let clean = ext.trim_start_matches('.').to_ascii_lowercase();
            if let Some(lang) = self.extension_map.get(&clean) {
                return Some(*lang);
            }
        }

        // Priority 2: Check Stage 01 detected language name if provided
        if let Some(name) = detected_language {
            let clean = name.trim().to_ascii_lowercase();
            if let Some(lang) = self.name_map.get(&clean) {
                return Some(*lang);
            }
        }

        None
    }

    /// Parse a source file on disk using automatic language detection or explicit detected language.
    pub fn parse_file(
        &self,
        path: &Path,
        detected_language: Option<&str>,
    ) -> Result<ParsedSource, ParserError> {
        let lang = self
            .detect_language(path, detected_language)
            .ok_or_else(|| {
                ParserError::UnsupportedLanguage(format!(
                    "Cannot parse '{}': unsupported language or extension",
                    path.display()
                ))
            })?;

        debug!(
            "Parsing '{}' as {:?} (max size: {} KB)",
            path.display(),
            lang,
            self.config.max_file_size / 1024
        );

        self.engine.parse_file(path, Some(lang), &self.config)
    }

    /// Parse in-memory source code with an explicitly specified language.
    pub fn parse_source(
        &self,
        source: &str,
        lang: SupportedLanguage,
    ) -> Result<ParsedSource, ParserError> {
        self.engine.parse_source(source, lang, &self.config)
    }

    /// List all supported languages.
    pub fn supported_languages(&self) -> &'static [SupportedLanguage] {
        SupportedLanguage::all()
    }

    /// List all supported file extensions without leading dot.
    pub fn supported_extensions(&self) -> Vec<&'static str> {
        let mut exts = Vec::new();
        for lang in SupportedLanguage::all() {
            exts.extend_from_slice(lang.extensions());
        }
        exts.sort_unstable();
        exts.dedup();
        exts
    }
}
