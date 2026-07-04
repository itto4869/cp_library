type Link<T> = Option<Box<Node<T>>>;

#[derive(Clone, Debug)]
struct Node<T> {
    value: T,
    priority: u64,
    size: usize,
    rev: bool,
    left: Link<T>,
    right: Link<T>,
}

impl<T> Node<T> {
    fn new(value: T, priority: u64) -> Self {
        Self {
            value,
            priority,
            size: 1,
            rev: false,
            left: None,
            right: None,
        }
    }

    fn size(node: &Link<T>) -> usize {
        node.as_ref().map_or(0, |node| node.size)
    }

    fn update(node: &mut Box<Self>) {
        node.size = 1 + Self::size(&node.left) + Self::size(&node.right);
    }

    fn toggle(node: &mut Link<T>) {
        if let Some(node) = node {
            node.rev ^= true;
        }
    }

    fn push(node: &mut Box<Self>) {
        if !node.rev {
            return;
        }
        node.rev = false;
        std::mem::swap(&mut node.left, &mut node.right);
        Self::toggle(&mut node.left);
        Self::toggle(&mut node.right);
    }

    fn merge(left: Link<T>, right: Link<T>) -> Link<T> {
        match (left, right) {
            (None, right) => right,
            (left, None) => left,
            (Some(mut left), Some(mut right)) => {
                if left.priority > right.priority {
                    Self::push(&mut left);
                    left.right = Self::merge(left.right.take(), Some(right));
                    Self::update(&mut left);
                    Some(left)
                } else {
                    Self::push(&mut right);
                    right.left = Self::merge(Some(left), right.left.take());
                    Self::update(&mut right);
                    Some(right)
                }
            }
        }
    }

    /// Splits the tree into the first `left_size` elements and the rest.
    fn split(root: Link<T>, left_size: usize) -> (Link<T>, Link<T>) {
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

    fn get(node: &mut Link<T>, index: usize) -> Option<&T> {
        let node = node.as_mut()?;
        Self::push(node);
        let left_size = Self::size(&node.left);

        match index.cmp(&left_size) {
            std::cmp::Ordering::Less => Self::get(&mut node.left, index),
            std::cmp::Ordering::Equal => Some(&node.value),
            std::cmp::Ordering::Greater => Self::get(&mut node.right, index - left_size - 1),
        }
    }

    fn get_mut(node: &mut Link<T>, index: usize) -> Option<&mut T> {
        let node = node.as_mut()?;
        Self::push(node);
        let left_size = Self::size(&node.left);

        match index.cmp(&left_size) {
            std::cmp::Ordering::Less => Self::get_mut(&mut node.left, index),
            std::cmp::Ordering::Equal => Some(&mut node.value),
            std::cmp::Ordering::Greater => Self::get_mut(&mut node.right, index - left_size - 1),
        }
    }

    fn collect(node: &mut Link<T>, out: &mut Vec<T>)
    where
        T: Clone,
    {
        if let Some(node) = node {
            Self::push(node);
            Self::collect(&mut node.left, out);
            out.push(node.value.clone());
            Self::collect(&mut node.right, out);
        }
    }

    fn collect_into(mut node: Box<Self>, out: &mut Vec<T>) {
        Self::push(&mut node);
        let Node {
            value, left, right, ..
        } = *node;

        if let Some(left) = left {
            Self::collect_into(left, out);
        }
        out.push(value);
        if let Some(right) = right {
            Self::collect_into(right, out);
        }
    }
}

#[derive(Clone, Debug)]
pub struct ImplicitTreap<T> {
    root: Link<T>,
    seed: u64,
}

impl<T> Default for ImplicitTreap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ImplicitTreap<T> {
    const DEFAULT_SEED: u64 = 0x9e37_79b9_7f4a_7c15;

    pub fn new() -> Self {
        Self::with_seed(Self::DEFAULT_SEED)
    }

    pub fn with_seed(seed: u64) -> Self {
        Self { root: None, seed }
    }

    pub fn len(&self) -> usize {
        Node::size(&self.root)
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub fn push_front(&mut self, value: T) {
        self.insert(0, value);
    }

    pub fn push_back(&mut self, value: T) {
        self.insert(self.len(), value);
    }

    pub fn insert(&mut self, index: usize, value: T) {
        assert!(index <= self.len());

        let node = Some(Box::new(Node::new(value, self.next_priority())));
        let (left, right) = Node::split(self.root.take(), index);
        self.root = Node::merge(Node::merge(left, node), right);
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len() {
            return None;
        }

        let (left, rest) = Node::split(self.root.take(), index);
        let (middle, right) = Node::split(rest, 1);
        self.root = Node::merge(left, right);

        middle.map(|node| {
            let Node { value, .. } = *node;
            value
        })
    }

    pub fn get(&mut self, index: usize) -> Option<&T> {
        if index >= self.len() {
            return None;
        }
        Node::get(&mut self.root, index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len() {
            return None;
        }
        Node::get_mut(&mut self.root, index)
    }

    pub fn set(&mut self, index: usize, value: T) -> bool {
        if let Some(current) = self.get_mut(index) {
            *current = value;
            true
        } else {
            false
        }
    }

    /// Reverses the half-open range `[left, right)`.
    pub fn reverse(&mut self, left: usize, right: usize) {
        assert!(left <= right);
        assert!(right <= self.len());

        if left == right {
            return;
        }

        let (prefix, rest) = Node::split(self.root.take(), left);
        let (mut middle, suffix) = Node::split(rest, right - left);
        Node::toggle(&mut middle);
        self.root = Node::merge(prefix, Node::merge(middle, suffix));
    }

    pub fn clear(&mut self) {
        self.root = None;
    }

    pub fn to_vec(&mut self) -> Vec<T>
    where
        T: Clone,
    {
        let mut values = Vec::with_capacity(self.len());
        Node::collect(&mut self.root, &mut values);
        values
    }

    pub fn into_vec(self) -> Vec<T> {
        let mut values = Vec::with_capacity(Node::size(&self.root));
        if let Some(root) = self.root {
            Node::collect_into(root, &mut values);
        }
        values
    }

    fn next_priority(&mut self) -> u64 {
        self.seed = self.seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.seed;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }
}

impl<T> Extend<T> for ImplicitTreap<T> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        for value in iter {
            self.push_back(value);
        }
    }
}

impl<T> std::iter::FromIterator<T> for ImplicitTreap<T> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut treap = Self::new();
        treap.extend(iter);
        treap
    }
}

impl<T> From<Vec<T>> for ImplicitTreap<T> {
    fn from(values: Vec<T>) -> Self {
        values.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn next(seed: &mut u64) -> u64 {
        *seed = seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = *seed;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }

    #[test]
    fn test_insert_remove_get_set() {
        let mut treap = ImplicitTreap::new();
        assert!(treap.is_empty());

        treap.push_back(1);
        treap.push_back(3);
        treap.insert(1, 2);
        treap.push_front(0);

        assert_eq!(treap.len(), 4);
        assert_eq!(treap.to_vec(), vec![0, 1, 2, 3]);
        assert_eq!(treap.get(2), Some(&2));
        assert!(treap.set(2, 20));
        assert_eq!(treap.to_vec(), vec![0, 1, 20, 3]);
        assert_eq!(treap.remove(2), Some(20));
        assert_eq!(treap.remove(100), None);
        assert_eq!(treap.into_vec(), vec![0, 1, 3]);
    }

    #[test]
    fn test_reverse_ranges() {
        let mut treap: ImplicitTreap<_> = (0..8).collect();

        treap.reverse(2, 7);
        assert_eq!(treap.to_vec(), vec![0, 1, 6, 5, 4, 3, 2, 7]);

        treap.reverse(0, 8);
        assert_eq!(treap.to_vec(), vec![7, 2, 3, 4, 5, 6, 1, 0]);

        treap.reverse(3, 3);
        assert_eq!(treap.to_vec(), vec![7, 2, 3, 4, 5, 6, 1, 0]);
    }

    #[test]
    fn test_random_operations_against_vec() {
        let mut treap = ImplicitTreap::new();
        let mut vec = Vec::new();
        let mut seed = 1;

        for _ in 0..1000 {
            match next(&mut seed) % 5 {
                0 => {
                    let index = if vec.is_empty() {
                        0
                    } else {
                        next(&mut seed) as usize % (vec.len() + 1)
                    };
                    let value = next(&mut seed) as i32 % 1000;
                    treap.insert(index, value);
                    vec.insert(index, value);
                }
                1 if !vec.is_empty() => {
                    let index = next(&mut seed) as usize % vec.len();
                    assert_eq!(treap.remove(index), Some(vec.remove(index)));
                }
                2 if !vec.is_empty() => {
                    let left = next(&mut seed) as usize % (vec.len() + 1);
                    let right = left + next(&mut seed) as usize % (vec.len() - left + 1);
                    treap.reverse(left, right);
                    vec[left..right].reverse();
                }
                3 if !vec.is_empty() => {
                    let index = next(&mut seed) as usize % vec.len();
                    assert_eq!(treap.get(index), Some(&vec[index]));
                }
                4 if !vec.is_empty() => {
                    let index = next(&mut seed) as usize % vec.len();
                    let value = next(&mut seed) as i32 % 1000;
                    assert!(treap.set(index, value));
                    vec[index] = value;
                }
                _ => {
                    let value = next(&mut seed) as i32 % 1000;
                    treap.push_back(value);
                    vec.push(value);
                }
            }

            assert_eq!(treap.to_vec(), vec);
            assert_eq!(treap.len(), vec.len());
        }
    }
}
