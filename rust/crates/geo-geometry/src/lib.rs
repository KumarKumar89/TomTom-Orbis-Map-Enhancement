//! `geo-geometry` — the deterministic geometry plane.
//!
//! All projection / tile-coordinate math is centralized here because Orbis Map
//! Display uses EPSG:3857 with a standard XYZ tile grid; scattering that math
//! through application code is a known source of subtle off-by-one bugs.

pub mod measure;
pub mod polyline;
pub mod projection;
pub mod similarity;

pub use measure::{bearing_deg, haversine_distance_m, planar_distance};
pub use projection::{web_mercator_from_lonlat, lonlat_from_web_mercator, LONLAT_MAX_LAT};
pub use similarity::{frechet_distance, hausdorff_distance};
