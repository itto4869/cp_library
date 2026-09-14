#[derive(Clone, Debug, Default)]
struct Node {
    children: [Option<usize>; 2],
    count: usize,
}

/// A multiset of `u64` values stored in a 64-bit binary trie.
///
/// Duplicates are counted separately. Operations take O(64) time, except
/// `new`, `len`, and `is_empty`, which take O(1). Deleted nodes are retained
/// for reuse along the same paths; space is O(1 + 64D), where D is the number
/// of distinct values ever inserted. `clear` releases all nodes.
#[derive(Clone, Debug)]
pub struct BinaryTrie {
    nodes: Vec<Node>,
}

impl Default for BinaryTrie {
    fn default() -> Self {
        Self::new()
    }
}

impl BinaryTrie {
    pub fn new() -> Self {
        Self {
            nodes: vec![Node::default()],
        }
    }

    /// Returns the number of values, including duplicates.
    pub fn len(&self) -> usize {
        self.nodes[0].count
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn insert(&mut self, value: u64) {
        let mut node = 0;
        self.nodes[node].count += 1;
        for bit in (0..64).rev() {
            let branch = ((value >> bit) & 1) as usize;
            let child = match self.nodes[node].children[branch] {
                Some(child) => child,
                None => {
                    let child = self.nodes.len();
                    self.nodes.push(Node::default());
                    self.nodes[node].children[branch] = Some(child);
                    child
                }
            };
            node = child;
            self.nodes[node].count += 1;
        }
    }

    /// Returns the multiplicity of `value`.
    pub fn count(&self, value: u64) -> usize {
        let mut node = 0;
        for bit in (0..64).rev() {
            let branch = ((value >> bit) & 1) as usize;
            match self.nodes[node].children[branch] {
                Some(child) => node = child,
                None => return 0,
            }
        }
        self.nodes[node].count
    }

    pub fn contains(&self, value: u64) -> bool {
        self.count(value) > 0
    }

    /// Removes one occurrence, returning whether it was present.
    pub fn remove(&mut self, value: u64) -> bool {
        if !self.contains(value) {
            return false;
        }
        let mut node = 0;
        self.nodes[node].count -= 1;
        for bit in (0..64).rev() {
            let branch = ((value >> bit) & 1) as usize;
            node = self.nodes[node].children[branch].unwrap();
            self.nodes[node].count -= 1;
        }
        true
    }

    /// Returns the minimum `value ^ x`, or `None` when empty.
    pub fn min_xor(&self, x: u64) -> Option<u64> {
        self.xor_extreme(x, false)
    }

    /// Returns the maximum `value ^ x`, or `None` when empty.
    pub fn max_xor(&self, x: u64) -> Option<u64> {
        self.xor_extreme(x, true)
    }

    fn xor_extreme(&self, x: u64, maximize: bool) -> Option<u64> {
        if self.is_empty() {
            return None;
        }
        let mut node = 0;
        let mut result = 0;
        for bit in (0..64).rev() {
            let x_bit = ((x >> bit) & 1) as usize;
            let preferred = x_bit ^ usize::from(maximize);
            let branch = if self.nodes[node].children[preferred]
                .is_some_and(|child| self.nodes[child].count > 0)
            {
                preferred
            } else {
                preferred ^ 1
            };
            result |= ((branch ^ x_bit) as u64) << bit;
            node = self.nodes[node].children[branch].unwrap();
        }
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::BinaryTrie;

    #[test]
    fn duplicates_deletion_and_full_width_values() {
        let mut trie = BinaryTrie::default();
        assert!(trie.is_empty());
        assert_eq!(trie.min_xor(0), None);
        assert_eq!(trie.max_xor(0), None);
        assert!(!trie.remove(0));
        for value in [0, 0, 1 << 63, u64::MAX] {
            trie.insert(value);
        }
        assert_eq!(trie.len(), 4);
        assert_eq!(trie.count(0), 2);
        assert_eq!(trie.min_xor(u64::MAX), Some(0));
        assert_eq!(trie.max_xor(0), Some(u64::MAX));
        assert!(trie.remove(0));
        assert!(trie.contains(0));
        assert!(trie.remove(0));
        assert!(!trie.remove(0));
        assert_eq!(trie.min_xor(0), Some(1 << 63));
        trie.clear();
        assert!(trie.is_empty());
        trie.insert(7);
        assert_eq!(trie.min_xor(3), Some(4));
        assert_eq!(trie.max_xor(3), Some(4));
    }

    #[test]
    fn mixed_operations_match_vec() {
        let mut trie = BinaryTrie::new();
        let mut values = Vec::new();
        let mut seed = 42u64;
        for step in 0..2000 {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let value = seed % 100;
            if step % 3 == 0 {
                let position = values.iter().position(|&v| v == value);
                assert_eq!(trie.remove(value), position.is_some());
                if let Some(position) = position {
                    values.swap_remove(position);
                }
            } else {
                trie.insert(value);
                values.push(value);
            }
            assert_eq!(trie.len(), values.len());
            assert_eq!(
                trie.count(value),
                values.iter().filter(|&&v| v == value).count()
            );
            assert_eq!(trie.min_xor(seed), values.iter().map(|v| v ^ seed).min());
            assert_eq!(trie.max_xor(seed), values.iter().map(|v| v ^ seed).max());
        }
    }
}
