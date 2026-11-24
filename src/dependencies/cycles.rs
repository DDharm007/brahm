//! Dependency cycle detection algorithm.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use super::types::DependencyCycle;

/// Detects circular dependencies within a directed file dependency graph.
pub struct CycleDetector;

impl CycleDetector {
    /// Detect all circular dependency loops in the given file-level dependency map.
    ///
    /// Graph edges represent `A -> B` meaning file `A` depends on/imports file `B`.
    /// Returns a list of unique detected cycles.
    pub fn find_cycles(dependencies: &HashMap<PathBuf, Vec<PathBuf>>) -> Vec<DependencyCycle> {
        let mut visited = HashSet::new();
        let mut rec_stack = Vec::new();
        let mut in_stack = HashSet::new();
        let mut raw_cycles = Vec::new();

        // Sort keys for deterministic cycle detection ordering
        let mut nodes: Vec<&PathBuf> = dependencies.keys().collect();
        nodes.sort();

        for node in nodes {
            if !visited.contains(node) {
                Self::dfs(
                    node,
                    dependencies,
                    &mut visited,
                    &mut rec_stack,
                    &mut in_stack,
                    &mut raw_cycles,
                );
            }
        }

        Self::deduplicate_cycles(raw_cycles)
    }

    fn dfs<'a>(
        current: &'a PathBuf,
        adj: &'a HashMap<PathBuf, Vec<PathBuf>>,
        visited: &mut HashSet<&'a PathBuf>,
        stack: &mut Vec<&'a PathBuf>,
        in_stack: &mut HashSet<&'a PathBuf>,
        found_cycles: &mut Vec<Vec<PathBuf>>,
    ) {
        visited.insert(current);
        stack.push(current);
        in_stack.insert(current);

        if let Some(neighbors) = adj.get(current) {
            for neighbor in neighbors {
                if in_stack.contains(neighbor) {
                    // Cycle detected! Extract loop from stack starting at neighbor
                    if let Some(start_idx) = stack.iter().position(|&item| item == neighbor) {
                        let mut cycle_slice: Vec<PathBuf> =
                            stack[start_idx..].iter().map(|&p| p.clone()).collect();
                        cycle_slice.push(neighbor.clone());
                        found_cycles.push(cycle_slice);
                    }
                } else if !visited.contains(neighbor) {
                    Self::dfs(neighbor, adj, visited, stack, in_stack, found_cycles);
                }
            }
        }

        stack.pop();
        in_stack.remove(current);
    }

    /// Deduplicate cycles by rotating each loop so the lexicographically smallest path comes first.
    fn deduplicate_cycles(raw: Vec<Vec<PathBuf>>) -> Vec<DependencyCycle> {
        let mut seen = HashSet::new();
        let mut unique_cycles = Vec::new();

        for cycle in raw {
            if cycle.len() <= 1 {
                continue;
            }

            // Exclude the closing duplicate node when finding canonical rotation
            let base_nodes = &cycle[..cycle.len() - 1];
            if base_nodes.is_empty() {
                continue;
            }

            // Find index of lexicographically smallest node
            let min_idx = base_nodes
                .iter()
                .enumerate()
                .min_by_key(|(_, path)| path.as_path())
                .map(|(idx, _)| idx)
                .unwrap_or(0);

            // Rotate cycle starting at minimum element
            let mut canonical = Vec::new();
            for item in &base_nodes[min_idx..] {
                canonical.push(item.clone());
            }
            for item in &base_nodes[..min_idx] {
                canonical.push(item.clone());
            }
            // Append starting node to close the loop
            canonical.push(canonical[0].clone());

            let signature: Vec<String> = canonical
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();

            if seen.insert(signature) {
                unique_cycles.push(DependencyCycle::new(canonical));
            }
        }

        unique_cycles
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cycle_detection_simple_loop() {
        let a = PathBuf::from("a.ts");
        let b = PathBuf::from("b.ts");
        let c = PathBuf::from("c.ts");

        let mut graph = HashMap::new();
        graph.insert(a.clone(), vec![b.clone()]);
        graph.insert(b.clone(), vec![c.clone()]);
        graph.insert(c.clone(), vec![a.clone()]); // A -> B -> C -> A

        let cycles = CycleDetector::find_cycles(&graph);
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].length(), 3);
        assert_eq!(
            cycles[0].cycle,
            vec![a.clone(), b.clone(), c.clone(), a.clone()]
        );
    }

    #[test]
    fn test_cycle_detection_no_cycles() {
        let a = PathBuf::from("a.ts");
        let b = PathBuf::from("b.ts");
        let c = PathBuf::from("c.ts");

        let mut graph = HashMap::new();
        graph.insert(a, vec![b.clone()]);
        graph.insert(b, vec![c]);

        let cycles = CycleDetector::find_cycles(&graph);
        assert!(cycles.is_empty());
    }

    #[test]
    fn test_cycle_deduplication() {
        let a = PathBuf::from("a.ts");
        let b = PathBuf::from("b.ts");

        let mut graph = HashMap::new();
        graph.insert(a.clone(), vec![b.clone()]);
        graph.insert(b.clone(), vec![a.clone()]); // Two nodes mutually importing

        let cycles = CycleDetector::find_cycles(&graph);
        assert_eq!(
            cycles.len(),
            1,
            "Mutually importing nodes must produce exactly 1 cycle"
        );
        assert_eq!(cycles[0].cycle, vec![a.clone(), b.clone(), a.clone()]);
    }
}
