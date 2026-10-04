//! `py-bridge` — the single PyO3 surface between Python and the Rust engine.
//!
//! Build the wheel with maturin:
//! ```text
//! cd rust/crates/py-bridge && maturin build --features python
//! ```
//! Without the `python` feature this crate compiles as a plain Rust library so
//! the workspace CI (fmt/clippy/test) never requires a Python toolchain. The
//! pure-Rust core below is what both the CLI and the Python extension call —
//! one implementation, two delivery channels.

pub mod core_api;

#[cfg(feature = "python")]
pub mod python {
    //! Thin PyO3 wrappers. Keep logic in `core_api`; this module only moves
    //! data across the FFI boundary.
    use crate::core_api;
    use pyo3::prelude::*;

    #[pyfunction]
    fn mercator_from_lonlat(py: Python<'_>, lon: f64, lat: f64) -> PyResult<(f64, f64)> {
        let _ = py;
        Ok(core_api::mercator_from_lonlat(lon, lat))
    }

    #[pyfunction]
    fn haversine_m(a: (f64, f64), b: (f64, f64)) -> f64 {
        core_api::haversine_m(a.0, a.1, b.0, b.1)
    }

    #[pyfunction]
    fn frechet(points_a: Vec<(f64, f64)>, points_b: Vec<(f64, f64)>) -> f64 {
        core_api::frechet(&points_a, &points_b)
    }

    /// Validate a canonical MapFeature JSON string against contract basics.
    #[pyfunction]
    fn validate_feature_json(json: &str) -> PyResult<bool> {
        core_api::validate_feature_json(json)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    #[pymodule]
    fn orbis_engine(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_function(wrap_pyfunction!(mercator_from_lonlat, m)?)?;
        m.add_function(wrap_pyfunction!(haversine_m, m)?)?;
        m.add_function(wrap_pyfunction!(frechet, m)?)?;
        m.add_function(wrap_pyfunction!(validate_feature_json, m)?)?;
        m.add("__engine_version__", env!("CARGO_PKG_VERSION"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::core_api;

    #[test]
    fn core_api_is_usable_without_python() {
        let (x, y) = core_api::mercator_from_lonlat(4.8936, 52.3738);
        assert!(x > 0.0 && y > 0.0);
        let d = core_api::haversine_m(4.8936, 52.3738, 4.4777, 51.9244);
        assert!((57_000.0..60_000.0).contains(&d));
    }
}
