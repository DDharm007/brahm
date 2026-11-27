//! Integration test suite for Milestone 05: Architecture & Dependency Graph Engine.

use brahm::dependencies::analyzer::DependencyAnalysisResult;
use brahm::graph::{
    ArchitectureGraph, EdgeKind, GraphBuilder, GraphEdge, GraphExport, GraphNode, NodeKind,
};
use brahm::parser::{ParserRegistry, SupportedLanguage};
use brahm::structure::analyzer::ApplicationStructureResult;
use brahm::structure::types::{ApiRoute, EntryPoint, EntryPointKind, HttpMethod};
use brahm::symbols::extractor::{DefaultSymbolExtractor, SymbolExtractor};
use brahm::symbols::SymbolIndex;
use std::path::PathBuf;

#[test]
fn test_01_graph_primitives_nodes_and_edges() {
    let mut graph = ArchitectureGraph::new();

    let n1 = GraphNode::new("file:src/main.rs", "main.rs", NodeKind::File);
    let n2 = GraphNode::new("file:src/lib.rs", "lib.rs", NodeKind::File);
    let n3 = GraphNode::new("pkg:serde", "serde", NodeKind::ExternalPackage);

    graph.add_node(n1);
    graph.add_node(n2);
    graph.add_node(n3);

    assert_eq!(graph.node_count(), 3);
    assert_eq!(graph.edge_count(), 0);

    // main.rs imports lib.rs
    graph.add_edge(
        "file:src/main.rs",
        "file:src/lib.rs",
        GraphEdge::new(EdgeKind::Imports),
    );
    // lib.rs imports serde
    graph.add_edge(
        "file:src/lib.rs",
        "pkg:serde",
        GraphEdge::new(EdgeKind::Imports),
    );

    assert_eq!(graph.edge_count(), 2);

    // Fan-in: serde has 1 incoming, main.rs has 0
    assert_eq!(graph.fan_in("pkg:serde"), 1);
    assert_eq!(graph.fan_in("file:src/main.rs"), 0);

    // Fan-out: main.rs has 1 outgoing, serde has 0
    assert_eq!(graph.fan_out("file:src/main.rs"), 1);
    assert_eq!(graph.fan_out("pkg:serde"), 0);

    assert!(!graph.has_cycles());
}

#[test]
fn test_02_topological_sort_and_dag_verification() {
    let mut graph = ArchitectureGraph::new();

    // A -> B -> C
    graph.add_node(GraphNode::new("A", "ModuleA", NodeKind::Module));
    graph.add_node(GraphNode::new("B", "ModuleB", NodeKind::Module));
    graph.add_node(GraphNode::new("C", "ModuleC", NodeKind::Module));

    graph.add_edge("A", "B", GraphEdge::new(EdgeKind::DependsOn));
    graph.add_edge("B", "C", GraphEdge::new(EdgeKind::DependsOn));

    assert!(!graph.has_cycles());

    let sorted = graph.topological_sort().expect("Graph should be DAG");
    assert_eq!(sorted.len(), 3);

    let ids: Vec<&str> = sorted.iter().map(|n| n.id.as_str()).collect();
    let idx_a = ids.iter().position(|&id| id == "A").unwrap();
    let idx_b = ids.iter().position(|&id| id == "B").unwrap();
    let idx_c = ids.iter().position(|&id| id == "C").unwrap();

    assert!(idx_a < idx_b);
    assert!(idx_b < idx_c);
}

#[test]
fn test_03_cycle_detection_and_tarjan_scc() {
    let mut graph = ArchitectureGraph::new();

    // Cycle: A -> B -> C -> A
    graph.add_node(GraphNode::new("A", "ModuleA", NodeKind::Module));
    graph.add_node(GraphNode::new("B", "ModuleB", NodeKind::Module));
    graph.add_node(GraphNode::new("C", "ModuleC", NodeKind::Module));
    graph.add_node(GraphNode::new("D", "IndependentD", NodeKind::Module));

    graph.add_edge("A", "B", GraphEdge::new(EdgeKind::Imports));
    graph.add_edge("B", "C", GraphEdge::new(EdgeKind::Imports));
    graph.add_edge("C", "A", GraphEdge::new(EdgeKind::Imports));
    graph.add_edge("A", "D", GraphEdge::new(EdgeKind::Imports));

    assert!(graph.has_cycles());
    assert!(graph.topological_sort().is_err());

    let cyclic_components = graph.find_cyclic_components();
    assert_eq!(cyclic_components.len(), 1);

    let cycle = &cyclic_components[0];
    assert_eq!(cycle.len(), 3);
    assert!(cycle.contains(&"A".to_string()));
    assert!(cycle.contains(&"B".to_string()));
    assert!(cycle.contains(&"C".to_string()));
    assert!(!cycle.contains(&"D".to_string()));
}

#[test]
fn test_04_blast_radius_impact_analysis() {
    let mut graph = ArchitectureGraph::new();

    // In architectural graphs: App -> Service -> DbHelper
    // (App depends on Service, Service depends on DbHelper)
    graph.add_node(GraphNode::new("app", "App", NodeKind::File));
    graph.add_node(GraphNode::new("service", "Service", NodeKind::File));
    graph.add_node(GraphNode::new("db", "DbHelper", NodeKind::File));

    graph.add_edge("app", "service", GraphEdge::new(EdgeKind::Imports));
    graph.add_edge("service", "db", GraphEdge::new(EdgeKind::Imports));

    // If "db" changes, Service directly depends on it, and App transitively depends on it
    let report = graph.blast_radius("db", None).expect("Node should exist");

    assert_eq!(report.root_id, "db");
    assert_eq!(report.direct_dependents, vec!["service"]);
    assert_eq!(report.transitive_dependents, vec!["app"]);
    assert_eq!(report.total_impacted_nodes, 2);
    assert!((report.impact_ratio - (2.0 / 3.0)).abs() < 1e-6);

    // If "app" changes, nothing depends on it
    let app_report = graph.blast_radius("app", None).expect("Node should exist");
    assert_eq!(app_report.total_impacted_nodes, 0);
    assert_eq!(app_report.impact_ratio, 0.0);
}

#[test]
fn test_05_graph_builder_from_analysis_results() {
    let registry = ParserRegistry::new();
    let extractor = DefaultSymbolExtractor::new();
    let builder = GraphBuilder::new();

    let main_code = r#"
fn main() {
    init();
}

fn init() {}
"#;
    let mut parsed_main = registry
        .parse_source(main_code, SupportedLanguage::Rust)
        .unwrap();
    let main_path = PathBuf::from("src/main.rs");
    parsed_main.path = Some(main_path.clone());

    let mut symbol_index = SymbolIndex::new();
    let symbols = extractor.extract_symbols(&parsed_main);
    symbol_index.extend(symbols);

    let mut dep_results = DependencyAnalysisResult {
        imports: Vec::new(),
        exports: Vec::new(),
        internal_dependencies: std::collections::HashMap::new(),
        external_dependencies: std::collections::HashMap::new(),
        unique_external_packages: vec!["tokio".to_string()].into_iter().collect(),
        cycles: Vec::new(),
        stats: brahm::dependencies::analyzer::DependencyStats::default(),
    };
    dep_results
        .external_dependencies
        .insert(main_path.clone(), vec!["tokio".to_string()]);

    let structure_results = ApplicationStructureResult {
        entrypoints: vec![EntryPoint {
            file_path: main_path.clone(),
            kind: EntryPointKind::Executable,
            symbol_name: Some("main".to_string()),
            span: Some(brahm::parser::SourceSpan::new(2, 0, 4, 1, 0, 20)),
            description: "Main executable entry point".to_string(),
            confidence: 1.0,
        }],
        routes: vec![ApiRoute {
            method: HttpMethod::Get,
            path_pattern: "/api/init".to_string(),
            handler_name: Some("init".to_string()),
            file_path: main_path.clone(),
            span: brahm::parser::SourceSpan::new(2, 0, 2, 20, 0, 20),
            framework: "Custom".to_string(),
        }],
        data_models: Vec::new(),
        summary: brahm::structure::analyzer::StructureSummary::default(),
    };

    let graph = builder.build(
        &[parsed_main],
        &symbol_index,
        &dep_results,
        &structure_results,
    );

    // Verify file node
    assert!(graph.contains_node("file:src/main.rs"));

    // Verify external package node
    assert!(graph.contains_node("pkg:tokio"));

    // Verify route node
    assert!(graph.contains_node("route:GET:/api/init"));

    // Verify symbols are present
    assert!(graph.node_count() >= 4);

    // Verify Route -> init handler edge
    let route_outgoing = graph.outgoing_edges("route:GET:/api/init");
    assert!(route_outgoing
        .iter()
        .any(|(n, e)| e.kind == EdgeKind::RoutesTo && n.name == "init"));
}

#[test]
fn test_06_graph_export_json_serialization_and_roundtrip() {
    let mut graph = ArchitectureGraph::new();

    graph.add_node(GraphNode::new("f1", "file1.rs", NodeKind::File));
    graph.add_node(GraphNode::new("f2", "file2.rs", NodeKind::File));
    graph.add_edge("f1", "f2", GraphEdge::new(EdgeKind::Imports));

    let export = GraphExport::from_graph(&graph);

    assert_eq!(export.version, "1.0");
    assert_eq!(export.summary.total_nodes, 2);
    assert_eq!(export.summary.total_edges, 1);
    assert!(export.summary.is_dag);

    // Serialize to JSON
    let json_str = serde_json::to_string_pretty(&export).expect("Serialization must succeed");
    assert!(json_str.contains("\"version\": \"1.0\""));
    assert!(json_str.contains("\"total_nodes\": 2"));

    // Deserialize back
    let deserialized: GraphExport =
        serde_json::from_str(&json_str).expect("Deserialization must succeed");
    assert_eq!(deserialized, export);
}
