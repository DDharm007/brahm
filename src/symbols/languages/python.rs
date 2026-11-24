//! Symbol extraction for Python source files.

use std::path::Path;
use tree_sitter::Node;

use crate::parser::ParsedSource;
use crate::symbols::id::generate_symbol_id;
use crate::symbols::types::{ParameterInfo, Symbol, SymbolKind, Visibility};

/// Extract all functions, methods, classes, and types from a parsed Python source file.
pub fn extract_python_symbols(parsed: &ParsedSource) -> Vec<Symbol> {
    let file_path = parsed.file_path().unwrap_or(Path::new("unknown.py"));
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
                    extract_function(parsed, file_path, &child, current_scope, parent_id, false)
                {
                    symbols.push(sym);
                }
            }
            "decorated_definition" => {
                // Decorated class or function
                if let Some(inner) = child.child_by_field_name("definition") {
                    if inner.kind() == "function_definition" {
                        if let Some(sym) = extract_function(
                            parsed,
                            file_path,
                            &inner,
                            current_scope,
                            parent_id,
                            true,
                        ) {
                            symbols.push(sym);
                        }
                    } else if inner.kind() == "class_definition" {
                        if let Some(sym) = extract_class(parsed, file_path, &inner, current_scope) {
                            let class_id = sym.id.clone();
                            let class_name = sym.qualified_name.clone();
                            symbols.push(sym);

                            if let Some(body) = inner.child_by_field_name("body") {
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
                }
            }
            "class_definition" => {
                if let Some(sym) = extract_class(parsed, file_path, &child, current_scope) {
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
    _is_decorated: bool,
) -> Option<Symbol> {
    let name_node = node.child_by_field_name("name")?;
    let name = parsed.node_text(&name_node).trim();

    let qualified_name = match current_scope {
        Some(scope) => format!("{}.{}", scope, name),
        None => name.to_string(),
    };

    let is_method = current_scope.is_some();
    let is_constructor = name == "__init__";
    let kind = if is_constructor {
        SymbolKind::Constructor
    } else if is_method {
        SymbolKind::Method
    } else {
        SymbolKind::Function
    };

    let id = generate_symbol_id(file_path, &qualified_name, kind);
    let span = parsed.node_span(node);

    let visibility = if name.starts_with("__") && !name.ends_with("__") {
        Visibility::Private
    } else if name.starts_with('_') && !name.ends_with("__") {
        Visibility::Internal
    } else {
        Visibility::Public
    };

    let return_type = node.child_by_field_name("return_type").map(|n| {
        parsed
            .node_text(&n)
            .trim()
            .trim_start_matches("->")
            .trim()
            .to_string()
    });

    let parameters = extract_parameters(parsed, node);

    // Check async keyword
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
        doc_comment: extract_python_docstring(parsed, node),
        is_async,
        is_static: false,
    })
}

fn extract_class(
    parsed: &ParsedSource,
    file_path: &Path,
    node: &Node,
    current_scope: Option<&str>,
) -> Option<Symbol> {
    let name_node = node.child_by_field_name("name")?;
    let name = parsed.node_text(&name_node).trim();

    let qualified_name = match current_scope {
        Some(scope) => format!("{}.{}", scope, name),
        None => name.to_string(),
    };

    let id = generate_symbol_id(file_path, &qualified_name, SymbolKind::Class);
    let span = parsed.node_span(node);

    let visibility = if name.starts_with('_') {
        Visibility::Internal
    } else {
        Visibility::Public
    };

    Some(Symbol {
        id,
        name: name.to_string(),
        qualified_name,
        kind: SymbolKind::Class,
        visibility,
        file_path: file_path.to_path_buf(),
        span,
        signature: None,
        return_type: None,
        parameters: Vec::new(),
        parent_id: None,
        doc_comment: extract_python_docstring(parsed, node),
        is_async: false,
        is_static: false,
    })
}

fn extract_parameters(parsed: &ParsedSource, node: &Node) -> Vec<ParameterInfo> {
    let mut params = Vec::new();
    if let Some(param_list) = node.child_by_field_name("parameters") {
        let mut cursor = param_list.walk();
        for child in param_list.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    params.push(ParameterInfo::new(parsed.node_text(&child).trim()));
                }
                "typed_parameter" | "typed_default_parameter" => {
                    let name = child
                        .child(0)
                        .map(|n| parsed.node_text(&n).trim().to_string())
                        .unwrap_or_else(|| "_".to_string());

                    let type_ann = child
                        .child_by_field_name("type")
                        .map(|n| parsed.node_text(&n).trim().to_string());

                    let default_val = child
                        .child_by_field_name("value")
                        .map(|n| parsed.node_text(&n).trim().to_string());

                    params.push(ParameterInfo {
                        name,
                        type_annotation: type_ann,
                        default_value: default_val,
                    });
                }
                "default_parameter" => {
                    let name = child
                        .child_by_field_name("name")
                        .map(|n| parsed.node_text(&n).trim().to_string())
                        .unwrap_or_else(|| "_".to_string());

                    let default_val = child
                        .child_by_field_name("value")
                        .map(|n| parsed.node_text(&n).trim().to_string());

                    params.push(ParameterInfo {
                        name,
                        type_annotation: None,
                        default_value: default_val,
                    });
                }
                _ => {}
            }
        }
    }
    params
}

fn extract_python_docstring(parsed: &ParsedSource, node: &Node) -> Option<String> {
    let body = node.child_by_field_name("body")?;
    let mut cursor = body.walk();
    if let Some(child) = body.children(&mut cursor).next() {
        if child.kind() == "expression_statement" {
            if let Some(str_node) = child.child(0) {
                if str_node.kind() == "string" {
                    let raw = parsed.node_text(&str_node).trim();
                    let cleaned = raw
                        .trim_start_matches("\"\"\"")
                        .trim_start_matches("'''")
                        .trim_end_matches("\"\"\"")
                        .trim_end_matches("'''")
                        .trim();
                    return Some(cleaned.to_string());
                }
            }
        }
    }
    None
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
