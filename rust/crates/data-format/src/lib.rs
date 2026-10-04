//! `data-format` — file-level IO for the landing zone.
//!
//! Phase 1 handles GeoJSON + NDJSON manifests (checksum, provenance, licensing).
//! PBF/FlatGeobuf/GeoParquet readers land here in Phase 2 behind the same
//! [`FeatureReader`] trait so pipelines never special-case a source format.

use geo_core::{
    BoundingBox, Confidence, FeatureId, FeatureType, GeoResult, MapFeature, MapVersion,
    ProvenanceRecord, SourceRef, Timestamp,
};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;

pub trait FeatureReader {
    fn read_features(&mut self) -> GeoResult<Vec<MapFeature>>;
}

#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("bad feature at index {index}: {reason}")]
    BadFeature { index: usize, reason: String },
}

impl From<FormatError> for geo_core::GeoError {
    fn from(e: FormatError) -> Self {
        geo_core::GeoError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
    }
}

// ---------------------------------------------------------------- GeoJSON ---

#[derive(Debug, Deserialize, Serialize)]
struct GeoJsonFeatureCollection {
    features: Vec<GeoJsonFeature>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GeoJsonFeature {
    geometry: serde_json::Value,
    #[serde(default)]
    properties: serde_json::Value,
}

/// Reader that adapts GeoJSON into canonical MapFeatures.
pub struct GeoJsonSource<R: Read> {
    inner: R,
    source_id: String,
    license_tag: String,
}

impl<R: Read> GeoJsonSource<R> {
    pub fn new(inner: R, source_id: impl Into<String>, license_tag: impl Into<String>) -> Self {
        Self { inner, source_id: source_id.into(), license_tag: license_tag.into() }
    }
}

impl<R: Read> FeatureReader for GeoJsonSource<R> {
    fn read_features(&mut self) -> GeoResult<Vec<MapFeature>> {
        let mut buf = String::new();
        self.inner.read_to_string(&mut buf)?;
        let fc: GeoJsonFeatureCollection = serde_json::from_str(&buf)?;
        let now = Timestamp::from_epoch_secs(chronoless_epoch_now());
        let mut out = vec![];
        for (i, gf) in fc.features.iter().enumerate() {
            let bbox = bbox_from_geometry(&gf.geometry).map_err(|_| FormatError::BadFeature {
                index: i,
                reason: "no coordinates found".into(),
            })?;
            let feature_type = classify(&gf.properties);
            out.push(MapFeature {
                id: FeatureId::new(format!("{}-{}", self.source_id, i)),
                version: MapVersion::new("ingest"),
                geometry: gf.geometry.clone(),
                bbox,
                feature_type,
                properties: gf.properties.clone(),
                sources: vec![SourceRef {
                    source_id: geo_core::SourceId::new(&self.source_id),
                    native_id: i.to_string(),
                    license_tag: self.license_tag.clone(),
                }],
                confidence: Confidence::certain(),
                observation_time: now,
                processing_version: concat!("data-format-", env!("CARGO_PKG_VERSION")).into(),
                provenance: vec![ProvenanceRecord {
                    step: "geojson-ingest".into(),
                    processing_version: env!("CARGO_PKG_VERSION").into(),
                    input_ids: vec![format!("{}/features/{}", self.source_id, i)],
                    timestamp: now,
                }],
                contract_version: geo_core::CONTRACT_VERSION.into(),
            });
        }
        Ok(out)
    }
}

fn classify(props: &serde_json::Value) -> FeatureType {
    match props.get("layer").and_then(|v| v.as_str()) {
        Some("roads") => FeatureType::Segment,
        Some("addresses") => FeatureType::Address,
        Some("places") => FeatureType::Place,
        _ => FeatureType::Place,
    }
}

fn bbox_from_geometry(g: &serde_json::Value) -> Result<BoundingBox, ()> {
    fn walk(v: &serde_json::Value, xs: &mut Vec<f64>, ys: &mut Vec<f64>) {
        match v {
            serde_json::Value::Array(items) => {
                // A coordinate pair is [num, num]; anything else recurses.
                if items.len() >= 2
                    && items[0].is_number()
                    && items[1].is_number()
                {
                    if let (Some(x), Some(y)) = (items[0].as_f64(), items[1].as_f64()) {
                        xs.push(x);
                        ys.push(y);
                    }
                } else {
                    for i in items {
                        walk(i, xs, ys);
                    }
                }
            }
            _ => {}
        }
    }
    let (mut xs, mut ys) = (vec![], vec![]);
    walk(g.get("coordinates").ok_or(())?, &mut xs, &mut ys);
    if xs.is_empty() {
        return Err(());
    }
    Ok(BoundingBox::new(
        xs.iter().cloned().fold(f64::INFINITY, f64::min),
        ys.iter().cloned().fold(f64::INFINITY, f64::min),
        xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
    ))
}

// ---------------------------------------------------------------- Manifest ---

/// One object-store artifact inside a landing-zone manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
    pub source_id: String,
    pub license_tag: String,
}

/// NDJSON manifest writer/reader used to track raw artifacts + provenance.
pub struct Manifest {
    pub dataset: String,
    pub entries: Vec<ManifestEntry>,
}

impl Manifest {
    pub fn new(dataset: impl Into<String>) -> Self {
        Self { dataset: dataset.into(), entries: vec![] }
    }

    pub fn add(&mut self, e: ManifestEntry) {
        self.entries.push(e);
    }

    pub fn write_ndjson<W: Write>(&self, mut w: W) -> GeoResult<()> {
        let header = serde_json::json!({"dataset": self.dataset, "kind": "header"});
        writeln!(w, "{header}")?;
        for e in &self.entries {
            writeln!(w, "{}", serde_json::to_string(e)?)?;
        }
        Ok(())
    }

    pub fn read_ndjson<R: Read>(r: R) -> GeoResult<Self> {
        let mut br = BufReader::new(r).lines();
        let first: serde_json::Value = match br.next() {
            Some(l) => serde_json::from_str(&l?)?,
            None => Err(geo_core::GeoError::Validation("empty manifest".into()))?,
        };
        if first.get("kind").and_then(|k| k.as_str()) != Some("header") {
            return Err(geo_core::GeoError::Validation("manifest missing header".into()));
        }
        let dataset = first["dataset"].as_str().unwrap_or_default().to_string();
        let mut m = Manifest::new(dataset);
        for line in br {
            let entry: ManifestEntry = serde_json::from_str(&line?)?;
            m.entries.push(entry);
        }
        Ok(m)
    }
}

/// FNV-1a based stable checksum for tests/fixtures. Production code paths will
/// use the `sha2` crate once added; keeping this dependency-free for phase 1.
pub fn fnv1a_hex(data: &[u8]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in data {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{h:016x}")
}

/// Placeholder epoch seconds; replaced by `tokio-time`/`chrono` in Phase 2.
fn chronoless_epoch_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub fn open_geojson_file(path: &Path, source_id: &str, license: &str) -> GeoResult<GeoJsonSource<std::fs::File>> {
    let f = std::fs::File::open(path)?;
    Ok(GeoJsonSource::new(f, source_id, license))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
      "type": "FeatureCollection",
      "features": [
        {"geometry": {"type": "LineString", "coordinates": [[4.89,52.37],[4.90,52.37]]},
         "properties": {"layer": "roads", "class": "primary"}},
        {"geometry": {"type": "Point", "coordinates": [4.91, 52.38]},
         "properties": {"layer": "addresses"}}
      ]
    }"#;

    #[test]
    fn geojson_normalizes_into_canonical_features() {
        let mut src = GeoJsonSource::new(SAMPLE.as_bytes(), "osm-sample", "ODbL");
        let feats = src.read_features().unwrap();
        assert_eq!(feats.len(), 2);
        assert_eq!(feats[0].feature_type, FeatureType::Segment);
        assert_eq!(feats[1].feature_type, FeatureType::Address);
        assert_eq!(feats[0].bbox.min_x, 4.89);
        assert_eq!(feats[0].sources[0].license_tag, "ODbL");
        assert_eq!(feats[0].contract_version, geo_core::CONTRACT_VERSION);
    }

    #[test]
    fn bad_geometry_reports_index() {
        let bad = r#"{"features":[{"geometry":{"type":"Point","coordinates":[]},"properties":{}}]}"#;
        let mut src = GeoJsonSource::new(bad.as_bytes(), "x", "ODbL");
        let err = src.read_features().unwrap_err();
        assert!(err.to_string().contains("io error"), "{err}");
    }

    #[test]
    fn manifest_roundtrip() {
        let mut m = Manifest::new("orbis-amsterdam-2026-10");
        m.add(ManifestEntry {
            path: "raw/tiles/base.web/v1/12/1215/2045.pbf".into(),
            sha256: fnv1a_hex(b"fake-bytes"),
            bytes: 10,
            source_id: "orbis".into(),
            license_tag: "TomTom-commercial".into(),
        });
        let mut buf = vec![];
        m.write_ndjson(&mut buf).unwrap();
        let back = Manifest::read_ndjson(buf.as_slice()).unwrap();
        assert_eq!(back.dataset, "orbis-amsterdam-2026-10");
        assert_eq!(back.entries.len(), 1);
        assert_eq!(back.entries[0].path, m.entries[0].path);
    }

    #[test]
    fn checksum_is_stable_and_distinct() {
        assert_eq!(fnv1a_hex(b"abc"), fnv1a_hex(b"abc"));
        assert_ne!(fnv1a_hex(b"abc"), fnv1a_hex(b"abd"));
    }
}
