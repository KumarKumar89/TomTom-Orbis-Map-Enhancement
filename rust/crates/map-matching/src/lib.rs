//! `map-matching` — snap probe/GPS trajectories to the road network.
//!
//! Phase 1 implements a deterministic greedy matcher with route-continuity
//! verification: each observation is snapped to the nearest segment via the
//! spatial index, then consecutive matched segments must be connected in the
//! routing graph within a bounded detour factor. A production-grade Viterbi/HMM
//! matcher is planned behind the same API surface (see docs/algorithms).

use geo_core::{Coordinate, GeoError, GeoResult};
use geo_geometry::polyline::snap_to_segment;
use routing_graph::{CostModel, RoutingGraph};
use spatial_index::{Indexed, RTree};
use std::collections::HashMap;
use topology_engine::RoadNetwork;

#[derive(Debug, Clone, PartialEq)]
pub struct Observation {
    pub t: i64,
    pub coord: Coordinate,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchedPoint {
    pub observation: Observation,
    pub segment_id: geo_core::FeatureId,
    /// Snapped position on the road geometry.
    pub projected: Coordinate,
    /// Perpendicular offset distance from raw observation to road.
    pub offset: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MatchStep {
    Direct(MatchedPoint),
    /// Gap bridged by routing through intermediate segments.
    Bridged { via: Vec<geo_core::FeatureId>, to: MatchedPoint },
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchResult {
    pub steps: Vec<MatchStep>,
    pub total_offset: f64,
    pub unmatched: usize,
}

/// Detour factor above which we refuse to bridge two consecutive matches.
const MAX_DETOUR_FACTOR: f64 = 3.0;
/// Observations further than this (dataset units) from any road are rejected.
const MAX_SNAP_DISTANCE: f64 = 50.0;

struct SegmentBBox(geo_core::Segment);
impl Indexed for SegmentBBox {
    fn bbox(&self) -> geo_core::BoundingBox {
        self.0.geometry.bbox().union(&geo_core::BoundingBox::new(
            self.0.geometry.bbox().min_x - MAX_SNAP_DISTANCE,
            self.0.geometry.bbox().min_y - MAX_SNAP_DISTANCE,
            self.0.geometry.bbox().max_x + MAX_SNAP_DISTANCE,
            self.0.geometry.bbox().max_y + MAX_SNAP_DISTANCE,
        ))
    }
}

pub struct MapMatcher {
    net: RoadNetwork,
    index: RTree<SegmentBBox>,
    graph: RoutingGraph,
}

impl MapMatcher {
    pub fn new(net: RoadNetwork) -> Self {
        let mut b = RTree::<SegmentBBox>::builder();
        for seg in net.segments.values() {
            b = b.add(SegmentBBox(seg.clone()));
        }
        let graph = RoutingGraph::from_network(&net, CostModel::Distance);
        Self { net, index: b.build(), graph }
    }

    /// Nearest segment + projected point + offset for one observation.
    fn snap_one(&self, c: &Coordinate) -> Option<(geo_core::FeatureId, Coordinate, f64)> {
        // Query a box around the point, then test real perpendicular distance.
        let q = geo_core::BoundingBox::new(
            c.x - MAX_SNAP_DISTANCE,
            c.y - MAX_SNAP_DISTANCE,
            c.x + MAX_SNAP_DISTANCE,
            c.y + MAX_SNAP_DISTANCE,
        );
        let mut best: Option<(f64, geo_core::FeatureId, Coordinate)> = None;
        for sb in self.index.query_bbox(q) {
            let coords = &sb.0.geometry.coords;
            for w in coords.windows(2) {
                let (p, d) = snap_to_segment(c, &w[0], &w[1]);
                if best.as_ref().map_or(true, |(bd, _, _)| d < *bd) && d <= MAX_SNAP_DISTANCE {
                    best = Some((d, sb.0.id.clone(), p));
                }
            }
        }
        best.map(|(d, id, p)| (id, p, d))
    }

    pub fn match_trajectory(&self, obs: &[Observation]) -> GeoResult<MatchResult> {
        let mut steps: Vec<MatchStep> = vec![];
        let mut total_offset = 0.0;
        let mut unmatched = 0usize;
        let mut prev_seg: Option<geo_core::FeatureId> = None;

        for o in obs {
            let Some((seg_id, projected, offset)) = self.snap_one(&o.coord) else {
                unmatched += 1;
                continue;
            };
            total_offset += offset;

            if let Some(p) = &prev_seg {
                if p != &seg_id && !segments_adjacent(&self.net, p, &seg_id) {
                    // Try to bridge with a short route between the two segments'
                    // endpoints; reject implausible jumps.
                    let straight = projected.euclidean_distance(&steps.last().map(|s| match s {
                        MatchStep::Direct(m) | MatchStep::Bridged { to: m, .. } => m.projected,
                    }).unwrap());
                    if let Some((cost, path)) = self.bridge_route(p, &seg_id)? {
                        if straight > 0.0 && cost / straight > MAX_DETOUR_FACTOR {
                            unmatched += 1;
                            continue;
                        }
                        steps.push(MatchStep::Bridged { via: path, to: MatchedPoint {
                            observation: o.clone(), segment_id: seg_id.clone(), projected, offset,
                        }});
                        prev_seg = Some(seg_id);
                        continue;
                    } else {
                        unmatched += 1;
                        continue;
                    }
                }
            }
            steps.push(MatchStep::Direct(MatchedPoint {
                observation: o.clone(), segment_id: seg_id.clone(), projected, offset,
            }));
            prev_seg = Some(seg_id);
        }

        Ok(MatchResult { steps, total_offset, unmatched })
    }

    fn bridge_route(
        &self,
        from: &geo_core::FeatureId,
        to: &geo_core::FeatureId,
    ) -> GeoResult<Option<(f64, Vec<geo_core::FeatureId>)>> {
        let fs = self.net.segments.get(from).ok_or(GeoError::Validation("unknown segment"))?;
        let ts = self.net.segments.get(to).ok_or(GeoError::Validation("unknown segment"))?;
        // Attempt every connector pairing (two-way streets make all four plausible).
        let starts = [&fs.to_connector, &fs.from_connector];
        let ends = [&ts.from_connector, &ts.to_connector];
        let mut best: Option<(f64, Vec<geo_core::FeatureId>)> = None;
        for s in starts {
            for e in ends {
                if let Some(res @ (cost, _)) = self.graph.shortest_path(s, e)? {
                    if best.as_ref().map_or(true, |(bc, _)| cost < *bc) {
                        best = Some(res);
                    }
                }
            }
        }
        Ok(best)
    }
}

fn segments_adjacent(
    net: &RoadNetwork,
    a: &geo_core::FeatureId,
    b: &geo_core::FeatureId,
) -> bool {
    let (Some(sa), Some(sb)) = (net.segments.get(a), net.segments.get(b)) else {
        return false;
    };
    let shared = |x: &geo_core::FeatureId, y: &geo_core::FeatureId| x == y;
    shared(&sa.to_connector, &sb.from_connector)
        || shared(&sa.to_connector, &sb.to_connector)
        || shared(&sa.from_connector, &sb.from_connector)
        || shared(&sa.from_connector, &sb.to_connector)
}

/// Sum of squared offsets — used as a quality score in regression tests.
pub fn match_quality(res: &MatchResult) -> f64 {
    res.steps
        .iter()
        .map(|s| match s {
            MatchStep::Direct(m) | MatchStep::Bridged { to: m, .. } => m.offset * m.offset,
        })
        .sum()
}

// Keep HashMap import meaningful for future multi-hypothesis scoring.
#[allow(dead_code)]
type Hypotheses = HashMap<usize, Vec<f64>>;

#[cfg(test)]
mod tests {
    use super::*;
    use geo_core::primitives::LineString;

    fn net_straight_two() -> RoadNetwork {
        let mut n = RoadNetwork::new();
        let g1 = LineString::new(vec![Coordinate::new(0., 0.), Coordinate::new(10., 0.)]).unwrap();
        let g2 = LineString::new(vec![Coordinate::new(10., 0.), Coordinate::new(20., 0.)]).unwrap();
        n.add_segment(geo_core::Segment {
            id: geo_core::FeatureId::new("s1"),
            geometry: g1,
            from_connector: geo_core::FeatureId::new("c0"),
            to_connector: geo_core::FeatureId::new("c1"),
            road_id: None, class: "primary".into(), lane_count: None, max_speed_kmh: None, one_way: false,
        });
        n.add_segment(geo_core::Segment {
            id: geo_core::FeatureId::new("s2"),
            geometry: g2,
            from_connector: geo_core::FeatureId::new("c1"),
            to_connector: geo_core::FeatureId::new("c2"),
            road_id: None, class: "primary".into(), lane_count: None, max_speed_kmh: None, one_way: false,
        });
        n
    }

    #[test]
    fn snaps_points_on_a_line() {
        let m = MapMatcher::new(net_straight_two());
        let obs = vec![
            Observation { t: 0, coord: Coordinate::new(2.0, 0.3) },
            Observation { t: 1, coord: Coordinate::new(12.0, -0.2) },
        ];
        let r = m.match_trajectory(&obs).unwrap();
        assert_eq!(r.steps.len(), 2);
        assert_eq!(r.unmatched, 0);
        let ids: Vec<String> = r.steps.iter().map(|s| match s {
            MatchStep::Direct(mm) | MatchStep::Bridged { to: mm, .. } => mm.segment_id.to_string(),
        }).collect();
        assert_eq!(ids, vec!["s1", "s2"]);
    }

    #[test]
    fn rejects_far_off_network_points() {
        let m = MapMatcher::new(net_straight_two());
        let obs = vec![Observation { t: 0, coord: Coordinate::new(5.0, 900.0) }];
        let r = m.match_trajectory(&obs).unwrap();
        assert_eq!(r.unmatched, 1);
        assert!(r.steps.is_empty());
    }

    #[test]
    fn quality_score_grows_with_noise() {
        let m = MapMatcher::new(net_straight_two());
        let clean = m.match_trajectory(&[Observation { t: 0, coord: Coordinate::new(5., 0.01) }]).unwrap();
        let noisy = m.match_trajectory(&[Observation { t: 0, coord: Coordinate::new(5., 5.0) }]).unwrap();
        assert!(match_quality(&clean) < match_quality(&noisy));
    }
}
