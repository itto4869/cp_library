/// Factorials and inverse factorials modulo a prime, with generic integer results.
///
/// `T` defaults to `u64`. Primitive signed and unsigned integers are supported
/// when the positive modulus fits `T`. Arithmetic uses `u128` intermediates
/// with overflow-safe modular multiplication, including for `u128` moduli.
/// The modulus must be prime; primality is a caller precondition.
///
/// Construction uses O(n + log(modulo)) modular multiplications and O(n) space.
/// A query uses O(1) modular multiplications. Each multiplication is O(1) when
/// its product fits `u128`, otherwise O(log(modulo)) using repeated doubling.
///
/// ```
/// use cp_math::combinations::Combination;
/// let comb = Combination::<usize>::new(10, 1_000_000_007);
/// assert_eq!(comb.n_c_r(5, 2), 10);
/// ```
pub struct Combination<T = u64> {
    fact: Vec<T>,
    inv_fact: Vec<T>,
    modulo: u128,
}

impl<T: Copy + TryInto<u128> + TryFrom<u128>> Combination<T> {
    /// Precomputes up to `max_n`. Requires a prime `modulo` and `max_n < modulo`.
    ///
    /// # Panics
    /// Panics if the modulus is not convertible to `u128`, is less than 2,
    /// `max_n >= modulo`, or a residue cannot be represented by `T`.
    pub fn new(max_n: usize, modulo: T) -> Self {
        let modulo = Self::to_u128(modulo);
        assert!(modulo >= 2, "modulo must be at least 2");
        assert!((max_n as u128) < modulo, "max_n must be less than modulo");
        let one = Self::from_u128(1);
        let mut fact = vec![one; max_n + 1];
        let mut inv_fact = vec![one; max_n + 1];
        for i in 1..=max_n {
            fact[i] = Self::from_u128(mul_mod(Self::to_u128(fact[i - 1]), i as u128, modulo));
        }
        inv_fact[max_n] = Self::from_u128(mod_pow(Self::to_u128(fact[max_n]), modulo - 2, modulo));
        for i in (1..=max_n).rev() {
            inv_fact[i - 1] =
                Self::from_u128(mul_mod(Self::to_u128(inv_fact[i]), i as u128, modulo));
        }
        Self {
            fact,
            inv_fact,
            modulo,
        }
    }

    /// Returns nCr. Returns zero for r > n; otherwise panics if n was not precomputed.
    pub fn n_c_r(&self, n: usize, r: usize) -> T {
        if r > n {
            return Self::from_u128(0);
        }
        self.multiply(
            self.fact[n],
            self.multiply(self.inv_fact[r], self.inv_fact[n - r]),
        )
    }

    /// Returns nPr. Returns zero for r > n; otherwise panics if n was not precomputed.
    pub fn n_p_r(&self, n: usize, r: usize) -> T {
        if r > n {
            return Self::from_u128(0);
        }
        self.multiply(self.fact[n], self.inv_fact[n - r])
    }

    /// Returns nHr = (n+r-1)Cr. Defines 0H0 = 1 and 0Hr = 0 for r > 0.
    /// Panics if the required factorial was not precomputed or the index overflows.
    pub fn n_h_r(&self, n: usize, r: usize) -> T {
        if n == 0 {
            return Self::from_u128(u128::from(r == 0));
        }
        let index = (n - 1)
            .checked_add(r)
            .expect("combination index exceeds usize");
        self.n_c_r(index, r)
    }

    /// Returns n! modulo the modulus. Panics if n was not precomputed.
    pub fn fact(&self, n: usize) -> T {
        self.fact[n]
    }

    /// Returns the inverse of n! modulo the modulus. Panics if n was not precomputed.
    pub fn inv_fact(&self, n: usize) -> T {
        self.inv_fact[n]
    }

    fn multiply(&self, a: T, b: T) -> T {
        Self::from_u128(mul_mod(Self::to_u128(a), Self::to_u128(b), self.modulo))
    }
    fn to_u128(value: T) -> u128 {
        value
            .try_into()
            .ok()
            .expect("value must be a nonnegative integer fitting u128")
    }
    fn from_u128(value: u128) -> T {
        T::try_from(value)
            .ok()
            .expect("residue does not fit the result type")
    }
}

// Assumes a, b < modulo. Avoids overflow even when modulo is near u128::MAX.
fn add_mod(a: u128, b: u128, modulo: u128) -> u128 {
    if a >= modulo - b {
        a - (modulo - b)
    } else {
        a + b
    }
}

fn mul_mod(mut a: u128, mut b: u128, modulo: u128) -> u128 {
    if let Some(product) = a.checked_mul(b) {
        return product % modulo;
    }
    let mut result = 0;
    while b > 0 {
        if b & 1 != 0 {
            result = add_mod(result, a, modulo);
        }
        a = add_mod(a, a, modulo);
        b >>= 1;
    }
    result
}

fn mod_pow(mut base: u128, mut exp: u128, modulo: u128) -> u128 {
    let mut result = 1;
    while exp > 0 {
        if exp & 1 != 0 {
            result = mul_mod(result, base, modulo);
        }
        base = mul_mod(base, base, modulo);
        exp >>= 1;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    const MOD: u64 = 1_000_000_007;

    fn compare_pascal<T>(modulo: T)
    where
        T: Copy + TryInto<u128> + TryFrom<u128>,
    {
        let m = modulo.try_into().ok().unwrap();
        let comb = Combination::new(20, modulo);
        let mut row = vec![1u128];
        for n in 0..=20 {
            for (r, &expected) in row.iter().enumerate() {
                assert_eq!(comb.n_c_r(n, r).try_into().ok().unwrap(), expected);
            }
            let mut next = vec![1; row.len() + 1];
            for r in 1..row.len() {
                next[r] = (row[r - 1] + row[r]) % m;
            }
            row = next;
        }
    }

    #[test]
    fn generic_integer_types_and_wide_modulus() {
        compare_pascal(251u8);
        compare_pascal(1_000_000_007u32);
        compare_pascal(1_000_000_007usize);
        compare_pascal(1_000_000_007i64);
        compare_pascal((1u128 << 127) - 1); // Mersenne prime
        let c = Combination::<u8>::new(6, 7);
        assert_eq!(c.n_p_r(5, 2), 6);
        assert_eq!(c.n_h_r(3, 2), 6);
        assert_eq!(c.fact(6), 6);
        assert_eq!(c.n_c_r(2, 3), 0);
        assert_eq!(Combination::<u32>::new(0, 2).fact(0), 1);
    }

    #[test]
    fn full_width_modular_multiplication() {
        let m = u128::MAX;
        assert_eq!(mul_mod(m - 1, m - 1, m), 1);
        assert_eq!(mul_mod(m - 2, m - 3, m), 6);
        assert_eq!(mul_mod(0, m - 1, m), 0);
    }

    #[test]
    fn rejects_invalid_moduli_and_noninvertible_factorial() {
        assert!(std::panic::catch_unwind(|| Combination::new(0, -7i32)).is_err());
        assert!(std::panic::catch_unwind(|| Combination::new(0, 1u8)).is_err());
        assert!(std::panic::catch_unwind(|| Combination::new(7, 7u8)).is_err());
    }

    #[test]
    fn test_combination() {
        let comb = Combination::new(100, MOD);

        // nCr
        assert_eq!(comb.n_c_r(5, 2), 10);
        assert_eq!(comb.n_c_r(5, 0), 1);
        assert_eq!(comb.n_c_r(5, 5), 1);
        assert_eq!(comb.n_c_r(5, 6), 0);

        // nPr
        assert_eq!(comb.n_p_r(5, 2), 20);
        assert_eq!(comb.n_p_r(5, 0), 1);
        assert_eq!(comb.n_p_r(5, 5), 120);

        // nHr
        // 3H2 = (3+2-1)C2 = 4C2 = 6
        assert_eq!(comb.n_h_r(3, 2), 6);
        // 0H0 = 1
        assert_eq!(comb.n_h_r(0, 0), 1);
    }
}
