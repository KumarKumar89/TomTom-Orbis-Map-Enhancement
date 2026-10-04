//! `tile-engine` — tile indexing + geometry preparation for vector tiles.
//!
//! The Orbis Map Display API uses EPSG:3857 with a standard XYZ tile grid; all
//! tile arithmetic lives here so producers (Rust) and consumers (MapLibre UI)
//! agree on boundaries down to the bit.

use geo_core::{BoundingBox, Coordinate, LineString};
use geo_geometry::projection::{lonlat_from_web_mercator, web_mercator_from_lonlat};

pub const WORLD_HALF_EXTENT: f64 = 20_037_508.342_789_244;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TileId {
    pub z: u8,
    pub x: u64,
    pub y: u64,
}

impl TileId {
    /// Bounding box of this tile in EPSG:3857 meters.
    pub fn bbox_3857(&self) -> BoundingBox {
        let n = (1u64 << self.z) as f64;
        let s = 2.0 * WORLD_HALF_EXTENT / n;
        let min_x = -WORLD_HALF_EXTENT + self.x as f64 * s;
        let max_x = min_x + s;
        // Row 0 is at the top (max y).
        let max_y = WORLD_HALF_EXTENT - self.y as f64 * s;
        let min_y = max_y - s;
        BoundingBox::new(min_x, min_y, max_x, max_y)
    }

    pub fn from_lonlat(lon: f64, lat: f64, zoom: u8) -> Self {
        let m = web_mercator_from_lonlat(lon, lat);
        let (x, y) = geo_geometry::projection::tile_indices(m.x, m.y, zoom);
        TileId { z: zoom, x, y }
    }

    /// All tiles at `zoom` intersecting a lon/lat bbox (input in degrees).
    pub fn covering(min_lon: f64, min_lat: f64, max_lon: f64, max_lat: f64, zoom: u8) -> Vec<TileId> {
        let a = web_mercator_from_lonlat(min_lon, min_lat);
        let b = web_mercator_from_lonlat(max_lon, max_lat);
        let (x0, y1) = geo_geometry::projection::tile_indices(a.x, a.y, zoom);
        let (x1, y0) = geo_geometry::projection::tile_indices(b.x, b.y, zoom);
        let mut out = vec![];
        for x in x0..=x1 {
            for y in y0..=y1 {
                out.push(TileId { z: zoom, x, y });
            }
        }
        out
    }
}

/// Clip a 3857-coordinate polyline to a tile bbox (Cohen–Sutherland style
/// Liang–Barsky per segment; sufficient and exact for tile-edge clipping).
pub fn clip_line_to_tile(line_3857: &LineString, tile: &TileId) -> Vec<LineString> {
    let bb = tile.bbox_3857();
    let mut out: Vec<LineString> = vec![];
    for w in line_3857.coords.windows(2) {
        if let Some(seg) = clip_segment(w[0], w[1], &bb) {
            match out.last_mut() {
                // Stitch adjacent clipped pieces sharing an endpoint.
                Some(prev) if prev.end() == seg.start() => {
                    prev.coords.push(seg.end());
                }
                _ => out.push(LineString::new(vec![seg.start(), seg.end()]).unwrap()),
            }
        }
    }
    out
}

fn clip_segment(a: Coordinate, b: Coordinate, bb: &BoundingBox) -> Option<LineString> {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let (mut t0, mut t1) = (0.0f64, 1.0f64);
    for (p, q) in [(-dx, a.x - bb.min_x), (dx, bb.max_x - a.x), (-dy, a.y - bb.min_y), (dy, bb.max_y - a.y)] {
        if p == 0.0 {
            if q < 0.0 {
                return None;
            }
        } else {
            let r = q / p;
            if p < 0.0 {
                if r > t1 {
                    return None;
                }
                if r > t0 {
                    t0 = r;
                }
            } else {
                if r < t0 {
                    return None;
                }
                if r < t1 {
                    t1 = r;
                }
            }
        }
    }
    if (t1 - t0).abs() < 1e-12 {
        return None; // degenerate touch
    }
    let p0 = Coordinate::new(a.x + t0 * dx, a.y + t0 * dy);
    let p1 = Coordinate::new(a.x + t1 * dx, a.y + t1 * dy);
    LineString::new(vec![p0, p1])
}

/// Convert a 3857 coordinate into tile-local normalized coordinates [0,1].
pub fn to_tile_local(c_3857: &Coordinate, tile: &TileId) -> (f64, f64) {
    let bb = tile.bbox_3857();
    let fx = (c_3857.x - bb.min_x) / (bb.max_x - bb.min_x);
    // Flip y so that 0 is at the top edge (screen convention).
    let fy = (bb.max_y - c_3857.y) / (bb.max_y - bb.min_y);
    (fx, fy)
}

/// Sanity helper used by tests and golden data generation.
pub fn tile_center_lonlat(tile: &TileId) -> (f64, f64) {
    let bb = tile.bbox_3857();
    lonlat_from_web_mercator(&Coordinate::new(
        (bb.min_x + bb.max_x) / 2.0,
        (bb.min_y + bb.max_y) / 2.0,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom0_is_whole_world() {
        let t = TileId { z: 0, x: 0, y: 0 };
        let bb = t.bbox_3857();
        assert!((bb.min_x + WORLD_HALF_EXTENT).abs() < 1e-6);
        assert!((bb.max_x - WORLD_HALF_EXTENT).abs() < 1e-6);
    }

    #[test]
    fn tile_covering_amsterdam_zoom10() {
        let tiles = TileId::covering(4.6, 52.2, 5.1, 52.5, 10);
        assert!(!tiles.is_empty());
        // Every returned tile must actually contain part of the query point set.
        let probe = TileId::from_lonlat(4.8936, 52.3738, 10);
        assert!(tiles.contains(&probe));
    }

    #[test]
    fn clip_keeps_inside_and_splits_outside() {
        let tile = TileId { z: 1, x: 1, y: 1 }; // covers 0..half-world quadrant
        let bb = tile.bbox_3857();
        let inside = LineString::new(vec![
            Coordinate::new(bb.min_x + 100., 0.0),
            Coordinate::new(bb.max_x - 100., 0.0),
        ])
        .unwrap();
        let parts = clip_line_to_tile(&inside, &tile);
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].length(), inside.length());

        let crossing = LineString::new(vec![
            Coordinate::new(-WORLD_HALF_EXTENT, 0.0),
            Coordinate::new(WORLD_HALF_EXTENT, 0.0),
        ])
        .unwrap();
        let parts = clip_line_to_tile(&crossing, &tile);
        assert!(!parts.is_empty());
        for p in &parts {
            let pb = p.bbox();
            assert!(pb.min_x >= bb.min_x - 1e-6 && pb.max_x <= bb.max_x + 1e-6);
        }
    }

    #[test]
    fn tile_local_flips_y() {
        let tile = TileId { z: 1, x: 1, y: 1 };
        let bb = tile.bbox_3857();
        let (fx, fy) = to_tile_local(&Coordinate::new(bb.min_x, bb.max_y), &tile);
        assert!((fx - 0.0).abs() < 1e-9);
        assert!((fy - 0.0).abs() < 1e-9); // top-left corner is (0,0)
    }

    #[test]
    fn tile_center_roundtrip() {
        let t = TileId::from_lonlat(4.8936, 52.3738, 12);
        let (lon, lat) = tile_center_lonlat(&t);
        let t2 = TileId::from_lonlat(lon, lat, 12);
        assert_eq!(t, t2);
    }
}
