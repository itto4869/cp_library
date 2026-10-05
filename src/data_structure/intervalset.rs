use std::collections::BTreeSet;

/// A union of half-open intervals `[l, r)` with `i64` endpoints.
///
/// Intervals are nonempty, disjoint, and nonadjacent, stored in a `BTreeSet`.
/// Insertion merges overlapping or adjacent intervals; removal may split them.
/// Empty intervals are allowed. Methods taking endpoints panic if `l > r`.
/// No sentinel values or endpoint arithmetic are used.
///
/// With n stored intervals and k affected intervals, insertion and removal take
/// O((k + 1) log(n + 1)) time. Queries take O(log(n + 1)) time.
/// Storage is O(n); `len` counts intervals, not covered integers.
///
/// # Examples
/// ```
/// use cp_library::data_structure::intervalset::IntervalSet;
///
/// let mut set = IntervalSet::new();
/// set.insert(1, 4);
/// set.insert(4, 8);
/// assert_eq!(set.iter().collect::<Vec<_>>(), vec![(1, 8)]);
/// set.remove(3, 6);
/// assert_eq!(set.iter().collect::<Vec<_>>(), vec![(1, 3), (6, 8)]);
/// assert!(set.contains(2));
/// assert!(!set.contains(3));
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IntervalSet {
    intervals: BTreeSet<(i64, i64)>,
}

impl IntervalSet {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of stored intervals in O(1) time.
    pub fn len(&self) -> usize {
        self.intervals.len()
    }

    pub fn is_empty(&self) -> bool {
        self.intervals.is_empty()
    }

    pub fn clear(&mut self) {
        self.intervals.clear();
    }

    /// Iterates over intervals in ascending order of their left endpoints.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = (i64, i64)> + ExactSizeIterator + '_ {
        self.intervals.iter().copied()
    }

    /// Returns the interval containing `point`, or `None` when uncovered.
    pub fn interval_containing(&self, point: i64) -> Option<(i64, i64)> {
        self.intervals
            .range(..=(point, i64::MAX))
            .next_back()
            .copied()
            .filter(|&(_, r)| point < r)
    }

    pub fn contains(&self, point: i64) -> bool {
        self.interval_containing(point).is_some()
    }

    /// Returns whether all of `[l, r)` is covered. Empty intervals return true.
    pub fn contains_range(&self, l: i64, r: i64) -> bool {
        assert!(l <= r, "interval endpoints must satisfy l <= r");
        l == r || self.interval_containing(l).is_some_and(|(_, end)| r <= end)
    }

    /// Adds `[l, r)`, merging overlapping and adjacent intervals.
    pub fn insert(&mut self, mut l: i64, mut r: i64) {
        assert!(l <= r, "interval endpoints must satisfy l <= r");
        if l == r {
            return;
        }
        if let Some((start, end)) = self.intervals.range(..=(l, i64::MAX)).next_back().copied() {
            if end >= l {
                self.intervals.remove(&(start, end));
                l = start;
                r = r.max(end);
            }
        }
        while let Some((start, end)) = self.intervals.range((l, i64::MIN)..).next().copied() {
            if start > r {
                break;
            }
            self.intervals.remove(&(start, end));
            r = r.max(end);
        }
        self.intervals.insert((l, r));
    }

    /// Removes `[l, r)`, retaining any portions outside the removed interval.
    pub fn remove(&mut self, l: i64, r: i64) {
        assert!(l <= r, "interval endpoints must satisfy l <= r");
        if l == r {
            return;
        }
        if let Some((start, end)) = self.interval_containing(l) {
            self.intervals.remove(&(start, end));
            if start < l {
                self.intervals.insert((start, l));
            }
            if end > r {
                self.intervals.insert((r, end));
                return;
            }
        }
        while let Some((start, end)) = self.intervals.range((l, i64::MIN)..).next().copied() {
            if start >= r {
                break;
            }
            self.intervals.remove(&(start, end));
            if end > r {
                self.intervals.insert((r, end));
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::IntervalSet;

    #[test]
    fn merges_splits_and_queries() {
        let mut set = IntervalSet::new();
        assert!(set.is_empty());
        assert!(set.contains_range(5, 5));
        for (l, r) in [(1, 3), (7, 9), (3, 7), (2, 4), (1, 9)] {
            set.insert(l, r);
        }
        assert_eq!(set.iter().collect::<Vec<_>>(), vec![(1, 9)]);
        assert_eq!(set.len(), 1);
        assert_eq!(set.interval_containing(1), Some((1, 9)));
        assert!(!set.contains(9));
        assert!(set.contains_range(1, 9));
        assert!(!set.contains_range(0, 9));
        set.remove(3, 7);
        assert_eq!(set.iter().collect::<Vec<_>>(), vec![(1, 3), (7, 9)]);
        assert!(!set.contains_range(2, 8));
        set.remove(3, 7);
        set.remove(2, 8);
        assert_eq!(set.iter().collect::<Vec<_>>(), vec![(1, 2), (8, 9)]);
        set.clear();
        assert!(set.is_empty());
    }

    #[test]
    fn endpoint_extremes_and_empty_intervals() {
        let mut set = IntervalSet::new();
        set.insert(i64::MIN, i64::MAX);
        set.insert(i64::MAX, i64::MAX);
        set.remove(0, 0);
        assert!(set.contains(i64::MIN));
        assert!(!set.contains(i64::MAX));
        set.remove(-1, 1);
        assert_eq!(
            set.iter().collect::<Vec<_>>(),
            vec![(i64::MIN, -1), (1, i64::MAX)]
        );
        set.insert(-1, 1);
        assert!(set.contains_range(i64::MIN, i64::MAX));
        set.remove(i64::MIN, i64::MAX);
        assert!(set.is_empty());
    }

    #[test]
    fn mixed_operations_match_integer_set() {
        let mut set = IntervalSet::new();
        let mut covered = std::collections::BTreeSet::new();
        let mut seed = 42u64;
        for step in 0..2000 {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let a = (seed % 41) as i64 - 20;
            let b = ((seed >> 8) % 41) as i64 - 20;
            let (l, r) = (a.min(b), a.max(b));
            if step % 2 == 0 {
                set.insert(l, r);
                covered.extend(l..r);
            } else {
                set.remove(l, r);
                for x in l..r {
                    covered.remove(&x);
                }
            }
            for x in -21..=21 {
                assert_eq!(set.contains(x), covered.contains(&x));
            }
            assert_eq!(
                set.contains_range(l, r),
                (l..r).all(|x| covered.contains(&x))
            );
            let intervals: Vec<_> = set.iter().collect();
            assert!(intervals.iter().all(|&(l, r)| l < r));
            assert!(intervals.windows(2).all(|pair| pair[0].1 < pair[1].0));
        }
    }

    #[test]
    #[should_panic(expected = "interval endpoints must satisfy l <= r")]
    fn rejects_reversed_insertion() {
        IntervalSet::new().insert(1, 0);
    }

    #[test]
    #[should_panic(expected = "interval endpoints must satisfy l <= r")]
    fn rejects_reversed_removal() {
        IntervalSet::new().remove(1, 0);
    }

    #[test]
    #[should_panic(expected = "interval endpoints must satisfy l <= r")]
    fn rejects_reversed_query() {
        IntervalSet::new().contains_range(1, 0);
    }
}
