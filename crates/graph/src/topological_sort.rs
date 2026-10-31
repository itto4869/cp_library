/// Returns a topological ordering of a directed graph, or `None` if cyclic.
///
/// `graph[v]` lists the destinations of outgoing edges from vertex `v`.
/// Vertex indices are zero-based. The input is neither consumed nor modified.
/// Every vertex appears exactly once in a successful result, and every edge
/// goes from an earlier vertex to a later one.
///
/// Uses Kahn's algorithm without recursion. Initially, zero-indegree vertices
/// are queued in ascending index order; newly available vertices are appended
/// in adjacency-list order. The result is deterministic for a fixed input, but
/// is not necessarily lexicographically smallest.
/// Empty graphs return `Some(vec![])`. Parallel edges are supported, and a
/// self-loop is a cycle. Time: O(V + E); additional space: O(V).
///
/// # Panics
/// Panics if any edge destination is outside `0..graph.len()`.
///
/// # Examples
/// ```
/// use cp_graph::topological_sort;
///
/// let graph = vec![vec![1, 2], vec![3], vec![3], vec![]];
/// assert_eq!(topological_sort(&graph), Some(vec![0, 1, 2, 3]));
/// assert_eq!(topological_sort(&[vec![1], vec![0]]), None);
/// ```
pub fn topological_sort(graph: &[Vec<usize>]) -> Option<Vec<usize>> {
    let n = graph.len();
    let mut indegree = vec![0usize; n];
    for edges in graph {
        for &to in edges {
            assert!(to < n, "edge destination is out of bounds");
            indegree[to] += 1;
        }
    }

    // The output vector also serves as a FIFO queue.
    let mut order: Vec<usize> = (0..n).filter(|&v| indegree[v] == 0).collect();
    let mut head = 0;
    while head < order.len() {
        let vertex = order[head];
        head += 1;
        for &to in &graph[vertex] {
            indegree[to] -= 1;
            if indegree[to] == 0 {
                order.push(to);
            }
        }
    }
    (order.len() == n).then_some(order)
}

#[cfg(test)]
mod tests {
    use super::topological_sort;

    fn assert_valid(graph: &[Vec<usize>], order: &[usize]) {
        assert_eq!(order.len(), graph.len());
        let mut position = vec![usize::MAX; graph.len()];
        for (i, &v) in order.iter().enumerate() {
            assert_eq!(position[v], usize::MAX);
            position[v] = i;
        }
        for (v, edges) in graph.iter().enumerate() {
            for &to in edges {
                assert!(position[v] < position[to]);
            }
        }
    }

    #[test]
    fn branching_disconnected_graph_with_parallel_edges() {
        let graph = vec![vec![], vec![3, 3, 2], vec![4], vec![4], vec![], vec![]];
        let order = topological_sort(&graph).unwrap();
        assert_eq!(order, vec![0, 1, 5, 3, 2, 4]);
        assert_valid(&graph, &order);
        assert_eq!(topological_sort(&graph), Some(order));
    }

    #[test]
    fn empty_graph_singleton_and_cycles() {
        assert_eq!(topological_sort(&[]), Some(vec![]));
        assert_eq!(topological_sort(&[vec![]]), Some(vec![0]));
        assert_eq!(topological_sort(&[vec![0]]), None);
        assert_eq!(topological_sort(&[vec![1], vec![0]]), None);
        // Acyclic vertices can be processed even when another component cycles.
        assert_eq!(topological_sort(&[vec![1], vec![], vec![3], vec![2]]), None);
    }

    #[test]
    fn exhaustive_three_vertex_graphs_match_reachability() {
        for mask in 0..1usize << 9 {
            let mut graph = vec![vec![]; 3];
            let mut reach = [[false; 3]; 3];
            for v in 0..3 {
                for to in 0..3 {
                    if mask & (1 << (v * 3 + to)) != 0 {
                        graph[v].push(to);
                        reach[v][to] = true;
                    }
                }
            }
            for k in 0..3 {
                for i in 0..3 {
                    for j in 0..3 {
                        reach[i][j] |= reach[i][k] && reach[k][j];
                    }
                }
            }
            let order = topological_sort(&graph);
            assert_eq!(order.is_none(), (0..3).any(|v| reach[v][v]));
            if let Some(order) = order {
                assert_valid(&graph, &order);
            }
        }
    }

    #[test]
    fn deep_chain_without_recursion() {
        let n = 100_000;
        let mut graph = vec![vec![]; n];
        for v in 1..n {
            graph[v].push(v - 1);
        }
        assert_eq!(topological_sort(&graph), Some((0..n).rev().collect()));
    }

    #[test]
    #[should_panic(expected = "edge destination is out of bounds")]
    fn rejects_invalid_destination_even_in_cyclic_component() {
        topological_sort(&[vec![], vec![1, 2]]);
    }
}
