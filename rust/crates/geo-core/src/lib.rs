//! `geo-core` — foundational type system for the Orbis enhancement platform.
//!
//! Every other crate in the Rust workspace depends on these types. Geometry and
//! map-feature structures are strongly typed on purpose: modules must not invent
//! their own feature representation (see docs/architecture/ADR-0002).

pub mod error;
pub mod feature;
pub mod identifiers;
pub mod primitives;
pub mod transportation;

pub use error::{GeoError, GeoResult};
pub use feature::{Confidence, FeatureType, MapFeature, ProvenanceRecord, SourceRef};
pub use identifiers::{FeatureId, MapVersion, SourceId};
pub use primitives::{BoundingBox, Coordinate, LineString, Point, Polygon, Timestamp};
pub use transportation::{
    AccessProperties, Connector, Destination, Lane, Restriction, Road, Segment,
};

/// Version of the canonical data contract itself. Bump on breaking change.
pub const CONTRACT_VERSION: &str = "1.0.0";
