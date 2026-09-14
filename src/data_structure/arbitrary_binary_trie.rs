#[derive(Clone, Debug, Default)]
struct Node {
    children: [Option<usize>; 2],
    count: usize,
}

/// A multiset of arbitrary-width unsigned integers represented as MSB-first bits.
///
/// Leading zeros are ignored; an empty slice represents zero. The stored width
/// grows automatically. XOR results are MSB-first with leading zeros removed,
/// and zero is returned as an empty vector. Duplicates are counted separately.
///
/// For input length L and the largest significant width ever inserted W,
/// insertion, removal, lookup and XOR queries take O(L + W) time. Nodes on
/// deleted paths are retained for reuse. Space is O(1 + D * W), where D is the
/// number of distinct values ever inserted. `clear` releases nodes and resets W.
/// All traversals are iterative.
///
/// # Examples
/// ```
/// use cp_library::data_structure::arbitrary_binary_trie::ArbitraryBinaryTrie;
///
/// let mut trie = ArbitraryBinaryTrie::new();
/// let mut large = vec![false; 201];
/// large[0] = true; // 2^200
/// trie.insert(&large);
/// trie.insert(&[true]); // 1
/// assert_eq!(trie.min_xor(&[]), Some(vec![true]));
/// assert_eq!(trie.max_xor(&[]), Some(large));
/// ```
#[derive(Clone, Debug)]
pub struct ArbitraryBinaryTrie {
    nodes: Vec<Node>,
    root: usize,
    width: usize,
}

impl Default for ArbitraryBinaryTrie {
    fn default() -> Self {
        Self::new()
    }
}

fn significant(bits: &[bool]) -> &[bool] {
    &bits[bits.iter().position(|&bit| bit).unwrap_or(bits.len())..]
}

fn padded(bits: &[bool], width: usize) -> impl Iterator<Item = bool> + '_ {
    std::iter::repeat_n(false, width - bits.len()).chain(bits.iter().copied())
}

impl ArbitraryBinaryTrie {
    pub fn new() -> Self {
        Self {
            nodes: vec![Node::default()],
            root: 0,
            width: 0,
        }
    }

    /// Returns the number of values, including duplicates, in O(1) time.
    pub fn len(&self) -> usize {
        self.nodes[self.root].count
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Removes all values, releases nodes, and resets the stored bit width.
    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn insert(&mut self, bits: &[bool]) {
        let bits = significant(bits);
        while self.width < bits.len() {
            let root = self.nodes.len();
            self.nodes.push(Node {
                children: [Some(self.root), None],
                count: self.len(),
            });
            self.root = root;
            self.width += 1;
        }
        let mut node = self.root;
        self.nodes[node].count += 1;
        for bit in padded(bits, self.width) {
            let branch = usize::from(bit);
            node = match self.nodes[node].children[branch] {
                Some(child) => child,
                None => {
                    let child = self.nodes.len();
                    self.nodes.push(Node::default());
                    self.nodes[node].children[branch] = Some(child);
                    child
                }
            };
            self.nodes[node].count += 1;
        }
    }

    /// Returns the multiplicity of the integer represented by `bits`.
    pub fn count(&self, bits: &[bool]) -> usize {
        let bits = significant(bits);
        if bits.len() > self.width {
            return 0;
        }
        let mut node = self.root;
        for bit in padded(bits, self.width) {
            match self.nodes[node].children[usize::from(bit)] {
                Some(child) => node = child,
                None => return 0,
            }
        }
        self.nodes[node].count
    }

    pub fn contains(&self, bits: &[bool]) -> bool {
        self.count(bits) > 0
    }

    /// Counts values less than or equal to `bits`, including duplicates.
    /// Leading zeros are ignored. Takes O(L + W) time and O(1) extra space.
    pub fn count_less(&self, bits: &[bool]) -> usize {
        self.count_inclusive(bits, false)
    }

    /// Counts values greater than or equal to `bits`, including duplicates.
    /// Leading zeros are ignored. Takes O(L + W) time and O(1) extra space.
    pub fn count_greater(&self, bits: &[bool]) -> usize {
        self.count_inclusive(bits, true)
    }

    fn count_inclusive(&self, bits: &[bool], greater: bool) -> usize {
        let bits = significant(bits);
        if bits.len() > self.width {
            return if greater { 0 } else { self.len() };
        }
        let mut node = self.root;
        let mut count = 0;
        for bit in padded(bits, self.width) {
            // The opposite subtree qualifies once this bit determines order.
            if bit != greater {
                if let Some(child) = self.nodes[node].children[usize::from(greater)] {
                    count += self.nodes[child].count;
                }
            }
            match self.nodes[node].children[usize::from(bit)] {
                Some(child) => node = child,
                None => return count,
            }
        }
        count + self.nodes[node].count
    }

    /// Removes one occurrence, returning whether the value was present.
    pub fn remove(&mut self, bits: &[bool]) -> bool {
        if !self.contains(bits) {
            return false;
        }
        let bits = significant(bits);
        let mut node = self.root;
        self.nodes[node].count -= 1;
        for bit in padded(bits, self.width) {
            node = self.nodes[node].children[usize::from(bit)].unwrap();
            self.nodes[node].count -= 1;
        }
        true
    }

    /// Returns the minimum numeric `value XOR x`, or `None` when empty.
    pub fn min_xor(&self, x: &[bool]) -> Option<Vec<bool>> {
        self.xor_extreme(x, false)
    }

    /// Returns the maximum numeric `value XOR x`, or `None` when empty.
    pub fn max_xor(&self, x: &[bool]) -> Option<Vec<bool>> {
        self.xor_extreme(x, true)
    }

    fn xor_extreme(&self, x: &[bool], maximize: bool) -> Option<Vec<bool>> {
        if self.is_empty() {
            return None;
        }
        let x = significant(x);
        // Bits above the stored width are XORed with zero for every value.
        let extra = x.len().saturating_sub(self.width);
        let mut result = x[..extra].to_vec();
        let mut node = self.root;
        for bit in padded(&x[extra..], self.width) {
            let preferred = usize::from(bit ^ maximize);
            let branch = if self.nodes[node].children[preferred]
                .is_some_and(|child| self.nodes[child].count > 0)
            {
                preferred
            } else {
                preferred ^ 1
            };
            result.push((branch != 0) ^ bit);
            node = self.nodes[node].children[branch].unwrap();
        }
        Some(significant(&result).to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::ArbitraryBinaryTrie;

    fn bits(value: u128) -> Vec<bool> {
        let width = 128 - value.leading_zeros();
        (0..width)
            .rev()
            .map(|bit| value & (1 << bit) != 0)
            .collect()
    }

    #[test]
    fn zero_duplicates_and_leading_zeros() {
        let mut trie = ArbitraryBinaryTrie::default();
        assert!(trie.is_empty());
        assert_eq!(trie.min_xor(&[]), None);
        assert_eq!(trie.max_xor(&[]), None);
        assert!(!trie.remove(&[]));
        trie.insert(&[]);
        trie.insert(&[false, false]);
        assert_eq!(trie.count(&[false]), 2);
        assert_eq!(trie.min_xor(&[]), Some(vec![]));
        assert_eq!(trie.max_xor(&[true, false]), Some(vec![true, false]));
        trie.insert(&[false, true]);
        assert_eq!(trie.count(&[true]), 1);
        assert_eq!(trie.count(&[]), 2);
        assert_eq!(trie.len(), 3);
        assert!(trie.remove(&[]));
        assert!(trie.remove(&[false]));
        assert!(!trie.remove(&[]));
        assert_eq!(trie.min_xor(&[]), Some(vec![true]));
        assert!(trie.remove(&[true]));
        assert_eq!(trie.max_xor(&[]), None);
        trie.insert(&[]);
        assert_eq!(trie.min_xor(&[]), Some(vec![]));
        trie.clear();
        assert!(trie.is_empty());
        trie.insert(&[true]);
        assert!(trie.contains(&[true]));
    }

    #[test]
    fn mixed_operations_match_u128_multiset() {
        let mut trie = ArbitraryBinaryTrie::new();
        let mut values = Vec::<u128>::new();
        let mut seed = 42u128;
        for step in 0..1500 {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let value = (seed % 40) << (step % 124);
            if step % 3 == 0 {
                let value = values.first().copied().unwrap_or(value);
                assert_eq!(trie.remove(&bits(value)), !values.is_empty());
                if !values.is_empty() {
                    values.swap_remove(0);
                }
            } else {
                trie.insert(&bits(value));
                values.push(value);
            }
            assert_eq!(trie.len(), values.len());
            for query in [0, value, seed, u128::MAX] {
                assert_eq!(
                    trie.count_less(&bits(query)),
                    values.iter().filter(|&&v| v <= query).count()
                );
                assert_eq!(
                    trie.count_greater(&bits(query)),
                    values.iter().filter(|&&v| v >= query).count()
                );
            }
            assert_eq!(
                trie.count(&bits(value)),
                values.iter().filter(|&&v| v == value).count()
            );
            assert_eq!(
                trie.min_xor(&bits(seed)),
                values.iter().map(|v| v ^ seed).min().map(bits)
            );
            assert_eq!(
                trie.max_xor(&bits(seed)),
                values.iter().map(|v| v ^ seed).max().map(bits)
            );
        }
    }

    #[test]
    fn very_wide_values_growth_deletion_and_long_queries() {
        let mut trie = ArbitraryBinaryTrie::new();
        trie.insert(&[true]);
        let mut large = vec![false; 10_000];
        large[0] = true;
        trie.insert(&large);
        assert_eq!(trie.min_xor(&[]), Some(vec![true]));
        assert_eq!(trie.max_xor(&[]), Some(large.clone()));
        assert_eq!(trie.min_xor(&large), Some(vec![]));
        let mut query = vec![false; 10_001];
        query[0] = true;
        let mut expected = query.clone();
        expected[10_000] = true;
        assert_eq!(trie.min_xor(&query), Some(expected));
        assert_eq!(trie.count(&query), 0);
        assert!(!trie.remove(&query));
        assert!(trie.remove(&large));
        assert_eq!(trie.max_xor(&[]), Some(vec![true]));
        trie.insert(&large);
        assert!(trie.contains(&large));
        assert_eq!(trie.count_less(&large), 2);
        assert_eq!(trie.count_greater(&large), 1);
        assert_eq!(trie.count_less(&query), 2);
        assert_eq!(trie.count_greater(&query), 0);
        assert!(trie.remove(&large));
        assert_eq!(trie.count_less(&large), 1);
        assert_eq!(trie.count_greater(&large), 0);
    }

    #[test]
    fn inclusive_counts_with_duplicates_zero_and_deleted_paths() {
        let mut trie = ArbitraryBinaryTrie::new();
        assert_eq!(trie.count_less(&[]), 0);
        assert_eq!(trie.count_greater(&[]), 0);
        trie.insert(&[]);
        trie.insert(&[false]);
        assert_eq!(trie.count_less(&[]), 2);
        assert_eq!(trie.count_greater(&[]), 2);
        for value in [2, 2, 5] {
            trie.insert(&bits(value));
        }
        for (query, less, greater) in [
            (0, 2, 5),
            (1, 2, 3),
            (2, 4, 3),
            (3, 4, 1),
            (5, 5, 1),
            (6, 5, 0),
        ] {
            let mut padded_query = vec![false; 3];
            padded_query.extend(bits(query));
            assert_eq!(trie.count_less(&padded_query), less);
            assert_eq!(trie.count_greater(&padded_query), greater);
        }
        assert!(trie.remove(&bits(2)));
        assert_eq!(trie.count_less(&bits(2)), 3);
        assert_eq!(trie.count_greater(&bits(2)), 2);
        for value in [0, 0, 2, 5] {
            assert!(trie.remove(&bits(value)));
        }
        assert_eq!(trie.count_less(&bits(2)), 0);
        assert_eq!(trie.count_greater(&bits(2)), 0);
        trie.clear();
        assert_eq!(trie.count_less(&bits(5)), 0);
        assert_eq!(trie.count_greater(&[]), 0);
    }
}
