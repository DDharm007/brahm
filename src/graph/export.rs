//! Serializable data representations for exporting the architecture graph to JSON.

use chrono::Utc;
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::engine::ArchitectureGraph;
use super::types::{EdgeKind, NodeKind};

/// Top-level export schema for BRAHM_DEPENDENCY_GRAPH.json.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphExport {
    /// Schema format version.
    pub version: String,
    /// Generation timestamp in UTC.
    pub generated_at: String,
    /// Summary metrics of the graph topology.
    pub summary: GraphSummary,
    /// List of vertices in the graph.
    pub nodes: Vec<GraphNodeExport>,
    /// List of directed edges in the graph.
    pub edges: Vec<GraphEdgeExport>,
    /// Pre-computed blast radius metrics for critical components.
    pub blast_radius_index: HashMap<String, usize>,
}

/// High-level topology statistics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphSummary {
    /// Total vertices.
    pub total_nodes: usize,
    /// Total directed edges.
    pub total_edges: usize,
    /// Whether the graph has zero directed cycles.
    pub is_dag: bool,
    /// Edge density ratio (0.0 to 1.0).
    pub density: f64,
    /// Number of strongly connected components with cycles.
    pub cyclic_components_count: usize,
    /// Breakdown of nodes by category.
    pub node_counts_by_kind: HashMap<String, usize>,
    /// Breakdown of edges by relationship kind.
    pub edge_counts_by_kind: HashMap<String, usize>,
    /// Top most depended-upon central components.
    pub top_central_nodes: Vec<CentralNodeExport>,
}

/// Centrality information for a key architectural node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CentralNodeExport {
    pub id: String,
    pub name: String,
    pub kind: NodeKind,
    pub fan_in: usize,
    pub fan_out: usize,
}

/// Exported node representation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphNodeExport {
    pub id: String,
    pub name: String,
    pub kind: NodeKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub metadata: HashMap<String, String>,
}

/// Exported edge representation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphEdgeExport {
    pub source: String,
    pub target: String,
    pub kind: EdgeKind,
    pub weight: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<String>,
}

impl GraphExport {
    /// Convert an in-memory `ArchitectureGraph` into a serializable `GraphExport`.
    pub fn from_graph(graph: &ArchitectureGraph) -> Self {
        let node_counts: HashMap<String, usize> = graph
            .count_nodes_by_kind()
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();

        let edge_counts: HashMap<String, usize> = graph
            .count_edges_by_kind()
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();

        let top_central_nodes: Vec<CentralNodeExport> = graph
            .rank_by_fan_in(10)
            .into_iter()
            .map(|(node, fan_in)| {
                let fan_out = graph.fan_out(&node.id);
                CentralNodeExport {
                    id: node.id,
                    name: node.name,
                    kind: node.kind,
                    fan_in,
                    fan_out,
                }
            })
            .collect();

        let cyclic_components = graph.find_cyclic_components();

        let summary = GraphSummary {
            total_nodes: graph.node_count(),
            total_edges: graph.edge_count(),
            is_dag: !graph.has_cycles(),
            density: graph.density(),
            cyclic_components_count: cyclic_components.len(),
            node_counts_by_kind: node_counts,
            edge_counts_by_kind: edge_counts,
            top_central_nodes,
        };

        let nodes: Vec<GraphNodeExport> = graph
            .nodes()
            .map(|n| GraphNodeExport {
                id: n.id.clone(),
                name: n.name.clone(),
                kind: n.kind,
                file_path: n
                    .file_path
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string()),
                metadata: n.metadata.clone(),
            })
            .collect();

        let mut edges = Vec::new();
        let inner = graph.inner();
        for edge in inner.edge_references() {
            let src = &inner[edge.source()].id;
            let tgt = &inner[edge.target()].id;
            let weight = edge.weight();
            edges.push(GraphEdgeExport {
                source: src.clone(),
                target: tgt.clone(),
                kind: weight.kind,
                weight: weight.weight,
                metadata: weight.metadata.clone(),
            });
        }

        let mut blast_radius_index = HashMap::new();
        for node in graph.nodes() {
            if let Some(report) = graph.blast_radius(&node.id, Some(3)) {
                if report.total_impacted_nodes > 0 {
                    blast_radius_index.insert(node.id.clone(), report.total_impacted_nodes);
                }
            }
        }

        Self {
            version: "1.0".to_string(),
            generated_at: Utc::now().to_rfc3339(),
            summary,
            nodes,
            edges,
            blast_radius_index,
        }
    }
}
