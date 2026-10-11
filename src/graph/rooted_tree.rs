/// Returns `(parent, order)` for an undirected tree rooted at `root`.
///
/// Each edge must appear once in each direction. Vertex indices are zero-based.
/// `parent[root] == root`; every other entry is the vertex's parent.
/// `order` contains every vertex once in breadth-first order (nondecreasing
/// depth), with ties determined by adjacency-list order. Reversing `order`
/// visits children before parents, which is useful for bottom-up tree DP.
///
/// Uses no recursion. Time and additional space are O(n) for an n-vertex tree.
///
/// # Panics
/// Panics if `root` or an edge destination is out of bounds, or the graph is
/// not an undirected tree (including missing reverse edges, parallel edges,
/// self-loops, cycles, and disconnected graphs). An empty graph has no valid root.
///
/// # Examples
/// ```
/// use cp_library::graph::rooted_tree;
///
/// let tree = vec![vec![1, 2], vec![0, 3], vec![0], vec![1]];
/// let (parent, order) = rooted_tree(&tree, 0);
/// assert_eq!(parent, vec![0, 0, 0, 1]);
/// assert_eq!(order, vec![0, 1, 2, 3]);
/// ```
pub fn rooted_tree(tree: &[Vec<usize>], root: usize) -> (Vec<usize>, Vec<usize>) {
    let n = tree.len();
    assert!(root < n, "root vertex is out of bounds");
    let mut parent = vec![usize::MAX; n];
    let mut order = Vec::with_capacity(n);
    parent[root] = root;
    order.push(root);

    let mut head = 0;
    while head < order.len() {
        let vertex = order[head];
        head += 1;
        let mut parent_edges = 0;
        for &to in &tree[vertex] {
            assert!(to < n, "edge destination is out of bounds");
            if vertex != root && to == parent[vertex] {
                parent_edges += 1;
                continue;
            }
            assert!(parent[to] == usize::MAX, "graph must be an undirected tree");
            parent[to] = vertex;
            order.push(to);
        }
        assert!(
            parent_edges == usize::from(vertex != root),
            "each tree edge must appear once in each direction"
        );
    }
    assert!(order.len() == n, "tree must be connected");
    (parent, order)
}

#[cfg(test)]
mod tests {
    use super::rooted_tree;

    #[test]
    fn nonzero_root_breadth_first_order_and_bottom_up_dp() {
        let tree = vec![
            vec![1, 2],
            vec![0, 3, 4],
            vec![0, 5],
            vec![1],
            vec![1],
            vec![2],
        ];
        let (parent, order) = rooted_tree(&tree, 2);
        assert_eq!(parent, vec![2, 0, 2, 1, 1, 2]);
        assert_eq!(order, vec![2, 0, 5, 1, 3, 4]);
        let mut size = vec![1; tree.len()];
        for &v in order.iter().rev() {
            if v != 2 {
                size[parent[v]] += size[v];
            }
        }
        assert_eq!(size, vec![4, 3, 6, 1, 1, 1]);
    }

    #[test]
    fn singleton() {
        assert_eq!(rooted_tree(&[vec![]], 0), (vec![0], vec![0]));
    }

    #[test]
    fn deep_path_without_recursion() {
        let n = 100_000;
        let mut tree = vec![vec![]; n];
        for v in 1..n {
            tree[v - 1].push(v);
            tree[v].push(v - 1);
        }
        let (parent, order) = rooted_tree(&tree, n - 1);
        assert_eq!(order, (0..n).rev().collect::<Vec<_>>());
        assert_eq!(parent[n - 1], n - 1);
        assert!((0..n - 1).all(|v| parent[v] == v + 1));
    }

    #[test]
    fn rejects_invalid_inputs() {
        let cases = vec![
            (vec![], 0),
            (vec![vec![]], 1),
            (vec![vec![1]], 0),
            (vec![vec![0]], 0),
            (vec![vec![], vec![]], 0),
            (vec![vec![1], vec![]], 0),
            (vec![vec![1, 1], vec![0, 0]], 0),
            (vec![vec![1], vec![0, 0]], 0),
            (vec![vec![1, 2], vec![0, 2], vec![0, 1]], 0),
        ];
        for (tree, root) in cases {
            assert!(
                std::panic::catch_unwind(|| rooted_tree(&tree, root)).is_err(),
                "tree={tree:?}, root={root}"
            );
        }
    }
}
