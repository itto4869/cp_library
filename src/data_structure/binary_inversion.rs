use ac_library::{Monoid, Segtree};

#[derive(Clone, Copy, Default)]
struct Counts {
    zeros: usize,
    ones: usize,
    inversions: usize,
}

#[derive(Clone)]
struct BinaryInversionMonoid;

impl Monoid for BinaryInversionMonoid {
    type S = Counts;
    fn identity() -> Counts {
        Counts::default()
    }
    fn binary_operation(left: &Counts, right: &Counts) -> Counts {
        Counts {
            zeros: left.zeros + right.zeros,
            ones: left.ones + right.ones,
            inversions: left
                .ones
                .checked_mul(right.zeros)
                .and_then(|cross| {
                    left.inversions
                        .checked_add(right.inversions)?
                        .checked_add(cross)
                })
                .expect("inversion count exceeds usize"),
        }
    }
}

fn leaf<T: TryInto<u8>>(value: T) -> Counts {
    let value = value.try_into().ok().expect("binary value must be 0 or 1");
    assert!(value <= 1, "binary value must be 0 or 1");
    Counts {
        zeros: usize::from(value == 0),
        ones: usize::from(value == 1),
        inversions: 0,
    }
}

/// Point updates and inversion counts on half-open subranges of a 0/1 array.
///
/// Inputs accept integer types convertible to `u8` (including `usize` and `i64`).
/// Conversion and 0/1 validation are checked. Counts use `usize`; construction
/// and updates panic if the total inversion count exceeds `usize::MAX`.
/// Discard the structure after an overflowing update.
///
/// An inversion is a pair `i < j` with values 1 and 0, respectively.
/// Construction takes O(n) time and space. `set` and `query` take O(log n)
/// time; `len`, `is_empty`, and `all_inversions` take O(1).
///
/// ```
/// use cp_library::data_structure::binary_inversion::BinaryInversion;
/// let mut tree = BinaryInversion::new(&[1, 0, 1, 0]);
/// assert_eq!(tree.query(0, 4), 3);
/// assert_eq!(tree.query(1, 4), 1);
/// tree.set(1, 1);
/// assert_eq!(tree.query(0, 3), 0);
/// ```
#[derive(Clone)]
pub struct BinaryInversion {
    tree: Segtree<BinaryInversionMonoid>,
    len: usize,
}

impl BinaryInversion {
    /// Creates a structure from 0/1 values. Empty arrays are supported.
    ///
    /// # Panics
    /// Panics if any value is neither 0 nor 1, or the inversion count overflows.
    pub fn new<T: Copy + TryInto<u8>>(values: &[T]) -> Self {
        Self {
            tree: Segtree::from(values.iter().copied().map(leaf).collect::<Vec<_>>()),
            len: values.len(),
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Assigns `value` at `index`.
    ///
    /// # Panics
    /// Panics if `index >= len()`, `value` is neither 0 nor 1, or the count overflows.
    pub fn set<T: TryInto<u8>>(&mut self, index: usize, value: T) {
        assert!(index < self.len, "index is out of bounds");
        self.tree.set(index, leaf(value));
    }

    /// Counts inversions entirely within `[l, r)`. Empty ranges return zero.
    ///
    /// # Panics
    /// Panics unless `l <= r <= len()`.
    pub fn query(&self, l: usize, r: usize) -> usize {
        assert!(l <= r && r <= self.len, "range must satisfy l <= r <= len");
        self.tree.prod(l..r).inversions
    }

    /// Returns the inversion count of the entire array.
    pub fn all_inversions(&self) -> usize {
        self.tree.all_prod().inversions
    }
}

#[cfg(test)]
mod tests {
    use super::BinaryInversion;

    fn brute(values: &[u8]) -> usize {
        (0..values.len())
            .map(|i| values[i + 1..].iter().filter(|&&v| values[i] > v).count())
            .sum()
    }

    #[test]
    fn updates_and_all_subranges_match_brute_force() {
        let mut seed = 42u64;
        for n in 1..20 {
            let mut values = vec![0; n];
            let mut tree = BinaryInversion::new(&values);
            for _ in 0..60 {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                let index = seed as usize % n;
                let value = (seed >> 8 & 1) as u8;
                values[index] = value;
                tree.set(index, value);
                assert_eq!(tree.len(), n);
                assert_eq!(tree.all_inversions(), brute(&values));
                for l in 0..=n {
                    for r in l..=n {
                        assert_eq!(tree.query(l, r), brute(&values[l..r]));
                    }
                }
            }
        }
    }

    #[test]
    fn empty_and_large_counts() {
        let empty = BinaryInversion::new::<usize>(&[]);
        assert!(empty.is_empty());
        assert_eq!(empty.query(0, 0), 0);
        assert_eq!(empty.all_inversions(), 0);
        let values: Vec<_> = std::iter::repeat_n(1, 10_000)
            .chain(std::iter::repeat_n(0, 10_000))
            .collect();
        assert_eq!(BinaryInversion::new(&values).all_inversions(), 100_000_000);
    }

    #[test]
    fn rejects_invalid_inputs() {
        assert!(std::panic::catch_unwind(|| BinaryInversion::new(&[-1i64])).is_err());
        assert!(std::panic::catch_unwind(|| BinaryInversion::new(&[256usize])).is_err());
        assert!(std::panic::catch_unwind(|| BinaryInversion::new(&[2])).is_err());
        assert!(std::panic::catch_unwind(|| BinaryInversion::new(&[0]).set(1, 0)).is_err());
        assert!(std::panic::catch_unwind(|| BinaryInversion::new(&[0]).set(0, 2)).is_err());
        assert!(std::panic::catch_unwind(|| BinaryInversion::new(&[0]).query(1, 0)).is_err());
        assert!(std::panic::catch_unwind(|| BinaryInversion::new(&[0]).query(0, 2)).is_err());
    }

    #[test]
    fn accepts_different_integer_types() {
        let mut tree = BinaryInversion::new(&[1usize, 0, 1, 0]);
        let count: usize = tree.all_inversions();
        assert_eq!(count, 3);
        tree.set(0, 0i64);
        assert_eq!(tree.query(0, 4), 1);
        assert_eq!(BinaryInversion::new(&[1i64, 0, 0]).all_inversions(), 2);
        assert_eq!(BinaryInversion::new(&[1u128, 0]).all_inversions(), 1);
    }

    #[test]
    #[should_panic(expected = "inversion count exceeds usize")]
    fn detects_count_overflow() {
        use super::{BinaryInversionMonoid, Counts};
        use ac_library::Monoid;
        BinaryInversionMonoid::binary_operation(
            &Counts {
                ones: usize::MAX,
                ..Counts::default()
            },
            &Counts {
                zeros: 2,
                ..Counts::default()
            },
        );
    }
}
