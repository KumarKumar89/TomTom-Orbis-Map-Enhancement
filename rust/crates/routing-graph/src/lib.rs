//! `routing-graph` — graph algorithms over the road network.
//!
//! Builds a directed multigraph from [`topology_engine::RoadNetwork`] connectors
//! (nodes) and segments (edges) and provides shortest-path searches used by the
//! quality engine's *route continuity* regression tests: a routing result that
//! changes across map versions without a corresponding data change is a defect.

use geo_core::{FeatureId, GeoResult};
use std::collections::{BinaryHeap, HashMap, HashSet};
use topology_engine::RoadNetwork;

/// Directed edge weight kinds used for cost models.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CostModel {
    /// Physical length only.
    Distance,
    /// Length penalized for class (motorway preferred).
    FastTripProxy,
}

#[derive(Clone)]
struct Edge {
    to: FeatureId,
    segment: FeatureId,
    length: f64,
}

pub struct RoutingGraph {
    adj: HashMap<FeatureId, Vec<Edge>>,
}

impl RoutingGraph {
    pub fn from_network(net: &RoadNetwork, model: CostModel) -> Self {
        let mut adj: HashMap<FeatureId, Vec<Edge>> = HashMap::new();
        for seg in net.segments.values() {
            let len = seg.geometry.length();
            let cost = match model {
                CostModel::Distance => len,
                CostModel::FastTripProxy => {
                    let factor = match seg.class.as_str() {
                        "motorway" | "trunk" => 0.5,
                        "primary" | "secondary" => 0.8,
                        _ => 1.0,
                    };
                    len * factor
                }
            };
            // Two-way streets get both directions.
            adj.entry(seg.from_connector.clone())
                .or_default()
                .push(Edge { to: seg.to_connector.clone(), segment: seg.id.clone(), length: cost });
            if !seg.one_way {
                adj.entry(seg.to_connector.clone())
                    .or_default()
                    .push(Edge { to: seg.from_connector.clone(), segment: seg.id.clone(), length: cost });
            }
        }
        Self { adj }
    }

    /// Dijkstra from `start` connector to `goal` connector. Returns
    /// (total_cost, ordered segment ids). Deterministic tie-breaking on node id.
    pub fn shortest_path(&self, start: &FeatureId, goal: &FeatureId) -> GeoResult<Option<(f64, Vec<FeatureId>)>> {
        if start == goal {
            return Ok(Some((0.0, vec![])));
        }
        #[derive(PartialEq)]
        struct State {
            cost: f64,
            node: FeatureId,
        }
        impl Eq for State {}
        impl Ord for State {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                // Reversed because BinaryHeap is a max-heap.
                other.cost.partial_cmp(&self.cost).unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| self.node.cmp(&other.node))
            }
        }
        impl PartialOrd for State {
            fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(o))
            }
        }

        let mut dist: HashMap<FeatureId, f64> = HashMap::new();
        let mut prev: HashMap<FeatureId, (FeatureId, FeatureId)> = HashMap::new();
        let mut heap = BinaryHeap::new();
        dist.insert(start.clone(), 0.0);
        heap.push(State { cost: 0.0, node: start.clone() });

        while let Some(State { cost, node }) = heap.pop() {
            if &node == goal {
                let mut path = vec![];
                let mut cur = node;
                while let Some((p, seg)) = prev.remove(&cur) {
                    path.push(seg);
                    cur = p;
                }
                path.reverse();
                return Ok(Some((cost, path)));
            }
            if cost > dist.get(&node).copied().unwrap_or(f64::INFINITY) {
                continue;
            }
            if let Some(edges) = self.adj.get(&node) {
                for e in edges {
                    let nc = cost + e.length;
                    if nc < dist.get(&e.to).copied().unwrap_or(f64::INFINITY) {
                        dist.insert(e.to.clone(), nc);
                        prev.insert(e.to.clone(), (node.clone(), e.segment.clone()));
                        heap.push(State { cost: nc, node: e.to.clone() });
                    }
                }
            }
        }
        Ok(None)
    }

    /// Set of connector ids reachable from `start` (connectivity diagnostics).
    ///
    /// Returns owned ids so callers are not entangled with the graph borrow
    /// lifetime (and so `HashMap` iteration order never leaks into results).
    pub fn reachable_from(&self, start: &FeatureId) -> HashSet<FeatureId> {
        let mut seen: HashSet<FeatureId> = HashSet::new();
        let mut stack: Vec<FeatureId> = vec![start.clone()];
        while let Some(n) = stack.pop() {
            if !seen.insert(n.clone()) {
                continue;
            }
            if let Some(edges) = self.adj.get(&n) {
                for e in edges {
                    if !seen.contains(&e.to) {
                        stack.push(e.to.clone());
                    }
                }
            }
        }
        seen
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_core::primitives::{Coordinate, LineString};

    fn seg(id: &str, from: &str, to: &str, one_way: bool, class: &str) -> geo_core::Segment {
        geo_core::Segment {
            id: FeatureId::new(id),
            geometry: LineString::new(vec![Coordinate::new(0., 0.), Coordinate::new(1., 0.)]).unwrap(),
            from_connector: FeatureId::new(from),
            to_connector: FeatureId::new(to),
            road_id: None,
            class: class.into(),
            lane_count: None,
            max_speed_kmh: None,
            one_way,
        }
    }

    #[test]
    fn finds_simple_route() {
        let mut net = RoadNetwork::new();
        net.add_segment(seg("s1", "c1", "c2", false, "residential"));
        net.add_segment(seg("s2", "c2", "c3", false, "residential"));
        let g = RoutingGraph::from_network(&net, CostModel::Distance);
        let (cost, path) = g
            .shortest_path(&FeatureId::new("c1"), &FeatureId::new("c3"))
            .unwrap()
            .expect("path exists");
        assert!((cost - 2.0).abs() < 1e-9);
        assert_eq!(path.len(), 2);
    }

    #[test]
    fn respects_one_way_directions() {
        let mut net = RoadNetwork::new();
        net.add_segment(seg("s1", "c1", "c2", true, "primary"));
        let g = RoutingGraph::from_network(&net, CostModel::Distance);
        assert!(g.shortest_path(&FeatureId::new("c1"), &FeatureId::new("c2")).unwrap().is_some());
        assert!(g.shortest_path(&FeatureId::new("c2"), &FeatureId::new("c1")).unwrap().is_none());
    }

    #[test]
    fn fast_trip_prefers_motorway() {
        let mut net = RoadNetwork::new();
        // Direct residential route length 2 vs motorway detour length 3.
        net.add_segment(seg("r1", "a", "b", false, "residential"));
        net.add_segment(seg("r2", "b", "z", false, "residential"));
        let mut m = seg("m1", "a", "y", false, "motorway");
        m.geometry = LineString::new(vec![Coordinate::new(0., 0.), Coordinate::new(1.5, 0.)]).unwrap();
        let mut m2 = seg("m2", "y", "z", false, "motorway");
        m2.geometry = LineString::new(vec![Coordinate::new(0., 0.), Coordinate::new(1.5, 0.)]).unwrap();
        net.add_segment(m);
        net.add_segment(m2);
        let g = RoutingGraph::from_network(&net, CostModel::FastTripProxy);
        let (_, path) = g
            .shortest_path(&FeatureId::new("a"), &FeatureId::new("z"))
            .unwrap()
            .expect("path exists");
        assert_eq!(path, vec![FeatureId::new("m1"), FeatureId::new("m2")]);
    }

    #[test]
    fn reachable_set_is_owned_and_correct() {
        let mut net = RoadNetwork::new();
        net.add_segment(seg("s1", "c1", "c2", true, "primary"));
        net.add_segment(seg("s2", "c2", "c3", false, "primary"));
        let g = RoutingGraph::from_network(&net, CostModel::Distance);
        let reach = g.reachable_from(&FeatureId::new("c1"));
        assert!(reach.contains(&FeatureId::new("c1")));
        assert!(reach.contains(&FeatureId::new("c2")));
        assert!(reach.contains(&FeatureId::new("c3")));
        // Reverse direction must NOT be reachable (one-way s1).
        let back = g.reachable_from(&FeatureId::new("c3"));
        assert!(!back.contains(&FeatureId::new("c1")));
    }
}
