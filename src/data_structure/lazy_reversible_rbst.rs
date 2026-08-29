//! A randomized balanced sequence supporting lazy range updates and reversal.
//!
//! [`LazyReversibleRbst`] stores a sequence rather than an ordered set.  It
//! supports insertion, removal, range products, range actions, and range
//! reversal in an expected logarithmic number of algebraic operations.

/// Algebraic operations used by [`LazyReversibleRbst`].
///
/// `Value` must form a monoid under [`identity`](Self::identity) and
/// [`combine`](Self::combine). `Action` acts on both individual values and
/// products through [`apply`](Self::apply).
///
/// The following laws are required:
///
/// - `combine` is associative and `identity` is its identity element.
/// - `apply(f, combine(x, y), x_len + y_len)` equals
///   `combine(apply(f, x, x_len), apply(f, y, y_len))`.
/// - `apply(compose(new, old), x, len)` equals
///   `apply(new, apply(old, x, len), len)`.
/// - `compose` is associative.
/// - `apply` commutes with reversing a sequence.
///
/// These laws cannot be checked by the compiler.
pub trait LazyReversibleRbstSpec {
    type Value: Clone;
    type Action: Clone;

    /// Returns the identity element of the value monoid.
    fn identity() -> Self::Value;

    /// Combines two adjacent products in left-to-right order.
    fn combine(left: &Self::Value, right: &Self::Value) -> Self::Value;

    /// Applies `action` to a product representing `len` elements.
    fn apply(action: &Self::Action, value: &Self::Value, len: usize) -> Self::Value;

    /// Composes actions in chronological order: `new` after `old`.
    fn compose(new: &Self::Action, old: &Self::Action) -> Self::Action;
}

type Link<S> = Option<Box<Node<S>>>;

struct Node<S>
where
    S: LazyReversibleRbstSpec,
{
    value: S::Value,
    product: S::Value,
    reverse_product: S::Value,
    lazy: Option<S::Action>,
    size: usize,
    reversed: bool,
    left: Link<S>,
    right: Link<S>,
}

impl<S> Clone for Node<S>
where
    S: LazyReversibleRbstSpec,
{
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            product: self.product.clone(),
            reverse_product: self.reverse_product.clone(),
            lazy: self.lazy.clone(),
            size: self.size,
            reversed: self.reversed,
            left: self.left.clone(),
            right: self.right.clone(),
        }
    }
}

impl<S> Node<S>
where
    S: LazyReversibleRbstSpec,
{
    fn new(value: S::Value) -> Self {
        Self {
            product: value.clone(),
            reverse_product: value.clone(),
            value,
            lazy: None,
            size: 1,
            reversed: false,
            left: None,
            right: None,
        }
    }

    fn size(node: &Link<S>) -> usize {
        node.as_ref().map_or(0, |node| node.size)
    }

    fn product(node: &Link<S>) -> S::Value {
        node.as_ref()
            .map_or_else(S::identity, |node| node.product.clone())
    }

    fn reverse_product(node: &Link<S>) -> S::Value {
        node.as_ref()
            .map_or_else(S::identity, |node| node.reverse_product.clone())
    }

    fn update(node: &mut Box<Self>) {
        node.size = 1 + Self::size(&node.left) + Self::size(&node.right);

        let left_product = Self::product(&node.left);
        let right_product = Self::product(&node.right);
        node.product = S::combine(&S::combine(&left_product, &node.value), &right_product);

        let right_reverse_product = Self::reverse_product(&node.right);
        let left_reverse_product = Self::reverse_product(&node.left);
        node.reverse_product = S::combine(
            &S::combine(&right_reverse_product, &node.value),
            &left_reverse_product,
        );
    }

    fn apply_action(node: &mut Link<S>, action: &S::Action) {
        let Some(node) = node else {
            return;
        };

        node.value = S::apply(action, &node.value, 1);
        node.product = S::apply(action, &node.product, node.size);
        node.reverse_product = S::apply(action, &node.reverse_product, node.size);
        node.lazy = Some(match node.lazy.take() {
            Some(old) => S::compose(action, &old),
            None => action.clone(),
        });
    }

    fn toggle(node: &mut Link<S>) {
        let Some(node) = node else {
            return;
        };

        std::mem::swap(&mut node.left, &mut node.right);
        std::mem::swap(&mut node.product, &mut node.reverse_product);
        node.reversed ^= true;
    }

    fn push(node: &mut Box<Self>) {
        if node.reversed {
            Self::toggle(&mut node.left);
            Self::toggle(&mut node.right);
            node.reversed = false;
        }

        if let Some(action) = node.lazy.take() {
            Self::apply_action(&mut node.left, &action);
            Self::apply_action(&mut node.right, &action);
        }
    }

    fn merge(left: Link<S>, right: Link<S>, random: &mut Random) -> Link<S> {
        match (left, right) {
            (None, right) => right,
            (left, None) => left,
            (Some(mut left), Some(mut right)) => {
                let left_size = left.size;
                let total_size = left_size + right.size;

                if random.index(total_size) < left_size {
                    Self::push(&mut left);
                    left.right = Self::merge(left.right.take(), Some(right), random);
                    Self::update(&mut left);
                    Some(left)
                } else {
                    Self::push(&mut right);
                    right.left = Self::merge(Some(left), right.left.take(), random);
                    Self::update(&mut right);
                    Some(right)
                }
            }
        }
    }

    /// Splits the tree into the first `left_size` elements and the rest.
    fn split(root: Link<S>, left_size: usize) -> (Link<S>, Link<S>) {
        match root {
            None => (None, None),
            Some(mut root) => {
                Self::push(&mut root);
                let current_left_size = Self::size(&root.left);

                if left_size <= current_left_size {
                    let (left, right) = Self::split(root.left.take(), left_size);
                    root.left = right;
                    Self::update(&mut root);
                    (left, Some(root))
                } else {
                    let (left, right) =
                        Self::split(root.right.take(), left_size - current_left_size - 1);
                    root.right = left;
                    Self::update(&mut root);
                    (Some(root), right)
                }
            }
        }
    }

    fn build(
        values: &mut [Option<S::Value>],
        left: usize,
        right: usize,
        random: &mut Random,
    ) -> Link<S> {
        if left == right {
            return None;
        }

        let root_index = left + random.index(right - left);
        let value = values[root_index]
            .take()
            .expect("each value is used exactly once while building");
        let mut root = Box::new(Self::new(value));
        root.left = Self::build(values, left, root_index, random);
        root.right = Self::build(values, root_index + 1, right, random);
        Self::update(&mut root);
        Some(root)
    }

    fn get(node: &mut Link<S>, index: usize) -> Option<&S::Value> {
        let node = node.as_mut()?;
        Self::push(node);
        let left_size = Self::size(&node.left);

        match index.cmp(&left_size) {
            std::cmp::Ordering::Less => Self::get(&mut node.left, index),
            std::cmp::Ordering::Equal => Some(&node.value),
            std::cmp::Ordering::Greater => Self::get(&mut node.right, index - left_size - 1),
        }
    }

    fn set(node: &mut Box<Self>, index: usize, value: S::Value) {
        Self::push(node);
        let left_size = Self::size(&node.left);

        match index.cmp(&left_size) {
            std::cmp::Ordering::Less => {
                Self::set(
                    node.left
                        .as_mut()
                        .expect("the index was checked before descending"),
                    index,
                    value,
                );
            }
            std::cmp::Ordering::Equal => node.value = value,
            std::cmp::Ordering::Greater => {
                Self::set(
                    node.right
                        .as_mut()
                        .expect("the index was checked before descending"),
                    index - left_size - 1,
                    value,
                );
            }
        }

        Self::update(node);
    }

    fn collect(node: &mut Link<S>, output: &mut Vec<S::Value>) {
        if let Some(node) = node {
            Self::push(node);
            Self::collect(&mut node.left, output);
            output.push(node.value.clone());
            Self::collect(&mut node.right, output);
        }
    }

    fn collect_into(mut node: Box<Self>, output: &mut Vec<S::Value>) {
        Self::push(&mut node);
        let Self {
            value, left, right, ..
        } = *node;

        if let Some(left) = left {
            Self::collect_into(left, output);
        }
        output.push(value);
        if let Some(right) = right {
            Self::collect_into(right, output);
        }
    }
}

#[derive(Clone)]
struct Random {
    state: u64,
}

impl Random {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }

    fn index(&mut self, upper_bound: usize) -> usize {
        debug_assert!(upper_bound > 0);
        let upper_bound = upper_bound as u64;
        let rejection_threshold = upper_bound.wrapping_neg() % upper_bound;

        loop {
            let product = self.next() as u128 * upper_bound as u128;
            // Reject the incomplete low bucket to avoid modulo bias.
            if product as u64 >= rejection_threshold {
                return (product >> 64) as usize;
            }
        }
    }
}

/// A randomized balanced sequence with lazy range actions and reversals.
///
/// The tree keeps both forward and backward products, so `Value` may use a
/// non-commutative monoid. All ranges are half-open `[left, right)` ranges.
/// Invalid insertion positions and invalid ranges panic.
///
/// Assuming the spec operations and cloning take `O(1)`, point operations,
/// range operations, [`split_off`](Self::split_off), and
/// [`append`](Self::append) take expected `O(log n)` time. Construction and
/// conversion to a vector take `O(n)` time. The expected bounds assume the
/// pseudorandom stream is independent of the operations performed.
pub struct LazyReversibleRbst<S>
where
    S: LazyReversibleRbstSpec,
{
    root: Link<S>,
    random: Random,
}

impl<S> Clone for LazyReversibleRbst<S>
where
    S: LazyReversibleRbstSpec,
{
    fn clone(&self) -> Self {
        Self {
            root: self.root.clone(),
            random: self.random.clone(),
        }
    }
}

impl<S> Default for LazyReversibleRbst<S>
where
    S: LazyReversibleRbstSpec,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<S> LazyReversibleRbst<S>
where
    S: LazyReversibleRbstSpec,
{
    const DEFAULT_SEED: u64 = 0x243f_6a88_85a3_08d3;

    /// Creates an empty tree using a fixed, deterministic default seed.
    pub fn new() -> Self {
        Self::with_seed(Self::DEFAULT_SEED)
    }

    /// Creates an empty tree with a deterministic random seed.
    pub fn with_seed(seed: u64) -> Self {
        Self {
            root: None,
            random: Random::new(seed),
        }
    }

    /// Builds a tree from an iterator in `O(n)` time, assuming `O(1)` spec
    /// operations and cloning.
    pub fn from_iter_with_seed<I>(iter: I, seed: u64) -> Self
    where
        I: IntoIterator<Item = S::Value>,
    {
        let mut values: Vec<_> = iter.into_iter().map(Some).collect();
        let mut random = Random::new(seed);
        let len = values.len();
        let root = Node::build(&mut values, 0, len, &mut random);
        Self { root, random }
    }

    pub fn len(&self) -> usize {
        Node::size(&self.root)
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub fn push_front(&mut self, value: S::Value) {
        self.insert(0, value);
    }

    pub fn push_back(&mut self, value: S::Value) {
        self.insert(self.len(), value);
    }

    /// Inserts `value` at `index`.
    ///
    /// Panics if `index > self.len()`.
    pub fn insert(&mut self, index: usize, value: S::Value) {
        assert!(index <= self.len(), "insertion index out of bounds");

        let (left, right) = Node::split(self.root.take(), index);
        let middle = Some(Box::new(Node::new(value)));
        let left = Node::merge(left, middle, &mut self.random);
        self.root = Node::merge(left, right, &mut self.random);
    }

    /// Removes and returns the value at `index`, or `None` if it is out of bounds.
    pub fn remove(&mut self, index: usize) -> Option<S::Value> {
        if index >= self.len() {
            return None;
        }

        let (left, rest) = Node::split(self.root.take(), index);
        let (middle, right) = Node::split(rest, 1);
        self.root = Node::merge(left, right, &mut self.random);
        middle.map(|node| node.value)
    }

    /// Returns the value at `index`, or `None` if it is out of bounds.
    ///
    /// This method takes `&mut self` because it may push lazy operations.
    pub fn get(&mut self, index: usize) -> Option<&S::Value> {
        if index >= self.len() {
            return None;
        }
        Node::get(&mut self.root, index)
    }

    /// Replaces the value at `index` and reports whether the index existed.
    pub fn set(&mut self, index: usize, value: S::Value) -> bool {
        if index >= self.len() {
            return false;
        }

        Node::set(
            self.root
                .as_mut()
                .expect("a valid index implies a non-empty tree"),
            index,
            value,
        );
        true
    }

    /// Returns the product of the half-open range `[left, right)`.
    ///
    /// This method takes `&mut self` because it splits and merges the tree.
    pub fn fold(&mut self, left: usize, right: usize) -> S::Value {
        self.assert_range(left, right);

        if left == right {
            return S::identity();
        }

        let (prefix, rest) = Node::split(self.root.take(), left);
        let (middle, suffix) = Node::split(rest, right - left);
        let result = Node::product(&middle);
        let rest = Node::merge(middle, suffix, &mut self.random);
        self.root = Node::merge(prefix, rest, &mut self.random);
        result
    }

    /// Alias of [`fold`](Self::fold).
    pub fn prod(&mut self, left: usize, right: usize) -> S::Value {
        self.fold(left, right)
    }

    /// Returns the product of the entire sequence using one `Value::clone`.
    pub fn all_prod(&self) -> S::Value {
        Node::product(&self.root)
    }

    /// Applies `action` to every element in `[left, right)`.
    pub fn apply(&mut self, left: usize, right: usize, action: S::Action) {
        self.assert_range(left, right);

        if left == right {
            return;
        }

        let (prefix, rest) = Node::split(self.root.take(), left);
        let (mut middle, suffix) = Node::split(rest, right - left);
        Node::apply_action(&mut middle, &action);
        let rest = Node::merge(middle, suffix, &mut self.random);
        self.root = Node::merge(prefix, rest, &mut self.random);
    }

    /// Reverses the half-open range `[left, right)`.
    pub fn reverse(&mut self, left: usize, right: usize) {
        self.assert_range(left, right);

        if left == right {
            return;
        }

        let (prefix, rest) = Node::split(self.root.take(), left);
        let (mut middle, suffix) = Node::split(rest, right - left);
        Node::toggle(&mut middle);
        let rest = Node::merge(middle, suffix, &mut self.random);
        self.root = Node::merge(prefix, rest, &mut self.random);
    }

    /// Reverses the entire sequence in `O(1)` time.
    pub fn reverse_all(&mut self) {
        Node::toggle(&mut self.root);
    }

    /// Splits the tree at `index`, returning the suffix `[index, len)`.
    ///
    /// Panics if `index > self.len()`.
    pub fn split_off(&mut self, index: usize) -> Self {
        assert!(index <= self.len(), "split index out of bounds");

        let (left, right) = Node::split(self.root.take(), index);
        self.root = left;
        Self {
            root: right,
            random: Random::new(self.random.next()),
        }
    }

    /// Moves all elements of `other` to the end of `self`.
    ///
    /// `other` is empty after this operation.
    pub fn append(&mut self, other: &mut Self) {
        self.root = Node::merge(self.root.take(), other.root.take(), &mut self.random);
    }

    pub fn clear(&mut self) {
        self.root = None;
    }

    /// Clones the current sequence into a vector.
    ///
    /// This method takes `&mut self` because it pushes all lazy operations.
    pub fn to_vec(&mut self) -> Vec<S::Value> {
        let mut values = Vec::with_capacity(self.len());
        Node::collect(&mut self.root, &mut values);
        values
    }

    /// Consumes the tree and returns its sequence as a vector.
    pub fn into_vec(self) -> Vec<S::Value> {
        let mut values = Vec::with_capacity(Node::size(&self.root));
        if let Some(root) = self.root {
            Node::collect_into(root, &mut values);
        }
        values
    }

    fn assert_range(&self, left: usize, right: usize) {
        assert!(left <= right, "range start exceeds range end");
        assert!(right <= self.len(), "range end out of bounds");
    }
}

impl<S> Extend<S::Value> for LazyReversibleRbst<S>
where
    S: LazyReversibleRbstSpec,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = S::Value>,
    {
        for value in iter {
            self.push_back(value);
        }
    }
}

impl<S> FromIterator<S::Value> for LazyReversibleRbst<S>
where
    S: LazyReversibleRbstSpec,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = S::Value>,
    {
        Self::from_iter_with_seed(iter, Self::DEFAULT_SEED)
    }
}

impl<S> From<Vec<S::Value>> for LazyReversibleRbst<S>
where
    S: LazyReversibleRbstSpec,
{
    fn from(values: Vec<S::Value>) -> Self {
        values.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct RangeAddRangeSum;

    impl LazyReversibleRbstSpec for RangeAddRangeSum {
        type Value = i64;
        type Action = i64;

        fn identity() -> Self::Value {
            0
        }

        fn combine(left: &Self::Value, right: &Self::Value) -> Self::Value {
            left + right
        }

        fn apply(action: &Self::Action, value: &Self::Value, len: usize) -> Self::Value {
            value + action * len as i64
        }

        fn compose(new: &Self::Action, old: &Self::Action) -> Self::Action {
            new + old
        }
    }

    #[derive(Clone, Copy)]
    struct Affine {
        multiplier: i64,
        constant: i64,
    }

    struct RangeAffineRangeSum;

    impl LazyReversibleRbstSpec for RangeAffineRangeSum {
        type Value = i64;
        type Action = Affine;

        fn identity() -> Self::Value {
            0
        }

        fn combine(left: &Self::Value, right: &Self::Value) -> Self::Value {
            left + right
        }

        fn apply(action: &Self::Action, value: &Self::Value, len: usize) -> Self::Value {
            action.multiplier * value + action.constant * len as i64
        }

        fn compose(new: &Self::Action, old: &Self::Action) -> Self::Action {
            Affine {
                multiplier: new.multiplier * old.multiplier,
                constant: new.multiplier * old.constant + new.constant,
            }
        }
    }

    struct Concatenation;

    impl LazyReversibleRbstSpec for Concatenation {
        type Value = String;
        type Action = bool;

        fn identity() -> Self::Value {
            String::new()
        }

        fn combine(left: &Self::Value, right: &Self::Value) -> Self::Value {
            let mut result = String::with_capacity(left.len() + right.len());
            result.push_str(left);
            result.push_str(right);
            result
        }

        fn apply(action: &Self::Action, value: &Self::Value, _: usize) -> Self::Value {
            if !action {
                return value.clone();
            }

            value
                .chars()
                .map(|character| {
                    if character.is_ascii_lowercase() {
                        character.to_ascii_uppercase()
                    } else {
                        character.to_ascii_lowercase()
                    }
                })
                .collect()
        }

        fn compose(new: &Self::Action, old: &Self::Action) -> Self::Action {
            new ^ old
        }
    }

    fn next(seed: &mut u64) -> u64 {
        *seed = seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = *seed;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }

    #[test]
    fn range_add_sum_and_sequence_operations() {
        let mut tree: LazyReversibleRbst<RangeAddRangeSum> = (1..=5).collect();

        assert_eq!(tree.len(), 5);
        assert!(!tree.is_empty());
        assert_eq!(tree.fold(0, 5), 15);
        assert_eq!(tree.fold(2, 2), 0);

        tree.apply(1, 4, 10);
        assert_eq!(tree.all_prod(), 45);
        assert_eq!(tree.prod(1, 4), 39);

        tree.reverse(1, 5);
        assert_eq!(tree.to_vec(), vec![1, 5, 14, 13, 12]);
        assert_eq!(tree.get(2), Some(&14));
        assert!(tree.set(2, -4));
        assert!(!tree.set(20, 0));
        assert_eq!(tree.remove(3), Some(13));
        assert_eq!(tree.remove(20), None);

        tree.push_front(9);
        tree.push_back(8);
        tree.insert(2, 7);
        assert_eq!(tree.to_vec(), vec![9, 1, 7, 5, -4, 12, 8]);

        tree.reverse_all();
        assert_eq!(tree.into_vec(), vec![8, 12, -4, 5, 7, 1, 9]);
    }

    #[test]
    fn composes_affine_actions_in_chronological_order() {
        let mut tree: LazyReversibleRbst<RangeAffineRangeSum> = vec![1, 2, 3, 4].into();

        tree.apply(
            0,
            4,
            Affine {
                multiplier: 2,
                constant: 3,
            },
        );
        tree.apply(
            0,
            4,
            Affine {
                multiplier: 5,
                constant: 7,
            },
        );
        assert_eq!(tree.all_prod(), 188);

        tree.apply(
            1,
            4,
            Affine {
                multiplier: 3,
                constant: -2,
            },
        );
        tree.reverse(1, 4);

        assert_eq!(tree.to_vec(), vec![32, 184, 154, 124]);
        assert_eq!(tree.fold(1, 4), 462);
    }

    #[test]
    fn reversal_preserves_non_commutative_products() {
        let values = ["a", "bc", "D", "ef", "G"].into_iter().map(str::to_owned);
        let mut tree: LazyReversibleRbst<Concatenation> = values.collect();

        assert_eq!(tree.all_prod(), "abcDefG");
        tree.reverse(1, 4);
        assert_eq!(tree.all_prod(), "aefDbcG");
        assert_eq!(tree.fold(1, 4), "efDbc");

        tree.apply(1, 4, true);
        assert_eq!(tree.all_prod(), "aEFdBCG");
        tree.reverse_all();
        assert_eq!(tree.all_prod(), "GBCdEFa");
        assert_eq!(tree.to_vec(), vec!["G", "BC", "d", "EF", "a"]);
    }

    #[test]
    fn split_and_append_keep_pending_operations() {
        let mut tree: LazyReversibleRbst<RangeAffineRangeSum> = (0..8).collect();
        tree.apply(
            0,
            8,
            Affine {
                multiplier: 2,
                constant: 1,
            },
        );
        tree.reverse(1, 7);

        let mut suffix = tree.split_off(3);
        assert_eq!(tree.all_prod(), 25);
        assert_eq!(suffix.all_prod(), 39);

        tree.apply(
            0,
            tree.len(),
            Affine {
                multiplier: -1,
                constant: 0,
            },
        );
        suffix.apply(
            0,
            suffix.len(),
            Affine {
                multiplier: 1,
                constant: 10,
            },
        );
        tree.append(&mut suffix);

        assert!(suffix.is_empty());
        assert_eq!(
            tree.clone().into_vec(),
            vec![-1, -13, -11, 19, 17, 15, 13, 25]
        );

        let mut whole = tree.split_off(0);
        assert!(tree.is_empty());
        tree.append(&mut whole);
        assert!(whole.is_empty());
        let empty = tree.split_off(tree.len());
        assert!(empty.is_empty());
        assert_eq!(tree.into_vec(), vec![-1, -13, -11, 19, 17, 15, 13, 25]);
    }

    #[test]
    fn random_operations_match_vec_without_eagerly_pushing_lazy_tags() {
        let mut tree = LazyReversibleRbst::<RangeAddRangeSum>::with_seed(7);
        let mut expected = Vec::<i64>::new();
        let mut seed = 1;

        for step in 0..5000 {
            match next(&mut seed) % 8 {
                0 => {
                    let index = next(&mut seed) as usize % (expected.len() + 1);
                    let value = next(&mut seed) as i64 % 1000 - 500;
                    tree.insert(index, value);
                    expected.insert(index, value);
                }
                1 if !expected.is_empty() => {
                    let index = next(&mut seed) as usize % expected.len();
                    assert_eq!(tree.remove(index), Some(expected.remove(index)));
                }
                2 => {
                    let left = next(&mut seed) as usize % (expected.len() + 1);
                    let right = left + next(&mut seed) as usize % (expected.len() - left + 1);
                    let addition = next(&mut seed) as i64 % 41 - 20;
                    tree.apply(left, right, addition);
                    for value in &mut expected[left..right] {
                        *value += addition;
                    }
                }
                3 => {
                    let left = next(&mut seed) as usize % (expected.len() + 1);
                    let right = left + next(&mut seed) as usize % (expected.len() - left + 1);
                    tree.reverse(left, right);
                    expected[left..right].reverse();
                }
                4 => {
                    let left = next(&mut seed) as usize % (expected.len() + 1);
                    let right = left + next(&mut seed) as usize % (expected.len() - left + 1);
                    assert_eq!(tree.fold(left, right), expected[left..right].iter().sum());
                }
                5 if !expected.is_empty() => {
                    let index = next(&mut seed) as usize % expected.len();
                    assert_eq!(tree.get(index), Some(&expected[index]));
                }
                6 if !expected.is_empty() => {
                    let index = next(&mut seed) as usize % expected.len();
                    let value = next(&mut seed) as i64 % 1000 - 500;
                    assert!(tree.set(index, value));
                    expected[index] = value;
                }
                7 => {
                    let index = next(&mut seed) as usize % (expected.len() + 1);
                    let mut suffix = tree.split_off(index);
                    let suffix_expected = expected.split_off(index);

                    if next(&mut seed) & 1 == 0 {
                        tree.append(&mut suffix);
                        expected.extend(suffix_expected);
                    } else {
                        suffix.append(&mut tree);
                        let mut rotated = suffix_expected;
                        rotated.append(&mut expected);
                        tree = suffix;
                        expected = rotated;
                    }
                }
                _ => {
                    let value = next(&mut seed) as i64 % 1000 - 500;
                    tree.push_back(value);
                    expected.push(value);
                }
            }

            assert_eq!(tree.len(), expected.len());
            if step % 17 == 0 {
                assert_eq!(tree.all_prod(), expected.iter().sum());
            }
            if step % 113 == 0 {
                assert_eq!(tree.clone().into_vec(), expected);
            }
        }

        assert_eq!(tree.to_vec(), expected);
    }

    #[test]
    fn clear_resets_the_sequence() {
        let mut tree: LazyReversibleRbst<RangeAddRangeSum> = (0..10).collect();
        tree.apply(0, 10, 5);
        tree.reverse_all();
        tree.clear();

        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
        assert_eq!(tree.all_prod(), 0);
        assert_eq!(tree.fold(0, 0), 0);
        assert_eq!(tree.get(0), None);
        assert_eq!(tree.remove(0), None);
    }

    #[test]
    #[should_panic(expected = "insertion index out of bounds")]
    fn insert_rejects_an_out_of_bounds_index() {
        let mut tree = LazyReversibleRbst::<RangeAddRangeSum>::new();
        tree.insert(1, 0);
    }

    #[test]
    #[should_panic(expected = "range start exceeds range end")]
    fn range_rejects_reversed_bounds() {
        let mut tree: LazyReversibleRbst<RangeAddRangeSum> = vec![1, 2].into();
        tree.apply(2, 1, 3);
    }

    #[test]
    #[should_panic(expected = "range end out of bounds")]
    fn range_rejects_an_out_of_bounds_end() {
        let mut tree: LazyReversibleRbst<RangeAddRangeSum> = vec![1, 2].into();
        tree.reverse(0, 3);
    }
}
