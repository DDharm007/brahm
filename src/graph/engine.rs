//! Core graph data structure and query algorithms backed by petgraph.

use petgraph::algo::{is_cyclic_directed, tarjan_scc, toposort};
use petgraph::graph::{DiGraph, EdgeIndex, NodeIndex};
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

use super::types::{EdgeKind, GraphEdge, GraphNode, NodeKind};

/// Report detailing the structural impact of modifying or removing a component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlastRadiusReport {
    /// ID of the target node evaluated.
    pub root_id: String,
    /// Nodes that directly depend on the target node.
    pub direct_dependents: Vec<String>,
    /// Nodes that indirectly/transitively depend on the target node.
    pub transitive_dependents: Vec<String>,
    /// Total count of uniquely impacted nodes.
    pub total_impacted_nodes: usize,
    /// Percentage of the total repository graph impacted (0.0 to 1.0).
    pub impact_ratio: f64,
}

/// The central architecture and dependency graph of BRAHM.
#[derive(Debug, Clone)]
pub struct ArchitectureGraph {
    /// Inner directed graph from petgraph.
    graph: DiGraph<GraphNode, GraphEdge>,
    /// Fast index mapping node IDs to petgraph NodeIndex.
    node_map: HashMap<String, NodeIndex>,
}

impl Default for ArchitectureGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl ArchitectureGraph {
    /// Initialize a new empty architecture graph.
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
        }
    }

    /// Add a node to the graph if not already present.
    /// If already present, updates the node data and returns the existing index.
    pub fn add_node(&mut self, node: GraphNode) -> NodeIndex {
        if let Some(&idx) = self.node_map.get(&node.id) {
            self.graph[idx] = node;
            idx
        } else {
            let id = node.id.clone();
            let idx = self.graph.add_node(node);
            self.node_map.insert(id, idx);
            idx
        }
    }

    /// Add a directed edge from `source_id` to `target_id`.
    /// Returns `Some(EdgeIndex)` if both nodes exist, or `None` if either node is missing.
    pub fn add_edge(
        &mut self,
        source_id: &str,
        target_id: &str,
        edge: GraphEdge,
    ) -> Option<EdgeIndex> {
        let &src_idx = self.node_map.get(source_id)?;
        let &tgt_idx = self.node_map.get(target_id)?;
        Some(self.graph.add_edge(src_idx, tgt_idx, edge))
    }

    /// Total number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Total number of edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    /// Retrieve a node by its unique identifier.
    pub fn get_node(&self, id: &str) -> Option<&GraphNode> {
        let &idx = self.node_map.get(id)?;
        self.graph.node_weight(idx)
    }

    /// Check if a node with the specified ID exists in the graph.
    pub fn contains_node(&self, id: &str) -> bool {
        self.node_map.contains_key(id)
    }

    /// Return an iterator over all nodes in the graph.
    pub fn nodes(&self) -> impl Iterator<Item = &GraphNode> {
        self.graph.node_weights()
    }

    /// Calculate in-degree (fan-in / count of incoming edges) for a given node.
    pub fn fan_in(&self, id: &str) -> usize {
        if let Some(&idx) = self.node_map.get(id) {
            self.graph
                .neighbors_directed(idx, Direction::Incoming)
                .count()
        } else {
            0
        }
    }

    /// Calculate out-degree (fan-out / count of outgoing edges) for a given node.
    pub fn fan_out(&self, id: &str) -> usize {
        if let Some(&idx) = self.node_map.get(id) {
            self.graph
                .neighbors_directed(idx, Direction::Outgoing)
                .count()
        } else {
            0
        }
    }

    /// List outgoing neighbors and their connecting edges.
    pub fn outgoing_edges(&self, id: &str) -> Vec<(&GraphNode, &GraphEdge)> {
        let mut results = Vec::new();
        if let Some(&idx) = self.node_map.get(id) {
            for edge in self.graph.edges_directed(idx, Direction::Outgoing) {
                let target_node = &self.graph[edge.target()];
                let edge_data = edge.weight();
                results.push((target_node, edge_data));
            }
        }
        results
    }

    /// List incoming neighbors and their connecting edges.
    pub fn incoming_edges(&self, id: &str) -> Vec<(&GraphNode, &GraphEdge)> {
        let mut results = Vec::new();
        if let Some(&idx) = self.node_map.get(id) {
            for edge in self.graph.edges_directed(idx, Direction::Incoming) {
                let source_node = &self.graph[edge.source()];
                let edge_data = edge.weight();
                results.push((source_node, edge_data));
            }
        }
        results
    }

    /// Check whether the directed graph contains at least one cycle.
    pub fn has_cycles(&self) -> bool {
        is_cyclic_directed(&self.graph)
    }

    /// Compute a topological sort of the graph nodes if it is a Directed Acyclic Graph (DAG).
    /// Returns `Ok(Vec<GraphNode>)` if acyclic, or `Err(Vec<String>)` containing cycle node IDs.
    pub fn topological_sort(&self) -> Result<Vec<GraphNode>, Vec<String>> {
        match toposort(&self.graph, None) {
            Ok(indices) => {
                let sorted = indices
                    .into_iter()
                    .map(|idx| self.graph[idx].clone())
                    .collect();
                Ok(sorted)
            }
            Err(cycle_err) => {
                let node_id = self.graph[cycle_err.node_id()].id.clone();
                Err(vec![node_id])
            }
        }
    }

    /// Detect strongly connected components (SCCs) using Tarjan's algorithm.
    /// Filters to components with more than 1 node or nodes with self-loops.
    pub fn find_cyclic_components(&self) -> Vec<Vec<String>> {
        let sccs = tarjan_scc(&self.graph);
        let mut cycles = Vec::new();

        for scc in sccs {
            if scc.len() > 1 {
                let component_ids: Vec<String> =
                    scc.iter().map(|&idx| self.graph[idx].id.clone()).collect();
                cycles.push(component_ids);
            } else if scc.len() == 1 {
                let idx = scc[0];
                // Check for self-loop
                if self.graph.contains_edge(idx, idx) {
                    cycles.push(vec![self.graph[idx].id.clone()]);
                }
            }
        }

        cycles
    }

    /// Compute the blast radius of modifying a node.
    ///
    /// In an architectural graph, edges point from dependent -> dependency (e.g., A imports B).
    /// Therefore, if component B changes, any component that imports or calls B (incoming edges to B)
    /// is impacted.
    pub fn blast_radius(&self, id: &str, max_depth: Option<usize>) -> Option<BlastRadiusReport> {
        let &start_idx = self.node_map.get(id)?;
        let max_d = max_depth.unwrap_or(usize::MAX);

        let mut direct_dependents = Vec::new();
        let mut transitive_dependents = Vec::new();
        let mut visited = HashSet::new();
        visited.insert(start_idx);

        // Queue: (NodeIndex, depth)
        let mut queue = VecDeque::new();
        queue.push_back((start_idx, 0));

        while let Some((curr_idx, depth)) = queue.pop_front() {
            if depth >= max_d {
                continue;
            }

            for neighbor_idx in self.graph.neighbors_directed(curr_idx, Direction::Incoming) {
                if visited.insert(neighbor_idx) {
                    let neighbor_id = self.graph[neighbor_idx].id.clone();
                    if depth == 0 {
                        direct_dependents.push(neighbor_id.clone());
                    } else {
                        transitive_dependents.push(neighbor_id.clone());
                    }
                    queue.push_back((neighbor_idx, depth + 1));
                }
            }
        }

        let total_impacted = direct_dependents.len() + transitive_dependents.len();
        let total_nodes = self.node_count();
        let impact_ratio = if total_nodes > 0 {
            total_impacted as f64 / total_nodes as f64
        } else {
            0.0
        };

        Some(BlastRadiusReport {
            root_id: id.to_string(),
            direct_dependents,
            transitive_dependents,
            total_impacted_nodes: total_impacted,
            impact_ratio,
        })
    }

    /// Get central nodes ranked by fan-in (most depended-upon components).
    pub fn rank_by_fan_in(&self, limit: usize) -> Vec<(GraphNode, usize)> {
        let mut ranked: Vec<(GraphNode, usize)> = self
            .graph
            .node_indices()
            .map(|idx| {
                let node = self.graph[idx].clone();
                let fan_in = self
                    .graph
                    .neighbors_directed(idx, Direction::Incoming)
                    .count();
                (node, fan_in)
            })
            .collect();

        ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.id.cmp(&b.0.id)));
        ranked.truncate(limit);
        ranked
    }

    /// Get component nodes ranked by fan-out (highest coupling to other components).
    pub fn rank_by_fan_out(&self, limit: usize) -> Vec<(GraphNode, usize)> {
        let mut ranked: Vec<(GraphNode, usize)> = self
            .graph
            .node_indices()
            .map(|idx| {
                let node = self.graph[idx].clone();
                let fan_out = self
                    .graph
                    .neighbors_directed(idx, Direction::Outgoing)
                    .count();
                (node, fan_out)
            })
            .collect();

        ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.id.cmp(&b.0.id)));
        ranked.truncate(limit);
        ranked
    }

    /// Count nodes by kind.
    pub fn count_nodes_by_kind(&self) -> HashMap<NodeKind, usize> {
        let mut counts = HashMap::new();
        for node in self.graph.node_weights() {
            *counts.entry(node.kind).or_insert(0) += 1;
        }
        counts
    }

    /// Count edges by kind.
    pub fn count_edges_by_kind(&self) -> HashMap<EdgeKind, usize> {
        let mut counts = HashMap::new();
        for edge in self.graph.edge_weights() {
            *counts.entry(edge.kind).or_insert(0) += 1;
        }
        counts
    }

    /// Compute the overall graph density: 2 * E / (V * (V - 1)) for directed graphs: E / (V * (V - 1)).
    pub fn density(&self) -> f64 {
        let v = self.node_count();
        if v <= 1 {
            return 0.0;
        }
        let e = self.edge_count();
        e as f64 / (v * (v - 1)) as f64
    }

    /// Direct access to inner petgraph reference.
    pub fn inner(&self) -> &DiGraph<GraphNode, GraphEdge> {
        &self.graph
    }
}
