use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// Returns shortest distances from `start` in a nonnegative weighted graph.
///
/// `graph[v]` contains `(destination, cost)` pairs. Vertex indices are zero-based.
/// For an undirected graph, insert each edge in both directions.
/// Unreachable vertices and distances at least `usize::MAX` are represented by
/// `usize::MAX`. Addition saturates rather than overflowing.
///
/// Time: O(V + E log(E + 1)); space: O(V + E).
///
/// # Panics
/// Panics if `start` or any edge destination is outside `0..graph.len()`.
///
/// # Examples
/// ```
/// use cp_library::graph::dijkstra;
///
/// let graph = vec![vec![(1, 4), (2, 1)], vec![], vec![(1, 2)], vec![]];
/// assert_eq!(dijkstra(&graph, 0), vec![0, 3, 1, usize::MAX]);
/// ```
pub fn dijkstra(graph: &[Vec<(usize, usize)>], start: usize) -> Vec<usize> {
    let n = graph.len();
    assert!(start < n, "start vertex is out of bounds");
    for edges in graph {
        for &(to, _) in edges {
            assert!(to < n, "edge destination is out of bounds");
        }
    }

    let mut distances = vec![usize::MAX; n];
    let mut queue = BinaryHeap::new();
    distances[start] = 0;
    queue.push(Reverse((0, start)));

    while let Some(Reverse((distance, vertex))) = queue.pop() {
        if distance != distances[vertex] {
            continue;
        }
        for &(to, cost) in &graph[vertex] {
            let next_distance = distance.saturating_add(cost);
            if next_distance < distances[to] {
                distances[to] = next_distance;
                queue.push(Reverse((next_distance, to)));
            }
        }
    }
    distances
}

#[cfg(test)]
mod tests {
    use super::dijkstra;

    #[test]
    fn shortest_paths_with_repeated_updates() {
        let graph = vec![
            vec![(1, 10), (2, 1)],
            vec![(3, 2)],
            vec![(1, 2), (3, 8)],
            vec![],
            vec![],
        ];
        assert_eq!(dijkstra(&graph, 0), vec![0, 3, 1, 5, usize::MAX]);
    }

    #[test]
    fn zero_cost_cycles_parallel_edges_and_nonzero_start() {
        let graph = vec![vec![(1, 0)], vec![(0, 0), (1, 0), (2, 8), (2, 3)], vec![]];
        assert_eq!(dijkstra(&graph, 1), vec![0, 0, 3]);
        assert_eq!(dijkstra(&graph, 2), vec![usize::MAX, usize::MAX, 0]);
    }

    #[test]
    fn single_vertex() {
        assert_eq!(dijkstra(&[vec![]], 0), vec![0]);
    }

    #[test]
    fn large_costs_do_not_wrap() {
        let graph = vec![
            vec![(1, usize::MAX - 1), (2, 5), (3, usize::MAX)],
            vec![(2, 10), (3, 1)],
            vec![],
            vec![],
        ];
        assert_eq!(dijkstra(&graph, 0), vec![0, usize::MAX - 1, 5, usize::MAX]);
    }

    #[test]
    #[should_panic(expected = "start vertex is out of bounds")]
    fn empty_graph() {
        dijkstra(&[], 0);
    }

    #[test]
    #[should_panic(expected = "start vertex is out of bounds")]
    fn invalid_start() {
        dijkstra(&[vec![]], 1);
    }

    #[test]
    #[should_panic(expected = "edge destination is out of bounds")]
    fn invalid_destination_in_unreachable_component() {
        dijkstra(&[vec![], vec![(2, 1)]], 0);
    }
}
