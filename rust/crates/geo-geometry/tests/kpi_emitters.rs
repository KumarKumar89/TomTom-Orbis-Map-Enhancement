//! CI/CD KPI emitters — geospatial regression measurements (README §14).
//!
//! These tests compute real metrics from the implemented engines and print
//! machine-parseable `KPI <id> <value>` lines, which
//! `kpi/tools/collect_kpis.py --stdin` ingests from the CI test run. Values
//! are deterministic (fixed seeds / fixed inputs) so trends across runs are
//! comparable and threshold breaches are always genuine regressions.
//!
//! KPI ids referenced here must exist in `kpi/definitions/kpis.yaml`
//! (the collector drops unknown ids and the Python contract tests assert
//! definitions stay coherent).

use geo_core::Coordinate;
use geo_geometry::projection::{lonlat_from_web_mercator, web_mercator_from_lonlat};

/// Print one measurement line consumed by `collect_kpis.py`.
macro_rules! kpi {
    ($id:expr, $value:expr) => {{
        let v: f64 = $value as f64;
        println!("KPI {} {}", $id, v);
    }};
}

/// Deterministic LCG so every CI run measures the same dataset.
fn lcg_points(seed: u64, n: usize) -> Vec<(f64, f64)> {
    let mut state = seed;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let x = (state >> 33) as f64 / (1u64 << 31) as f64 * 360.0 - 180.0;
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let y = (state >> 33) as f64 / (1u64 << 31) as f64 * 154.0 - 77.0; // within clamp range
        out.push((x, y));
    }
    out
}

/// KPI-003 Projection Round-Trip Error 4326->3857->4326 (max absolute error, m).
#[test]
fn emit_projection_roundtrip_error() {
    let pts = lcg_points(0x0B15, 10_000);
    let mut max_err_m = 0.0f64;
    for (lon, lat) in pts {
        let m = web_mercator_from_lonlat(lon, lat);
        let (lon2, lat2) = lonlat_from_web_mercator(&m);
        let dlon = (lon - lon2).abs();
        let dlat = (lat - lat2).abs();
        // Convert the worst angular residual to approximate ground meters.
        let err_m = dlat.max(dlon / lat.clamp(1e-6, 90.0).to_radians().cos()) * 111_320.0;
        max_err_m = max_err_m.max(err_m);
    }
    kpi!("KPI-003", max_err_m);
    // Hard sanity bound well above the warn threshold (0.05 m): a real
    // projection regression trips this before the KPI gate does.
    assert!(max_err_m < 1.0, "projection round-trip error blew up: {max_err_m} m");
}

/// KPI-001 Geometry Positional Accuracy RMSE against an analytic truth set.
///
/// Ground truth here is the exact position of published landmark coordinates
/// re-projected through the engine pipeline; RMSE in meters.
#[test]
fn emit_geometry_positional_rmse() {
    // (lon, lat) reference landmarks with published coordinates.
    let landmarks = [
        (4.8936, 52.3738),     // Amsterdam Centraal area
        (13.404958, 52.5200),  // Berlin Brandenburg Gate
        (-73.9857, 40.7484),   // Empire State Building
        (2.2945, 48.8584),     // Eiffel Tower
        (139.7454, 35.6586),   // Tokyo Tower
    ];
    let mut sq_sum = 0.0f64;
    for (lon, lat) in landmarks {
        let truth = Coordinate::new(lon, lat);
        let m = web_mercator_from_lonlat(lon, lat);
        let back = lonlat_from_web_mercator(&m);
        let round = Coordinate::new(back.0, back.1);
        let dlon = (round.x - truth.x) * lat.to_radians().cos() * 111_320.0;
        let dlat = (round.y - truth.y) * 110_574.0;
        sq_sum += dlon * dlon + dlat * dlat;
    }
    let rmse = (sq_sum / landmarks.len() as f64).sqrt();
    kpi!("KPI-001", rmse);
    assert!(rmse < 0.1, "landmark positional RMSE regressed: {rmse} m");
}
