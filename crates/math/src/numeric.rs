/// GCD (Greatest Common Divisor) and LCM (Least Common Multiple) trait.
pub trait GCD {
    /// Calculates the Greatest Common Divisor of two numbers.
    fn gcd(self, other: Self) -> Self;

    /// Calculates the Least Common Multiple of two numbers.
    fn lcm(self, other: Self) -> Self;
}

macro_rules! impl_gcd {
    ($($t:ty),*) => {
        $(
            impl GCD for $t {
                fn gcd(self, other: Self) -> Self {
                    let mut a = self;
                    let mut b = other;
                    while b != 0 {
                        let t = b;
                        b = a % b;
                        a = t;
                    }
                    a
                }

                fn lcm(self, other: Self) -> Self {
                    if self == 0 && other == 0 {
                        return 0;
                    }
                    (self / self.gcd(other)) * other
                }
            }
        )*
    };
}

impl_gcd!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

/// Calculates the Greatest Common Divisor of two numbers.
pub fn gcd<T: GCD>(a: T, b: T) -> T {
    a.gcd(b)
}

/// Calculates the Least Common Multiple of two numbers.
pub fn lcm<T: GCD>(a: T, b: T) -> T {
    a.lcm(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcd_u64() {
        assert_eq!(gcd(12u64, 18), 6);
        assert_eq!(gcd(18u64, 12), 6);
        assert_eq!(gcd(17u64, 13), 1);
        assert_eq!(gcd(12u64, 0), 12);
        assert_eq!(gcd(0u64, 12), 12);
        assert_eq!(gcd(0u64, 0), 0);
    }

    #[test]
    fn test_lcm_u64() {
        assert_eq!(lcm(12u64, 18), 36);
        assert_eq!(lcm(3u64, 4), 12);
        assert_eq!(lcm(0u64, 5), 0);
        assert_eq!(lcm(5u64, 0), 0);
    }

    #[test]
    fn test_gcd_u128() {
        let a: u128 = 12345678901234567890;
        let b: u128 = 98765432109876543210;
        let g = gcd(a, b);
        assert!(a % g == 0);
        assert!(b % g == 0);
    }

    #[test]
    fn test_gcd_usize() {
        assert_eq!(gcd(100usize, 25), 25);
    }

    #[test]
    fn test_gcd_signed() {
        assert_eq!(gcd(12i32, 18), 6);
        assert_eq!(gcd(-12i32, 18), 6);
    }
}
