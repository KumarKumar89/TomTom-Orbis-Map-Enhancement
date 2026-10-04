//! Web-Mercator (EPSG:3857) <-> WGS84 (EPSG:4326) transforms and tile math.

use geo_core::Coordinate;

/// Clamping latitude used by the Mercator projection (EPSG:3857 valid range).
pub const LONLAT_MAX_LAT: f64 = 85.051_128_779_806_6;

const EARTH_RADIUS_M: f64 = 6_378_137.0; // WGS84 semi-major axis

/// Project lon/lat degrees to EPSG:3857 meters.
pub fn web_mercator_from_lonlat(lon: f64, lat: f64) -> Coordinate {
    let lat = lat.clamp(-LONLAT_MAX_LAT, LONLAT_MAX_LAT);
    let x = EARTH_RADIUS_M.to_radians() * lon;
    let s = lat.to_radians().sin();
    let y = EARTH_RADIUS_M * ((1.0 + s) / (1.0 - s)).ln() / 2.0;
    Coordinate::new(x, y)
}

/// Inverse Web-Mercator projection: EPSG:3857 meters to lon/lat degrees.
pub fn lonlat_from_web_mercator(c: &Coordinate) -> (f64, f64) {
    let lon = c.x / EARTH_RADIUS_M * 180.0 / std::f64::consts::PI;
    let lat = (c.y / EARTH_RADIUS_M).sinh().atan().to_degrees();
    (lon, lat)
}

/// Number of tiles along one axis at zoom level z (XYZ / slippy scheme).
pub fn tile_count(zoom: u8) -> u64 {
    1u64 << zoom
}

/// Tile indices containing an EPSG:3857 point at the given zoom.
///
/// The 3857 world spans [-20037508.342789244, +20037508.342789244] on both axes.
pub fn tile_indices(x_3857: f64, y_3857: f64, zoom: u8) -> (u64, u64) {
    const HALF: f64 = 20_037_508.342_789_244;
    let n = tile_count(zoom) as f64;
    let xf = (x_3857 + HALF) / (2.0 * HALF);
    // Y flips: tile row 0 is at the top (max y in 3857).
    let yf = (HALF - y_3857) / (2.0 * HALF);
    let tx = (xf * n).clamp(0.0, n - 1.0) as u64;
    let ty = (yf * n).clamp(0.0, n - 1.0) as u64;
    (tx, ty)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mercator_roundtrip_amsterdam() {
        let (lon, lat) = (4.8936, 52.3738);
        let m = web_mercator_from_lonlat(lon, lat);
        let (lon2, lat2) = lonlat_from_web_mercator(&m);
        assert!((lon - lon2).abs() < 1e-9);
        assert!((lat - lat2).abs() < 1e-9);
    }

    #[test]
    fn origin_maps_to_tile_center_boundary() {
        // Equator/prime meridian should sit exactly on the middle tile edge.
        let m = web_mercator_from_lonlat(0.0, 0.0);
        assert!(m.x.abs() < 1e-9 && m.y.abs() < 1e-9);
        let (tx, ty) = tile_indices(m.x, m.y, 1);
        assert_eq!((tx, ty), (1, 1)); // 2x2 grid, center corner
    }

    #[test]
    fn tile_indices_clamp_at_extremes() {
        let (tx, _) = tile_indices(20_037_508.34, 0.0, 3);
        assert_eq!(tx, 7); // last tile index at zoom 3
        let (_, ty) = tile_indices(0.0, 20_037_508.34, 2);
        assert_eq!(ty, 0); // top row
    }

    #[test]
    fn high_latitude_is_clamped_not_nan() {
        let m = web_mercator_from_lonlat(0.0, 89.9);
        assert!(m.y.is_finite());
    }
}
