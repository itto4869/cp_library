/// A directed graph in which every vertex has exactly one outgoing edge.
///
/// Vertex indices are zero-based. An empty graph is allowed.
#[derive(Clone, Debug)]
pub struct FunctionalGraph {
    next: Vec<usize>,
}

impl FunctionalGraph {
    /// Creates a graph with an edge `v -> next[v]` for each vertex.
    ///
    /// # Panics
    /// Panics if any destination is outside `0..next.len()`.
    pub fn new(next: Vec<usize>) -> Self {
        assert!(
            next.iter().all(|&to| to < next.len()),
            "edge destination is out of bounds"
        );
        Self { next }
    }

    /// Returns the number of vertices.
    pub fn len(&self) -> usize {
        self.next.len()
    }

    /// Returns whether the graph has no vertices.
    pub fn is_empty(&self) -> bool {
        self.next.is_empty()
    }

    /// Returns the destination of the outgoing edge from `vertex`.
    ///
    /// # Panics
    /// Panics if `vertex` is outside `0..self.len()`.
    pub fn next(&self, vertex: usize) -> usize {
        self.next[vertex]
    }

    /// Returns all cycles, excluding vertices that only lead into a cycle.
    ///
    /// Each cycle lists vertices in edge order, without repeating its first
    /// vertex at the end. Each starts at its smallest vertex; cycles are ordered
    /// by those starting vertices. A self-loop is a one-element cycle.
    ///
    /// Time and additional space: O(n).
    ///
    /// # Examples
    /// ```
    /// use cp_library::graph::FunctionalGraph;
    ///
    /// let graph = FunctionalGraph::new(vec![1, 2, 1, 4, 3, 4, 6]);
    /// assert_eq!(graph.cycles(), vec![vec![1, 2], vec![3, 4], vec![6]]);
    /// ```
    pub fn cycles(&self) -> Vec<Vec<usize>> {
        let mut indegree = vec![0usize; self.len()];
        for &to in &self.next {
            indegree[to] += 1;
        }

        let mut queue: Vec<usize> = (0..self.len()).filter(|&v| indegree[v] == 0).collect();
        let mut head = 0;
        while head < queue.len() {
            let to = self.next[queue[head]];
            head += 1;
            indegree[to] -= 1;
            if indegree[to] == 0 {
                queue.push(to);
            }
        }

        // Only cycle vertices retain positive indegree after pruning.
        let mut cycles = Vec::new();
        for start in 0..self.len() {
            if indegree[start] == 0 {
                continue;
            }
            let mut cycle = Vec::new();
            let mut vertex = start;
            loop {
                cycle.push(vertex);
                indegree[vertex] = 0;
                vertex = self.next[vertex];
                if vertex == start {
                    break;
                }
            }
            cycles.push(cycle);
        }
        cycles
    }
}

#[cfg(test)]
mod tests {
    use super::FunctionalGraph;

    #[test]
    fn extracts_multiple_cycles_without_incoming_trees() {
        let graph = FunctionalGraph::new(vec![1, 2, 1, 4, 3, 4, 6, 0, 0]);
        let expected = vec![vec![1, 2], vec![3, 4], vec![6]];
        assert_eq!(graph.cycles(), expected);
        assert_eq!(graph.cycles(), expected);
        assert_eq!(graph.len(), 9);
        assert!(!graph.is_empty());
        assert_eq!(graph.next(7), 0);
    }

    #[test]
    fn preserves_edge_order() {
        let graph = FunctionalGraph::new(vec![2, 0, 1]);
        assert_eq!(graph.cycles(), vec![vec![0, 2, 1]]);
    }

    #[test]
    fn empty_graph() {
        let graph = FunctionalGraph::new(vec![]);
        assert!(graph.is_empty());
        assert_eq!(graph.len(), 0);
        assert!(graph.cycles().is_empty());
    }

    #[test]
    fn self_loops() {
        assert_eq!(FunctionalGraph::new(vec![0]).cycles(), vec![vec![0]]);
        assert_eq!(
            FunctionalGraph::new(vec![0, 1, 2]).cycles(),
            vec![vec![0], vec![1], vec![2]]
        );
    }

    #[test]
    fn long_chain_is_iterative() {
        let n = 100_000;
        let mut next: Vec<_> = (1..=n).collect();
        next[n - 1] = n - 1;
        assert_eq!(FunctionalGraph::new(next).cycles(), vec![vec![n - 1]]);
    }

    #[test]
    #[should_panic(expected = "edge destination is out of bounds")]
    fn invalid_destination() {
        FunctionalGraph::new(vec![1]);
    }
}
