//! Common [`ac_library::MapMonoid`] implementations for [`ac_library::LazySegtree`].
//!
//! `composition(f, g)` applies `g` first, then `f`. Assignment actions use
//! `None` for the identity and `Some(value)` for assignment, including zero.
//! Integer arithmetic must fit its type (including intermediate sums, products,
//! and composed actions). Generic sum types must obey the usual semiring laws;
//! use ACL modints for modular arithmetic. Lengths must fit `i64`.
//!
//! Build trees from leaves: `LazySegtree::new(n)` contains monoid identities,
//! which represent empty segments, not n zero-valued elements.
//!
//! ```
//! use ac_library::{LazySegtree, ModInt998244353 as Mint};
//! use cp_library::data_structure::lazy_segtree_map_monoid::{
//!     RangeAddSum, RangeAffineSum, RangeAssignMin, SumLen,
//! };
//! let mut sums = LazySegtree::<RangeAddSum>::from(
//!     vec![1, 2, 3].into_iter().map(SumLen::new).collect::<Vec<_>>());
//! sums.apply_range(0..2, 10);
//! assert_eq!(sums.all_prod().sum, 26);
//! let mut mins = LazySegtree::<RangeAssignMin>::from(vec![Some(3), Some(7)]);
//! mins.apply_range(.., Some(0));
//! assert_eq!(mins.all_prod(), Some(0));
//! let mut affine = LazySegtree::<RangeAffineSum<Mint>>::from(
//!     vec![SumLen::new(Mint::new(2)); 3]);
//! affine.apply_range(.., (Mint::new(3), Mint::new(1))); // x -> 3x + 1
//! assert_eq!(affine.all_prod().sum.val(), 21);
//! ```

use ac_library::{MapMonoid, Monoid};
use std::marker::PhantomData;
use std::ops::{Add, Mul};

/// Segment sum and number of elements. An ordinary leaf has length one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SumLen<T = i64> {
    pub sum: T,
    pub len: usize,
}

impl<T> SumLen<T> {
    pub fn new(value: T) -> Self {
        Self { sum: value, len: 1 }
    }
}

/// Sum monoid whose identity is `(sum = 0, len = 0)`.
#[derive(Clone, Copy, Debug)]
pub struct SumMonoid<T = i64>(PhantomData<T>);

impl<T: Copy + From<i64> + Add<Output = T>> Monoid for SumMonoid<T> {
    type S = SumLen<T>;
    fn identity() -> Self::S {
        SumLen {
            sum: T::from(0),
            len: 0,
        }
    }
    fn binary_operation(a: &Self::S, b: &Self::S) -> Self::S {
        SumLen {
            sum: a.sum + b.sum,
            len: a.len + b.len,
        }
    }
}

/// Range addition / range sum. Action: the amount to add.
#[derive(Clone, Copy, Debug)]
pub struct RangeAddSum<T = i64>(PhantomData<T>);

impl<T> MapMonoid for RangeAddSum<T>
where
    T: Copy + From<i64> + Add<Output = T> + Mul<Output = T>,
{
    type M = SumMonoid<T>;
    type F = T;
    fn identity_map() -> T {
        T::from(0)
    }
    fn mapping(f: &T, x: &SumLen<T>) -> SumLen<T> {
        SumLen {
            sum: x.sum + *f * T::from(i64::try_from(x.len).expect("length exceeds i64")),
            len: x.len,
        }
    }
    fn composition(f: &T, g: &T) -> T {
        *f + *g
    }
}

/// Range assignment / range sum. `None` is the identity action.
#[derive(Clone, Copy, Debug)]
pub struct RangeAssignSum<T = i64>(PhantomData<T>);

impl<T> MapMonoid for RangeAssignSum<T>
where
    T: Copy + From<i64> + Add<Output = T> + Mul<Output = T>,
{
    type M = SumMonoid<T>;
    type F = Option<T>;
    fn identity_map() -> Self::F {
        None
    }
    fn mapping(f: &Self::F, x: &SumLen<T>) -> SumLen<T> {
        match f {
            Some(value) => SumLen {
                sum: *value * T::from(i64::try_from(x.len).expect("length exceeds i64")),
                len: x.len,
            },
            None => *x,
        }
    }
    fn composition(f: &Self::F, g: &Self::F) -> Self::F {
        f.or(*g)
    }
}

/// Range affine transformation / range sum. `(a, b)` maps x to a*x + b.
/// Composition is `(a_f*a_g, a_f*b_g + b_f)` (g first, then f).
#[derive(Clone, Copy, Debug)]
pub struct RangeAffineSum<T = i64>(PhantomData<T>);

impl<T> MapMonoid for RangeAffineSum<T>
where
    T: Copy + From<i64> + Add<Output = T> + Mul<Output = T>,
{
    type M = SumMonoid<T>;
    type F = (T, T);
    fn identity_map() -> Self::F {
        (T::from(1), T::from(0))
    }
    fn mapping(f: &Self::F, x: &SumLen<T>) -> SumLen<T> {
        SumLen {
            sum: f.0 * x.sum + f.1 * T::from(i64::try_from(x.len).expect("length exceeds i64")),
            len: x.len,
        }
    }
    fn composition(f: &Self::F, g: &Self::F) -> Self::F {
        (f.0 * g.0, f.0 * g.1 + f.1)
    }
}

macro_rules! extrema {
    ($monoid:ident, $add:ident, $assign:ident, $op:ident, $doc:literal) => {
        #[doc = $doc]
        /// `None` is the empty segment; use `Some(value)` for ordinary leaves.
        #[derive(Clone, Copy, Debug)]
        pub struct $monoid;
        impl Monoid for $monoid {
            type S = Option<i64>;
            fn identity() -> Self::S { None }
            fn binary_operation(a: &Self::S, b: &Self::S) -> Self::S {
                match (*a, *b) {
                    (Some(a), Some(b)) => Some(a.$op(b)),
                    (a, b) => a.or(b),
                }
            }
        }
        #[doc = concat!("Range addition with `", stringify!($monoid), "`. Action: `i64`.")]
        #[derive(Clone, Copy, Debug)]
        pub struct $add;
        impl MapMonoid for $add {
            type M = $monoid;
            type F = i64;
            fn identity_map() -> Self::F { 0 }
            fn mapping(f: &Self::F, x: &Option<i64>) -> Option<i64> { x.map(|x| x + *f) }
            fn composition(f: &Self::F, g: &Self::F) -> Self::F { *f + *g }
        }
        #[doc = concat!("Range assignment with `", stringify!($monoid), "`. `None` is the identity action.")]
        #[derive(Clone, Copy, Debug)]
        pub struct $assign;
        impl MapMonoid for $assign {
            type M = $monoid;
            type F = Option<i64>;
            fn identity_map() -> Self::F { None }
            fn mapping(f: &Self::F, x: &Option<i64>) -> Option<i64> { x.map(|x| f.unwrap_or(x)) }
            fn composition(f: &Self::F, g: &Self::F) -> Self::F { f.or(*g) }
        }
    };
}

extrema!(
    MinMonoid,
    RangeAddMin,
    RangeAssignMin,
    min,
    "Minimum monoid for i64."
);
extrema!(
    MaxMonoid,
    RangeAddMax,
    RangeAssignMax,
    max,
    "Maximum monoid for i64."
);

#[cfg(test)]
mod tests {
    use super::*;
    use ac_library::{LazySegtree, ModInt998244353 as Mint};
    use std::fmt::Debug;

    fn check<M: MapMonoid>(
        leaf: impl Fn(i64) -> <M::M as Monoid>::S,
        action: impl Fn(usize) -> (M::F, i64, i64),
    ) where
        <M::M as Monoid>::S: PartialEq + Debug,
    {
        let mut values = vec![3, -2, 0, 8, 1, -5, 4];
        let mut tree =
            LazySegtree::<M>::from(values.iter().copied().map(&leaf).collect::<Vec<_>>());
        let identity = M::identity_element();
        let empty = LazySegtree::<M>::new(0);
        assert_eq!(empty.all_prod(), identity);
        let x = M::binary_operation(&leaf(2), &leaf(-3));
        let y = leaf(5);
        assert_eq!(M::mapping(&M::identity_map(), &x), x);
        for step in 0..80 {
            let (f, a, b) = action(step);
            let (g, _, _) = action(step + 1);
            assert_eq!(M::mapping(&f, &identity), identity);
            assert_eq!(
                M::mapping(&M::composition(&f, &g), &x),
                M::mapping(&f, &M::mapping(&g, &x))
            );
            assert_eq!(
                M::mapping(&M::composition(&M::identity_map(), &f), &x),
                M::mapping(&f, &x)
            );
            assert_eq!(
                M::mapping(&M::composition(&f, &M::identity_map()), &x),
                M::mapping(&f, &x)
            );
            assert_eq!(
                M::mapping(&f, &M::binary_operation(&x, &y)),
                M::binary_operation(&M::mapping(&f, &x), &M::mapping(&f, &y))
            );
            let l = step * 3 % 8;
            let r = step * 5 % 8;
            let (l, r) = (l.min(r), l.max(r));
            tree.apply_range(l..r, f);
            for value in &mut values[l..r] {
                *value = a * *value + b;
            }
            if step % 7 == 0 {
                let p = step % 7;
                values[p] = -3;
                tree.set(p, leaf(-3));
            }
            for l in 0..=values.len() {
                for r in l..=values.len() {
                    let expected = values[l..r].iter().fold(M::identity_element(), |acc, &v| {
                        M::binary_operation(&acc, &leaf(v))
                    });
                    assert_eq!(tree.prod(l..r), expected);
                }
            }
            for (i, &v) in values.iter().enumerate() {
                assert_eq!(tree.get(i), leaf(v));
            }
        }
    }

    #[test]
    fn all_maps_match_vec_and_obey_action_laws() {
        let add = |step: usize| {
            let b = step as i64 % 5 - 2;
            (b, 1, b)
        };
        let assign = |step: usize| {
            if step % 3 == 0 {
                (None, 1, 0)
            } else {
                let b = step as i64 % 5 - 2;
                (Some(b), 0, b)
            }
        };
        check::<RangeAddSum>(SumLen::new, add);
        check::<RangeAssignSum>(SumLen::new, assign);
        check::<RangeAffineSum>(SumLen::new, |step| {
            let a = step as i64 % 3 - 1;
            let b = step as i64 % 5 - 2;
            ((a, b), a, b)
        });
        check::<RangeAddMin>(Some, add);
        check::<RangeAddMax>(Some, add);
        check::<RangeAssignMin>(Some, assign);
        check::<RangeAssignMax>(Some, assign);
    }

    #[test]
    fn affine_order_and_modular_sums() {
        assert_eq!(
            RangeAffineSum::<i64>::composition(&(2, 3), &(5, 7)),
            (10, 17)
        );
        let mut values: Vec<Mint> = (0..7).map(Mint::new).collect();
        let mut tree = LazySegtree::<RangeAffineSum<Mint>>::from(
            values.iter().copied().map(SumLen::new).collect::<Vec<_>>(),
        );
        for step in 0..60 {
            let a = Mint::new(123456789 + step);
            let b = Mint::new(-987654321 + step);
            let l = step as usize % 7;
            tree.apply_range(l.., (a, b));
            for x in &mut values[l..] {
                *x = a * *x + b;
            }
        }
        assert_eq!(tree.all_prod().sum, values.iter().copied().sum::<Mint>());
        for (i, &value) in values.iter().enumerate() {
            assert_eq!(tree.get(i).sum, value);
        }
        let mut add = LazySegtree::<RangeAddSum<Mint>>::from(vec![SumLen::new(Mint::new(1)); 3]);
        add.apply_range(.., Mint::new(-2));
        assert_eq!(add.all_prod().sum, Mint::new(-3));
        let mut assign =
            LazySegtree::<RangeAssignSum<Mint>>::from(vec![SumLen::new(Mint::new(1)); 3]);
        assign.apply_range(.., Some(Mint::new(0)));
        assert_eq!(
            assign.all_prod(),
            SumLen {
                sum: Mint::new(0),
                len: 3
            }
        );
    }

    #[test]
    fn extrema_support_full_integer_range_and_empty_segments() {
        let mut min = LazySegtree::<RangeAssignMin>::from(vec![Some(i64::MAX); 3]);
        min.apply_range(.., Some(i64::MIN));
        assert_eq!(min.all_prod(), Some(i64::MIN));
        assert_eq!(min.prod(1..1), None);
        let mut max = LazySegtree::<RangeAssignMax>::from(vec![Some(i64::MIN); 3]);
        max.apply_range(.., Some(i64::MAX));
        assert_eq!(max.all_prod(), Some(i64::MAX));
        assert_eq!(max.prod(1..1), None);
    }
}
