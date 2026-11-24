//! Symbol extraction for Go source files.

use std::path::Path;
use tree_sitter::Node;

use crate::parser::ParsedSource;
use crate::symbols::id::generate_symbol_id;
use crate::symbols::types::{ParameterInfo, Symbol, SymbolKind, Visibility};

/// Extract all functions, methods, structs, and interfaces from a parsed Go source file.
pub fn extract_go_symbols(parsed: &ParsedSource) -> Vec<Symbol> {
    let file_path = parsed.file_path().unwrap_or(Path::new("unknown.go"));
    let mut symbols = Vec::new();
    let root = parsed.root_node();

    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        match child.kind() {
            "function_declaration" => {
                if let Some(sym) = extract_function(parsed, file_path, &child) {
                    symbols.push(sym);
                }
            }
            "method_declaration" => {
                if let Some(sym) = extract_method(parsed, file_path, &child) {
                    symbols.push(sym);
                }
            }
            "type_declaration" => {
                extract_type_declaration(parsed, file_path, &child, &mut symbols);
            }
            "const_declaration" => {
                extract_const_declaration(parsed, file_path, &child, &mut symbols);
            }
            _ => {}
        }
    }

    symbols
}

fn extract_function(parsed: &ParsedSource, file_path: &Path, node: &Node) -> Option<Symbol> {
    let name_node = node.child_by_field_name("name")?;
    let name = parsed.node_text(&name_node).trim();

    let id = generate_symbol_id(file_path, name, SymbolKind::Function);
    let span = parsed.node_span(node);
    let visibility = go_visibility(name);

    let return_type = node
        .child_by_field_name("result")
        .map(|n| parsed.node_text(&n).trim().to_string());

    let parameters = extract_parameters(parsed, node);

    let signature = node.child_by_field_name("parameters").map(|p| {
        let p_text = parsed.node_text(&p);
        match &return_type {
            Some(ret) => format!("{} {}", p_text, ret),
            None => p_text.to_string(),
        }
    });

    Some(Symbol {
        id,
        name: name.to_string(),
        qualified_name: name.to_string(),
        kind: SymbolKind::Function,
        visibility,
        file_path: file_path.to_path_buf(),
        span,
        signature,
        return_type,
        parameters,
        parent_id: None,
        doc_comment: extract_go_doc_comment(parsed, node),
        is_async: false,
        is_static: false,
    })
}

fn extract_method(parsed: &ParsedSource, file_path: &Path, node: &Node) -> Option<Symbol> {
    let name_node = node.child_by_field_name("name")?;
    let name = parsed.node_text(&name_node).trim();

    let receiver_node = node.child_by_field_name("receiver")?;
    let receiver_text = parsed.node_text(&receiver_node).trim();
    // Clean receiver type e.g. "(s *Server)" -> "Server"
    let receiver_type = extract_receiver_type_name(receiver_text);

    let qualified_name = format!("{}.{}", receiver_type, name);
    let id = generate_symbol_id(file_path, &qualified_name, SymbolKind::Method);
    let span = parsed.node_span(node);
    let visibility = go_visibility(name);

    let return_type = node
        .child_by_field_name("result")
        .map(|n| parsed.node_text(&n).trim().to_string());

    let parameters = extract_parameters(parsed, node);

    let signature = node.child_by_field_name("parameters").map(|p| {
        let p_text = parsed.node_text(&p);
        match &return_type {
            Some(ret) => format!("{} {} {}", receiver_text, p_text, ret),
            None => format!("{} {}", receiver_text, p_text),
        }
    });

    let parent_id = Some(generate_symbol_id(
        file_path,
        &receiver_type,
        SymbolKind::Struct,
    ));

    Some(Symbol {
        id,
        name: name.to_string(),
        qualified_name,
        kind: SymbolKind::Method,
        visibility,
        file_path: file_path.to_path_buf(),
        span,
        signature,
        return_type,
        parameters,
        parent_id,
        doc_comment: extract_go_doc_comment(parsed, node),
        is_async: false,
        is_static: false,
    })
}

fn extract_type_declaration(
    parsed: &ParsedSource,
    file_path: &Path,
    node: &Node,
    symbols: &mut Vec<Symbol>,
) {
    let mut cursor = node.walk();
    for spec in node.children(&mut cursor) {
        if spec.kind() == "type_spec" {
            let name_node = match spec.child_by_field_name("name") {
                Some(n) => n,
                None => continue,
            };
            let name = parsed.node_text(&name_node).trim();

            let type_node = match spec.child_by_field_name("type") {
                Some(t) => t,
                None => continue,
            };

            let kind = match type_node.kind() {
                "struct_type" => SymbolKind::Struct,
                "interface_type" => SymbolKind::Interface,
                _ => SymbolKind::TypeAlias,
            };

            let id = generate_symbol_id(file_path, name, kind);
            let span = parsed.node_span(&spec);
            let visibility = go_visibility(name);

            symbols.push(Symbol {
                id,
                name: name.to_string(),
                qualified_name: name.to_string(),
                kind,
                visibility,
                file_path: file_path.to_path_buf(),
                span,
                signature: None,
                return_type: None,
                parameters: Vec::new(),
                parent_id: None,
                doc_comment: extract_go_doc_comment(parsed, node),
                is_async: false,
                is_static: false,
            });
        }
    }
}

fn extract_const_declaration(
    parsed: &ParsedSource,
    file_path: &Path,
    node: &Node,
    symbols: &mut Vec<Symbol>,
) {
    let mut cursor = node.walk();
    for spec in node.children(&mut cursor) {
        if spec.kind() == "const_spec" {
            if let Some(name_node) = spec.child_by_field_name("name") {
                let name = parsed.node_text(&name_node).trim();
                let id = generate_symbol_id(file_path, name, SymbolKind::Constant);
                let span = parsed.node_span(&spec);

                symbols.push(Symbol {
                    id,
                    name: name.to_string(),
                    qualified_name: name.to_string(),
                    kind: SymbolKind::Constant,
                    visibility: go_visibility(name),
                    file_path: file_path.to_path_buf(),
                    span,
                    signature: None,
                    return_type: spec
                        .child_by_field_name("type")
                        .map(|n| parsed.node_text(&n).trim().to_string()),
                    parameters: Vec::new(),
                    parent_id: None,
                    doc_comment: extract_go_doc_comment(parsed, node),
                    is_async: false,
                    is_static: false,
                });
            }
        }
    }
}

fn extract_parameters(parsed: &ParsedSource, node: &Node) -> Vec<ParameterInfo> {
    let mut params = Vec::new();
    if let Some(param_list) = node.child_by_field_name("parameters") {
        let mut cursor = param_list.walk();
        for child in param_list.children(&mut cursor) {
            if child.kind() == "parameter_declaration" {
                let name = child
                    .child_by_field_name("name")
                    .map(|n| parsed.node_text(&n).trim().to_string())
                    .unwrap_or_else(|| "_".to_string());

                let type_ann = child
                    .child_by_field_name("type")
                    .map(|n| parsed.node_text(&n).trim().to_string());

                params.push(ParameterInfo {
                    name,
                    type_annotation: type_ann,
                    default_value: None,
                });
            }
        }
    }
    params
}

fn go_visibility(name: &str) -> Visibility {
    if let Some(first) = name.chars().next() {
        if first.is_uppercase() {
            Visibility::Public
        } else {
            Visibility::Internal
        }
    } else {
        Visibility::Unknown
    }
}

fn extract_receiver_type_name(receiver_str: &str) -> String {
    let cleaned = receiver_str
        .trim_start_matches('(')
        .trim_end_matches(')')
        .trim();
    let parts: Vec<&str> = cleaned.split_whitespace().collect();
    let type_part = parts.last().copied().unwrap_or(cleaned);
    type_part.trim_start_matches('*').to_string()
}

fn extract_go_doc_comment(parsed: &ParsedSource, node: &Node) -> Option<String> {
    let mut prev = node.prev_sibling();
    let mut comments = Vec::new();

    while let Some(sibling) = prev {
        if sibling.kind() == "comment" {
            let text = parsed.node_text(&sibling).trim();
            comments.push(text.to_string());
            prev = sibling.prev_sibling();
        } else {
            break;
        }
    }

    if comments.is_empty() {
        None
    } else {
        comments.reverse();
        Some(comments.join("\n"))
    }
}
