//! Fast prime factorization for 64-bit unsigned integers.

/// Returns the prime factorization of `n` as `(prime, exponent)` pairs.
///
/// The prime factors are sorted in ascending order. This implementation uses
/// deterministic Miller-Rabin primality testing and Pollard's rho algorithm,
/// so it is also suitable for numbers whose prime factors are much larger than
/// a trial-division limit.
///
/// # Panics
///
/// Panics if `n` is zero, because zero has no prime factorization.
///
/// # Examples
///
/// ```
/// use cp_library::math::prime_factorization::prime_factorize;
///
/// assert_eq!(prime_factorize(360), vec![(2, 3), (3, 2), (5, 1)]);
/// assert!(prime_factorize(1).is_empty());
/// ```
pub fn prime_factorize(n: u64) -> Vec<(u64, u32)> {
    assert!(n > 0, "zero has no prime factorization");

    let mut factors = Vec::new();
    collect_prime_factors(n, &mut factors);
    factors.sort_unstable();

    let mut result = Vec::new();
    for factor in factors {
        if let Some((last_factor, exponent)) = result.last_mut() {
            if *last_factor == factor {
                *exponent += 1;
                continue;
            }
        }
        result.push((factor, 1));
    }
    result
}

fn collect_prime_factors(n: u64, factors: &mut Vec<u64>) {
    if n == 1 {
        return;
    }
    if is_prime(n) {
        factors.push(n);
        return;
    }

    let divisor = pollard_rho(n);
    collect_prime_factors(divisor, factors);
    collect_prime_factors(n / divisor, factors);
}

fn is_prime(n: u64) -> bool {
    const SMALL_PRIMES: [u64; 12] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];

    if n < 2 {
        return false;
    }
    for prime in SMALL_PRIMES {
        if n % prime == 0 {
            return n == prime;
        }
    }

    let exponent_of_two = (n - 1).trailing_zeros();
    let odd_part = (n - 1) >> exponent_of_two;

    // These bases make Miller-Rabin deterministic for every 64-bit integer.
    const BASES: [u64; 7] = [2, 325, 9_375, 28_178, 450_775, 9_780_504, 1_795_265_022];
    'next_base: for base in BASES {
        let base = base % n;
        if base == 0 {
            continue;
        }

        let mut value = mod_pow(base, odd_part, n);
        if value == 1 || value == n - 1 {
            continue;
        }
        for _ in 1..exponent_of_two {
            value = mod_mul(value, value, n);
            if value == n - 1 {
                continue 'next_base;
            }
        }
        return false;
    }
    true
}

fn pollard_rho(n: u64) -> u64 {
    for prime in [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
        if n % prime == 0 {
            return prime;
        }
    }

    let mut random = XorShift64::new(n ^ 0x9e37_79b9_7f4a_7c15);
    loop {
        let mut y = random.next() % (n - 2) + 2;
        let constant = random.next() % (n - 1) + 1;
        let batch_size = 128u64;
        let mut cycle_length = 1u64;
        let mut gcd = 1u64;
        let mut x = 0u64;
        let mut saved_y = 0u64;

        // Brent's cycle detection batches gcd calculations, which is notably
        // faster than taking a gcd after every polynomial evaluation.
        while gcd == 1 {
            x = y;
            for _ in 0..cycle_length {
                y = polynomial(y, constant, n);
            }

            let mut processed = 0u64;
            while processed < cycle_length && gcd == 1 {
                saved_y = y;
                let count = batch_size.min(cycle_length - processed);
                let mut product = 1u64;
                for _ in 0..count {
                    y = polynomial(y, constant, n);
                    product = mod_mul(product, x.abs_diff(y), n);
                }
                gcd = gcd_u64(product, n);
                processed += count;
            }
            cycle_length = cycle_length.saturating_mul(2);
        }

        if gcd == n {
            loop {
                saved_y = polynomial(saved_y, constant, n);
                gcd = gcd_u64(x.abs_diff(saved_y), n);
                if gcd > 1 {
                    break;
                }
            }
        }

        if gcd != n {
            return gcd;
        }
    }
}

fn polynomial(value: u64, constant: u64, modulo: u64) -> u64 {
    ((value as u128 * value as u128 + constant as u128) % modulo as u128) as u64
}

fn mod_mul(lhs: u64, rhs: u64, modulo: u64) -> u64 {
    (lhs as u128 * rhs as u128 % modulo as u128) as u64
}

fn mod_pow(mut base: u64, mut exponent: u64, modulo: u64) -> u64 {
    let mut result = 1u64;
    while exponent > 0 {
        if exponent & 1 == 1 {
            result = mod_mul(result, base, modulo);
        }
        base = mod_mul(base, base, modulo);
        exponent >>= 1;
    }
    result
}

fn gcd_u64(mut lhs: u64, mut rhs: u64) -> u64 {
    while rhs != 0 {
        (lhs, rhs) = (rhs, lhs % rhs);
    }
    lhs
}

struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                0x4d59_5df4_d0f3_3173
            } else {
                seed
            },
        }
    }

    fn next(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factorizes_basic_values() {
        assert_eq!(prime_factorize(1), vec![]);
        assert_eq!(prime_factorize(2), vec![(2, 1)]);
        assert_eq!(prime_factorize(360), vec![(2, 3), (3, 2), (5, 1)]);
        assert_eq!(prime_factorize(3u64.pow(40)), vec![(3, 40)]);
    }

    #[test]
    fn factorizes_large_prime() {
        assert_eq!(
            prime_factorize(18_446_744_073_709_551_557),
            vec![(18_446_744_073_709_551_557, 1)]
        );
    }

    #[test]
    fn factorizes_large_semiprime() {
        let smaller_prime = 4_294_967_279u64;
        let larger_prime = 4_294_967_291u64;
        let n = smaller_prime * larger_prime;
        assert_eq!(
            prime_factorize(n),
            vec![(smaller_prime, 1), (larger_prime, 1)]
        );
    }

    #[test]
    fn factorizes_largest_u64() {
        assert_eq!(
            prime_factorize(u64::MAX),
            vec![
                (3, 1),
                (5, 1),
                (17, 1),
                (257, 1),
                (641, 1),
                (65_537, 1),
                (6_700_417, 1),
            ]
        );
    }

    #[test]
    fn matches_trial_division_for_small_values() {
        for n in 1..10_000 {
            assert_eq!(prime_factorize(n), trial_division(n), "n = {n}");
        }
    }

    #[test]
    #[should_panic(expected = "zero has no prime factorization")]
    fn rejects_zero() {
        prime_factorize(0);
    }

    fn trial_division(mut n: u64) -> Vec<(u64, u32)> {
        let mut result = Vec::new();
        let mut divisor = 2;
        while divisor * divisor <= n {
            let mut exponent = 0;
            while n % divisor == 0 {
                n /= divisor;
                exponent += 1;
            }
            if exponent > 0 {
                result.push((divisor, exponent));
            }
            divisor += 1;
        }
        if n > 1 {
            result.push((n, 1));
        }
        result
    }
}
