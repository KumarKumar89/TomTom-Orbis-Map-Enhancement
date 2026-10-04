//! `topology-engine` — segment ↔ connector consistency and road-graph invariants.
//!
//! Overture models connectors as the physical connection points between segments;
//! that relationship is the natural place to enforce network validity. The
//! quality engine runs these checks on every release candidate, and CI fails the
//! build when any error-severity invariant is violated.

use geo_core::{
    Connector, FeatureId, GeoError, GeoResult, Restriction, Segment,
};
use std::collections::{HashMap, HashSet};

/// Severity of a topology finding. Only `Error` findings gate releases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
    pub severity: Severity,
    pub code: &'static str,
    pub message: String,
    pub feature_ids: Vec<FeatureId>,
}

impl Finding {
    fn error(code: &'static str, message: impl Into<String>, ids: Vec<FeatureId>) -> Self {
        Self { severity: Severity::Error, code, message: message.into(), feature_ids: ids }
    }
    fn warning(code: &'static str, message: impl Into<String>, ids: Vec<FeatureId>) -> Self {
        Self { severity: Severity::Warning, code, message: message.into(), feature_ids: ids }
    }
}

/// An in-memory directed road network assembled from Overture-style primitives.
#[derive(Default, Clone)]
pub struct RoadNetwork {
    pub segments: HashMap<FeatureId, Segment>,
    pub connectors: HashMap<FeatureId, Connector>,
    pub restrictions: Vec<Restriction>,
}

impl RoadNetwork {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_segment(&mut self, s: Segment) {
        self.connectors
            .entry(s.from_connector.clone())
            .or_insert_with(|| Connector {
                id: s.from_connector.clone(),
                segment_ids: vec![],
                is_dead_end: false,
                sub_type: geo_core::transportation::ConnectorSubType::Junction,
            })
            .segment_ids
            .push(s.id.clone());
        self.connectors
            .entry(s.to_connector.clone())
            .or_insert_with(|| Connector {
                id: s.to_connector.clone(),
                segment_ids: vec![],
                is_dead_end: false,
                sub_type: geo_core::transportation::ConnectorSubType::Junction,
            })
            .segment_ids
            .push(s.id.clone());
        self.segments.insert(s.id.clone(), s);
    }

    /// Register an explicit connector (e.g. dead-end or interchange metadata).
    pub fn add_connector(&mut self, c: Connector) {
        self.connectors.insert(c.id.clone(), c);
    }

    pub fn add_restriction(&mut self, r: Restriction) {
        self.restrictions.push(r);
    }

    /// Run all structural invariants and return findings sorted deterministically.
    pub fn validate(&self) -> GeoResult<Vec<Finding>> {
        let mut findings = vec![];

        // INV-1: dangling references — segment endpoints must exist.
        for (id, seg) in &self.segments {
            for cn in [&seg.from_connector, &seg.to_connector] {
                if !self.connectors.contains_key(cn) {
                    findings.push(Finding::error(
                        "TOPO-001-dangling-connector-ref",
                        format!("segment {id} references missing connector {cn}"),
                        vec![id.clone()],
                    ));
                }
            }
        }

        // INV-2: orphan connectors — must reference at least one segment.
        for (id, cn) in &self.connectors {
            let live: Vec<_> = cn
                .segment_ids
                .iter()
                .filter(|s| self.segments.contains_key(*s))
                .collect();
            if live.is_empty() {
                findings.push(Finding::error(
                    "TOPO-002-orphan-connector",
                    format!("connector {id} has no attached segments"),
                    vec![id.clone()],
                ));
            } else if live.len() == 1 && !cn.is_dead_end {
                // A single attached segment that is not declared a dead end is
                // almost certainly a missing junction.
                findings.push(Finding::warning(
                    "TOPO-003-undeclared-dead-end",
                    format!("connector {id} attaches exactly one segment but is not marked dead-end"),
                    vec![id.clone()],
                ));
            }
        }

        // INV-3: bidirectional consistency — connector.segment_ids must agree
        // with segment.from/to declarations.
        for (cid, cn) in &self.connectors {
            for sid in &cn.segment_ids {
                if let Some(seg) = self.segments.get(sid) {
                    if &seg.from_connector != cid && &seg.to_connector != cid {
                        findings.push(Finding::error(
                            "TOPO-004-connector-list-mismatch",
                            format!("connector {cid} lists segment {sid} which does not reference it"),
                            vec![cid.clone(), sid.clone()],
                        ));
                    }
                }
            }
        }

        // INV-4: restrictions must point at existing features, and transitions
        // must be traversable through their connector ("no impossible turn").
        for r in &self.restrictions {
            if let Some(s) = &r.applies_to_segment {
                if !self.segments.contains_key(s) {
                    findings.push(Finding::error(
                        "TOPO-005-restriction-unknown-segment",
                        format!("restriction {} targets unknown segment {s}", r.id),
                        vec![s.clone()],
                    ));
                }
            }
            if let Some((from, to)) = &r.transition {
                let (Some(fs), Some(ts)) = (self.segments.get(from), self.segments.get(to)) else {
                    findings.push(Finding::error(
                        "TOPO-006-transition-unknown-segment",
                        format!("restriction transition ({from} -> {to}) references unknown segments"),
                        vec![from.clone(), to.clone()],
                    ));
                    continue;
                };
                // The turn is only physically possible if fs.to_connector == ts.from_connector
                // (or the reverse traversal for two-way streets).
                let shared = fs.to_connector == ts.from_connector
                    || fs.to_connector == ts.to_connector
                    || fs.from_connector == ts.from_connector
                    || fs.from_connector == ts.to_connector;
                if !shared {
                    findings.push(Finding::error(
                        "TOPO-007-impossible-turn",
                        format!("restriction transition ({from} -> {to}) shares no connector"),
                        vec![from.clone(), to.clone()],
                    ));
                }
            }
        }

        // INV-8: duplicate canonical ids are impossible by construction here
        // (HashMap), but duplicated geometry endpoints flag double digitization.
        let mut endpoint_seen: HashMap<(String, String), &FeatureId> = HashMap::new();
        let mut ids: Vec<_> = self.segments.keys().cloned().collect();
        ids.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        for id in &ids {
            let seg = &self.segments[id];
            let key = (seg.from_connector.to_string(), seg.to_connector.to_string());
            let rkey = (seg.to_connector.to_string(), seg.from_connector.to_string());
            if let Some(prev) = endpoint_seen.get(&key).or_else(|| endpoint_seen.get(&rkey)) {
                findings.push(Finding::warning(
                    "TOPO-008-duplicate-candidate",
                    format!("segments {prev} and {id} connect the same connector pair"),
                    vec![(*prev).clone(), id.clone()],
                ));
            } else {
                endpoint_seen.insert(key, id);
            }
        }

        findings.sort_by(|a, b| {
            a.code.cmp(b.code).then_with(|| {
                let ka: Vec<_> = a.feature_ids.iter().map(|f| f.as_str()).collect();
                let kb: Vec<_> = b.feature_ids.iter().map(|f| f.as_str()).collect();
                ka.cmp(&kb)
            })
        });
        Ok(findings)
    }

    /// Reachability check: can we drive from any node to `target`? Returns the
    /// set of segments unreachable *from* `start_id` following travel direction.
    pub fn unreachable_segments(&self, start_id: &FeatureId) -> GeoResult<HashSet<FeatureId>> {
        let start = self
            .segments
            .get(start_id)
            .ok_or_else(|| GeoError::TopologyViolation(format!("unknown start segment {start_id}")))?;
        let mut visited: HashSet<FeatureId> = HashSet::new();
        let mut stack = vec![start.to_connector.clone()];
        while let Some(conn) = stack.pop() {
            if !visited.insert(conn.clone()) {
                continue;
            }
            for (sid, seg) in &self.segments {
                if &seg.from_connector == &conn {
                    stack.push(seg.to_connector.clone());
                    let _ = sid;
                }
            }
        }
        Ok(self
            .segments
            .keys()
            .filter(|s| {
                let seg = &self.segments[*s];
                !visited.contains(&seg.from_connector) && !visited.contains(&seg.to_connector)
            })
            .cloned()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_core::primitives::{Coordinate, LineString};
    use geo_core::transportation::ConnectorSubType;

    fn line() -> LineString {
        LineString::new(vec![Coordinate::new(0., 0.), Coordinate::new(1., 1.)]).unwrap()
    }

    fn seg(id: &str, from: &str, to: &str) -> Segment {
        Segment {
            id: FeatureId::new(id),
            geometry: line(),
            from_connector: FeatureId::new(from),
            to_connector: FeatureId::new(to),
            road_id: None,
            class: "residential".into(),
            lane_count: None,
            max_speed_kmh: None,
            one_way: false,
        }
    }

    #[test]
    fn clean_network_has_no_findings() {
        let mut n = RoadNetwork::new();
        n.add_segment(seg("s1", "c1", "c2"));
        n.add_segment(seg("s2", "c2", "c3"));
        // c2 has 2 segments; c1/c3 have 1 each -> mark as dead ends.
        for cid in ["c1", "c3"] {
            n.add_connector(Connector {
                id: FeatureId::new(cid),
                segment_ids: vec![FeatureId::new(if cid == "c1" { "s1" } else { "s2" })],
                is_dead_end: true,
                sub_type: ConnectorSubType::Junction,
            });
        }
        let f = n.validate().unwrap();
        assert_eq!(
            f.iter().map(|x| x.code).collect::<Vec<_>>(),
            Vec::<&str>::new(),
            "unexpected findings: {f:?}"
        );
    }

    #[test]
    fn detects_orphan_connector() {
        let mut n = RoadNetwork::new();
        n.add_segment(seg("s1", "c1", "c2"));
        n.add_connector(Connector {
            id: FeatureId::new("ghost"),
            segment_ids: vec![],
            is_dead_end: false,
            sub_type: ConnectorSubType::Junction,
        });
        let codes: Vec<_> = n.validate().unwrap().into_iter().map(|f| f.code).collect();
        assert!(codes.contains(&"TOPO-002-orphan-connector"));
    }

    #[test]
    fn detects_impossible_turn() {
        let mut n = RoadNetwork::new();
        n.add_segment(seg("s1", "c1", "c2"));
        n.add_segment(seg("s2", "c3", "c4")); // disconnected from s1
        n.add_restriction(Restriction {
            id: FeatureId::new("r1"),
            applies_to_segment: None,
            applies_to_connector: None,
            transition: Some((FeatureId::new("s1"), FeatureId::new("s2"))),
            kind: geo_core::transportation::RestrictionKind::Prohibited,
            access: Default::default(),
            when: None,
        });
        let codes: Vec<_> = n.validate().unwrap().into_iter().map(|f| f.code).collect();
        assert!(codes.contains(&"TOPO-007-impossible-turn"));
    }

    #[test]
    fn reachability_finds_disconnected_island() {
        let mut n = RoadNetwork::new();
        n.add_segment(seg("s1", "c1", "c2"));
        n.add_segment(seg("s2", "c2", "c3"));
        n.add_segment(seg("island", "i1", "i2"));
        let un = n.unreachable_segments(&FeatureId::new("s1")).unwrap();
        assert!(un.contains(&FeatureId::new("island")));
        assert!(!un.contains(&FeatureId::new("s2")));
    }

    #[test]
    fn unknown_start_errors() {
        let n = RoadNetwork::new();
        assert!(n.unreachable_segments(&FeatureId::new("nope")).is_err());
    }
}
