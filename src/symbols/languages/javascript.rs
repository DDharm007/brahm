//! Symbol extraction for JavaScript, TypeScript, and TSX source files.

use std::path::Path;
use tree_sitter::Node;

use crate::parser::ParsedSource;
use crate::symbols::id::generate_symbol_id;
use crate::symbols::types::{ParameterInfo, Symbol, SymbolKind, Visibility};

/// Extract all declarations and symbols from a parsed JS/TS/TSX source file.
pub fn extract_javascript_symbols(parsed: &ParsedSource) -> Vec<Symbol> {
    let file_path = parsed.file_path().unwrap_or(Path::new("unknown.js"));
    let mut symbols = Vec::new();
    let root = parsed.root_node();

    extract_nodes(parsed, file_path, &root, None, None, false, &mut symbols);

    symbols
}

fn extract_nodes(
    parsed: &ParsedSource,
    file_path: &Path,
    node: &Node,
    current_scope: Option<&str>,
    parent_id: Option<&str>,
    is_exported_context: bool,
    symbols: &mut Vec<Symbol>,
) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "export_statement" => {
                // Descend into export statement with is_exported_context = true
                extract_nodes(
                    parsed,
                    file_path,
                    &child,
                    current_scope,
                    parent_id,
                    true,
                    symbols,
                );
            }
            "function_declaration" => {
                if let Some(sym) = extract_function(
                    parsed,
                    file_path,
                    &child,
                    current_scope,
                    parent_id,
                    is_exported_context,
                ) {
                    symbols.push(sym);
                }
            }
            "class_declaration" => {
                if let Some(sym) = extract_class(
                    parsed,
                    file_path,
                    &child,
                    current_scope,
                    is_exported_context,
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
                            false,
                            symbols,
                        );
                    }
                }
            }
            "method_definition" => {
                if let Some(sym) =
                    extract_method(parsed, file_path, &child, current_scope, parent_id)
                {
                    symbols.push(sym);
                }
            }
            "interface_declaration" => {
                if let Some(sym) = extract_interface(
                    parsed,
                    file_path,
                    &child,
                    current_scope,
                    is_exported_context,
                ) {
                    symbols.push(sym);
                }
            }
            "type_alias_declaration" => {
                if let Some(sym) = extract_type_alias(
                    parsed,
                    file_path,
                    &child,
                    current_scope,
                    is_exported_context,
                ) {
                    symbols.push(sym);
                }
            }
            "enum_declaration" => {
                if let Some(sym) = extract_enum(
                    parsed,
                    file_path,
                    &child,
                    current_scope,
                    is_exported_context,
                ) {
                    symbols.push(sym);
                }
            }
            "lexical_declaration" | "variable_declaration" => {
                extract_variable_declarations(
                    parsed,
                    file_path,
                    &child,
                    current_scope,
                    parent_id,
                    is_exported_context,
                    symbols,
                );
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
    is_exported: bool,
) -> Option<Symbol> {
    let name_node = node.child_by_field_name("name")?;
    let name = parsed.node_text(&name_node).trim();

    let qualified_name = match current_scope {
        Some(scope) => format!("{}.{}", scope, name),
        None => name.to_string(),
    };

    let id = generate_symbol_id(file_path, &qualified_name, SymbolKind::Function);
    let span = parsed.node_span(node);
    let visibility = if is_exported {
        Visibility::Public
    } else {
        Visibility::Internal
    };

    let return_type = extract_js_return_type(parsed, node);
    let parameters = extract_parameters(parsed, node);
    let is_async = node_has_async_keyword(parsed, node);

    let signature = find_js_parameters_node(node).map(|p| {
        let p_text = parsed.node_text(&p);
        match &return_type {
            Some(ret) => format!("{}: {}", p_text, ret),
            None => p_text.to_string(),
        }
    });

    Some(Symbol {
        id,
        name: name.to_string(),
        qualified_name,
        kind: SymbolKind::Function,
        visibility,
        file_path: file_path.to_path_buf(),
        span,
        signature,
        return_type,
        parameters,
        parent_id: parent_id.map(|s| s.to_string()),
        doc_comment: extract_jsdoc_comment(parsed, node),
        is_async,
        is_static: false,
    })
}

fn extract_class(
    parsed: &ParsedSource,
    file_path: &Path,
    node: &Node,
    current_scope: Option<&str>,
    is_exported: bool,
) -> Option<Symbol> {
    let name_node = node.child_by_field_name("name")?;
    let name = parsed.node_text(&name_node).trim();

    let qualified_name = match current_scope {
        Some(scope) => format!("{}.{}", scope, name),
        None => name.to_string(),
    };

    let id = generate_symbol_id(file_path, &qualified_name, SymbolKind::Class);
    let span = parsed.node_span(node);
    let visibility = if is_exported {
        Visibility::Public
    } else {
        Visibility::Internal
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
        doc_comment: extract_jsdoc_comment(parsed, node),
        is_async: false,
        is_static: false,
    })
}

fn extract_method(
    parsed: &ParsedSource,
    file_path: &Path,
    node: &Node,
    current_scope: Option<&str>,
    parent_id: Option<&str>,
) -> Option<Symbol> {
    let name_node = node.child_by_field_name("name")?;
    let name = parsed.node_text(&name_node).trim();

    let is_constructor = name == "constructor";
    let kind = if is_constructor {
        SymbolKind::Constructor
    } else {
        SymbolKind::Method
    };

    let qualified_name = match current_scope {
        Some(scope) => format!("{}.{}", scope, name),
        None => name.to_string(),
    };

    let id = generate_symbol_id(file_path, &qualified_name, kind);
    let span = parsed.node_span(node);

    let is_static = node_has_keyword(parsed, node, "static");
    let is_async = node_has_async_keyword(parsed, node);

    // Private in JS methods starts with '#' or visibility modifier
    let visibility = if name.starts_with('#') || node_has_keyword(parsed, node, "private") {
        Visibility::Private
    } else if node_has_keyword(parsed, node, "protected") {
        Visibility::Protected
    } else {
        Visibility::Public
    };

    let return_type = extract_js_return_type(parsed, node);
    let parameters = extract_parameters(parsed, node);

    let signature = find_js_parameters_node(node).map(|p| {
        let p_text = parsed.node_text(&p);
        match &return_type {
            Some(ret) => format!("{}: {}", p_text, ret),
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
        doc_comment: extract_jsdoc_comment(parsed, node),
        is_async,
        is_static,
    })
}

fn extract_interface(
    parsed: &ParsedSource,
    file_path: &Path,
    node: &Node,
    current_scope: Option<&str>,
    is_exported: bool,
) -> Option<Symbol> {
    let name_node = node.child_by_field_name("name")?;
    let name = parsed.node_text(&name_node).trim();

    let qualified_name = match current_scope {
        Some(scope) => format!("{}.{}", scope, name),
        None => name.to_string(),
    };

    let id = generate_symbol_id(file_path, &qualified_name, SymbolKind::Interface);
    let span = parsed.node_span(node);
    let visibility = if is_exported {
        Visibility::Public
    } else {
        Visibility::Internal
    };

    Some(Symbol {
        id,
        name: name.to_string(),
        qualified_name,
        kind: SymbolKind::Interface,
        visibility,
        file_path: file_path.to_path_buf(),
        span,
        signature: None,
        return_type: None,
        parameters: Vec::new(),
        parent_id: None,
        doc_comment: extract_jsdoc_comment(parsed, node),
        is_async: false,
        is_static: false,
    })
}

fn extract_type_alias(
    parsed: &ParsedSource,
    file_path: &Path,
    node: &Node,
    current_scope: Option<&str>,
    is_exported: bool,
) -> Option<Symbol> {
    let name_node = node.child_by_field_name("name")?;
    let name = parsed.node_text(&name_node).trim();

    let qualified_name = match current_scope {
        Some(scope) => format!("{}.{}", scope, name),
        None => name.to_string(),
    };

    let id = generate_symbol_id(file_path, &qualified_name, SymbolKind::TypeAlias);
    let span = parsed.node_span(node);
    let visibility = if is_exported {
        Visibility::Public
    } else {
        Visibility::Internal
    };

    Some(Symbol {
        id,
        name: name.to_string(),
        qualified_name,
        kind: SymbolKind::TypeAlias,
        visibility,
        file_path: file_path.to_path_buf(),
        span,
        signature: None,
        return_type: node
            .child_by_field_name("value")
            .map(|n| parsed.node_text(&n).trim().to_string()),
        parameters: Vec::new(),
        parent_id: None,
        doc_comment: extract_jsdoc_comment(parsed, node),
        is_async: false,
        is_static: false,
    })
}

fn extract_enum(
    parsed: &ParsedSource,
    file_path: &Path,
    node: &Node,
    current_scope: Option<&str>,
    is_exported: bool,
) -> Option<Symbol> {
    let name_node = node.child_by_field_name("name")?;
    let name = parsed.node_text(&name_node).trim();

    let qualified_name = match current_scope {
        Some(scope) => format!("{}.{}", scope, name),
        None => name.to_string(),
    };

    let id = generate_symbol_id(file_path, &qualified_name, SymbolKind::Enum);
    let span = parsed.node_span(node);
    let visibility = if is_exported {
        Visibility::Public
    } else {
        Visibility::Internal
    };

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
        doc_comment: extract_jsdoc_comment(parsed, node),
        is_async: false,
        is_static: false,
    })
}

fn extract_variable_declarations(
    parsed: &ParsedSource,
    file_path: &Path,
    node: &Node,
    current_scope: Option<&str>,
    parent_id: Option<&str>,
    is_exported: bool,
    symbols: &mut Vec<Symbol>,
) {
    let mut cursor = node.walk();
    for decl in node.children(&mut cursor) {
        if decl.kind() == "variable_declarator" {
            let name_node = match decl.child_by_field_name("name") {
                Some(n) => n,
                None => continue,
            };
            let name = parsed.node_text(&name_node).trim();

            let val_node = match decl.child_by_field_name("value") {
                Some(v) => v,
                None => continue,
            };

            let qualified_name = match current_scope {
                Some(scope) => format!("{}.{}", scope, name),
                None => name.to_string(),
            };

            // Detect arrow functions or anonymous functions assigned to variables
            if val_node.kind() == "arrow_function" || val_node.kind() == "function" {
                let id = generate_symbol_id(file_path, &qualified_name, SymbolKind::Function);
                let span = parsed.node_span(&decl);
                let is_async = node_has_async_keyword(parsed, &val_node);
                let parameters = extract_parameters(parsed, &val_node);
                let return_type = extract_js_return_type(parsed, &val_node);

                let signature = find_js_parameters_node(&val_node).map(|p| {
                    let p_text = parsed.node_text(&p);
                    match &return_type {
                        Some(ret) => format!("{}: {}", p_text, ret),
                        None => p_text.to_string(),
                    }
                });

                symbols.push(Symbol {
                    id,
                    name: name.to_string(),
                    qualified_name,
                    kind: SymbolKind::Function,
                    visibility: if is_exported {
                        Visibility::Public
                    } else {
                        Visibility::Internal
                    },
                    file_path: file_path.to_path_buf(),
                    span,
                    signature,
                    return_type,
                    parameters,
                    parent_id: parent_id.map(|s| s.to_string()),
                    doc_comment: extract_jsdoc_comment(parsed, node),
                    is_async,
                    is_static: false,
                });
            } else if is_exported {
                // If it's an exported constant
                let id = generate_symbol_id(file_path, &qualified_name, SymbolKind::Constant);
                let span = parsed.node_span(&decl);

                symbols.push(Symbol {
                    id,
                    name: name.to_string(),
                    qualified_name,
                    kind: SymbolKind::Constant,
                    visibility: Visibility::Public,
                    file_path: file_path.to_path_buf(),
                    span,
                    signature: None,
                    return_type: decl.child_by_field_name("type").map(|n| {
                        parsed
                            .node_text(&n)
                            .trim()
                            .trim_start_matches(':')
                            .trim()
                            .to_string()
                    }),
                    parameters: Vec::new(),
                    parent_id: parent_id.map(|s| s.to_string()),
                    doc_comment: extract_jsdoc_comment(parsed, node),
                    is_async: false,
                    is_static: false,
                });
            }
        }
    }
}

fn extract_parameters(parsed: &ParsedSource, node: &Node) -> Vec<ParameterInfo> {
    let mut params = Vec::new();
    if let Some(param_list) = find_js_parameters_node(node) {
        let mut cursor = param_list.walk();
        for child in param_list.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    params.push(ParameterInfo::new(parsed.node_text(&child).trim()));
                }
                "required_parameter" | "optional_parameter" => {
                    let name = child
                        .child_by_field_name("pattern")
                        .map(|n| parsed.node_text(&n).trim().to_string())
                        .unwrap_or_else(|| {
                            child
                                .child(0)
                                .map(|n| parsed.node_text(&n).trim().to_string())
                                .unwrap_or_else(|| "_".to_string())
                        });

                    let type_ann = extract_js_return_type(parsed, &child);

                    params.push(ParameterInfo {
                        name,
                        type_annotation: type_ann,
                        default_value: None,
                    });
                }
                _ => {}
            }
        }
    }
    params
}

fn extract_js_return_type(parsed: &ParsedSource, node: &Node) -> Option<String> {
    if let Some(n) = node.child_by_field_name("return_type") {
        return Some(
            parsed
                .node_text(&n)
                .trim()
                .trim_start_matches(':')
                .trim()
                .to_string(),
        );
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "type_annotation" {
                return Some(
                    parsed
                        .node_text(&child)
                        .trim()
                        .trim_start_matches(':')
                        .trim()
                        .to_string(),
                );
            }
        }
    }
    None
}

fn find_js_parameters_node<'a>(node: &'a Node) -> Option<Node<'a>> {
    if let Some(p) = node.child_by_field_name("parameters") {
        return Some(p);
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "formal_parameters" || child.kind() == "parameters" {
                return Some(child);
            }
        }
    }
    None
}

fn extract_jsdoc_comment(parsed: &ParsedSource, node: &Node) -> Option<String> {
    if let Some(comment) = find_comment_before(parsed, node) {
        return Some(comment);
    }
    if let Some(parent) = node.parent() {
        if parent.kind() == "export_statement" {
            if let Some(comment) = find_comment_before(parsed, &parent) {
                return Some(comment);
            }
        }
    }
    None
}

fn find_comment_before(parsed: &ParsedSource, node: &Node) -> Option<String> {
    let mut prev = node.prev_sibling();
    while let Some(sibling) = prev {
        if sibling.kind() == "comment" {
            let text = parsed.node_text(&sibling).trim();
            if text.starts_with("/**") {
                return Some(text.to_string());
            }
        }
        prev = sibling.prev_sibling();
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

fn node_has_async_keyword(parsed: &ParsedSource, node: &Node) -> bool {
    node_has_keyword(parsed, node, "async")
}
