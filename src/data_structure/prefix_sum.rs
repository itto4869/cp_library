use std::ops::Add;

/// Borrows a slice and returns its prefix sums with a leading zero.
/// The input is neither consumed nor modified.
///
/// For an input of length n, the result has length n + 1, and `prefix[i]`
/// is the sum of the first i elements. Empty input returns `[T::default()]`.
/// The sum of a half-open range `[l, r)` is `prefix[r] - prefix[l]` for
/// types supporting subtraction, with `0 <= l <= r <= n`.
///
/// `T::default()` must be zero. Integer prefix sums and range sums must fit
/// in `T`; floating-point arithmetic may introduce rounding error.
/// Takes O(n) time and O(n) space, assuming constant-time arithmetic.
///
/// ```
/// use cp_library::data_structure::prefix_sum::prefix_sum;
///
/// let values = vec![2, -1, 4, 3];
/// let sums = prefix_sum(&values);
/// assert_eq!(sums, vec![0, 2, 1, 5, 8]);
/// assert_eq!(sums[3] - sums[1], 3); // sum of [1, 3)
/// ```
pub fn prefix_sum<T>(values: &[T]) -> Vec<T>
where
    T: Copy + Default + Add<Output = T>,
{
    let mut prefix = Vec::with_capacity(values.len() + 1);
    let mut total = T::default();
    prefix.push(total);
    for &value in values {
        total = total + value;
        prefix.push(total);
    }
    prefix
}

#[cfg(test)]
mod tests {
    use super::prefix_sum;

    #[test]
    fn all_ranges_match_direct_sum() {
        let values: Vec<i64> = (0..80).map(|i| (i * 17 % 23) - 11).collect();
        let sums = prefix_sum(&values);
        assert_eq!(sums.len(), values.len() + 1);
        assert_eq!(sums[0], 0);
        for l in 0..=values.len() {
            for r in l..=values.len() {
                assert_eq!(sums[r] - sums[l], values[l..r].iter().sum::<i64>());
            }
        }
    }

    #[test]
    fn empty_singleton_and_numeric_types() {
        assert_eq!(prefix_sum::<usize>(&[]), vec![0]);
        assert_eq!(prefix_sum(&[usize::MAX]), vec![0, usize::MAX]);
        let wide = prefix_sum(&[i128::from(i64::MAX), 1]);
        assert_eq!(
            wide,
            vec![0, i128::from(i64::MAX), i128::from(i64::MAX) + 1]
        );
    }

    #[test]
    fn modular_sums() {
        use ac_library::ModInt998244353 as Mint;
        let sums = prefix_sum(&[Mint::new(-1), Mint::new(3), Mint::new(5)]);
        assert_eq!(sums[3] - sums[0], Mint::new(7));
        assert_eq!(sums[3] - sums[1], Mint::new(8));
        assert_eq!(sums[2], Mint::new(2));
    }
}
