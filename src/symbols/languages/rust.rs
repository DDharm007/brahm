//! Symbol extraction for Rust source files.

use std::path::Path;
use tree_sitter::Node;

use crate::parser::ParsedSource;
use crate::symbols::id::generate_symbol_id;
use crate::symbols::types::{ParameterInfo, Symbol, SymbolKind, Visibility};

/// Extract all declarations and symbols from a parsed Rust source file.
pub fn extract_rust_symbols(parsed: &ParsedSource) -> Vec<Symbol> {
    let file_path = parsed.file_path().unwrap_or(Path::new("unknown.rs"));
    let mut symbols = Vec::new();
    let root = parsed.root_node();

    // Track active impl blocks so methods can be associated with their target type
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
            "function_item" => {
                if let Some(sym) =
                    extract_function(parsed, file_path, &child, current_scope, parent_id)
                {
                    symbols.push(sym);
                }
            }
            "struct_item" => {
                if let Some(sym) = extract_struct(parsed, file_path, &child, current_scope) {
                    let struct_id = sym.id.clone();
                    let struct_name = sym.qualified_name.clone();
                    symbols.push(sym);
                    extract_nodes(
                        parsed,
                        file_path,
                        &child,
                        Some(&struct_name),
                        Some(&struct_id),
                        symbols,
                    );
                }
            }
            "enum_item" => {
                if let Some(sym) = extract_enum(parsed, file_path, &child, current_scope) {
                    let enum_id = sym.id.clone();
                    let enum_name = sym.qualified_name.clone();
                    symbols.push(sym);
                    extract_nodes(
                        parsed,
                        file_path,
                        &child,
                        Some(&enum_name),
                        Some(&enum_id),
                        symbols,
                    );
                }
            }
            "trait_item" => {
                if let Some(sym) = extract_trait(parsed, file_path, &child, current_scope) {
                    let trait_id = sym.id.clone();
                    let trait_name = sym.qualified_name.clone();
                    symbols.push(sym);
                    extract_nodes(
                        parsed,
                        file_path,
                        &child,
                        Some(&trait_name),
                        Some(&trait_id),
                        symbols,
                    );
                }
            }
            "impl_item" => {
                // Determine target type for the impl block
                let target_type = child
                    .child_by_field_name("type")
                    .map(|n| parsed.node_text(&n).trim().to_string())
                    .unwrap_or_else(|| "AnonymousImpl".to_string());

                let qualified_target = match current_scope {
                    Some(scope) => format!("{}::{}", scope, target_type),
                    None => target_type.clone(),
                };

                let impl_id = generate_symbol_id(file_path, &qualified_target, SymbolKind::Struct);

                // Traverse inside impl block, passing the type as scope and impl_id as parent
                if let Some(body) = child.child_by_field_name("body") {
                    extract_nodes(
                        parsed,
                        file_path,
                        &body,
                        Some(&qualified_target),
                        Some(&impl_id),
                        symbols,
                    );
                } else {
                    extract_nodes(
                        parsed,
                        file_path,
                        &child,
                        Some(&qualified_target),
                        Some(&impl_id),
                        symbols,
                    );
                }
            }
            "mod_item" => {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let mod_name = parsed.node_text(&name_node).trim();
                    let qualified = match current_scope {
                        Some(scope) => format!("{}::{}", scope, mod_name),
                        None => mod_name.to_string(),
                    };
                    let span = parsed.node_span(&child);
                    let id = generate_symbol_id(file_path, &qualified, SymbolKind::Module);
                    let vis = extract_visibility(parsed, &child);

                    symbols.push(Symbol {
                        id: id.clone(),
                        name: mod_name.to_string(),
                        qualified_name: qualified.clone(),
                        kind: SymbolKind::Module,
                        visibility: vis,
                        file_path: file_path.to_path_buf(),
                        span,
                        signature: None,
                        return_type: None,
                        parameters: Vec::new(),
                        parent_id: parent_id.map(|s| s.to_string()),
                        doc_comment: extract_doc_comment(parsed, &child),
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
            "type_item" => {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = parsed.node_text(&name_node).trim();
                    let qualified = match current_scope {
                        Some(scope) => format!("{}::{}", scope, name),
                        None => name.to_string(),
                    };
                    let span = parsed.node_span(&child);
                    let id = generate_symbol_id(file_path, &qualified, SymbolKind::TypeAlias);

                    symbols.push(Symbol {
                        id,
                        name: name.to_string(),
                        qualified_name: qualified,
                        kind: SymbolKind::TypeAlias,
                        visibility: extract_visibility(parsed, &child),
                        file_path: file_path.to_path_buf(),
                        span,
                        signature: None,
                        return_type: child
                            .child_by_field_name("type")
                            .map(|n| parsed.node_text(&n).trim().to_string()),
                        parameters: Vec::new(),
                        parent_id: parent_id.map(|s| s.to_string()),
                        doc_comment: extract_doc_comment(parsed, &child),
                        is_async: false,
                        is_static: false,
                    });
                }
            }
            "const_item" => {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = parsed.node_text(&name_node).trim();
                    let qualified = match current_scope {
                        Some(scope) => format!("{}::{}", scope, name),
                        None => name.to_string(),
                    };
                    let span = parsed.node_span(&child);
                    let id = generate_symbol_id(file_path, &qualified, SymbolKind::Constant);

                    symbols.push(Symbol {
                        id,
                        name: name.to_string(),
                        qualified_name: qualified,
                        kind: SymbolKind::Constant,
                        visibility: extract_visibility(parsed, &child),
                        file_path: file_path.to_path_buf(),
                        span,
                        signature: None,
                        return_type: child
                            .child_by_field_name("type")
                            .map(|n| parsed.node_text(&n).trim().to_string()),
                        parameters: Vec::new(),
                        parent_id: parent_id.map(|s| s.to_string()),
                        doc_comment: extract_doc_comment(parsed, &child),
                        is_async: false,
                        is_static: false,
                    });
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
    let name_node = node.child_by_field_name("name")?;
    let name = parsed.node_text(&name_node).trim();

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
    let visibility = extract_visibility(parsed, node);

    let return_type = node.child_by_field_name("return_type").map(|n| {
        parsed
            .node_text(&n)
            .trim()
            .trim_start_matches("->")
            .trim()
            .to_string()
    });

    let parameters = extract_parameters(parsed, node);

    let is_async = node_has_keyword(parsed, node, "async");

    let signature = node.child_by_field_name("parameters").map(|p| {
        let p_text = parsed.node_text(&p);
        match &return_type {
            Some(ret) => format!("{} -> {}", p_text, ret),
            None => p_text.to_string(),
        }
    });

    Some(Symbol {
        id,
        name: name.to_string(),
        qualified_name,
        kind,
        visibility,
        file_path: file_path.to_path_buf(),
        span,
        signature,
        return_type,
        parameters,
        parent_id: parent_id.map(|s| s.to_string()),
        doc_comment: extract_doc_comment(parsed, node),
        is_async,
        is_static: false,
    })
}

fn extract_struct(
    parsed: &ParsedSource,
    file_path: &Path,
    node: &Node,
    current_scope: Option<&str>,
) -> Option<Symbol> {
    let name_node = node.child_by_field_name("name")?;
    let name = parsed.node_text(&name_node).trim();

    let qualified_name = match current_scope {
        Some(scope) => format!("{}::{}", scope, name),
        None => name.to_string(),
    };

    let id = generate_symbol_id(file_path, &qualified_name, SymbolKind::Struct);
    let span = parsed.node_span(node);
    let visibility = extract_visibility(parsed, node);

    Some(Symbol {
        id,
        name: name.to_string(),
        qualified_name,
        kind: SymbolKind::Struct,
        visibility,
        file_path: file_path.to_path_buf(),
        span,
        signature: None,
        return_type: None,
        parameters: Vec::new(),
        parent_id: None,
        doc_comment: extract_doc_comment(parsed, node),
        is_async: false,
        is_static: false,
    })
}

fn extract_enum(
    parsed: &ParsedSource,
    file_path: &Path,
    node: &Node,
    current_scope: Option<&str>,
) -> Option<Symbol> {
    let name_node = node.child_by_field_name("name")?;
    let name = parsed.node_text(&name_node).trim();

    let qualified_name = match current_scope {
        Some(scope) => format!("{}::{}", scope, name),
        None => name.to_string(),
    };

    let id = generate_symbol_id(file_path, &qualified_name, SymbolKind::Enum);
    let span = parsed.node_span(node);
    let visibility = extract_visibility(parsed, node);

    Some(Symbol {
        id,
        name: name.to_string(),
        qualified_name,
        kind: SymbolKind::Enum,
        visibility,
        file_path: file_path.to_path_buf(),
        span,
        signature: None,
        return_type: None,
        parameters: Vec::new(),
        parent_id: None,
        doc_comment: extract_doc_comment(parsed, node),
        is_async: false,
        is_static: false,
    })
}

fn extract_trait(
    parsed: &ParsedSource,
    file_path: &Path,
    node: &Node,
    current_scope: Option<&str>,
) -> Option<Symbol> {
    let name_node = node.child_by_field_name("name")?;
    let name = parsed.node_text(&name_node).trim();

    let qualified_name = match current_scope {
        Some(scope) => format!("{}::{}", scope, name),
        None => name.to_string(),
    };

    let id = generate_symbol_id(file_path, &qualified_name, SymbolKind::Trait);
    let span = parsed.node_span(node);
    let visibility = extract_visibility(parsed, node);

    Some(Symbol {
        id,
        name: name.to_string(),
        qualified_name,
        kind: SymbolKind::Trait,
        visibility,
        file_path: file_path.to_path_buf(),
        span,
        signature: None,
        return_type: None,
        parameters: Vec::new(),
        parent_id: None,
        doc_comment: extract_doc_comment(parsed, node),
        is_async: false,
        is_static: false,
    })
}

fn extract_visibility(parsed: &ParsedSource, node: &Node) -> Visibility {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "visibility_modifier" {
                let text = parsed.node_text(&child).trim();
                if text.starts_with("pub(crate)") || text.starts_with("pub(super)") {
                    return Visibility::Internal;
                } else if text.starts_with("pub") {
                    return Visibility::Public;
                }
            }
        }
    }
    Visibility::Private
}

fn extract_parameters(parsed: &ParsedSource, node: &Node) -> Vec<ParameterInfo> {
    let mut params = Vec::new();
    if let Some(param_list) = node.child_by_field_name("parameters") {
        let mut cursor = param_list.walk();
        for child in param_list.children(&mut cursor) {
            if child.kind() == "parameter" {
                let name = child
                    .child_by_field_name("pattern")
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
            } else if child.kind() == "self_parameter" {
                params.push(ParameterInfo {
                    name: "self".to_string(),
                    type_annotation: None,
                    default_value: None,
                });
            }
        }
    }
    params
}

fn extract_doc_comment(parsed: &ParsedSource, node: &Node) -> Option<String> {
    let mut prev = node.prev_sibling();
    let mut comments = Vec::new();

    while let Some(sibling) = prev {
        if sibling.kind() == "line_comment" {
            let text = parsed.node_text(&sibling).trim();
            if text.starts_with("///") || text.starts_with("//!") {
                comments.push(text.to_string());
                prev = sibling.prev_sibling();
                continue;
            }
        } else if sibling.kind() == "block_comment" {
            let text = parsed.node_text(&sibling).trim();
            if text.starts_with("/**") {
                comments.push(text.to_string());
                prev = sibling.prev_sibling();
                continue;
            }
        }
        break;
    }

    if comments.is_empty() {
        None
    } else {
        comments.reverse();
        Some(comments.join("\n"))
    }
}

fn node_has_keyword(parsed: &ParsedSource, node: &Node, kw: &str) -> bool {
    let count = node.child_count();
    for i in 0..count {
        if let Some(child) = node.child(i) {
            if child.kind() == kw || parsed.node_text(&child).trim() == kw {
                return true;
            }
        }
    }
    false
}
