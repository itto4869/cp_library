use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// Cost operations for Dijkstra's algorithm.
///
/// `ZERO` must be the additive identity and least valid cost, `MAX` the
/// infinity sentinel. Addition of nonnegative costs must preserve order and
/// saturate at `MAX`. Primitive signed and unsigned integers are supported;
/// negative edge costs are rejected by `dijkstra`.
pub trait DijkstraCost: Copy + Ord {
    const ZERO: Self;
    const MAX: Self;
    fn saturating_add(self, other: Self) -> Self;
}

macro_rules! impl_cost {
    ($($t:ty),*) => { $(
        impl DijkstraCost for $t {
            const ZERO: Self = 0;
            const MAX: Self = <$t>::MAX;
            fn saturating_add(self, other: Self) -> Self {
                self.saturating_add(other)
            }
        }
    )* };
}
impl_cost!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);

/// Returns shortest distances from `start` in a nonnegative weighted graph.
///
/// `graph[v]` contains `(destination, cost)` pairs. Vertex indices are zero-based.
/// For an undirected graph, insert each edge in both directions.
/// Unreachable vertices and distances at least `T::MAX` are represented by
/// `T::MAX`. Addition saturates rather than overflowing.
///
/// Time: O(V + E log(E + 1)); space: O(V + E).
///
/// # Panics
/// Panics if `start` or any edge destination is outside `0..graph.len()`,
/// or if any edge cost is negative.
///
/// # Examples
/// ```
/// use cp_graph::dijkstra;
///
/// let graph = vec![vec![(1, 4), (2, 1)], vec![], vec![(1, 2)], vec![]];
/// assert_eq!(dijkstra(&graph, 0), vec![0, 3, 1, usize::MAX]);
/// ```
pub fn dijkstra<T: DijkstraCost>(graph: &[Vec<(usize, T)>], start: usize) -> Vec<T> {
    let n = graph.len();
    assert!(start < n, "start vertex is out of bounds");
    for edges in graph {
        for &(to, cost) in edges {
            assert!(to < n, "edge destination is out of bounds");
            assert!(cost >= T::ZERO, "edge cost must be nonnegative");
        }
    }

    let mut distances = vec![T::MAX; n];
    let mut queue = BinaryHeap::new();
    distances[start] = T::ZERO;
    queue.push(Reverse((T::ZERO, start)));

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
        assert_eq!(dijkstra::<usize>(&[vec![]], 0), vec![0]);
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
        dijkstra::<usize>(&[], 0);
    }

    #[test]
    #[should_panic(expected = "start vertex is out of bounds")]
    fn invalid_start() {
        dijkstra::<usize>(&[vec![]], 1);
    }

    #[test]
    #[should_panic(expected = "edge destination is out of bounds")]
    fn invalid_destination_in_unreachable_component() {
        dijkstra(&[vec![], vec![(2, 1)]], 0);
    }

    #[test]
    fn generic_costs_and_saturation() {
        let graph = vec![vec![(1, u128::MAX - 2)], vec![(2, 1)], vec![], vec![]];
        assert_eq!(
            dijkstra(&graph, 0),
            vec![0, u128::MAX - 2, u128::MAX - 1, u128::MAX]
        );
        let graph = vec![vec![(1, 250u8), (2, 20)], vec![(2, 10)], vec![]];
        assert_eq!(dijkstra(&graph, 0), vec![0, 250, 20]);
        let graph = vec![vec![(1, 7i64)], vec![(2, 4)], vec![], vec![]];
        assert_eq!(dijkstra(&graph, 0), vec![0, 7, 11, i64::MAX]);
    }

    #[test]
    #[should_panic(expected = "edge cost must be nonnegative")]
    fn rejects_negative_cost_even_in_unreachable_component() {
        dijkstra(&[vec![], vec![(1, -1i64)]], 0);
    }
}
