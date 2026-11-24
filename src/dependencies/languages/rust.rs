//! Import extraction for Rust source files.

use std::path::Path;
use tree_sitter::Node;

use crate::dependencies::types::{ExportStatement, ImportKind, ImportSpecifier, ImportStatement};
use crate::parser::ParsedSource;

/// Extract all `use` and external `mod` declarations from a Rust source file.
pub fn extract_rust_imports(parsed: &ParsedSource) -> (Vec<ImportStatement>, Vec<ExportStatement>) {
    let file_path = parsed.file_path().unwrap_or(Path::new("unknown.rs"));
    let mut imports = Vec::new();
    let mut exports = Vec::new();
    let root = parsed.root_node();

    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        match child.kind() {
            "use_declaration" => {
                if let Some(imp) = extract_use_declaration(parsed, file_path, &child) {
                    let is_pub = node_is_pub(parsed, &child);
                    if is_pub {
                        // Public use is a re-export
                        exports.push(ExportStatement {
                            source_file: file_path.to_path_buf(),
                            span: parsed.node_span(&child),
                            symbol_name: None,
                            is_default: false,
                            reexport_from: Some(imp.raw_path.clone()),
                        });
                    }
                    imports.push(imp);
                }
            }
            "mod_item" => {
                // mod foo; without a body is an internal file import
                if child.child_by_field_name("body").is_none() {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = parsed.node_text(&name_node).trim();
                        let raw_path = format!("self::{}", name);
                        let span = parsed.node_span(&child);

                        imports.push(ImportStatement {
                            raw_path,
                            source_file: file_path.to_path_buf(),
                            span,
                            kind: ImportKind::Default,
                            specifiers: vec![ImportSpecifier::new(name)],
                            is_relative: true,
                            resolved_target: None,
                            is_external: false,
                        });
                    }
                }
            }
            _ => {}
        }
    }

    (imports, exports)
}

fn extract_use_declaration(
    parsed: &ParsedSource,
    file_path: &Path,
    node: &Node,
) -> Option<ImportStatement> {
    let span = parsed.node_span(node);
    let raw_text = parsed.node_text(node).trim();

    // Strip "pub ", "pub(crate) ", "use ", ";"
    let cleaned = raw_text
        .trim_start_matches("pub(crate)")
        .trim_start_matches("pub(super)")
        .trim_start_matches("pub")
        .trim()
        .trim_start_matches("use")
        .trim()
        .trim_end_matches(';')
        .trim();

    let is_relative = cleaned.starts_with("crate::")
        || cleaned.starts_with("super::")
        || cleaned.starts_with("self::");

    let kind = if cleaned.ends_with("::*") {
        ImportKind::Wildcard
    } else {
        ImportKind::Named
    };

    let specifiers = extract_rust_specifiers(cleaned);

    Some(ImportStatement {
        raw_path: cleaned.to_string(),
        source_file: file_path.to_path_buf(),
        span,
        kind,
        specifiers,
        is_relative,
        resolved_target: None,
        is_external: !is_relative,
    })
}

fn extract_rust_specifiers(use_str: &str) -> Vec<ImportSpecifier> {
    let mut specifiers = Vec::new();

    if let Some(open) = use_str.find('{') {
        if let Some(close) = use_str.rfind('}') {
            let inner = &use_str[open + 1..close];
            for part in inner.split(',') {
                let p = part.trim();
                if !p.is_empty() {
                    if p.contains(" as ") {
                        let mut tokens = p.split(" as ");
                        let orig = tokens.next().unwrap_or("").trim();
                        let alias = tokens.next().unwrap_or("").trim();
                        specifiers.push(ImportSpecifier::with_alias(orig, alias));
                    } else {
                        specifiers.push(ImportSpecifier::new(p));
                    }
                }
            }
            return specifiers;
        }
    }

    let last_part = use_str.split("::").last().unwrap_or(use_str).trim();
    if last_part.contains(" as ") {
        let mut tokens = last_part.split(" as ");
        let orig = tokens.next().unwrap_or("").trim();
        let alias = tokens.next().unwrap_or("").trim();
        specifiers.push(ImportSpecifier::with_alias(orig, alias));
    } else if !last_part.is_empty() {
        specifiers.push(ImportSpecifier::new(last_part));
    }

    specifiers
}

fn node_is_pub(parsed: &ParsedSource, node: &Node) -> bool {
    let text = parsed.node_text(node);
    text.starts_with("pub")
}
