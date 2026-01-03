/// Returns the length of the longest increasing subsequence of the given sequence.
/// The subsequence is strictly increasing.
pub fn lis<T: Ord>(seq: &[T]) -> usize {
    let mut dp: Vec<&T> = Vec::new();
    for x in seq {
        let idx = dp.partition_point(|item| *item < x);
        if idx < dp.len() {
            dp[idx] = x;
        } else {
            dp.push(x);
        }
    }
    dp.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lis_simple() {
        let seq = vec![1, 3, 5, 2, 4, 6];
        assert_eq!(lis(&seq), 4); // 1, 2, 4, 6 or 1, 3, 5, 6 etc.
    }

    #[test]
    fn test_lis_empty() {
        let seq: Vec<i32> = vec![];
        assert_eq!(lis(&seq), 0);
    }

    #[test]
    fn test_lis_sorted() {
        let seq = vec![1, 2, 3, 4, 5];
        assert_eq!(lis(&seq), 5);
    }

    #[test]
    fn test_lis_reverse_sorted() {
        let seq = vec![5, 4, 3, 2, 1];
        assert_eq!(lis(&seq), 1);
    }

    #[test]
    fn test_lis_with_duplicates() {
        let seq = vec![1, 2, 2, 3];
        assert_eq!(lis(&seq), 3); // 1, 2, 3
    }
}
