"""Contract tests for the kpi/ review framework.

Run: pytest tests/python/test_kpi_framework.py
Validates the invariants the whole scorecard depends on:
  * 92 domain KPIs across 20 categories (+ integrated infra/perf baseline)
  * every KPI has Ideal Target vs Actual semantics (ideal/pass/warn coherent)
  * status evaluation PASS/WARN/FAIL/MISSING is direction-aware
  * gate exit codes behave for CI/CD automation
"""
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[2]
KPI = ROOT / "kpi"
sys.path.insert(0, str(KPI / "tools"))

import kpi_report as kr  # noqa: E402


defs = kr.load_definitions()  # module-level: definitions are pure data


def test_92_domain_kpis_across_20_categories():
    cats = {d.category for d in defs}
    domain = [d for d in defs if d.id.startswith("KPI-")]
    assert len(domain) == 92
    # 19 domain categories + the integrated infrastructure/performance one.
    assert len(cats) == 20
    assert "infra_performance" in cats


def test_ids_contiguous_and_unique():
    ids = [d.id for d in defs]
    assert len(ids) == len(set(ids))
    nums = sorted(int(d.id.split("-")[1]) for d in defs if d.id.startswith("KPI-"))
    assert nums == list(range(1, 93))


def test_key_review_areas_present():
    need = {
        "geospatial_core", "topology", "lanes", "conflation", "change_detection",
        "data_quality", "computer_vision", "routing", "spatial_indexing",
        "regression", "orbis_integration", "overture_integration", "osm_integration",
        "sensors_probe", "rendering_tiles", "coverage", "freshness",
    }
    assert need <= {d.category for d in defs}


def test_thresholds_coherent_with_direction():
    for d in defs:
        if d.direction == "higher":
            assert d.pass_th >= d.warn_th, d.id
        else:
            assert d.pass_th <= d.warn_th, d.id


def test_status_model_direction_aware():
    hi = next(d for d in defs if d.direction == "higher")
    lo = next(d for d in defs if d.direction == "lower")
    assert kr.evaluate(hi, None) == "MISSING"
    assert kr.evaluate(hi, hi.pass_th) == "PASS"
    assert kr.evaluate(hi, hi.warn_th) == "WARN"
    assert kr.evaluate(hi, hi.warn_th - 1e-9) == "FAIL"
    assert kr.evaluate(lo, lo.pass_th) == "PASS"
    assert kr.evaluate(lo, lo.warn_th) == "WARN"
    assert kr.evaluate(lo, lo.warn_th + 1e-9) == "FAIL"


def test_current_measurements_reference_known_ids():
    cur_path = KPI / "history" / "current.json"
    assert cur_path.exists(), "current.json missing — record measurements via collect_kpis.py"
    known = {d.id for d in defs}
    cur = json.loads(cur_path.read_text())
    assert not (set(cur) - known), f"unknown ids: {sorted(set(cur) - known)}"


def test_report_end_to_end_gate_fails_on_breach(tmp_path):
    """A gated FAIL must produce exit code 1 and a JUnit failure entry."""
    bad = tmp_path / "current.json"
    g = next(d for d in defs if d.gate and d.direction == "higher")
    bad.write_text(json.dumps({g.id: g.warn_th - 1.0}))
    rc = subprocess.run(
        [sys.executable, str(KPI / "tools" / "kpi_report.py"),
         "--current", str(bad), "--out-dir", str(tmp_path / "out")],
        capture_output=True, text=True)
    assert rc.returncode == 1, rc.stdout + rc.stderr
    junit = (tmp_path / "out" / "junit_kpi.xml").read_text()
    assert "<failure" in junit
    summary = json.loads((tmp_path / "out" / "kpi_summary.json").read_text())
    assert g.id in summary["gate"]["failed"]


def test_report_passes_when_all_gated_meet_warn(tmp_path):
    good = tmp_path / "current.json"
    payload = {}
    for d in defs:
        # actual exactly at pass threshold => everything green.
        payload[d.id] = d.pass_th
    good.write_text(json.dumps(payload))
    rc = subprocess.run(
        [sys.executable, str(KPI / "tools" / "kpi_report.py"),
         "--current", str(good), "--out-dir", str(tmp_path / "out")],
        capture_output=True, text=True)
    assert rc.returncode == 0, rc.stdout + rc.stderr
    md = (tmp_path / "out" / "kpi_report.md").read_text()
    assert "GATE PASSED" in md
    assert "Ideal Target" in md  # every row rendered as Ideal vs Actual


def test_unknown_id_rejected(tmp_path):
    bad = tmp_path / "current.json"
    bad.write_text(json.dumps({"KPI-999": 1.0}))
    rc = subprocess.run(
        [sys.executable, str(KPI / "tools" / "kpi_report.py"),
         "--current", str(bad), "--out-dir", str(tmp_path / "out")],
        capture_output=True, text=True)
    assert rc.returncode == 2


def test_collector_ingests_stdin_lines(tmp_path, monkeypatch):
    hist = tmp_path / "history"
    hist.mkdir()
    src = (KPI / "tools" / "collect_kpis.py").read_text()
    # run collector with a sandboxed ROOT by patching module constant via env-free approach:
    mod = tmp_path / "collect_sandbox.py"
    patched = src.replace('ROOT = Path(__file__).resolve().parents[1]',
                          f'ROOT = Path(r"{KPI}")\nOUT_ROOT = Path(r"{hist.parent}")')
    patched = patched.replace('hist = ROOT / "history"', 'hist = OUT_ROOT / "history"')
    patched = patched.replace('from kpi_report import', f'sys.path.insert(0, r"{KPI / "tools"}"); from kpi_report import')
    mod.write_text(patched)
    rc = subprocess.run([sys.executable, str(mod), "--stdin", "--run-id", "sbx"],
                        input="noise\nKPI KPI-057 100.0\nKPI INFRA-001 1.5\n",
                        capture_output=True, text=True)
    assert rc.returncode == 0, rc.stderr
    snap = json.loads((hist / "sbx.json").read_text())
    assert snap["metrics"] == {"KPI-057": 100.0, "INFRA-001": 1.5}
