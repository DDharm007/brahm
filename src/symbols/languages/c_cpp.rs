//! Symbol extraction for C and C++ source files.

use std::path::Path;
use tree_sitter::Node;

use crate::parser::ParsedSource;
use crate::symbols::id::generate_symbol_id;
use crate::symbols::types::{Symbol, SymbolKind, Visibility};

/// Extract functions, classes, structs, enums, and namespaces from parsed C / C++ source code.
pub fn extract_c_cpp_symbols(parsed: &ParsedSource) -> Vec<Symbol> {
    let file_path = parsed.file_path().unwrap_or(Path::new("unknown.cpp"));
    let mut symbols = Vec::new();
    let root = parsed.root_node();

    extract_nodes(parsed, file_path, &root, None, None, &mut symbols);

    symbols
}

fn extract_nodes(
    parsed: &ParsedSource,
    file_path: &Path,
    node: &Node,
    current_scope: Option<&str>,
    parent_id: Option<&str>,
    symbols: &mut Vec<Symbol>,
) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "function_definition" => {
                if let Some(sym) =
                    extract_function(parsed, file_path, &child, current_scope, parent_id)
                {
                    symbols.push(sym);
                }
            }
            "class_specifier" => {
                if let Some(sym) = extract_class_specifier(
                    parsed,
                    file_path,
                    &child,
                    current_scope,
                    SymbolKind::Class,
                ) {
                    let class_id = sym.id.clone();
                    let class_name = sym.qualified_name.clone();
                    symbols.push(sym);

                    if let Some(body) = child.child_by_field_name("body") {
                        extract_nodes(
                            parsed,
                            file_path,
                            &body,
                            Some(&class_name),
                            Some(&class_id),
                            symbols,
                        );
                    }
                }
            }
            "struct_specifier" => {
                if let Some(sym) = extract_class_specifier(
                    parsed,
                    file_path,
                    &child,
                    current_scope,
                    SymbolKind::Struct,
                ) {
                    let struct_id = sym.id.clone();
                    let struct_name = sym.qualified_name.clone();
                    symbols.push(sym);

                    if let Some(body) = child.child_by_field_name("body") {
                        extract_nodes(
                            parsed,
                            file_path,
                            &body,
                            Some(&struct_name),
                            Some(&struct_id),
                            symbols,
                        );
                    }
                }
            }
            "enum_specifier" => {
                if let Some(sym) = extract_enum_specifier(parsed, file_path, &child, current_scope)
                {
                    symbols.push(sym);
                }
            }
            "namespace_definition" => {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let ns_name = parsed.node_text(&name_node).trim();
                    let qualified = match current_scope {
                        Some(scope) => format!("{}::{}", scope, ns_name),
                        None => ns_name.to_string(),
                    };
                    let span = parsed.node_span(&child);
                    let id = generate_symbol_id(file_path, &qualified, SymbolKind::Module);

                    symbols.push(Symbol {
                        id: id.clone(),
                        name: ns_name.to_string(),
                        qualified_name: qualified.clone(),
                        kind: SymbolKind::Module,
                        visibility: Visibility::Public,
                        file_path: file_path.to_path_buf(),
                        span,
                        signature: None,
                        return_type: None,
                        parameters: Vec::new(),
                        parent_id: parent_id.map(|s| s.to_string()),
                        doc_comment: extract_doxygen_comment(parsed, &child),
                        is_async: false,
                        is_static: false,
                    });

                    if let Some(body) = child.child_by_field_name("body") {
                        extract_nodes(
                            parsed,
                            file_path,
                            &body,
                            Some(&qualified),
                            Some(&id),
                            symbols,
                        );
                    }
                }
            }
            "type_definition" => {
                if let Some(sym) = extract_typedef(parsed, file_path, &child, current_scope) {
                    symbols.push(sym);
                }
            }
            _ => {}
        }
    }
}

fn extract_function(
    parsed: &ParsedSource,
    file_path: &Path,
    node: &Node,
    current_scope: Option<&str>,
    parent_id: Option<&str>,
) -> Option<Symbol> {
    let declarator = node.child_by_field_name("declarator")?;
    let (name, params_text) = find_declarator_name_and_params(parsed, &declarator)?;

    let qualified_name = match current_scope {
        Some(scope) => format!("{}::{}", scope, name),
        None => name.to_string(),
    };

    let kind = if current_scope.is_some() {
        SymbolKind::Method
    } else {
        SymbolKind::Function
    };

    let id = generate_symbol_id(file_path, &qualified_name, kind);
    let span = parsed.node_span(node);

    let return_type = node
        .child_by_field_name("type")
        .map(|n| parsed.node_text(&n).trim().to_string());

    let signature = match &return_type {
        Some(ret) => format!("{} {}{}", ret, name, params_text),
        None => format!("{}{}", name, params_text),
    };

    Some(Symbol {
        id,
        name: name.to_string(),
        qualified_name,
        kind,
        visibility: Visibility::Public,
        file_path: file_path.to_path_buf(),
        span,
        signature: Some(signature),
        return_type,
        parameters: Vec::new(),
        parent_id: parent_id.map(|s| s.to_string()),
        doc_comment: extract_doxygen_comment(parsed, node),
        is_async: false,
        is_static: false,
    })
}

fn extract_class_specifier(
    parsed: &ParsedSource,
    file_path: &Path,
    node: &Node,
    current_scope: Option<&str>,
    kind: SymbolKind,
) -> Option<Symbol> {
    let name_node = node.child_by_field_name("name")?;
    let name = parsed.node_text(&name_node).trim();
    if name.is_empty() {
        return None;
    }

    let qualified_name = match current_scope {
        Some(scope) => format!("{}::{}", scope, name),
        None => name.to_string(),
    };

    let id = generate_symbol_id(file_path, &qualified_name, kind);
    let span = parsed.node_span(node);

    Some(Symbol {
        id,
        name: name.to_string(),
        qualified_name,
        kind,
        visibility: Visibility::Public,
        file_path: file_path.to_path_buf(),
        span,
        signature: None,
        return_type: None,
        parameters: Vec::new(),
        parent_id: None,
        doc_comment: extract_doxygen_comment(parsed, node),
        is_async: false,
        is_static: false,
    })
}

fn extract_enum_specifier(
    parsed: &ParsedSource,
    file_path: &Path,
    node: &Node,
    current_scope: Option<&str>,
) -> Option<Symbol> {
    let name_node = node.child_by_field_name("name")?;
    let name = parsed.node_text(&name_node).trim();
    if name.is_empty() {
        return None;
    }

    let qualified_name = match current_scope {
        Some(scope) => format!("{}::{}", scope, name),
        None => name.to_string(),
    };

    let id = generate_symbol_id(file_path, &qualified_name, SymbolKind::Enum);
    let span = parsed.node_span(node);

    Some(Symbol {
        id,
        name: name.to_string(),
        qualified_name,
        kind: SymbolKind::Enum,
        visibility: Visibility::Public,
        file_path: file_path.to_path_buf(),
        span,
        signature: None,
        return_type: None,
        parameters: Vec::new(),
        parent_id: None,
        doc_comment: extract_doxygen_comment(parsed, node),
        is_async: false,
        is_static: false,
    })
}

fn extract_typedef(
    parsed: &ParsedSource,
    file_path: &Path,
    node: &Node,
    current_scope: Option<&str>,
) -> Option<Symbol> {
    let declarator = node.child_by_field_name("declarator")?;
    let name = parsed.node_text(&declarator).trim();
    if name.is_empty() {
        return None;
    }

    let qualified_name = match current_scope {
        Some(scope) => format!("{}::{}", scope, name),
        None => name.to_string(),
    };

    let id = generate_symbol_id(file_path, &qualified_name, SymbolKind::TypeAlias);
    let span = parsed.node_span(node);

    Some(Symbol {
        id,
        name: name.to_string(),
        qualified_name,
        kind: SymbolKind::TypeAlias,
        visibility: Visibility::Public,
        file_path: file_path.to_path_buf(),
        span,
        signature: None,
        return_type: node
            .child_by_field_name("type")
            .map(|n| parsed.node_text(&n).trim().to_string()),
        parameters: Vec::new(),
        parent_id: None,
        doc_comment: extract_doxygen_comment(parsed, node),
        is_async: false,
        is_static: false,
    })
}

fn find_declarator_name_and_params<'a>(
    parsed: &'a ParsedSource,
    declarator: &Node,
) -> Option<(&'a str, String)> {
    if declarator.kind() == "function_declarator" {
        let name_node = declarator.child_by_field_name("declarator")?;
        let name = parsed.node_text(&name_node).trim();
        let params_text = declarator
            .child_by_field_name("parameters")
            .map(|p| parsed.node_text(&p).to_string())
            .unwrap_or_else(|| "()".to_string());
        return Some((name, params_text));
    }

    // Direct identifier or nested pointer declarator
    let mut curr = *declarator;
    while let Some(child) = curr.child_by_field_name("declarator") {
        if child.kind() == "function_declarator" {
            return find_declarator_name_and_params(parsed, &child);
        }
        curr = child;
    }

    let text = parsed.node_text(&curr).trim();
    if !text.is_empty() {
        Some((text, "()".to_string()))
    } else {
        None
    }
}

fn extract_doxygen_comment(parsed: &ParsedSource, node: &Node) -> Option<String> {
    let mut prev = node.prev_sibling();
    while let Some(sibling) = prev {
        if sibling.kind() == "comment" {
            let text = parsed.node_text(&sibling).trim();
            if text.starts_with("/**") || text.starts_with("///") {
                return Some(text.to_string());
            }
        }
        prev = sibling.prev_sibling();
    }
    None
}
