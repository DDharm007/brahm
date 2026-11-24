//! Import extraction for Python source files.

use std::path::Path;
use tree_sitter::Node;

use crate::dependencies::types::{ExportStatement, ImportKind, ImportSpecifier, ImportStatement};
use crate::parser::ParsedSource;

/// Extract all import statements and module-level exposed symbols from a Python source file.
pub fn extract_python_imports(
    parsed: &ParsedSource,
) -> (Vec<ImportStatement>, Vec<ExportStatement>) {
    let file_path = parsed.file_path().unwrap_or(Path::new("unknown.py"));
    let mut imports = Vec::new();
    let exports = Vec::new(); // Python defines exports dynamically via __all__ or module-level defs
    let root = parsed.root_node();

    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        match child.kind() {
            "import_statement" => {
                extract_direct_imports(parsed, file_path, &child, &mut imports);
            }
            "import_from_statement" => {
                if let Some(imp) = extract_from_import(parsed, file_path, &child) {
                    imports.push(imp);
                }
            }
            _ => {}
        }
    }

    (imports, exports)
}

fn extract_direct_imports(
    parsed: &ParsedSource,
    file_path: &Path,
    node: &Node,
    imports: &mut Vec<ImportStatement>,
) {
    let span = parsed.node_span(node);
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        if child.kind() == "dotted_name" {
            let raw_path = parsed.node_text(&child).trim().to_string();
            let is_relative = raw_path.starts_with('.');

            imports.push(ImportStatement {
                raw_path: raw_path.clone(),
                source_file: file_path.to_path_buf(),
                span,
                kind: ImportKind::Default,
                specifiers: vec![ImportSpecifier::new(&raw_path)],
                is_relative,
                resolved_target: None,
                is_external: !is_relative,
            });
        } else if child.kind() == "aliased_import" {
            if let Some(orig_node) = child.child_by_field_name("name") {
                let raw_path = parsed.node_text(&orig_node).trim().to_string();
                let alias = child
                    .child_by_field_name("alias")
                    .map(|a| parsed.node_text(&a).trim().to_string());

                let is_relative = raw_path.starts_with('.');
                let spec = match alias {
                    Some(al) => ImportSpecifier::with_alias(&raw_path, al),
                    None => ImportSpecifier::new(&raw_path),
                };

                imports.push(ImportStatement {
                    raw_path: raw_path.clone(),
                    source_file: file_path.to_path_buf(),
                    span,
                    kind: ImportKind::Default,
                    specifiers: vec![spec],
                    is_relative,
                    resolved_target: None,
                    is_external: !is_relative,
                });
            }
        }
    }
}

fn extract_from_import(
    parsed: &ParsedSource,
    file_path: &Path,
    node: &Node,
) -> Option<ImportStatement> {
    let span = parsed.node_span(node);

    // Module name or relative dots
    let module_name = if let Some(mod_node) = node.child_by_field_name("module_name") {
        parsed.node_text(&mod_node).trim().to_string()
    } else {
        // Look for relative import dots e.g. `from . import foo`
        let mut dots = String::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "import_prefix" || parsed.node_text(&child).trim().starts_with('.') {
                dots.push_str(parsed.node_text(&child).trim());
            }
        }
        if !dots.is_empty() {
            dots
        } else {
            return None;
        }
    };

    let is_relative = module_name.starts_with('.');

    // Check wildcard import
    let mut kind = ImportKind::Named;
    let mut specifiers = Vec::new();

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "wildcard_import" {
            kind = ImportKind::Wildcard;
            specifiers.push(ImportSpecifier::new("*"));
        } else if child.kind() == "dotted_name"
            && node.child_by_field_name("module_name") != Some(child)
        {
            let name = parsed.node_text(&child).trim();
            specifiers.push(ImportSpecifier::new(name));
        } else if child.kind() == "aliased_import" {
            if let Some(name_node) = child.child_by_field_name("name") {
                let name = parsed.node_text(&name_node).trim();
                let alias = child
                    .child_by_field_name("alias")
                    .map(|a| parsed.node_text(&a).trim().to_string());

                if let Some(al) = alias {
                    specifiers.push(ImportSpecifier::with_alias(name, al));
                } else {
                    specifiers.push(ImportSpecifier::new(name));
                }
            }
        }
    }

    Some(ImportStatement {
        raw_path: module_name,
        source_file: file_path.to_path_buf(),
        span,
        kind,
        specifiers,
        is_relative,
        resolved_target: None,
        is_external: !is_relative,
    })
}
