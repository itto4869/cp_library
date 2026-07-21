//! Prime enumeration using the sieve of Eratosthenes.

/// Returns all primes less than or equal to `upper_bound` in ascending order.
///
/// # Complexity
///
/// - Time: `O(upper_bound log log upper_bound)`
/// - Space: `O(upper_bound)`
///
/// # Examples
///
/// ```
/// use cp_library::math::prime_enumeration::enumerate_primes;
///
/// assert_eq!(enumerate_primes(10), vec![2, 3, 5, 7]);
/// assert!(enumerate_primes(1).is_empty());
/// ```
pub fn enumerate_primes(upper_bound: usize) -> Vec<usize> {
    if upper_bound < 2 {
        return Vec::new();
    }

    let mut is_prime = vec![true; upper_bound + 1];
    is_prime[0] = false;
    is_prime[1] = false;

    let mut prime = 2;
    while prime <= upper_bound / prime {
        if is_prime[prime] {
            for multiple in (prime * prime..=upper_bound).step_by(prime) {
                is_prime[multiple] = false;
            }
        }
        prime += 1;
    }

    is_prime
        .into_iter()
        .enumerate()
        .filter_map(|(number, prime)| prime.then_some(number))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enumerates_primes_up_to_upper_bound() {
        assert_eq!(
            enumerate_primes(30),
            vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29]
        );
    }

    #[test]
    fn handles_small_upper_bounds() {
        assert_eq!(enumerate_primes(0), vec![]);
        assert_eq!(enumerate_primes(1), vec![]);
        assert_eq!(enumerate_primes(2), vec![2]);
    }

    #[test]
    fn excludes_composite_upper_bound() {
        assert_eq!(enumerate_primes(49).last(), Some(&47));
    }
}
