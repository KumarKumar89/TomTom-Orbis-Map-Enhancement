//! Reusable emitter helper for Rust quality-engine tests.
//!
//! Copy into a crate's `src/bin/kpi_probe.rs` or inline the macro below in
//! evaluation code. Every measurement prints one machine-parseable line:
//!
//! ```text
//! KPI KPI-026 97.4
//! ```
//!
//! which `kpi/tools/collect_kpis.py --stdin` ingests from `cargo test -- --nocapture`.
//!
//! ```ignore
//! // usage inside an evaluation test:
//! let precision = evaluate_conflation_precision(&golden_set);
//! kpi!("KPI-026", precision * 100.0);
//! assert!(precision >= 0.95, "conflation precision below warn threshold");
//! ```
#[macro_export]
macro_rules! kpi {
    ($id:expr, $value:expr) => {{
        let v: f64 = $value as f64;
        println!("KPI {} {}", $id, v);
    }};
}
