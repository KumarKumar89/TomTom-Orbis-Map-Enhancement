//! Property-based tests for the R-tree spatial index (README §12: proptest is
//! "extremely valuable" for generating pathological geometry inputs). Run at
//! higher case counts in the nightly lane via PROPTEST_CASES.

use geo_core::{BoundingBox, Coordinate};
use proptest::prelude::*;
use spatial_index::{Indexed, RTree};

#[derive(Clone, Debug, PartialEq)]
struct Pt {
    id: u64,
    c: Coordinate,
}

impl Indexed for Pt {
    fn bbox(&self) -> BoundingBox {
        BoundingBox::new(self.c.x, self.c.y, self.c.x, self.c.y)
    }
}

fn any_bbox() -> impl Strategy<Value = BoundingBox> {
    // Finite, modest magnitudes: NaN/inf would model corrupt data, not a bug.
    (-1e6f64..1e6f64, -1e6f64..1e6f64)
        .prop_map(|(a, b)| BoundingBox::new(a.min(b), b.min(a), a.max(b), b.max(a)))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(
        std::env::var("PROPTEST_CASES").ok().and_then(|v| v.parse().ok()).unwrap_or(256)
    ))]

    /// Every indexed point whose bbox intersects the query must be returned,
    /// and nothing else — for arbitrary point sets and arbitrary queries.
    #[test]
    fn bbox_query_is_exact_against_brute_force(
        points in prop::collection::vec((-1e5f64..1e5f64, -1e5f64..1e5f64), 0..400),
        query in any_bbox(),
    ) {
        let mut b = RTree::<Pt>::builder();
        let mut all = Vec::new();
        for (i, (x, y)) in points.iter().enumerate() {
            let p = Pt { id: i as u64, c: Coordinate::new(*x, *y) };
            b = b.add(p.clone());
            all.push(p);
        }
        let tree = b.build();
        let mut got: Vec<u64> = tree.query_bbox(query).into_iter().map(|p| p.id).collect();
        got.sort_unstable();
        let mut want: Vec<u64> = all.iter()
            .filter(|p| p.bbox().intersects(&query))
            .map(|p| p.id).collect();
        want.sort_unstable();
        prop_assert_eq!(got, want);
    }

    /// Bulk loading never loses or duplicates items.
    #[test]
    fn build_preserves_item_count(points in prop::collection::vec(
        (-1e5f64..1e5f64, -1e5f64..1e5f64), 0..500)) {
        let mut b = RTree::<Pt>::builder();
        for (i, (x, y)) in points.iter().enumerate() {
            b = b.add(Pt { id: i as u64, c: Coordinate::new(*x, *y) });
        }
        let tree = b.build();
        prop_assert_eq!(tree.len_items(), points.len());
    }

    /// nearest(k) returns k items sorted no worse than brute force on the
    /// same distance metric (bbox-center euclidean, matching the engine).
    #[test]
    fn nearest_matches_brute_force(
        points in prop::collection::vec((-1e4f64..1e4f64, -1e4f64..1e4f64), 1..200),
        target in (-1e4f64..1e4f64, -1e4f64..1e4f64),
        k in 1usize..10,
    ) {
        let mut b = RTree::<Pt>::builder();
        let mut all = Vec::new();
        for (i, (x, y)) in points.iter().enumerate() {
            let p = Pt { id: i as u64, c: Coordinate::new(*x, *y) };
            b = b.add(p.clone());
            all.push(p);
        }
        let tree = b.build();
        let t = Coordinate::new(target.0, target.1);
        let got = tree.nearest(t, k);
        prop_assert_eq!(got.len(), k.min(points.len()));
        // The got-set must be among the true k-nearest by distance.
        let dist = |p: &Pt| p.c.euclidean_distance(&t);
        let mut ranked: Vec<f64> = all.iter().map(dist).collect();
        ranked.sort_by(|a, c| a.partial_cmp(c).unwrap());
        let kth = ranked[k.min(ranked.len()) - 1];
        for p in &got {
            prop_assert!(dist(p) <= kth + 1e-9,
                "returned {:?} at {} but kth-nearest is {}", p.id, dist(p), kth);
        }
    }
}
