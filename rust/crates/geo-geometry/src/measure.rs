//! Distance and bearing measures. Spherical (haversine) for geographic coords,
//! planar for projected ones.

use geo_core::Coordinate;

const EARTH_MEAN_RADIUS_M: f64 = 6_371_008.8;

/// Great-circle distance in meters between two lon/lat coordinates (degrees).
pub fn haversine_distance_m(a: &Coordinate, b: &Coordinate) -> f64 {
    let r = EARTH_MEAN_RADIUS_M;
    let dlat = (b.y - a.y).to_radians();
    let dlon = (b.x - a.x).to_radians();
    let la = a.y.to_radians();
    let lb = b.y.to_radians();
    let h = (dlat / 2.0).sin().powi(2) + la.cos() * lb.cos() * (dlon / 2.0).sin().powi(2);
    2.0 * r * h.sqrt().asin()
}

/// Planar Euclidean distance (use only within a projected CRS such as EPSG:3857).
pub fn planar_distance(a: &Coordinate, b: &Coordinate) -> f64 {
    a.euclidean_distance(b)
}

/// Initial bearing from `a` to `b` in degrees clockwise from north, [0, 360).
pub fn bearing_deg(a: &Coordinate, b: &Coordinate) -> f64 {
    let lam = a.y.to_radians();
    let lbe = b.y.to_radians();
    let dl = (b.x - a.x).to_radians();
    let y = dl.sin() * lbe.cos();
    let x = lam.cos() * lbe.sin() - lam.sin() * lbe.cos() * dl.cos();
    let th = y.atan2(x).to_degrees();
    (th + 360.0) % 360.0
}

/// Smallest angular difference between two headings in degrees, in [0, 180].
pub fn heading_diff_deg(a: f64, b: f64) -> f64 {
    let d = (a - b).abs() % 360.0;
    if d > 180.0 {
        360.0 - d
    } else {
        d
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(x: f64, y: f64) -> Coordinate {
        Coordinate::new(x, y)
    }

    #[test]
    fn haversine_amsterdam_rotterdam() {
        // Known approx 58 km between these city centers.
        let ams = c(4.8936, 52.3738);
        let rot = c(4.4777, 51.9244);
        let d = haversine_distance_m(&ams, &rot);
        assert!((57_000.0..60_000.0).contains(&d), "got {d}");
    }

    #[test]
    fn bearing_north_is_zero() {
        let south = c(0.0, 0.0);
        let north = c(0.0, 1.0);
        let b = bearing_deg(&south, &north);
        assert!(b < 1e-6 || (b - 360.0).abs() < 1e-6);
    }

    #[test]
    fn bearing_east_is_90() {
        let a = c(0.0, 52.0);
        let b = c(1.0, 52.0);
        let brg = bearing_deg(&a, &b);
        assert!((brg - 90.0).abs() < 0.5, "got {brg}");
    }

    #[test]
    fn heading_diff_wraps() {
        assert!((heading_diff_deg(350.0, 10.0) - 20.0).abs() < 1e-9);
        assert!((heading_diff_deg(0.0, 180.0) - 180.0).abs() < 1e-9);
    }
}
