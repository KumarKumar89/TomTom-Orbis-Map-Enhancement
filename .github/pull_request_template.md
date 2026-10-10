## Summary

- 

## Checklist before requesting a review

- [ ] `cargo fmt --all -- --check` passes (run in `rust/`)
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `cargo test --workspace` passes and emits any new `KPI <id> <value>` lines
- [ ] `pytest tests/python` passes (KPI framework contract tests)
- [ ] `python3 kpi/tools/kpi_report.py` gate still passes (or KPI deltas explained below)
- [ ] New/changed gated KPIs are defined in `kpi/definitions/kpis.yaml` first (single source of truth)
- [ ] No secrets, API keys, or large data files added (gitleaks + secret scanning will block them)
- [ ] Performance-sensitive changes include a benchmark note (Criterion budget: no >5% regression)

## KPI impact

| KPI id | Old | New | Reason |
|---|---|---|---|

