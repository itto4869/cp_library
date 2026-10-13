/// Counts pairs `(i, j)` such that `i < j` and `values[i] > values[j]`.
///
/// Equal values do not form inversions. The input is not modified.
/// Supports all integer types, and more generally any `Ord` type without cloning.
/// Uses coordinate compression and a Fenwick tree: O(n log n) time and O(n)
/// extra space, assuming constant-time comparisons. Returns zero for empty input.
///
/// # Panics
/// Panics if the inversion count exceeds `usize::MAX`.
///
/// ```
/// use cp_library::algorithm::inversion_count;
/// assert_eq!(inversion_count(&[3, -1, 2, -1]), 4);
/// ```
pub fn inversion_count<T: Ord>(values: &[T]) -> usize {
    let mut sorted: Vec<&T> = values.iter().collect();
    sorted.sort_unstable();
    sorted.dedup();
    let mut bit = vec![0usize; sorted.len() + 1];
    let mut result = 0usize;
    for (seen, value) in values.iter().enumerate() {
        let rank = sorted.binary_search(&value).unwrap() + 1;
        let mut index = rank;
        let mut less_or_equal = 0;
        while index > 0 {
            less_or_equal += bit[index];
            index &= index - 1;
        }
        result = result
            .checked_add(seen - less_or_equal)
            .expect("inversion count exceeds usize");
        index = rank;
        while index < bit.len() {
            bit[index] += 1;
            index += index & index.wrapping_neg();
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::inversion_count;

    #[test]
    fn boundaries_and_duplicates() {
        assert_eq!(inversion_count::<i64>(&[]), 0);
        assert_eq!(inversion_count(&[1]), 0);
        assert_eq!(inversion_count(&[1, 1, 1]), 0);
        assert_eq!(inversion_count(&[-3, -2, -1, 0]), 0);
        assert_eq!(inversion_count(&[3, -1, 2, -1]), 4);
        assert_eq!(inversion_count(&[i128::MAX, 0, i128::MIN]), 3);
        assert_eq!(inversion_count(&[u128::MAX, 0]), 1);
        let descending: Vec<_> = (0..10_000).rev().collect();
        assert_eq!(inversion_count(&descending), 10_000usize * 9_999 / 2);
    }

    #[test]
    fn matches_pair_enumeration() {
        let mut seed = 42u64;
        for n in 0..80 {
            let values: Vec<_> = (0..n)
                .map(|_| {
                    seed ^= seed << 13;
                    seed ^= seed >> 7;
                    seed ^= seed << 17;
                    (seed % 15) as i64 - 7
                })
                .collect();
            let expected = (0..n)
                .map(|i| (i + 1..n).filter(|&j| values[i] > values[j]).count())
                .sum::<usize>();
            assert_eq!(inversion_count(&values), expected);
        }
    }
}
