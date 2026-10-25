use std::cmp::Ordering;

/// Returns points sorted counterclockwise from the positive x-axis.
///
/// Points are vectors from the origin, ordered by angles in `[0, 2*pi)`.
/// Origins `(0, 0)` come first. Points on the same ray retain input order;
/// their distances from the origin are not used as a tie-breaker.
/// The input is not modified. Empty inputs are supported.
///
/// Coordinates are converted to `i128` for exact integer comparisons, without
/// floating-point arithmetic. All `i64` coordinates are supported, as are
/// other integer types when conversion and cross-product terms fit `i128`.
/// Time: O(n log n); additional space including the result: O(n).
///
/// # Panics
/// Panics if a coordinate cannot be converted to `i128`, or a multiplication
/// in an angle comparison overflows `i128`.
///
/// # Examples
/// ```
/// use cp_library::algorithm::polar_angle_sort;
///
/// let points = vec![(0, -1), (-1, 0), (0, 1), (1, 0)];
/// assert_eq!(polar_angle_sort(&points), vec![(1, 0), (0, 1), (-1, 0), (0, -1)]);
/// ```
pub fn polar_angle_sort<T: Copy + TryInto<i128>>(points: &[(T, T)]) -> Vec<(T, T)> {
    let mut points: Vec<_> = points
        .iter()
        .map(|&(x, y)| {
            let convert = |value: T| value.try_into().ok().expect("coordinate does not fit i128");
            ((x, y), (convert(x), convert(y)))
        })
        .collect();
    points.sort_by(|a, b| compare_angle(a.1, b.1));
    points.into_iter().map(|(point, _)| point).collect()
}

fn compare_angle(a: (i128, i128), b: (i128, i128)) -> Ordering {
    let class = |(x, y)| {
        if x == 0 && y == 0 {
            0
        } else if y > 0 || (y == 0 && x > 0) {
            1
        } else {
            2
        }
    };
    class(a).cmp(&class(b)).then_with(|| {
        let left = a.0.checked_mul(b.1).expect("angle product exceeds i128");
        let right = a.1.checked_mul(b.0).expect("angle product exceeds i128");
        // Comparing terms instead of subtracting avoids determinant overflow.
        right.cmp(&left)
    })
}

#[cfg(test)]
mod tests {
    use super::polar_angle_sort;

    #[test]
    fn axes_quadrants_origins_and_stable_ties() {
        let points = vec![
            (-1, -1),
            (0, 0),
            (0, -1),
            (2, 2),
            (-1, 1),
            (1, 0),
            (0, 1),
            (-1, 0),
            (1, -1),
            (1, 1),
            (0, 0),
            (1, 1),
            (2, 0),
        ];
        assert_eq!(
            polar_angle_sort(&points),
            vec![
                (0, 0),
                (0, 0),
                (1, 0),
                (2, 0),
                (2, 2),
                (1, 1),
                (1, 1),
                (0, 1),
                (-1, 1),
                (-1, 0),
                (-1, -1),
                (0, -1),
                (1, -1),
            ]
        );
        assert_eq!(points[0], (-1, -1));
        assert!(polar_angle_sort::<i64>(&[]).is_empty());
        assert_eq!(polar_angle_sort(&[(0, 0)]), vec![(0, 0)]);
    }

    #[test]
    fn integer_types_and_extreme_coordinates() {
        let lo = i64::MIN;
        let hi = i64::MAX;
        let ordered = vec![
            (hi, 0),
            (hi, hi),
            (0, hi),
            (lo, hi),
            (lo, 0),
            (lo, lo),
            (0, lo),
            (hi, lo),
        ];
        let reversed: Vec<_> = ordered.iter().copied().rev().collect();
        assert_eq!(polar_angle_sort(&reversed), ordered);
        assert_eq!(
            polar_angle_sort(&[(0usize, 1), (1, 0)]),
            vec![(1, 0), (0, 1)]
        );
        assert_eq!(
            polar_angle_sort(&[(0i128, 1), (1, 0)]),
            vec![(1, 0), (0, 1)]
        );
        // These directions are too close for a floating-point comparison.
        assert_eq!(
            polar_angle_sort(&[(hi - 1, hi), (hi, hi - 1)]),
            vec![(hi, hi - 1), (hi - 1, hi)]
        );
    }

    #[test]
    fn matches_atan2_for_small_coordinates() {
        let points: Vec<_> = (-10..=10)
            .flat_map(|x| (-10..=10).map(move |y| (x, y)))
            .collect();
        let mut expected = points.clone();
        let angle = |(x, y): (i32, i32)| {
            if (x, y) == (0, 0) {
                -1.0
            } else {
                (y as f64).atan2(x as f64).rem_euclid(std::f64::consts::TAU)
            }
        };
        expected.sort_by(|&a, &b| angle(a).partial_cmp(&angle(b)).unwrap());
        assert_eq!(polar_angle_sort(&points), expected);
    }

    #[test]
    #[should_panic(expected = "coordinate does not fit i128")]
    fn rejects_unrepresentable_coordinate() {
        polar_angle_sort(&[(u128::MAX, 0)]);
    }

    #[test]
    #[should_panic(expected = "angle product exceeds i128")]
    fn detects_product_overflow() {
        polar_angle_sort(&[(i128::MAX, 1), (1, i128::MAX)]);
    }
}
