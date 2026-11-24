//! Import extraction for Java source files.

use std::path::Path;

use crate::dependencies::types::{ExportStatement, ImportKind, ImportSpecifier, ImportStatement};
use crate::parser::ParsedSource;

/// Extract all import statements from a Java source file.
pub fn extract_java_imports(parsed: &ParsedSource) -> (Vec<ImportStatement>, Vec<ExportStatement>) {
    let file_path = parsed.file_path().unwrap_or(Path::new("unknown.java"));
    let mut imports = Vec::new();
    let exports = Vec::new();
    let root = parsed.root_node();

    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if child.kind() == "import_declaration" {
            let span = parsed.node_span(&child);
            let text = parsed.node_text(&child).trim();

            let cleaned = text
                .trim_start_matches("import")
                .trim_start_matches("static")
                .trim()
                .trim_end_matches(';')
                .trim();

            let is_wildcard = cleaned.ends_with(".*");
            let kind = if is_wildcard {
                ImportKind::Wildcard
            } else {
                ImportKind::Default
            };

            let is_external = cleaned.starts_with("java.") || cleaned.starts_with("javax.");

            imports.push(ImportStatement {
                raw_path: cleaned.to_string(),
                source_file: file_path.to_path_buf(),
                span,
                kind,
                specifiers: vec![ImportSpecifier::new(cleaned)],
                is_relative: false,
                resolved_target: None,
                is_external,
            });
        }
    }

    (imports, exports)
}
