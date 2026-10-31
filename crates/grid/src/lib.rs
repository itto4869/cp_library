/// Returns 4-directional neighbors (up, down, left, right)
/// within the grid boundaries (H x W).
///
/// # Arguments
/// * `r` - Row index
/// * `c` - Column index
/// * `h` - Height of the grid
/// * `w` - Width of the grid
pub fn neighbors4(r: usize, c: usize, h: usize, w: usize) -> Vec<(usize, usize)> {
    let mut neighbors = Vec::with_capacity(4);
    let dr = [0, 0, 1, !0]; // right, left, down, up (using !0 for -1)
    let dc = [1, !0, 0, 0];

    for i in 0..4 {
        let nr = r.wrapping_add(dr[i]);
        let nc = c.wrapping_add(dc[i]);

        if nr < h && nc < w {
            neighbors.push((nr, nc));
        }
    }
    neighbors
}

/// Returns 8-directional neighbors (horizontal, vertical, diagonal)
/// within the grid boundaries (H x W).
///
/// # Arguments
/// * `r` - Row index
/// * `c` - Column index
/// * `h` - Height of the grid
/// * `w` - Width of the grid
pub fn neighbors8(r: usize, c: usize, h: usize, w: usize) -> Vec<(usize, usize)> {
    let mut neighbors = Vec::with_capacity(8);
    // (dr, dc) for 8 directions
    for dr in [!0, 0, 1] {
        for dc in [!0, 0, 1] {
            if dr == 0 && dc == 0 {
                continue;
            }
            let nr = r.wrapping_add(dr);
            let nc = c.wrapping_add(dc);

            if nr < h && nc < w {
                neighbors.push((nr, nc));
            }
        }
    }
    neighbors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neighbors4() {
        let h = 3;
        let w = 3;

        // Center
        let n = neighbors4(1, 1, h, w);
        assert_eq!(n.len(), 4);
        assert!(n.contains(&(0, 1)));
        assert!(n.contains(&(2, 1)));
        assert!(n.contains(&(1, 0)));
        assert!(n.contains(&(1, 2)));

        // Top-left
        let n = neighbors4(0, 0, h, w);
        assert_eq!(n.len(), 2);
        assert!(n.contains(&(0, 1)));
        assert!(n.contains(&(1, 0)));

        // Bottom-right
        let n = neighbors4(2, 2, h, w);
        assert_eq!(n.len(), 2);
        assert!(n.contains(&(2, 1)));
        assert!(n.contains(&(1, 2)));
    }

    #[test]
    fn test_neighbors8() {
        let h = 3;
        let w = 3;

        // Center
        let n = neighbors8(1, 1, h, w);
        assert_eq!(n.len(), 8);
        assert!(n.contains(&(0, 0)));
        assert!(n.contains(&(0, 1)));
        assert!(n.contains(&(0, 2)));
        assert!(n.contains(&(1, 0)));
        assert!(n.contains(&(1, 2)));
        assert!(n.contains(&(2, 0)));
        assert!(n.contains(&(2, 1)));
        assert!(n.contains(&(2, 2)));

        // Top-left
        let n = neighbors8(0, 0, h, w);
        assert_eq!(n.len(), 3);
        assert!(n.contains(&(0, 1)));
        assert!(n.contains(&(1, 0)));
        assert!(n.contains(&(1, 1)));

        // Bottom-right
        let n = neighbors8(2, 2, h, w);
        assert_eq!(n.len(), 3);
        assert!(n.contains(&(1, 1)));
        assert!(n.contains(&(1, 2)));
        assert!(n.contains(&(2, 1)));
    }
}
