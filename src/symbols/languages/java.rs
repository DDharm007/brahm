//! Symbol extraction for Java source files.

use std::path::Path;
use tree_sitter::Node;

use crate::parser::ParsedSource;
use crate::symbols::id::generate_symbol_id;
use crate::symbols::types::{ParameterInfo, Symbol, SymbolKind, Visibility};

/// Extract all classes, interfaces, enums, methods, and constructors from a parsed Java source file.
pub fn extract_java_symbols(parsed: &ParsedSource) -> Vec<Symbol> {
    let file_path = parsed.file_path().unwrap_or(Path::new("unknown.java"));
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
            "class_declaration" => {
                if let Some(sym) =
                    extract_type_decl(parsed, file_path, &child, current_scope, SymbolKind::Class)
                {
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
            "interface_declaration" => {
                if let Some(sym) = extract_type_decl(
                    parsed,
                    file_path,
                    &child,
                    current_scope,
                    SymbolKind::Interface,
                ) {
                    let iface_id = sym.id.clone();
                    let iface_name = sym.qualified_name.clone();
                    symbols.push(sym);

                    if let Some(body) = child.child_by_field_name("body") {
                        extract_nodes(
                            parsed,
                            file_path,
                            &body,
                            Some(&iface_name),
                            Some(&iface_id),
                            symbols,
                        );
                    }
                }
            }
            "enum_declaration" => {
                if let Some(sym) =
                    extract_type_decl(parsed, file_path, &child, current_scope, SymbolKind::Enum)
                {
                    let enum_id = sym.id.clone();
                    let enum_name = sym.qualified_name.clone();
                    symbols.push(sym);

                    if let Some(body) = child.child_by_field_name("body") {
                        extract_nodes(
                            parsed,
                            file_path,
                            &body,
                            Some(&enum_name),
                            Some(&enum_id),
                            symbols,
                        );
                    }
                }
            }
            "method_declaration" => {
                if let Some(sym) =
                    extract_method(parsed, file_path, &child, current_scope, parent_id)
                {
                    symbols.push(sym);
                }
            }
            "constructor_declaration" => {
                if let Some(sym) =
                    extract_constructor(parsed, file_path, &child, current_scope, parent_id)
                {
                    symbols.push(sym);
                }
            }
            _ => {}
        }
    }
}

fn extract_type_decl(
    parsed: &ParsedSource,
    file_path: &Path,
    node: &Node,
    current_scope: Option<&str>,
    kind: SymbolKind,
) -> Option<Symbol> {
    let name_node = node.child_by_field_name("name")?;
    let name = parsed.node_text(&name_node).trim();

    let qualified_name = match current_scope {
        Some(scope) => format!("{}.{}", scope, name),
        None => name.to_string(),
    };

    let id = generate_symbol_id(file_path, &qualified_name, kind);
    let span = parsed.node_span(node);
    let visibility = extract_java_visibility(parsed, node);

    Some(Symbol {
        id,
        name: name.to_string(),
        qualified_name,
        kind,
        visibility,
        file_path: file_path.to_path_buf(),
        span,
        signature: None,
        return_type: None,
        parameters: Vec::new(),
        parent_id: None,
        doc_comment: extract_javadoc(parsed, node),
        is_async: false,
        is_static: node_has_modifier(parsed, node, "static"),
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

    let qualified_name = match current_scope {
        Some(scope) => format!("{}.{}", scope, name),
        None => name.to_string(),
    };

    let id = generate_symbol_id(file_path, &qualified_name, SymbolKind::Method);
    let span = parsed.node_span(node);
    let visibility = extract_java_visibility(parsed, node);
    let is_static = node_has_modifier(parsed, node, "static");

    let return_type = node
        .child_by_field_name("type")
        .map(|n| parsed.node_text(&n).trim().to_string());

    let parameters = extract_parameters(parsed, node);

    let signature = node.child_by_field_name("parameters").map(|p| {
        let p_text = parsed.node_text(&p);
        match &return_type {
            Some(ret) => format!("{} {}{}", ret, name, p_text),
            None => format!("{}{}", name, p_text),
        }
    });

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
        parent_id: parent_id.map(|s| s.to_string()),
        doc_comment: extract_javadoc(parsed, node),
        is_async: false,
        is_static,
    })
}

fn extract_constructor(
    parsed: &ParsedSource,
    file_path: &Path,
    node: &Node,
    current_scope: Option<&str>,
    parent_id: Option<&str>,
) -> Option<Symbol> {
    let name_node = node.child_by_field_name("name")?;
    let name = parsed.node_text(&name_node).trim();

    let qualified_name = match current_scope {
        Some(scope) => format!("{}.{}", scope, name),
        None => name.to_string(),
    };

    let id = generate_symbol_id(file_path, &qualified_name, SymbolKind::Constructor);
    let span = parsed.node_span(node);
    let visibility = extract_java_visibility(parsed, node);
    let parameters = extract_parameters(parsed, node);

    let signature = node
        .child_by_field_name("parameters")
        .map(|p| format!("{}{}", name, parsed.node_text(&p)));

    Some(Symbol {
        id,
        name: name.to_string(),
        qualified_name,
        kind: SymbolKind::Constructor,
        visibility,
        file_path: file_path.to_path_buf(),
        span,
        signature,
        return_type: None,
        parameters,
        parent_id: parent_id.map(|s| s.to_string()),
        doc_comment: extract_javadoc(parsed, node),
        is_async: false,
        is_static: false,
    })
}

fn extract_parameters(parsed: &ParsedSource, node: &Node) -> Vec<ParameterInfo> {
    let mut params = Vec::new();
    if let Some(param_list) = node.child_by_field_name("parameters") {
        let mut cursor = param_list.walk();
        for child in param_list.children(&mut cursor) {
            if child.kind() == "formal_parameter" {
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

fn extract_java_visibility(parsed: &ParsedSource, node: &Node) -> Visibility {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "modifiers" {
                let text = parsed.node_text(&child);
                if text.contains("public") {
                    return Visibility::Public;
                } else if text.contains("private") {
                    return Visibility::Private;
                } else if text.contains("protected") {
                    return Visibility::Protected;
                }
            }
        }
    }
    // Java package-private default
    Visibility::Internal
}

fn node_has_modifier(parsed: &ParsedSource, node: &Node, modifier: &str) -> bool {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "modifiers" && parsed.node_text(&child).contains(modifier) {
                return true;
            }
        }
    }
    false
}

fn extract_javadoc(parsed: &ParsedSource, node: &Node) -> Option<String> {
    let mut prev = node.prev_sibling();
    while let Some(sibling) = prev {
        if sibling.kind() == "block_comment" {
            let text = parsed.node_text(&sibling).trim();
            if text.starts_with("/**") {
                return Some(text.to_string());
            }
        }
        prev = sibling.prev_sibling();
    }
    None
}
