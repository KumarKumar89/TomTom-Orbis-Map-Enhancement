//! Overture-aligned transportation model: segments + connectors and their semantics.
//!
//! Overture represents the road network as **segments** (directed pieces of road
//! between physical connection points) joined by **connectors**. Access
//! restrictions, destinations and lanes attach to these primitives. This module
//! mirrors that structure so validation invariants can be expressed directly.

use crate::identifiers::FeatureId;
use crate::primitives::LineString;
use serde::{Deserialize, Serialize};

/// A directed piece of road between two connectors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Segment {
    pub id: FeatureId,
    /// Ordered centerline geometry.
    pub geometry: LineString,
    /// Physical connector at the start of travel direction.
    pub from_connector: FeatureId,
    /// Physical connector at the end of travel direction.
    pub to_connector: FeatureId,
    /// Road this segment belongs to (may be absent for ungrouped data).
    pub road_id: Option<FeatureId>,
    pub class: String,
    /// Number of travel lanes in the digitized direction.
    pub lane_count: Option<u8>,
    /// Speed limit in km/h where known.
    pub max_speed_kmh: Option<u16>,
    pub one_way: bool,
}

/// Physical connection point between segments (an intersection node).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Connector {
    pub id: FeatureId,
    /// Segments attached to this connector. Overture invariant: >= 2 except
    /// for declared dead-ends (tracked by the topology engine).
    pub segment_ids: Vec<FeatureId>,
    pub is_dead_end: bool,
    pub sub_type: ConnectorSubType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorSubType {
    /// Ordinary junction between segments of the same road(s).
    Junction,
    /// Connection across grade-separated roads.
    Interchange,
    /// Border crossing etc.
    Other,
}

/// Logical road grouping several segments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Road {
    pub id: FeatureId,
    pub name: String,
    pub class: String,
    pub segment_ids: Vec<FeatureId>,
}

/// Travel lane within a segment cross-section.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lane {
    pub id: FeatureId,
    pub segment_id: FeatureId,
    /// 0-based index from the curb on the right side of travel.
    pub index: u8,
    pub kind: LaneKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LaneKind {
    General,
    Turn,
    MergedThrough,
    Bike,
    Bus,
    Other,
}

/// Access restriction attached to a segment and/or connector transition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Restriction {
    pub id: FeatureId,
    pub applies_to_segment: Option<FeatureId>,
    pub applies_to_connector: Option<FeatureId>,
    /// Transition: (from_segment, to_segment) through the connector.
    pub transition: Option<(FeatureId, FeatureId)>,
    pub kind: RestrictionKind,
    pub access: AccessProperties,
    /// ISO-8601-like free text schedule ("Mo-Fa 07:00-10:00") where applicable.
    pub when: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RestrictionKind {
    Prohibited,
    OnlyAllowed,
    MandatoryMovement,
    Informational,
}

/// Vehicle/access class applicability of a restriction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AccessProperties {
    pub passenger_vehicle: bool,
    pub hgv: bool,
    pub bus: bool,
    pub pedestrian: bool,
    pub bicycle: bool,
    pub emergency: bool,
}

impl Default for AccessProperties {
    fn default() -> Self {
        Self {
            passenger_vehicle: true,
            hgv: true,
            bus: true,
            pedestrian: false,
            bicycle: false,
            emergency: true,
        }
    }
}

/// Signposted destination for a transition through a connector.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Destination {
    pub id: FeatureId,
    pub connector_id: FeatureId,
    pub from_segment_id: FeatureId,
    pub to_segment_id: FeatureId,
    /// Localised destination names (e.g. {"en": "Airport", "nl": "Luchthaven"}).
    pub name_i18n: std::collections::HashMap<String, String>,
    /// Route numbers / shields referenced (e.g. ["A9", "S105"]).
    pub route_numbers: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::Coordinate;

    fn line() -> LineString {
        LineString::new(vec![Coordinate::new(0., 0.), Coordinate::new(1., 1.)]).unwrap()
    }

    #[test]
    fn segment_connector_refs_serialize() {
        let seg = Segment {
            id: FeatureId::new("seg-1"),
            geometry: line(),
            from_connector: FeatureId::new("cn-1"),
            to_connector: FeatureId::new("cn-2"),
            road_id: None,
            class: "residential".into(),
            lane_count: Some(1),
            max_speed_kmh: Some(50),
            one_way: false,
        };
        let json = serde_json::to_string(&seg).unwrap();
        let back: Segment = serde_json::from_str(&json).unwrap();
        assert_eq!(seg, back);
    }

    #[test]
    fn access_default_covers_common_classes() {
        let a = AccessProperties::default();
        assert!(a.passenger_vehicle);
        assert!(!a.pedestrian);
    }
}
