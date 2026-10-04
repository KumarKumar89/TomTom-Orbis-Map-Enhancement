//! `spatial-index` — an STR (sort-tile-recursive) packed R-tree.
//!
//! Built once per dataset version; candidate search in the conflation engine and
//! map-matching both depend on it to stay near O(log N + k) per query instead of
//! O(N). Bulk loading via STR packing is deterministic for a given input set,
//! which keeps CI regression tests reproducible.

use geo_core::BoundingBox;

/// A record that can be indexed by its bounding box.
pub trait Indexed {
    fn bbox(&self) -> BoundingBox;
}

struct Node<T> {
    bbox: BoundingBox,
    children: Vec<Node<T>>,
    items: Vec<(BoundingBox, T)>, // leaf payloads only
}

impl<T: Clone> Clone for Node<T> {
    fn clone(&self) -> Self {
        Node { bbox: self.bbox, children: self.children.clone(), items: self.items.clone() }
    }
}

impl<T> Node<T> {
    fn leaf(items: Vec<(BoundingBox, T)>) -> Self {
        let bbox = items
            .iter()
            .map(|(b, _)| *b)
            .reduce(|a, b| a.union(&b))
            // Leaves are never built empty in this implementation.
            .unwrap_or(BoundingBox::new(0., 0., 0., 0.));
        Node { bbox, children: vec![], items }
    }

    fn internal(children: Vec<Node<T>>) -> Self {
        let bbox = children
            .iter()
            .map(|c| c.bbox)
            .reduce(|a, b| a.union(&b))
            .expect("internal node needs children");
        Node { bbox, children, items: vec![] }
    }
}

/// Bulk-loaded, immutable 2D R-tree.
pub struct RTree<T> {
    root: Option<Node<T>>,
    leaf_capacity: usize,
}

impl<T: Indexed + Clone + std::fmt::Debug> RTree<T> {
    pub fn builder() -> RTreeBuilder<T> {
        RTreeBuilder { items: vec![], leaf_capacity: 16 }
    }
}

pub struct RTreeBuilder<T> {
    items: Vec<(BoundingBox, T)>,
    leaf_capacity: usize,
}

impl<T: Indexed + Clone + std::fmt::Debug> RTreeBuilder<T> {
    pub fn leaf_capacity(mut self, n: usize) -> Self {
        assert!(n >= 2, "leaf capacity must be >= 2");
        self.leaf_capacity = n;
        self
    }

    pub fn add(mut self, item: T) -> Self {
        let b = item.bbox();
        self.items.push((b, item));
        self
    }

    /// STR bulk load: sort by x into vertical strips, sort each strip by y.
    pub fn build(self) -> RTree<T> {
        let cap = self.leaf_capacity;
        let mut items = self.items;
        if items.is_empty() {
            return RTree { root: None, leaf_capacity: cap };
        }
        // Sort into vertical slices, then leaves by y within slice. Deterministic
        // tie-breakers keep the tree byte-stable across runs/platforms.
        items.sort_by(|a, b| {
            a.0.min_x
                .partial_cmp(&b.0.min_x)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.0.min_y.partial_cmp(&b.0.min_y).unwrap_or(std::cmp::Ordering::Equal))
        });
        let num_leaves = items.len().div_ceil(cap);
        let slice_size = (num_leaves as f64).sqrt().ceil() as usize;
        let mut leaves = Vec::with_capacity(num_leaves);
        for slice in items.chunks(slice_size * cap) {
            let mut s: Vec<_> = slice.to_vec();
            s.sort_by(|a, b| {
                a.0.min_y
                    .partial_cmp(&b.0.min_y)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(a.0.min_x.partial_cmp(&b.0.min_x).unwrap_or(std::cmp::Ordering::Equal))
            });
            leaves.extend(s.chunks(cap).map(|chunk| Node::leaf(chunk.to_vec())));
        }
        // Pack leaves bottom-up until one root remains.
        while leaves.len() > 1 {
            leaves = leaves
                .chunks(cap)
                .map(|group| Node::internal(group.to_vec()))
                .collect();
        }
        RTree { root: Some(leaves.pop().unwrap()), leaf_capacity: cap }
    }
}

impl<T: Clone + std::fmt::Debug> RTree<T> {
    pub fn len_items(&self) -> usize {
        fn count<T>(n: &Node<T>) -> usize {
            n.items.len() + n.children.iter().map(count).sum::<usize>()
        }
        self.root.as_ref().map(count).unwrap_or(0)
    }

    /// All items whose stored bbox intersects `query`.
    pub fn query_bbox(&self, query: BoundingBox) -> Vec<T> {
        let mut out = vec![];
        if let Some(root) = &self.root {
            collect(root, query, &mut out);
        }
        out
    }

    /// k nearest items by center distance (priority-first traversal, exact for
    /// our small-to-medium datasets; simple enough to stay verifiable).
    pub fn nearest(&self, point: geo_core::Coordinate, k: usize) -> Vec<T> {
        let mut cand: Vec<(f64, T)> = vec![];
        if let Some(root) = &self.root {
            nearest_walk(root, &point, k, &mut cand);
            cand.sort_by(|a, b| {
                a.0.partial_cmp(&b.0)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| format!("{:?}", a.1).cmp(&format!("{:?}", b.1)))
            });
        }
        cand.truncate(k);
        cand.into_iter().map(|(_, t)| t).collect()
    }
}

fn collect<T: Clone>(node: &Node<T>, query: BoundingBox, out: &mut Vec<T>) {
    if !node.bbox.intersects(&query) {
        return;
    }
    for (b, item) in &node.items {
        if b.intersects(&query) {
            out.push(item.clone());
        }
    }
    for c in &node.children {
        collect(c, query, out);
    }
}

fn nearest_walk<T: Clone>(
    node: &Node<T>,
    p: &geo_core::Coordinate,
    k: usize,
    cand: &mut Vec<(f64, T)>,
) {
    // Prune when we already have k candidates all closer than min possible dist.
    if cand.len() == k {
        let worst = cand.iter().map(|(d, _)| *d).fold(0.0f64, f64::max);
        if bbox_min_distance(&node.bbox, p) > worst {
            return;
        }
    }
    for (b, item) in &node.items {
        let d = bbox_center(b).euclidean_distance(p);
        if cand.len() < k {
            cand.push((d, item.clone()));
        } else {
            cand.sort_by(|a, b2| a.0.partial_cmp(&b2.0).unwrap_or(std::cmp::Ordering::Equal));
            if d < cand.last().map(|x| x.0).unwrap_or(f64::MAX) {
                cand.pop();
                cand.push((d, item.clone()));
            }
        }
    }
    // Visit nearer child subtrees first for better pruning.
    let mut kids: Vec<_> = node.children.iter().collect();
    kids.sort_by(|a, b| {
        bbox_min_distance(&a.bbox, p)
            .partial_cmp(&bbox_min_distance(&b.bbox, p))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for c in kids {
        nearest_walk(c, p, k, cand);
    }
}

fn bbox_center(b: &BoundingBox) -> geo_core::Coordinate {
    geo_core::Coordinate::new((b.min_x + b.max_x) / 2.0, (b.min_y + b.max_y) / 2.0)
}

fn bbox_min_distance(b: &BoundingBox, p: &geo_core::Coordinate) -> f64 {
    let dx = (b.min_x - p.x).max(p.x - b.max_x).max(0.0);
    let dy = (b.min_y - p.y).max(p.y - b.max_y).max(0.0);
    (dx * dx + dy * dy).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_core::Coordinate;

    #[derive(Clone, Debug, PartialEq)]
    struct Pt {
        id: u32,
        c: Coordinate,
    }
    impl Indexed for Pt {
        fn bbox(&self) -> BoundingBox {
            BoundingBox::new(self.c.x, self.c.y, self.c.x, self.c.y)
        }
    }

    fn grid(n: u32) -> RTree<Pt> {
        let mut b = RTree::<Pt>::builder();
        for i in 0..n {
            for j in 0..n {
                b = b.add(Pt { id: i * n + j, c: Coordinate::new(i as f64, j as f64) });
            }
        }
        b.build()
    }

    #[test]
    fn empty_tree_queries_cleanly() {
        let t = RTree::<Pt>::builder().build();
        assert_eq!(t.len_items(), 0);
        assert!(t.query_bbox(BoundingBox::new(-1., -1., 1., 1.)).is_empty());
        assert!(t.nearest(Coordinate::new(0., 0.), 5).is_empty());
    }

    #[test]
    fn item_count_survives_packing() {
        let t = grid(10);
        assert_eq!(t.len_items(), 100);
    }

    #[test]
    fn bbox_query_matches_brute_force() {
        let t = grid(20);
        let q = BoundingBox::new(3.5, 3.5, 9.5, 9.5);
        let mut got: Vec<u32> = t.query_bbox(q).into_iter().map(|p| p.id).collect();
        got.sort();
        let mut want: Vec<u32> = (0..20u32)
            .flat_map(|i| (0..20u32).map(move |j| (i, j)))
            .filter(|(i, j)| (3.5..=9.5).contains(&(*i as f64)) && (3.5..=9.5).contains(&(*j as f64)))
            .map(|(i, j)| i * 20 + j)
            .collect();
        want.sort();
        assert_eq!(got, want);
    }

    #[test]
    fn nearest_returns_closest_points() {
        let t = grid(10);
        let near = t.nearest(Coordinate::new(4.9, 5.1), 3);
        let ids: Vec<u32> = near.iter().map(|p| p.id).collect();
        // Points (5,5)=55 and (5,4)? note id = i*n + j with x=i,y=j => (5,5)->55, (4,5)->45? careful: id=i*10+j where c=(i,j). (5,5)->55, (5,4)->54? no: c=(i as f64, j as f64). Distance from (4.9,5.1): (5,5)->~0.14, (4,5)->1.1, (5,6) invalid (grid 0..9 ok (5,6)->~1.08). So closest is id 55.
        assert!(ids.contains(&55));
        assert_eq!(ids.len(), 3);
    }
}
