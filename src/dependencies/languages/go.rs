//! Import extraction for Go source files.

use std::path::Path;
use tree_sitter::Node;

use crate::dependencies::types::{ExportStatement, ImportKind, ImportSpecifier, ImportStatement};
use crate::parser::ParsedSource;

/// Extract all `import` declarations from a Go source file.
pub fn extract_go_imports(parsed: &ParsedSource) -> (Vec<ImportStatement>, Vec<ExportStatement>) {
    let file_path = parsed.file_path().unwrap_or(Path::new("unknown.go"));
    let mut imports = Vec::new();
    let exports = Vec::new();
    let root = parsed.root_node();

    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if child.kind() == "import_declaration" {
            extract_import_declaration(parsed, file_path, &child, &mut imports);
        }
    }

    (imports, exports)
}

fn extract_import_declaration(
    parsed: &ParsedSource,
    file_path: &Path,
    node: &Node,
    imports: &mut Vec<ImportStatement>,
) {
    let span = parsed.node_span(node);
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        if child.kind() == "import_spec" {
            if let Some(path_node) = child.child_by_field_name("path") {
                let raw_path = clean_string_literal(parsed.node_text(&path_node));
                let alias = child
                    .child_by_field_name("name")
                    .map(|n| parsed.node_text(&n).trim().to_string());

                let spec = match alias {
                    Some(al) => ImportSpecifier::with_alias(&raw_path, al),
                    None => ImportSpecifier::new(&raw_path),
                };

                imports.push(ImportStatement {
                    raw_path,
                    source_file: file_path.to_path_buf(),
                    span,
                    kind: ImportKind::Default,
                    specifiers: vec![spec],
                    is_relative: false,
                    resolved_target: None,
                    is_external: true,
                });
            }
        } else if child.kind() == "import_spec_list" {
            let mut list_cursor = child.walk();
            for item in child.children(&mut list_cursor) {
                if item.kind() == "import_spec" {
                    if let Some(path_node) = item.child_by_field_name("path") {
                        let raw_path = clean_string_literal(parsed.node_text(&path_node));
                        let alias = item
                            .child_by_field_name("name")
                            .map(|n| parsed.node_text(&n).trim().to_string());

                        let spec = match alias {
                            Some(al) => ImportSpecifier::with_alias(&raw_path, al),
                            None => ImportSpecifier::new(&raw_path),
                        };

                        imports.push(ImportStatement {
                            raw_path,
                            source_file: file_path.to_path_buf(),
                            span,
                            kind: ImportKind::Default,
                            specifiers: vec![spec],
                            is_relative: false,
                            resolved_target: None,
                            is_external: true,
                        });
                    }
                }
            }
        }
    }
}

fn clean_string_literal(raw: &str) -> String {
    raw.trim().trim_matches(['\'', '"', '`']).to_string()
}
