//! Import and export extraction for JavaScript, TypeScript, and TSX source files.

use std::path::Path;
use tree_sitter::Node;

use crate::dependencies::types::{ExportStatement, ImportKind, ImportSpecifier, ImportStatement};
use crate::parser::ParsedSource;

/// Extract all ES module imports, CommonJS requires, and exports from JS/TS/TSX files.
pub fn extract_javascript_imports(
    parsed: &ParsedSource,
) -> (Vec<ImportStatement>, Vec<ExportStatement>) {
    let file_path = parsed.file_path().unwrap_or(Path::new("unknown.js"));
    let mut imports = Vec::new();
    let mut exports = Vec::new();
    let root = parsed.root_node();

    extract_from_node(parsed, file_path, &root, &mut imports, &mut exports);

    (imports, exports)
}

fn extract_from_node(
    parsed: &ParsedSource,
    file_path: &Path,
    node: &Node,
    imports: &mut Vec<ImportStatement>,
    exports: &mut Vec<ExportStatement>,
) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "import_statement" => {
                if let Some(imp) = extract_import_statement(parsed, file_path, &child) {
                    imports.push(imp);
                }
            }
            "export_statement" => {
                extract_export_statement(parsed, file_path, &child, imports, exports);
            }
            "call_expression" => {
                // Check for dynamic require('...') or import('...')
                if let Some(imp) = extract_dynamic_import(parsed, file_path, &child) {
                    imports.push(imp);
                }
            }
            _ => {
                // Descend into expressions and blocks to catch dynamic requires
                extract_from_node(parsed, file_path, &child, imports, exports);
            }
        }
    }
}

fn extract_import_statement(
    parsed: &ParsedSource,
    file_path: &Path,
    node: &Node,
) -> Option<ImportStatement> {
    let source_node = node.child_by_field_name("source")?;
    let raw_path = clean_string_literal(parsed.node_text(&source_node));
    let span = parsed.node_span(node);

    let is_relative =
        raw_path.starts_with("./") || raw_path.starts_with("../") || raw_path.starts_with('/');

    let mut specifiers = Vec::new();
    let mut kind = ImportKind::Default;

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "import_clause" {
            let mut clause_cursor = child.walk();
            for clause_child in child.children(&mut clause_cursor) {
                match clause_child.kind() {
                    "identifier" => {
                        // Default import
                        specifiers
                            .push(ImportSpecifier::new(parsed.node_text(&clause_child).trim()));
                    }
                    "named_imports" => {
                        kind = ImportKind::Named;
                        extract_named_specifiers(parsed, &clause_child, &mut specifiers);
                    }
                    "namespace_import" => {
                        kind = ImportKind::Wildcard;
                        if let Some(alias_node) = clause_child.child_by_field_name("alias") {
                            specifiers.push(ImportSpecifier::with_alias(
                                "*",
                                parsed.node_text(&alias_node).trim(),
                            ));
                        } else {
                            specifiers.push(ImportSpecifier::new("*"));
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    if specifiers.is_empty() {
        kind = ImportKind::SideEffect;
    }

    Some(ImportStatement {
        raw_path,
        source_file: file_path.to_path_buf(),
        span,
        kind,
        specifiers,
        is_relative,
        resolved_target: None,
        is_external: !is_relative,
    })
}

fn extract_export_statement(
    parsed: &ParsedSource,
    file_path: &Path,
    node: &Node,
    imports: &mut Vec<ImportStatement>,
    exports: &mut Vec<ExportStatement>,
) {
    let span = parsed.node_span(node);
    let is_default = node.child_by_field_name("default").is_some()
        || parsed.node_text(node).contains("export default");

    if let Some(source_node) = node.child_by_field_name("source") {
        // Re-export: `export { a } from './b'` or `export * from './b'`
        let raw_path = clean_string_literal(parsed.node_text(&source_node));
        let is_relative = raw_path.starts_with("./") || raw_path.starts_with("../");

        // Also track as an import requirement
        imports.push(ImportStatement {
            raw_path: raw_path.clone(),
            source_file: file_path.to_path_buf(),
            span,
            kind: ImportKind::ReExport,
            specifiers: Vec::new(),
            is_relative,
            resolved_target: None,
            is_external: !is_relative,
        });

        exports.push(ExportStatement {
            source_file: file_path.to_path_buf(),
            span,
            symbol_name: None,
            is_default,
            reexport_from: Some(raw_path),
        });
    } else {
        // Standard export
        let mut symbol_name = None;
        if let Some(decl) = node.child_by_field_name("declaration") {
            if let Some(name_node) = decl.child_by_field_name("name") {
                symbol_name = Some(parsed.node_text(&name_node).trim().to_string());
            }
        }

        exports.push(ExportStatement {
            source_file: file_path.to_path_buf(),
            span,
            symbol_name,
            is_default,
            reexport_from: None,
        });
    }
}

fn extract_dynamic_import(
    parsed: &ParsedSource,
    file_path: &Path,
    node: &Node,
) -> Option<ImportStatement> {
    let fn_node = node.child_by_field_name("function")?;
    let fn_name = parsed.node_text(&fn_node).trim();

    if fn_name == "require" || fn_name == "import" {
        let args = node.child_by_field_name("arguments")?;
        let arg0 = args.child(1)?; // child 0 is '('
        if arg0.kind() == "string" {
            let raw_path = clean_string_literal(parsed.node_text(&arg0));
            let is_relative = raw_path.starts_with("./") || raw_path.starts_with("../");
            let span = parsed.node_span(node);

            return Some(ImportStatement {
                raw_path,
                source_file: file_path.to_path_buf(),
                span,
                kind: ImportKind::Dynamic,
                specifiers: Vec::new(),
                is_relative,
                resolved_target: None,
                is_external: !is_relative,
            });
        }
    }
    None
}

fn extract_named_specifiers(
    parsed: &ParsedSource,
    named_imports_node: &Node,
    specifiers: &mut Vec<ImportSpecifier>,
) {
    let mut cursor = named_imports_node.walk();
    for child in named_imports_node.children(&mut cursor) {
        if child.kind() == "import_specifier" {
            let name_node = match child.child_by_field_name("name") {
                Some(n) => n,
                None => continue,
            };
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

fn clean_string_literal(raw: &str) -> String {
    raw.trim().trim_matches(['\'', '"', '`']).to_string()
}
