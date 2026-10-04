//! Pure-Rust functions that the PyO3 layer exposes. Every function here is
//! deterministic and independently unit-tested, so Python-side behaviour is
//! exactly the tested Rust behaviour.

use geo_core::{Coordinate, MapFeature};
use geo_geometry::measure::haversine_distance_m;
use geo_geometry::projection::web_mercator_from_lonlat;
use geo_geometry::similarity::frechet_distance;

pub fn mercator_from_lonlat(lon: f64, lat: f64) -> (f64, f64) {
    let c = web_mercator_from_lonlat(lon, lat);
    (c.x, c.y)
}

pub fn haversine_m(lon1: f64, lat1: f64, lon2: f64, lat2: f64) -> f64 {
    haversine_distance_m(&Coordinate::new(lon1, lat1), &Coordinate::new(lon2, lat2))
}

pub fn frechet(a: &[(f64, f64)], b: &[(f64, f64)]) -> f64 {
    let pa: Vec<Coordinate> = a.iter().map(|p| Coordinate::new(p.0, p.1)).collect();
    let pb: Vec<Coordinate> = b.iter().map(|p| Coordinate::new(p.0, p.1)).collect();
    if pa.is_empty() || pb.is_empty() {
        return f64::NAN; // Python side maps this to ValueError via wrapper if needed
    }
    frechet_distance(&pa, &pb)
}

/// Parse + structurally validate a canonical MapFeature JSON document.
pub fn validate_feature_json(json: &str) -> Result<bool, serde_json::Error> {
    let f: MapFeature = serde_json::from_str(json)?;
    let ok = f.contract_version == geo_core::CONTRACT_VERSION
        && !f.sources.is_empty()
        && f.confidence.value() >= 0.0;
    if ok {
        Ok(true)
    } else {
        Err(serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "feature failed contract validation",
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frechet_of_same_line_is_zero() {
        let l = vec![(0.0, 0.0), (1.0, 1.0)];
        assert!(frechet(&l, &l).abs() < 1e-12);
    }

    #[test]
    fn empty_input_returns_nan_not_panic() {
        assert!(frechet(&[], &[(0., 0.)]).is_nan());
    }

    #[test]
    fn valid_feature_json_passes() {
        let feat = serde_json::json!({
            "id": "f1",
            "version": "v",
            "geometry": {"type":"Point","coordinates":[0.0,0.0]},
            "bbox": {"min_x":0.0,"min_y":0.0,"max_x":0.0,"max_y":0.0},
            "feature_type": "segment",
            "properties": {},
            "sources": [{"source_id":"osm","native_id":"1","license_tag":"ODbL"}],
            "confidence": 0.9,
            "observation_time": 0,
            "processing_version": "t",
            "provenance": [],
            "contract_version": geo_core::CONTRACT_VERSION
        });
        assert!(validate_feature_json(&feat.to_string()).unwrap());
    }

    #[test]
    fn wrong_contract_version_fails() {
        let feat = serde_json::json!({
            "id": "f1", "version": "v",
            "geometry": {"type":"Point","coordinates":[0.0,0.0]},
            "bbox": {"min_x":0.0,"min_y":0.0,"max_x":0.0,"max_y":0.0},
            "feature_type": "segment", "properties": {},
            "sources": [{"source_id":"osm","native_id":"1","license_tag":"ODbL"}],
            "confidence": 0.9, "observation_time": 0,
            "processing_version": "t", "provenance": [],
            "contract_version": "0.0.1"
        });
        assert!(validate_feature_json(&feat.to_string()).is_err());
    }
}
