//! The canonical, versioned map feature contract.
//!
//! Every source adapter (Orbis, Overture, OSM, CV, customer data) must normalize
//! into [`MapFeature`] — no module may invent its own feature representation.
//! Mirrors `schemas/map-feature.schema.json`; keep both in sync via the
//! `data-validation.yml` CI job.

use crate::identifiers::{FeatureId, MapVersion, SourceId};
use crate::primitives::{BoundingBox, Timestamp};
use serde::{Deserialize, Serialize};

/// Feature classification aligned with Overture theme/feature pairs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureType {
    Segment,
    Connector,
    Road,
    Lane,
    Restriction,
    Destination,
    Intersection,
    Place,
    Address,
}

/// Calibrated confidence in [0, 1]. Constructed via [`Confidence::new`] which
/// rejects NaN/out-of-range values so scoring bugs fail fast.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Confidence(f64);

impl Confidence {
    pub fn new(value: f64) -> Option<Self> {
        if value.is_finite() && (0.0..=1.0).contains(&value) {
            Some(Self(value))
        } else {
            None
        }
    }

    pub fn certain() -> Self {
        Self(1.0)
    }

    pub fn value(&self) -> f64 {
        self.0
    }
}

/// Reference to a contributing upstream source of this feature.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceRef {
    pub source_id: SourceId,
    /// Source-native feature identifier (e.g. Overture uuid, OSM way id).
    pub native_id: String,
    /// Licensing / terms-of-use tag; never omit for external data.
    pub license_tag: String,
}

/// One provenance hop: which pipeline step produced or transformed this feature.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProvenanceRecord {
    pub step: String,
    pub processing_version: String,
    pub input_ids: Vec<String>,
    pub timestamp: Timestamp,
}

/// The canonical feature envelope shared by every plane of the system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapFeature {
    pub id: FeatureId,
    pub version: MapVersion,
    /// GeoJSON-ish geometry payload kept as JSON to avoid coupling the contract
    /// to one geometry library; validated against schemas/ at the boundary.
    pub geometry: serde_json::Value,
    pub bbox: BoundingBox,
    pub feature_type: FeatureType,
    pub properties: serde_json::Value,
    pub sources: Vec<SourceRef>,
    pub confidence: Confidence,
    pub observation_time: Timestamp,
    pub processing_version: String,
    pub provenance: Vec<ProvenanceRecord>,
    /// Contract version stamp for schema-drift detection in CI.
    pub contract_version: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confidence_rejects_invalid_values() {
        assert!(Confidence::new(f64::NAN).is_none());
        assert!(Confidence::new(-0.1).is_none());
        assert!(Confidence::new(1.1).is_none());
        assert_eq!(Confidence::new(0.5).unwrap().value(), 0.5);
    }

    #[test]
    fn feature_roundtrips_through_json() {
        let f = MapFeature {
            id: FeatureId::new("feat-1"),
            version: MapVersion::new("2026-10-04"),
            geometry: serde_json::json!({"type": "Point", "coordinates": [4.9, 52.3]}),
            bbox: BoundingBox::new(4.9, 52.3, 4.9, 52.3),
            feature_type: FeatureType::Segment,
            properties: serde_json::json!({"class": "motorway"}),
            sources: vec![SourceRef {
                source_id: SourceId::new("overture"),
                native_id: "08d5cefab..." .to_string(),
                license_tag: "CDLA-Permissive-2.0".to_string(),
            }],
            confidence: Confidence::certain(),
            observation_time: Timestamp::from_epoch_secs(1_760_000_000),
            processing_version: "pipeline-0.1.0".to_string(),
            provenance: vec![],
            contract_version: crate::CONTRACT_VERSION.to_string(),
        };
        let json = serde_json::to_string(&f).unwrap();
        let back: MapFeature = serde_json::from_str(&json).unwrap();
        assert_eq!(f, back);
    }
}
