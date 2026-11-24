//! Include extraction for C and C++ source files.

use std::path::Path;
use tree_sitter::Node;

use crate::dependencies::types::{ExportStatement, ImportKind, ImportSpecifier, ImportStatement};
use crate::parser::ParsedSource;

/// Extract all `#include` directives from C and C++ source files.
pub fn extract_c_cpp_imports(
    parsed: &ParsedSource,
) -> (Vec<ImportStatement>, Vec<ExportStatement>) {
    let file_path = parsed.file_path().unwrap_or(Path::new("unknown.cpp"));
    let mut imports = Vec::new();
    let exports = Vec::new();
    let root = parsed.root_node();

    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if child.kind() == "preproc_include" {
            if let Some(imp) = extract_include(parsed, file_path, &child) {
                imports.push(imp);
            }
        }
    }

    (imports, exports)
}

fn extract_include(
    parsed: &ParsedSource,
    file_path: &Path,
    node: &Node,
) -> Option<ImportStatement> {
    let path_node = node.child_by_field_name("path")?;
    let raw = parsed.node_text(&path_node).trim();
    let span = parsed.node_span(node);

    let is_quoted = raw.starts_with('"') && raw.ends_with('"');
    let cleaned = raw.trim_matches(['"', '<', '>']).to_string();

    Some(ImportStatement {
        raw_path: cleaned.clone(),
        source_file: file_path.to_path_buf(),
        span,
        kind: ImportKind::Default,
        specifiers: vec![ImportSpecifier::new(&cleaned)],
        is_relative: is_quoted,
        resolved_target: None,
        is_external: !is_quoted,
    })
}
