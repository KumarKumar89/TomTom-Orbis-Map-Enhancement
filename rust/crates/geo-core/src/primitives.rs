use serde::{Deserialize, Serialize};
use std::fmt;

/// A geographic or projected coordinate.
///
/// Semantics depend on the CRS context carried by the containing geometry:
/// - WGS84 lon/lat degrees for interchange / Overture alignment
/// - EPSG:3857 meters for tile math (Orbis Map Display uses 3857 + tile grid)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Coordinate {
    pub x: f64,
    pub y: f64,
}

impl Coordinate {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Construct from longitude/latitude in WGS84 degrees, validating ranges.
    pub fn from_lon_lat(lon: f64, lat: f64) -> Option<Self> {
        if lon.abs() <= 180.0 && lat.abs() <= 90.0 {
            Some(Self::new(lon, lat))
        } else {
            None
        }
    }

    pub fn euclidean_distance(&self, other: &Coordinate) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

impl fmt::Display for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

pub type Point = Coordinate;

/// An ordered list of coordinates with >= 2 positions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineString {
    pub coords: Vec<Coordinate>,
}

impl LineString {
    /// Returns `None` for degenerate lines (< 2 points).
    pub fn new(coords: Vec<Coordinate>) -> Option<Self> {
        if coords.len() >= 2 {
            Some(Self { coords })
        } else {
            None
        }
    }

    pub fn start(&self) -> Coordinate {
        self.coords[0]
    }

    pub fn end(&self) -> Coordinate {
        self.coords[self.coords.len() - 1]
    }

    pub fn is_closed(&self) -> bool {
        self.coords.first() == self.coords.last()
    }

    pub fn length(&self) -> f64 {
        self.coords
            .windows(2)
            .map(|w| w[0].euclidean_distance(&w[1]))
            .sum()
    }

    pub fn bbox(&self) -> BoundingBox {
        BoundingBox::from_coords(&self.coords)
    }
}

/// A simple polygon ring: closed, >= 4 positions (first == last).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Polygon {
    pub exterior: Vec<Coordinate>,
    pub interiors: Vec<Vec<Coordinate>>,
}

impl Polygon {
    /// Returns `None` when the exterior ring is not a valid closed ring.
    pub fn new(exterior: Vec<Coordinate>, interiors: Vec<Vec<Coordinate>>) -> Option<Self> {
        if exterior.len() < 4 || !exterior.first().zip(exterior.last()).map_or(true, |(a, b)| a == b)
        {
            return None;
        }
        if interiors.iter().any(|r| r.len() < 4) {
            return None;
        }
        Some(Self { exterior, interiors })
    }

    pub fn bbox(&self) -> BoundingBox {
        BoundingBox::from_coords(&self.exterior)
    }
}

/// Axis-aligned bounding box with normalized ordering (min <= max).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl BoundingBox {
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            min_x: min_x.min(max_x),
            min_y: min_y.min(max_y),
            max_x: max_x.max(min_x),
            max_y: max_y.max(min_y),
        }
    }

    pub fn from_coords(coords: &[Coordinate]) -> Self {
        let mut it = coords.iter();
        let first = it
            .next()
            .copied()
            .unwrap_or(Coordinate::new(0.0, 0.0));
        it.fold(Self::new(first.x, first.y, first.x, first.y), |acc, c| {
            Self::new(
                acc.min_x.min(c.x),
                acc.min_y.min(c.y),
                acc.max_x.max(c.x),
                acc.max_y.max(c.y),
            )
        })
    }

    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.min_x <= other.max_x
            && self.max_x >= other.min_x
            && self.min_y <= other.max_y
            && self.max_y >= other.min_y
    }

    pub fn contains(&self, c: &Coordinate) -> bool {
        c.x >= self.min_x && c.x <= self.max_x && c.y >= self.min_y && c.y <= self.max_y
    }

    pub fn union(&self, other: &BoundingBox) -> BoundingBox {
        Self::new(
            self.min_x.min(other.min_x),
            self.min_y.min(other.min_y),
            self.max_x.max(other.max_x),
            self.max_y.max(other.max_y),
        )
    }

    pub fn area(&self) -> f64 {
        (self.max_x - self.min_x) * (self.max_y - self.min_y)
    }
}

/// Unix epoch seconds. All ingestion timestamps must be normalized to UTC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp(pub i64);

impl Timestamp {
    pub fn from_epoch_secs(secs: i64) -> Self {
        Self(secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(x: f64, y: f64) -> Coordinate {
        Coordinate::new(x, y)
    }

    #[test]
    fn coordinate_lon_lat_validation() {
        assert!(Coordinate::from_lon_lat(4.9, 52.3).is_some());
        assert!(Coordinate::from_lon_lat(181.0, 0.0).is_none());
        assert!(Coordinate::from_lon_lat(0.0, 91.0).is_none());
    }

    #[test]
    fn linestring_requires_two_points() {
        assert!(LineString::new(vec![c(0., 0.)]).is_none());
        let ls = LineString::new(vec![c(0., 0.), c(3., 4.)]).unwrap();
        assert!((ls.length() - 5.0).abs() < 1e-12);
    }

    #[test]
    fn polygon_rejects_open_ring() {
        let open = vec![c(0., 0.), c(1., 0.), c(1., 1.), c(0., 1.)];
        assert!(Polygon::new(open, vec![]).is_none());
        let closed = vec![c(0., 0.), c(1., 0.), c(1., 1.), c(0., 0.)];
        assert!(Polygon::new(closed, vec![]).is_some());
    }

    #[test]
    fn bbox_normalizes_and_intersects() {
        let a = BoundingBox::new(10.0, 10.0, 0.0, 0.0); // unnormalized input
        assert_eq!(a, BoundingBox::new(0.0, 0.0, 10.0, 10.0));
        let b = BoundingBox::new(5.0, 5.0, 20.0, 20.0);
        assert!(a.intersects(&b));
        let far = BoundingBox::new(100.0, 100.0, 110.0, 110.0);
        assert!(!a.intersects(&far));
        assert!(a.contains(&c(5.0, 5.0)));
    }
}
