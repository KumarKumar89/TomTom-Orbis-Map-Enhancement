//! Curve similarity metrics used by the conflation engine's geometry score.

use geo_core::Coordinate;

/// Discrete Hausdorff distance between two polylines (max of min distances).
pub fn hausdorff_distance(a: &[Coordinate], b: &[Coordinate]) -> f64 {
    assert!(!a.is_empty() && !b.is_empty(), "hausdorff needs non-empty lines");
    let forward = a.iter().map(|p| min_dist_to_line(p, b)).fold(0.0f64, f64::max);
    let backward = b.iter().map(|p| min_dist_to_line(p, a)).fold(0.0f64, f64::max);
    forward.max(backward)
}

/// Discrete Fréchet distance (dynamic programming over vertex pairs).
/// Captures curve *ordering*, unlike Hausdorff — important for direction-aware
/// road matching.
pub fn frechet_distance(a: &[Coordinate], b: &[Coordinate]) -> f64 {
    assert!(!a.is_empty() && !b.is_empty(), "frechet needs non-empty lines");
    let n = a.len();
    let m = b.len();
    let inf = f64::INFINITY;
    let mut dp = vec![vec![inf; m]; n];
    for i in 0..n {
        for j in 0..m {
            let d = a[i].euclidean_distance(&b[j]);
            dp[i][j] = if i == 0 && j == 0 {
                d
            } else if i == 0 {
                d.max(dp[0][j - 1])
            } else if j == 0 {
                d.max(dp[i - 1][0])
            } else {
                d.max(dp[i - 1][j].min(dp[i][j - 1]).min(dp[i - 1][j - 1]))
            };
        }
    }
    dp[n - 1][m - 1]
}

fn min_dist_to_line(p: &Coordinate, line: &[Coordinate]) -> f64 {
    line.iter()
        .map(|q| p.euclidean_distance(q))
        .fold(f64::INFINITY, f64::min)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(x: f64, y: f64) -> Coordinate {
        Coordinate::new(x, y)
    }

    #[test]
    fn identical_lines_have_zero_distance() {
        let l = vec![c(0., 0.), c(1., 1.), c(2., 0.)];
        assert!(hausdorff_distance(&l, &l).abs() < 1e-12);
        assert!(frechet_distance(&l, &l).abs() < 1e-12);
    }

    #[test]
    fn shifted_parallel_line() {
        let a = vec![c(0., 0.), c(10., 0.)];
        let b = vec![c(0., 1.), c(10., 1.)];
        assert!((hausdorff_distance(&a, &b) - 1.0).abs() < 1e-9);
        assert!((frechet_distance(&a, &b) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn frechet_detects_reversed_ordering() {
        // Same vertex set, reversed traversal: Hausdorff is 0 but Frechet is not.
        let a = vec![c(0., 0.), c(5., 5.), c(10., 0.)];
        let b: Vec<_> = a.iter().rev().copied().collect();
        assert!(hausdorff_distance(&a, &b) < 1e-9);
        assert!(frechet_distance(&a, &b) > 1.0);
    }
}
