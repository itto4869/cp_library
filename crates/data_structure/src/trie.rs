use std::collections::BTreeMap;

#[derive(Clone, Debug, Default)]
struct Node {
    children: BTreeMap<char, usize>,
    count: usize,
    terminal_count: usize,
}

/// A multiset of UTF-8 strings, indexed by Unicode scalar values (`char`).
///
/// Empty strings and duplicates are supported. No Unicode normalization is
/// performed. String operations take O(L log(B + 1)) time, except `count_less`
/// and `count_greater`, which take O(1 + L * (B + 1)) time, where L is the
/// number of characters and B the maximum branching factor. Deleted nodes
/// remain available for reuse; space is proportional to the distinct prefixes
/// ever inserted. `clear` releases all nodes. Traversal and deletion are iterative.
#[derive(Clone, Debug)]
pub struct Trie {
    nodes: Vec<Node>,
}

impl Default for Trie {
    fn default() -> Self {
        Self::new()
    }
}

impl Trie {
    pub fn new() -> Self {
        Self {
            nodes: vec![Node::default()],
        }
    }

    /// Returns the number of strings, including duplicates, in O(1) time.
    pub fn len(&self) -> usize {
        self.nodes[0].count
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn insert(&mut self, word: &str) {
        let mut node = 0;
        self.nodes[node].count += 1;
        for ch in word.chars() {
            let child = match self.nodes[node].children.get(&ch) {
                Some(&child) => child,
                None => {
                    let child = self.nodes.len();
                    self.nodes.push(Node::default());
                    self.nodes[node].children.insert(ch, child);
                    child
                }
            };
            node = child;
            self.nodes[node].count += 1;
        }
        self.nodes[node].terminal_count += 1;
    }

    fn find(&self, text: &str) -> Option<usize> {
        let mut node = 0;
        for ch in text.chars() {
            node = *self.nodes[node].children.get(&ch)?;
        }
        Some(node)
    }

    /// Returns the number of exact matches.
    pub fn count(&self, word: &str) -> usize {
        self.find(word)
            .map_or(0, |node| self.nodes[node].terminal_count)
    }

    pub fn contains(&self, word: &str) -> bool {
        self.count(word) > 0
    }

    /// Counts strings lexicographically less than or equal to `word`.
    /// Includes duplicates. Order matches Rust's `str` ordering (no locale or
    /// normalization rules). Takes O(1 + L * (B + 1)) time and O(1) extra space.
    pub fn count_less(&self, word: &str) -> usize {
        let (less, equal) = self.count_before_and_equal(word);
        less + equal
    }

    /// Counts strings lexicographically greater than or equal to `word`.
    /// Includes duplicates. Order matches Rust's `str` ordering (no locale or
    /// normalization rules). Takes O(1 + L * (B + 1)) time and O(1) extra space.
    pub fn count_greater(&self, word: &str) -> usize {
        let (less, _) = self.count_before_and_equal(word);
        self.len() - less
    }

    fn count_before_and_equal(&self, word: &str) -> (usize, usize) {
        let mut node = 0;
        let mut less = 0;
        for ch in word.chars() {
            // A proper prefix and all smaller next characters precede word.
            less += self.nodes[node].terminal_count;
            for (_, &child) in self.nodes[node].children.range(..ch) {
                less += self.nodes[child].count;
            }
            match self.nodes[node].children.get(&ch) {
                Some(&child) => node = child,
                None => return (less, 0),
            }
        }
        (less, self.nodes[node].terminal_count)
    }

    /// Counts strings beginning with `prefix`, including duplicates.
    /// An empty prefix matches every stored string.
    pub fn prefix_count(&self, prefix: &str) -> usize {
        self.find(prefix).map_or(0, |node| self.nodes[node].count)
    }

    /// Returns whether any stored string starts with `prefix`.
    /// Returns false for all prefixes, including empty, when the trie is empty.
    pub fn starts_with(&self, prefix: &str) -> bool {
        self.prefix_count(prefix) > 0
    }

    /// Removes one exact occurrence, returning whether it was present.
    pub fn remove(&mut self, word: &str) -> bool {
        if !self.contains(word) {
            return false;
        }
        let mut node = 0;
        self.nodes[node].count -= 1;
        for ch in word.chars() {
            node = self.nodes[node].children[&ch];
            self.nodes[node].count -= 1;
        }
        self.nodes[node].terminal_count -= 1;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::Trie;

    #[test]
    fn shared_prefixes_duplicates_and_deletion() {
        let mut trie = Trie::new();
        for word in ["app", "apple", "app", "apt", "bat"] {
            trie.insert(word);
        }
        assert_eq!(trie.len(), 5);
        assert_eq!(trie.count("app"), 2);
        assert_eq!(trie.prefix_count("ap"), 4);
        assert!(!trie.contains("ap"));
        assert!(!trie.remove("ap"));
        assert!(trie.remove("app"));
        assert!(trie.remove("app"));
        assert!(!trie.remove("app"));
        assert!(trie.starts_with("app"));
        assert_eq!(trie.prefix_count("app"), 1);
        assert!(trie.remove("apple"));
        assert!(!trie.starts_with("app"));
        assert!(trie.contains("apt"));
        trie.insert("app");
        assert_eq!(trie.count("app"), 1);
        assert_eq!(trie.len(), 3);
    }

    #[test]
    fn empty_strings_unicode_and_clear() {
        let mut trie = Trie::default();
        assert!(trie.is_empty());
        assert!(!trie.starts_with(""));
        assert!(!trie.remove(""));
        for word in ["", "", "日本", "日本語", "🦀Rust"] {
            trie.insert(word);
        }
        assert_eq!(trie.count(""), 2);
        assert_eq!(trie.prefix_count(""), 5);
        assert_eq!(trie.prefix_count("日"), 2);
        assert!(trie.contains("🦀Rust"));
        assert!(trie.remove(""));
        assert!(trie.remove("日本"));
        assert_eq!(trie.count(""), 1);
        assert_eq!(trie.prefix_count("日本"), 1);
        trie.clear();
        assert!(trie.is_empty());
        assert!(!trie.contains("日本語"));
        trie.insert("new");
        assert_eq!(trie.len(), 1);
    }

    #[test]
    fn long_word_is_iterative() {
        let word = "a".repeat(100_000);
        let mut trie = Trie::new();
        trie.insert(&word);
        assert!(trie.contains(&word));
        assert_eq!(trie.count_less(&word), 1);
        assert_eq!(trie.count_greater(&word), 1);
        assert!(trie.remove(&word));
        assert!(trie.is_empty());
    }

    #[test]
    fn inclusive_counts_match_string_order_after_updates() {
        let mut trie = Trie::new();
        let mut words = Vec::new();
        let queries = [
            "",
            "a",
            "ap",
            "app",
            "apple",
            "b",
            "é",
            "e\u{301}",
            "日",
            "日本",
            "🦀",
            "\u{10ffff}",
        ];
        let check = |trie: &Trie, words: &Vec<&str>| {
            for query in queries {
                assert_eq!(
                    trie.count_less(query),
                    words.iter().filter(|&&w| w <= query).count(),
                    "<= {query:?}"
                );
                assert_eq!(
                    trie.count_greater(query),
                    words.iter().filter(|&&w| w >= query).count(),
                    ">= {query:?}"
                );
            }
        };
        check(&trie, &words);
        for word in [
            "",
            "",
            "app",
            "apple",
            "app",
            "bat",
            "日本",
            "日本語",
            "é",
            "e\u{301}",
            "🦀Rust",
        ] {
            trie.insert(word);
            words.push(word);
            check(&trie, &words);
        }
        assert!(!trie.remove("ap"));
        while let Some(word) = words.pop() {
            assert!(trie.remove(word));
            check(&trie, &words);
        }
        trie.insert("app");
        words.push("app");
        check(&trie, &words);
        trie.clear();
        words.clear();
        check(&trie, &words);
    }
}
