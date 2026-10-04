//! Polyline operations: simplification, densification, snapping, offsetting.

use geo_core::Coordinate;

/// Ramer–Douglas–Peucker simplification with perpendicular tolerance `eps`.
pub fn simplify(points: &[Coordinate], eps: f64) -> Vec<Coordinate> {
    if points.len() < 3 {
        return points.to_vec();
    }
    let mut keep = vec![false; points.len()];
    keep[0] = true;
    keep[points.len() - 1] = true;
    rdp(points, eps, 0, points.len() - 1, &mut keep);
    points
        .iter()
        .zip(keep.iter())
        .filter_map(|(p, k)| if *k { Some(*p) } else { None })
        .collect()
}

fn rdp(points: &[Coordinate], eps: f64, first: usize, last: usize, keep: &mut [bool]) {
    if last <= first + 1 {
        return;
    }
    let mut worst = -1.0f64;
    let mut idx = first;
    for i in (first + 1)..last {
        let d = perp_distance(&points[i], &points[first], &points[last]);
        if d > worst {
            worst = d;
            idx = i;
        }
    }
    if worst > eps {
        keep[idx] = true;
        rdp(points, eps, first, idx, keep);
        rdp(points, eps, idx, last, keep);
    }
}

fn perp_distance(p: &Coordinate, a: &Coordinate, b: &Coordinate) -> f64 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let len2 = dx * dx + dy * dy;
    if len2 == 0.0 {
        return p.euclidean_distance(a);
    }
    ((dy * p.x - dx * p.y + b.x * a.y - b.y * a.x).abs()) / len2.sqrt()
}

/// Insert intermediate points so no segment exceeds `max_spacing` length.
pub fn densify(points: &[Coordinate], max_spacing: f64) -> Vec<Coordinate> {
    assert!(max_spacing > 0.0, "max_spacing must be positive");
    let mut out = Vec::with_capacity(points.len());
    for w in points.windows(2) {
        out.push(w[0]);
        let d = w[0].euclidean_distance(&w[1]);
        let n = (d / max_spacing).ceil() as usize;
        for i in 1..n {
            let t = i as f64 / n as f64;
            out.push(Coordinate::new(
                w[0].x + t * (w[1].x - w[0].x),
                w[0].y + t * (w[1].y - w[0].y),
            ));
        }
    }
    if let Some(&last) = points.last() {
        out.push(last);
    }
    out
}

/// Closest point on segment `ab` to `p`, returned as (coordinate, distance).
pub fn snap_to_segment(p: &Coordinate, a: &Coordinate, b: &Coordinate) -> (Coordinate, f64) {
    let abx = b.x - a.x;
    let aby = b.y - a.y;
    let len2 = abx * abx + aby * aby;
    let t = if len2 == 0.0 {
        0.0
    } else {
        (((p.x - a.x) * abx + (p.y - a.y) * aby) / len2).clamp(0.0, 1.0)
    };
    let q = Coordinate::new(a.x + t * abx, a.y + t * aby);
    (q, p.euclidean_distance(&q))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(x: f64, y: f64) -> Coordinate {
        Coordinate::new(x, y)
    }

    #[test]
    fn simplify_removes_collinear_noise() {
        let line = vec![c(0., 0.), c(1., 0.001), c(2., 0.), c(3., 0.)];
        let s = simplify(&line, 0.01);
        assert_eq!(s.len(), 2);
        assert_eq!(s[0], c(0., 0.));
        assert_eq!(s[1], c(3., 0.));
    }

    #[test]
    fn simplify_keeps_sharp_corner() {
        let l = vec![c(0., 0.), c(1., 5.), c(2., 0.)];
        assert_eq!(simplify(&l, 0.1).len(), 3);
    }

    #[test]
    fn densify_respects_max_spacing() {
        let out = densify(&[c(0., 0.), c(10., 0.)], 3.0);
        assert_eq!(out.len(), 5); // 0,3,6,9,10
        for w in out.windows(2) {
            assert!(w[0].euclidean_distance(&w[1]) <= 3.0 + 1e-9);
        }
    }

    #[test]
    fn snap_clamps_to_endpoints() {
        let (q, d) = snap_to_segment(&c(-5., 1.), &c(0., 0.), &c(10., 0.));
        assert_eq!(q, c(0., 0.));
        assert!((d - (26.0f64).sqrt()).abs() < 1e-9);
        let (mid, md) = snap_to_segment(&c(5., 2.), &c(0., 0.), &c(10., 0.));
        assert_eq!(mid, c(5., 0.));
        assert!((md - 2.0).abs() < 1e-9);
    }
}
