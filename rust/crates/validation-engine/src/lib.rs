//! `validation-engine` — the automated release quality gate.
//!
//! Combines contract-schema checks, topology findings and statistical drift into
//! a single machine-readable [`QualityReport`] that CI can fail on:
//!
//! ```text
//! features -> schema validation -> topology invariants -> anomaly scan -> report
//! ```

use geo_core::{FeatureId, GeoError, GeoResult, MapFeature};
use serde::Serialize;
use std::collections::HashMap;
use topology_engine::{RoadNetwork, Severity};

#[derive(Debug, Clone, Serialize)]
pub struct GateViolation {
    pub code: String,
    pub severity: String,
    pub message: String,
    pub feature_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct QualityMetrics {
    pub total_features: usize,
    pub invalid_geometries: usize,
    pub topology_errors: usize,
    pub topology_warnings: usize,
    pub duplicate_candidates: usize,
    pub orphan_features: usize,
    /// Mean confidence across features (drift-tracked metric).
    pub mean_confidence: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct QualityReport {
    pub passed: bool,
    pub metrics: QualityMetrics,
    pub violations: Vec<GateViolation>,
}

impl QualityReport {
    /// Exit-code semantics for the CLI: 0 = pass, 1 = gated failures.
    pub fn exit_code(&self) -> i32 {
        if self.passed {
            0
        } else {
            1
        }
    }
}

/// Validate canonical features against the contract basics we can check without
/// an external JSON-Schema runner (full schema run happens in Python CI).
pub fn validate_feature_contract(f: &MapFeature) -> Vec<GateViolation> {
    let mut v = vec![];
    if f.contract_version != geo_core::CONTRACT_VERSION {
        v.push(GateViolation {
            code: "SCHEMA-001-contract-version".into(),
            severity: "error".into(),
            message: format!(
                "feature {} has contract {}, expected {}",
                f.id, f.contract_version, geo_core::CONTRACT_VERSION
            ),
            feature_ids: vec![f.id.to_string()],
        });
    }
    if f.sources.is_empty() {
        v.push(GateViolation {
            code: "SCHEMA-002-missing-source".into(),
            severity: "error".into(),
            message: format!("feature {} has no source provenance", f.id),
            feature_ids: vec![f.id.to_string()],
        });
    }
    // bbox must be normalized
    if f.bbox.min_x > f.bbox.max_x || f.bbox.min_y > f.bbox.max_y {
        v.push(GateViolation {
            code: "GEO-001-inverted-bbox".into(),
            severity: "error".into(),
            message: format!("feature {} has inverted bbox", f.id),
            feature_ids: vec![f.id.to_string()],
        });
    }
    v
}

/// Run the full gate over a network + its canonical feature envelope.
pub fn run_release_gate(
    features: &[MapFeature],
    network: &RoadNetwork,
    max_topology_errors: usize,
    min_mean_confidence: f64,
) -> GeoResult<QualityReport> {
    let mut violations = vec![];
    let mut invalid_geometries = 0usize;

    for f in features {
        violations.extend(validate_feature_contract(f));
        if !geometry_payload_is_sane(&f.geometry) {
            invalid_geometries += 1;
            violations.push(GateViolation {
                code: "GEO-002-invalid-geometry-payload".into(),
                severity: "error".into(),
                message: format!("feature {} geometry payload failed structural check", f.id),
                feature_ids: vec![f.id.to_string()],
            });
        }
    }

    let topo = network.validate()?;
    let mut topo_err = 0;
    let mut topo_warn = 0;
    let mut dupes = 0;
    let mut orphans = 0;
    for f in &topo {
        match f.severity {
            Severity::Error => {
                topo_err += 1;
                violations.push(GateViolation {
                    code: f.code.into(),
                    severity: "error".into(),
                    message: f.message.clone(),
                    feature_ids: f.feature_ids.iter().map(|i| i.to_string()).collect(),
                });
            }
            Severity::Warning => {
                topo_warn += 1;
                if f.code.contains("duplicate") {
                    dupes += 1;
                }
                if f.code.contains("orphan") {
                    orphans += 1;
                }
                violations.push(GateViolation {
                    code: f.code.into(),
                    severity: "warning".into(),
                    message: f.message.clone(),
                    feature_ids: f.feature_ids.iter().map(|i| i.to_string()).collect(),
                });
            }
            Severity::Info => {}
        }
    }

    let mean_confidence = if features.is_empty() {
        1.0
    } else {
        features.iter().map(|f| f.confidence.value()).sum::<f64>() / features.len() as f64
    };

    if topo_err > max_topology_errors {
        violations.push(GateViolation {
            code: "GATE-001-topology-error-budget".into(),
            severity: "error".into(),
            message: format!("{topo_err} topology errors exceed budget {max_topology_errors}"),
            feature_ids: vec![],
        });
    }
    if mean_confidence < min_mean_confidence {
        violations.push(GateViolation {
            code: "GATE-002-confidence-floor".into(),
            severity: "error".into(),
            message: format!(
                "mean confidence {mean_confidence:.3} below floor {min_mean_confidence}"
            ),
            feature_ids: vec![],
        });
    }

    let passed = !violations.iter().any(|v| v.severity == "error");
    Ok(QualityReport {
        passed,
        metrics: QualityMetrics {
            total_features: features.len(),
            invalid_geometries,
            topology_errors: topo_err,
            topology_warnings: topo_warn,
            duplicate_candidates: dupes,
            orphan_features: orphans,
            mean_confidence,
        },
        violations,
    })
}

fn geometry_payload_is_sane(g: &serde_json::Value) -> bool {
    // Structural sanity: type present, coordinates finite numbers.
    let Some(t) = g.get("type").and_then(|t| t.as_str()) else {
        return false;
    };
    if !matches!(t, "Point" | "LineString" | "Polygon" | "MultiLineString") {
        return false;
    }
    fn finite(v: &serde_json::Value) -> bool {
        match v {
            serde_json::Value::Array(xs) => xs.iter().all(finite),
            serde_json::Value::Number(n) => n.as_f64().map_or(false, |x| x.is_finite()),
            _ => false,
        }
    }
    g.get("coordinates").map(finite).unwrap_or(false)
}

/// Compare two reports to detect regression between map versions.
pub fn diff_reports(before: &QualityReport, after: &QualityReport) -> HashMap<&'static str, String> {
    let mut out = HashMap::new();
    let cmp = |name: &'static str, a: usize, b: usize, out: &mut HashMap<&'static str, String>| {
        if b > a {
            out.insert(name, format!("regressed {a} -> {b}"));
        }
    };
    cmp("topology_errors", before.metrics.topology_errors, after.metrics.topology_errors, &mut out);
    cmp("invalid_geometries", before.metrics.invalid_geometries, after.metrics.invalid_geometries, &mut out);
    cmp("duplicate_candidates", before.metrics.duplicate_candidates, after.metrics.duplicate_candidates, &mut out);
    if after.metrics.mean_confidence + 1e-9 < before.metrics.mean_confidence {
        out.insert(
            "mean_confidence",
            format!("{:.4} -> {:.4}", before.metrics.mean_confidence, after.metrics.mean_confidence),
        );
    }
    out
}

// Re-export commonly used id type so downstream crates need fewer imports.
pub use geo_core::FeatureId as Id;
#[allow(unused_imports)]
use std::convert::Infallible as _Unused;

#[cfg(test)]
mod tests {
    use super::*;
    use geo_core::{BoundingBox, Confidence, FeatureType, MapVersion, SourceRef, Timestamp};
    use geo_core::identifiers::SourceId;
    use geo_core::primitives::{Coordinate, LineString};

    fn feature(id: &str, conf: f64) -> MapFeature {
        MapFeature {
            id: FeatureId::new(id),
            version: MapVersion::new("v1"),
            geometry: serde_json::json!({"type": "LineString", "coordinates": [[0.0,0.0],[1.0,1.0]]}),
            bbox: BoundingBox::new(0., 0., 1., 1.),
            feature_type: FeatureType::Segment,
            properties: serde_json::json!({}),
            sources: vec![SourceRef {
                source_id: SourceId::new("overture"),
                native_id: id.into(),
                license_tag: "CDLA-Permissive-2.0".into(),
            }],
            confidence: Confidence::new(conf).unwrap(),
            observation_time: Timestamp::from_epoch_secs(0),
            processing_version: "test".into(),
            provenance: vec![],
            contract_version: geo_core::CONTRACT_VERSION.into(),
        }
    }

    fn empty_network() -> RoadNetwork {
        RoadNetwork::new()
    }

    #[test]
    fn clean_input_passes_gate() {
        let feats = vec![feature("f1", 0.95)];
        let rep = run_release_gate(&feats, &empty_network(), 0, 0.9).unwrap();
        assert!(rep.passed, "{:?}", rep.violations);
        assert_eq!(rep.exit_code(), 0);
    }

    #[test]
    fn confidence_floor_blocks_release() {
        let feats = vec![feature("f1", 0.5)];
        let rep = run_release_gate(&feats, &empty_network(), 0, 0.9).unwrap();
        assert!(!rep.passed);
        assert!(rep.violations.iter().any(|v| v.code == "GATE-002-confidence-floor"));
    }

    #[test]
    fn missing_source_is_error() {
        let mut f = feature("f1", 1.0);
        f.sources.clear();
        let rep = run_release_gate(&[f], &empty_network(), 0, 0.0).unwrap();
        assert!(!rep.passed);
        assert!(rep.violations.iter().any(|v| v.code == "SCHEMA-002-missing-source"));
    }

    #[test]
    fn invalid_geometry_payload_detected() {
        let mut f = feature("f1", 1.0);
        f.geometry = serde_json::json!({"type": "LineString", "coordinates": [[0.0, f64::NAN],[1.0,1.0]]});
        let rep = run_release_gate(&[f], &empty_network(), 0, 0.0).unwrap();
        assert!(rep.violations.iter().any(|v| v.code == "GEO-002-invalid-geometry-payload"));
    }

    #[test]
    fn regression_diff_flags_worse_metrics() {
        let good = run_release_gate(&[feature("f1", 0.9)], &empty_network(), 5, 0.5).unwrap();
        let bad_feats = vec![feature("f1", 0.6)];
        let mut net = RoadNetwork::new();
        net.add_connector(geo_core::Connector {
            id: FeatureId::new("ghost"),
            segment_ids: vec![],
            is_dead_end: false,
            sub_type: geo_core::transportation::ConnectorSubType::Junction,
        });
        let bad = run_release_gate(&bad_feats, &net, 5, 0.5).unwrap();
        let d = diff_reports(&good, &bad);
        assert!(d.contains_key("topology_errors"));
        assert!(d.contains_key("mean_confidence"));
    }
}
