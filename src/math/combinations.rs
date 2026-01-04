/// Combinatorics utility struct using precomputed factorials.
pub struct Combination {
    fact: Vec<u64>,
    inv_fact: Vec<u64>,
    modulo: u64,
}

impl Combination {
    /// Precomputes factorials and inverse factorials modulo `modulo` up to `max_n`.
    ///
    /// # Arguments
    /// * `max_n` - Maximum value of n to support.
    /// * `modulo` - The modulo (must be prime for valid inverses via Fermat's Little Theorem).
    pub fn new(max_n: usize, modulo: u64) -> Self {
        let mut fact = vec![1; max_n + 1];
        let mut inv_fact = vec![1; max_n + 1];

        for i in 1..=max_n {
            fact[i] = (fact[i - 1] * i as u64) % modulo;
        }

        inv_fact[max_n] = Self::mod_pow(fact[max_n], modulo - 2, modulo);
        for i in (1..=max_n).rev() {
            inv_fact[i - 1] = (inv_fact[i] * i as u64) % modulo;
        }

        Combination {
            fact,
            inv_fact,
            modulo,
        }
    }

    /// Calculates nCr (n choose r) modulo M.
    pub fn n_c_r(&self, n: usize, r: usize) -> u64 {
        if r > n {
            return 0;
        }
        let numer = self.fact[n];
        let denom = (self.inv_fact[r] * self.inv_fact[n - r]) % self.modulo;
        (numer * denom) % self.modulo
    }

    /// Calculates nPr (n permute r) modulo M.
    pub fn n_p_r(&self, n: usize, r: usize) -> u64 {
        if r > n {
            return 0;
        }
        let numer = self.fact[n];
        let denom = self.inv_fact[n - r];
        (numer * denom) % self.modulo
    }

    /// Calculates nHr (n multichoose r) modulo M.
    /// nHr = (n+r-1)Cr
    pub fn n_h_r(&self, n: usize, r: usize) -> u64 {
        if n == 0 && r == 0 {
            return 1;
        }
        if n == 0 {
            return 0; // if n=0, r>0, impossible to choose
        }
        self.n_c_r(n + r - 1, r)
    }

    /// Returns factorial of n modulo M.
    pub fn fact(&self, n: usize) -> u64 {
        self.fact[n]
    }

    /// Returns inverse factorial of n modulo M.
    pub fn inv_fact(&self, n: usize) -> u64 {
        self.inv_fact[n]
    }

    fn mod_pow(mut base: u64, mut exp: u64, modulo: u64) -> u64 {
        let mut res = 1;
        base %= modulo;
        while exp > 0 {
            if exp % 2 == 1 {
                res = (res * base) % modulo;
            }
            base = (base * base) % modulo;
            exp /= 2;
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MOD: u64 = 1_000_000_007;

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
