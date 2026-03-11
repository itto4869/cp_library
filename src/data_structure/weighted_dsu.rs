use std::ops::{Add, Neg, Sub};

#[derive(Clone, Debug)]
pub struct WeightedDsu<T> {
    parent_or_size: Vec<i32>,
    diff_weight: Vec<T>,
}

impl<T> WeightedDsu<T>
where
    T: Copy + Default + Add<Output = T> + Sub<Output = T> + Neg<Output = T>,
{
    pub fn new(size: usize) -> Self {
        Self {
            parent_or_size: vec![-1; size],
            diff_weight: vec![T::default(); size],
        }
    }

    pub fn find(&mut self, x: usize) -> usize {
        if self.parent_or_size[x] < 0 {
            return x;
        }
        let p = self.parent_or_size[x] as usize;
        let root = self.find(p);
        self.diff_weight[x] = self.diff_weight[x] + self.diff_weight[p];
        self.parent_or_size[x] = root as i32;
        root
    }

    pub fn weight(&mut self, x: usize) -> T {
        self.find(x); // Compress path
        self.diff_weight[x]
    }

    /// Returns the difference `weight(y) - weight(x)` if they are in the same set.
    /// Returns `None` otherwise.
    pub fn diff(&mut self, x: usize, y: usize) -> Option<T> {
        if self.same(x, y) {
            Some(self.weight(y) - self.weight(x))
        } else {
            None
        }
    }

    pub fn same(&mut self, x: usize, y: usize) -> bool {
        self.find(x) == self.find(y)
    }

    /// Merges the sets containing `x` and `y` such that `weight(y) = weight(x) + w`.
    /// Returns `true` if they were successfully merged (they were in different sets).
    /// Returns `false` if they were already in the same set.
    pub fn merge(&mut self, mut x: usize, mut y: usize, mut w: T) -> bool {
        let root_x = self.find(x);
        let root_y = self.find(y);
        if root_x == root_y {
            return false;
        }

        w = w + self.weight(x) - self.weight(y);
        x = root_x;
        y = root_y;

        if -self.parent_or_size[x] < -self.parent_or_size[y] {
            std::mem::swap(&mut x, &mut y);
            w = -w;
        }

        self.parent_or_size[x] += self.parent_or_size[y];
        self.parent_or_size[y] = x as i32;
        self.diff_weight[y] = w;

        true
    }

    pub fn size(&mut self, x: usize) -> usize {
        let root = self.find(x);
        -self.parent_or_size[root] as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weighted_dsu() {
        let mut dsu = WeightedDsu::<i32>::new(5);
        dsu.merge(0, 1, 2); // weight(1) = weight(0) + 2
        dsu.merge(1, 2, 3); // weight(2) = weight(1) + 3 -> weight(0) + 5
        dsu.merge(3, 4, 10); // weight(4) = weight(3) + 10

        assert_eq!(dsu.diff(0, 2), Some(5)); // w(2) - w(0) = 5
        assert_eq!(dsu.diff(0, 1), Some(2));
        assert_eq!(dsu.diff(1, 2), Some(3));
        assert_eq!(dsu.diff(1, 0), Some(-2));
        
        assert_eq!(dsu.same(0, 2), true);
        assert_eq!(dsu.same(0, 3), false);
        assert_eq!(dsu.diff(0, 3), None);

        dsu.merge(2, 4, 5); // weight(4) = weight(2) + 5 -> weight(0) + 10
        // weight(4) was weight(3) + 10, so weight(3) 
        // = weight(4) - 10 = (weight(0) + 10) - 10 = weight(0)

        assert_eq!(dsu.diff(0, 4), Some(10));
        assert_eq!(dsu.diff(0, 3), Some(0));
        assert_eq!(dsu.size(0), 5);
    }
}
