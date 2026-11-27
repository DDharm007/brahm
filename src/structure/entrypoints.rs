//! Detection of application entry points and execution landmarks.

use std::path::Path;

use super::types::{EntryPoint, EntryPointKind};
use crate::parser::ParsedSource;
use crate::symbols::types::SymbolKind;
use crate::symbols::SymbolIndex;

/// Detector responsible for locating verified and inferred application entry points.
#[derive(Debug, Default, Clone)]
pub struct EntryPointDetector;

impl EntryPointDetector {
    pub fn new() -> Self {
        Self
    }

    /// Detect entry points from a single parsed file and its symbol index.
    pub fn detect_file_entrypoints(
        &self,
        parsed: &ParsedSource,
        symbols: &SymbolIndex,
    ) -> Vec<EntryPoint> {
        let mut entrypoints = Vec::new();
        let file_path = parsed.file_path().unwrap_or(Path::new("unknown"));
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        // 1. Check for standard `main` functions in symbols
        for sym in symbols.get_by_name("main") {
            if sym.kind == SymbolKind::Function || sym.kind == SymbolKind::Method {
                let kind = if file_name.contains("cli") {
                    EntryPointKind::CliEntry
                } else {
                    EntryPointKind::Executable
                };

                entrypoints.push(EntryPoint {
                    file_path: file_path.to_path_buf(),
                    kind,
                    symbol_name: Some("main".to_string()),
                    span: Some(sym.span),
                    description: format!("Standard {} function entry point", sym.kind),
                    confidence: 1.0,
                });
            }
        }

        // 2. Python-specific `if __name__ == '__main__':` pattern
        if parsed.source.contains("__main__") && parsed.source.contains("__name__") {
            let already_detected = entrypoints
                .iter()
                .any(|e| e.symbol_name.as_deref() == Some("main"));
            if !already_detected {
                entrypoints.push(EntryPoint {
                    file_path: file_path.to_path_buf(),
                    kind: EntryPointKind::Executable,
                    symbol_name: Some("__main__ block".to_string()),
                    span: None,
                    description: "Python '__main__' execution block".to_string(),
                    confidence: 0.95,
                });
            }
        }

        // 3. File naming conventions & roles
        if file_name == "lib.rs" {
            entrypoints.push(EntryPoint {
                file_path: file_path.to_path_buf(),
                kind: EntryPointKind::LibraryRoot,
                symbol_name: None,
                span: None,
                description: "Rust crate root library definition".to_string(),
                confidence: 1.0,
            });
        } else if file_name == "server.js" || file_name == "server.ts" {
            entrypoints.push(EntryPoint {
                file_path: file_path.to_path_buf(),
                kind: EntryPointKind::ServerEntry,
                symbol_name: None,
                span: None,
                description: "Web server startup module".to_string(),
                confidence: 0.9,
            });
        } else if file_name == "wsgi.py" || file_name == "asgi.py" {
            entrypoints.push(EntryPoint {
                file_path: file_path.to_path_buf(),
                kind: EntryPointKind::ServerEntry,
                symbol_name: None,
                span: None,
                description: "Python WSGI/ASGI web application gateway entry point".to_string(),
                confidence: 0.95,
            });
        } else if file_name == "manage.py" {
            entrypoints.push(EntryPoint {
                file_path: file_path.to_path_buf(),
                kind: EntryPointKind::CliEntry,
                symbol_name: None,
                span: None,
                description: "Django management CLI entry point".to_string(),
                confidence: 1.0,
            });
        }

        entrypoints
    }
}
