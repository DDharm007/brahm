//! Assembly and construction of the architecture graph from parsed code, symbols, dependencies, and structure.

use std::collections::HashMap;
use std::path::Path;

use super::engine::ArchitectureGraph;
use super::types::{EdgeKind, GraphEdge, GraphNode, NodeKind};
use crate::dependencies::analyzer::DependencyAnalysisResult;
use crate::parser::ParsedSource;
use crate::structure::analyzer::ApplicationStructureResult;
use crate::symbols::types::SymbolKind;
use crate::symbols::SymbolIndex;

/// Builder responsible for converting static analysis findings into an interconnected architecture graph.
#[derive(Debug, Default, Clone)]
pub struct GraphBuilder;

impl GraphBuilder {
    /// Create a new graph builder.
    pub fn new() -> Self {
        Self
    }

    /// Construct a complete architecture graph from all analysis phases.
    pub fn build(
        &self,
        parsed_files: &[ParsedSource],
        symbol_index: &SymbolIndex,
        dependency_results: &DependencyAnalysisResult,
        structure_results: &ApplicationStructureResult,
    ) -> ArchitectureGraph {
        let mut graph = ArchitectureGraph::new();

        // 1. Add File nodes
        for parsed in parsed_files {
            if let Some(path) = &parsed.path {
                let file_id = normalize_file_id(path);
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let mut node = GraphNode::new(&file_id, file_name, NodeKind::File)
                    .with_file_path(path.clone());
                node.metadata
                    .insert("language".to_string(), parsed.language.name().to_string());
                graph.add_node(node);
            }
        }

        // 2. Add External Package nodes and File -> Package Import edges
        for pkg_name in &dependency_results.unique_external_packages {
            let pkg_id = format!("pkg:{}", pkg_name);
            let mut pkg_node = GraphNode::new(&pkg_id, pkg_name, NodeKind::ExternalPackage);
            pkg_node
                .metadata
                .insert("package_name".to_string(), pkg_name.clone());
            graph.add_node(pkg_node);
        }

        // Add internal file-to-file import edges
        for (src_file, targets) in &dependency_results.internal_dependencies {
            let src_file_id = normalize_file_id(src_file);
            if !graph.contains_node(&src_file_id) {
                let file_name = src_file
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                graph.add_node(
                    GraphNode::new(&src_file_id, file_name, NodeKind::File)
                        .with_file_path(src_file.clone()),
                );
            }

            for tgt_file in targets {
                let tgt_file_id = normalize_file_id(tgt_file);
                if !graph.contains_node(&tgt_file_id) {
                    let file_name = tgt_file
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    graph.add_node(
                        GraphNode::new(&tgt_file_id, file_name, NodeKind::File)
                            .with_file_path(tgt_file.clone()),
                    );
                }
                graph.add_edge(
                    &src_file_id,
                    &tgt_file_id,
                    GraphEdge::new(EdgeKind::Imports),
                );
            }
        }

        // Add external package import edges
        for (src_file, packages) in &dependency_results.external_dependencies {
            let src_file_id = normalize_file_id(src_file);
            if !graph.contains_node(&src_file_id) {
                let file_name = src_file
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                graph.add_node(
                    GraphNode::new(&src_file_id, file_name, NodeKind::File)
                        .with_file_path(src_file.clone()),
                );
            }

            for pkg_name in packages {
                let pkg_id = format!("pkg:{}", pkg_name);
                if !graph.contains_node(&pkg_id) {
                    let pkg_node = GraphNode::new(&pkg_id, pkg_name, NodeKind::ExternalPackage);
                    graph.add_node(pkg_node);
                }
                graph.add_edge(&src_file_id, &pkg_id, GraphEdge::new(EdgeKind::Imports));
            }
        }

        // 3. Add Symbol nodes and File -> Symbol Containment edges
        let mut symbol_name_to_ids: HashMap<String, Vec<String>> = HashMap::new();

        for sym in symbol_index.iter() {
            let sym_id = format!("sym:{}", sym.id);
            let kind = match sym.kind {
                SymbolKind::Function => NodeKind::Function,
                SymbolKind::Method => NodeKind::Method,
                SymbolKind::Class => NodeKind::Class,
                SymbolKind::Interface => NodeKind::Interface,
                SymbolKind::Struct => NodeKind::Struct,
                SymbolKind::Enum => NodeKind::Enum,
                SymbolKind::Module => NodeKind::Module,
                _ => NodeKind::Struct,
            };

            let mut node =
                GraphNode::new(&sym_id, &sym.name, kind).with_file_path(sym.file_path.clone());
            node.metadata
                .insert("visibility".to_string(), format!("{:?}", sym.visibility));
            if let Some(sig) = &sym.signature {
                node.metadata.insert("signature".to_string(), sig.clone());
            }
            graph.add_node(node);

            symbol_name_to_ids
                .entry(sym.name.clone())
                .or_default()
                .push(sym_id.clone());

            // Connect File -> Symbol
            let file_id = normalize_file_id(&sym.file_path);
            if graph.contains_node(&file_id) {
                graph.add_edge(&file_id, &sym_id, GraphEdge::new(EdgeKind::Contains));
            }

            // Connect Parent Symbol -> Child Symbol (e.g. Class -> Method)
            if let Some(parent_id) = &sym.parent_id {
                let parent_sym_id = format!("sym:{}", parent_id);
                graph.add_edge(&parent_sym_id, &sym_id, GraphEdge::new(EdgeKind::Contains));
            }
        }

        // 4. Add API Routes and Route -> Handler / File connections
        for route in &structure_results.routes {
            let route_id = format!("route:{}:{}", route.method, route.path_pattern);
            let mut route_node = GraphNode::new(
                &route_id,
                format!("{} {}", route.method, route.path_pattern),
                NodeKind::ApiRoute,
            )
            .with_file_path(route.file_path.clone());
            route_node
                .metadata
                .insert("method".to_string(), format!("{:?}", route.method));
            route_node
                .metadata
                .insert("path_pattern".to_string(), route.path_pattern.clone());
            route_node
                .metadata
                .insert("framework".to_string(), route.framework.clone());
            graph.add_node(route_node);

            // Connect File -> Route
            let file_id = normalize_file_id(&route.file_path);
            if graph.contains_node(&file_id) {
                graph.add_edge(&file_id, &route_id, GraphEdge::new(EdgeKind::Contains));
            }

            // Connect Route -> Handler function
            if let Some(handler_name) = &route.handler_name {
                if let Some(sym_ids) = symbol_name_to_ids.get(handler_name) {
                    for sym_id in sym_ids {
                        graph.add_edge(&route_id, sym_id, GraphEdge::new(EdgeKind::RoutesTo));
                    }
                }
            }
        }

        // 5. Add Data Models and File -> Model connections
        for model in &structure_results.data_models {
            let model_id = format!("model:{}", model.name);
            let mut model_node = GraphNode::new(&model_id, &model.name, NodeKind::DataModel)
                .with_file_path(model.file_path.clone());
            model_node
                .metadata
                .insert("kind".to_string(), format!("{:?}", model.kind));
            model_node
                .metadata
                .insert("framework".to_string(), model.framework_or_orm.clone());
            graph.add_node(model_node);

            // Connect File -> Model
            let file_id = normalize_file_id(&model.file_path);
            if graph.contains_node(&file_id) {
                graph.add_edge(&file_id, &model_id, GraphEdge::new(EdgeKind::Contains));
            }
        }

        // 6. Calls / Invocation relationships: Heuristic matching
        for parsed in parsed_files {
            let file_symbols: Vec<_> = symbol_index
                .iter()
                .filter(|s| {
                    parsed
                        .path
                        .as_ref()
                        .map(|p| normalize_file_id(p) == normalize_file_id(&s.file_path))
                        .unwrap_or(false)
                })
                .collect();

            for caller in &file_symbols {
                if caller.kind != SymbolKind::Function && caller.kind != SymbolKind::Method {
                    continue;
                }

                let caller_node_id = format!("sym:{}", caller.id);

                // Scan caller signature / doc / source context for known target symbol calls
                for callee in symbol_index.iter() {
                    if callee.id == caller.id {
                        continue;
                    }
                    if callee.kind != SymbolKind::Function && callee.kind != SymbolKind::Method {
                        continue;
                    }

                    let call_pattern = format!("{}(", callee.name);
                    if parsed.source.contains(&call_pattern) {
                        // Avoid adding duplicate edges
                        let callee_node_id = format!("sym:{}", callee.id);
                        graph.add_edge(
                            &caller_node_id,
                            &callee_node_id,
                            GraphEdge::new(EdgeKind::Calls),
                        );
                    }
                }
            }
        }

        graph
    }
}

/// Helper function to create consistent normalized file IDs across OS platforms.
pub fn normalize_file_id(path: &Path) -> String {
    let norm = path.to_string_lossy().replace('\\', "/");
    format!("file:{}", norm)
}
