# BRAHM Architecture & Dependency Graph Engine (`src/graph/`)

## 1. Directed Graph Model (`petgraph`)

`ArchitectureGraph` wraps `petgraph::graph::DiGraph<GraphNode, GraphEdge>` paired with an $O(1)$ `HashMap<String, NodeIndex>` lookup table.

### Vertex Kinds (`NodeKind`)
- `File` (`file:<rel_path>`)
- `Module`, `Function`, `Method`, `Class`, `Interface`, `Struct`, `Enum` (`sym:<blake3_id>`)
- `ApiRoute` (`route:<METHOD>:<path>`)
- `DataModel` (`model:<name>`)
- `ExternalPackage` (`pkg:<package_name>`)

### Edge Kinds (`EdgeKind`)
- `Contains` — Hierarchical ownership (`File -> Symbol`, `Class -> Method`, `File -> ApiRoute`, `File -> DataModel`)
- `Imports` — Dependency coupling (`File -> File`, `File -> ExternalPackage`)
- `RoutesTo` — Endpoint dispatch (`ApiRoute -> Function/Method` handler)
- `Calls` — Heuristic invocation reference (`Function -> Function`)
- `InheritsFrom` / `DependsOn` — Structural type or component relationships

---

## 2. Analytical Queries (`src/graph/engine.rs`)

- **Topological Sort (`topological_sort`):** Uses `petgraph::algo::toposort` to establish valid build/initialization layering or report exact cyclic blockers.
- **Strongly Connected Components (`find_cyclic_components`):** Uses Tarjan's SCC algorithm (`petgraph::algo::tarjan_scc`), identifying multi-node cycles and self-loops.
- **Centrality (`rank_by_fan_in`, `rank_by_fan_out`):** Measures incoming dependency weight (popularity / coupling concentration) and outgoing dependency breadth.
- **Blast Radius Analysis (`blast_radius`):** Performs bounded reverse-BFS along incoming dependent edges to determine direct dependents, transitive dependents, and normalized impact ratio (`total_impacted / total_nodes`) if a target node is modified or tampered with.
- **JSON Serialization (`BRAHM_DEPENDENCY_GRAPH.json`):** Serialized via `GraphExport` with graph summary metrics (`is_dag`, `density`, `node_counts_by_kind`, `edge_counts_by_kind`, top central nodes, and blast radius index).
