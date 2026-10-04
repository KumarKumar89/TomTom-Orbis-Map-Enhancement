use criterion::{black_box, criterion_group, criterion_main, Criterion};
use geo_core::{BoundingBox, Coordinate};
use spatial_index::{Indexed, RTree};

#[derive(Clone, Debug)]
struct Pt {
    c: Coordinate,
}
impl Indexed for Pt {
    fn bbox(&self) -> BoundingBox {
        BoundingBox::new(self.c.x, self.c.y, self.c.x, self.c.y)
    }
}

fn deterministic_lcg(seed: u64, n: usize) -> RTree<Pt> {
    let mut state = seed;
    let mut b = RTree::<Pt>::builder();
    for _ in 0..n {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let x = (state >> 33) as f64 / (1u64 << 31) as f64 * 1000.0;
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let y = (state >> 33) as f64 / (1u64 << 31) as f64 * 1000.0;
        b = b.add(Pt { c: Coordinate::new(x, y) });
    }
    b.build()
}

fn bench_build(c: &mut Criterion) {
    c.bench_function("rtree_build_10k", |bencher| {
        bencher.iter(|| black_box(deterministic_lcg(42, 10_000)));
    });
}

fn bench_query(c: &mut Criterion) {
    let tree = deterministic_lcg(42, 100_000);
    c.bench_function("rtree_bbox_query_100k", |bencher| {
        bencher.iter(|| {
            let hits = tree.query_bbox(black_box(BoundingBox::new(100., 100., 300., 300.)));
            hits.len()
        });
    });
    c.bench_function("rtree_nearest_100k_k10", |bencher| {
        bencher.iter(|| tree.nearest(black_box(Coordinate::new(500., 500.)), 10).len());
    });
}

criterion_group!(benches, bench_build, bench_query);
criterion_main!(benches);
