#!/usr/bin/env python3
"""KPI review framework — Ideal Target vs Actual with threshold evaluation.

Reads:
  kpi/definitions/kpis.yaml   (single source of truth for targets/thresholds)
  kpi/history/current.json    (latest measurements; produced by CI or manually)

Writes:
  kpi/reports/kpi_report.md     human-readable scorecard (Ideal vs Actual)
  kpi/reports/kpi_summary.json  machine-readable summary (dashboards/alerts)
  kpi/reports/junit_kpi.xml     JUnit gate report (CI annotations / failures)

Exit codes:
  0  all gated KPIs PASS/WARN-within-budget (WARN never fails the gate unless
     --strict-warn is given)
  1  at least one gated KPI FAIL, or a gated KPI is MISSING under --require-all
  2  input validation error (bad definitions / unknown ids in current.json)

Status model (mirrors kpi/definitions/kpis.yaml header):
  PASS    actual meets `pass`
  WARN    actual meets `warn` but not `pass`
  FAIL    actual worse than `warn`
  MISSING no measurement recorded yet
"""
from __future__ import annotations

import argparse
import json
import os
import sys
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from xml.sax.saxutils import escape

import yaml

ROOT = Path(__file__).resolve().parents[1]
DEFINITIONS = ROOT / "definitions" / "kpis.yaml"

STATUS_ORDER = ["PASS", "WARN", "FAIL", "MISSING"]
ICON = {"PASS": "✅", "WARN": "🟡", "FAIL": "🔴", "MISSING": "⚪"}


@dataclass
class KpiDef:
    id: str
    name: str
    category: str
    direction: str  # higher | lower
    unit: str
    ideal: float
    pass_th: float
    warn_th: float
    source: str
    review: str
    gate: bool


@dataclass
class KpiResult:
    definition: KpiDef
    actual: float | None
    status: str
    delta_vs_ideal: float | None
    history: list[float] = field(default_factory=list)

    @property
    def gap_pct(self) -> float | None:
        """% of the way from warn-threshold to pass-threshold that actual sits."""
        d, a = self.definition, self.actual
        if a is None:
            return None
        if d.direction == "higher":
            span = d.pass_th - d.warn_th
            return None if span <= 0 else max(0.0, min(1.0, (a - d.warn_th) / span)) * 100.0
        span = d.warn_th - d.pass_th
        return None if span <= 0 else max(0.0, min(1.0, (d.warn_th - a) / span)) * 100.0


def load_definitions(path: Path = DEFINITIONS) -> list[KpiDef]:
    raw = yaml.safe_load(path.read_text())
    defs: list[KpiDef] = []
    seen: set[str] = set()
    for cat, items in raw["categories"].items():
        for it in items:
            for req in ("id", "name", "direction", "unit", "ideal", "pass", "warn", "source", "review"):
                if req not in it:
                    raise ValueError(f"{it.get('id','?')}: missing field {req!r}")
            if it["direction"] not in ("higher", "lower"):
                raise ValueError(f"{it['id']}: direction must be higher|lower")
            ok = (it["pass"] >= it["warn"]) if it["direction"] == "higher" else (it["pass"] <= it["warn"])
            if not ok:
                raise ValueError(f"{it['id']}: pass threshold must be at least as strict as warn")
            if it["id"] in seen:
                raise ValueError(f"duplicate KPI id {it['id']}")
            seen.add(it["id"])
            defs.append(KpiDef(
                id=it["id"], name=it["name"], category=cat, direction=it["direction"],
                unit=it["unit"], ideal=float(it["ideal"]), pass_th=float(it["pass"]),
                warn_th=float(it["warn"]), source=it["source"], review=it["review"],
                gate=bool(it.get("gate", False)),
            ))
    return defs


def evaluate(d: KpiDef, actual: float | None) -> str:
    if actual is None:
        return "MISSING"
    meets_pass = actual >= d.pass_th if d.direction == "higher" else actual <= d.pass_th
    if meets_pass:
        return "PASS"
    meets_warn = actual >= d.warn_th if d.direction == "higher" else actual <= d.warn_th
    return "WARN" if meets_warn else "FAIL"


def load_current(path: Path) -> dict[str, float | None]:
    if not path.exists():
        return {}
    data = json.loads(path.read_text())
    out: dict[str, float | None] = {}
    for k, v in data.items():
        out[k] = None if v is None else float(v)
    return out


def load_history(root: Path) -> dict[str, list[float]]:
    """history/<run_id>.json snapshots -> per-KPI ordered series for trends."""
    series: dict[str, list[float]] = {}
    hdir = root / "history"
    if not hdir.is_dir():
        return series
    runs = sorted(p.name for p in hdir.glob("*.json"))
    for name in runs:
        try:
            snap = json.loads((hdir / name).read_text())
        except json.JSONDecodeError:
            continue
        for k, v in snap.get("metrics", {}).items():
            if v is not None:
                series.setdefault(k, []).append(float(v))
    return series


def fmt(v: float | None, unit: str) -> str:
    if v is None:
        return "—"
    if unit in ("pct", "%"):
        return f"{v:.2f}%"
    if abs(v) >= 1000 or (abs(v) < 0.01 and v != 0):
        return f"{v:,.4g}"
    return f"{v:g}"


def build_results(defs: list[KpiDef], current: dict[str, float | None],
                  history: dict[str, list[float]]) -> list[KpiResult]:
    results: list[KpiResult] = []
    for d in defs:
        actual = current.get(d.id)
        status = evaluate(d, actual)
        delta = None if actual is None else (actual - d.ideal if d.direction == "higher"
                                             else d.ideal - actual)
        hist = [v for v in history.get(d.id, []) if v is not None][-12:]
        results.append(KpiResult(d, actual, status, delta, hist))
    return results


def render_markdown(results: list[KpiResult], run_meta: dict) -> str:
    counts = {s: sum(1 for r in results if r.status == s) for s in STATUS_ORDER}
    lines: list[str] = []
    lines.append("# Orbis Map Enhancement — KPI Review Scorecard")
    lines.append("")
    lines.append(f"*Run `{run_meta.get('run_id','local')}` · pipeline `{run_meta.get('pipeline','manual')}`"
                 f" · commit `{run_meta.get('commit','unknown')}` · generated {run_meta.get('generated_utc','')}*")
    lines.append("")
    lines.append("## Executive summary")
    lines.append("")
    lines.append("| Status | Count | Share |")
    lines.append("|---|---:|---:|")
    total = len(results) or 1
    for s in STATUS_ORDER:
        lines.append(f"| {ICON[s]} {s} | {counts[s]} | {counts[s] * 100 // total}% |")
    gate_fail = [r for r in results if r.definition.gate and r.status == "FAIL"]
    gate_missing = [r for r in results if r.definition.gate and r.status == "MISSING"]
    verdict = "❌ GATE FAILED" if gate_fail else ("⚠️ gate incomplete (missing measurements)" if gate_missing else "✅ GATE PASSED")
    lines.append("")
    lines.append(f"**CI/CD gate verdict:** {verdict} — {len(gate_fail)} gated FAIL, "
                 f"{len(gate_missing)} gated MISSING of {sum(1 for r in results if r.definition.gate)} gated KPIs.")
    lines.append("")
    lines.append("> Every row below is read as **Ideal Target vs Actual**. `Gap→Pass` is how far the "
                 "actual has closed the distance between the warn and pass thresholds (100% = pass met).")
    lines.append("")

    by_cat: dict[str, list[KpiResult]] = {}
    for r in results:
        by_cat.setdefault(r.definition.category, []).append(r)

    lines.append("## Category overview")
    lines.append("")
    lines.append("| Category | KPIs | ✅ | 🟡 | 🔴 | ⚪ |")
    lines.append("|---|---:|---:|---:|---:|---:|")
    for cat, rs in by_cat.items():
        c = {s: sum(1 for r in rs if r.status == s) for s in STATUS_ORDER}
        lines.append(f"| {cat} | {len(rs)} | {c['PASS']} | {c['WARN']} | {c['FAIL']} | {c['MISSING']} |")
    lines.append("")

    for cat, rs in by_cat.items():
        lines.append(f"## {cat}")
        lines.append("")
        lines.append("| ID | KPI | Direction | Unit | Ideal Target | Pass ≤/≥ | Warn ≤/≥ | **Actual** | **Status** | Gap→Pass | Trend | Gate | Source | Cadence |")
        lines.append("|---|---|---|---|---|---|---|---|---|---:|---|---|---|")
        for r in sorted(rs, key=lambda x: x.definition.id):
            d = r.definition
            trend = sparkline(r.history + ([r.actual] if r.actual is not None else []))
            gap = "—" if r.gap_pct is None else f"{r.gap_pct:.0f}%"
            lines.append(
                f"| {d.id} | {d.name} | {'↑' if d.direction=='higher' else '↓'} | {d.unit} "
                f"| {fmt(d.ideal, d.unit)} | {fmt(d.pass_th, d.unit)} | {fmt(d.warn_th, d.unit)} "
                f"| **{fmt(r.actual, d.unit)}** | {ICON[r.status]} {r.status} | {gap} | {trend} "
                f"| {'🔒' if d.gate else ''} | {d.source} | {d.review} |")
        lines.append("")

    problems = [r for r in results if r.status in ("FAIL", "MISSING") and r.definition.gate]
    lines.append("## Action items (gated KPIs needing attention)")
    lines.append("")
    if not problems:
        lines.append("None — all gated KPIs are meeting their thresholds.")
    for r in problems:
        d = r.definition
        if r.status == "FAIL":
            lines.append(f"- **{d.id} {d.name}**: actual {fmt(r.actual, d.unit)} breaches warn "
                         f"{fmt(d.warn_th, d.unit)} ({d.unit}, target ideal {fmt(d.ideal, d.unit)}) — fix before merge/release.")
        else:
            lines.append(f"- **{d.id} {d.name}**: no measurement recorded — wire up `{d.source}` emitter.")
    lines.append("")
    return "\n".join(lines) + "\n"


def sparkline(vals: list[float]) -> str:
    if len(vals) < 2:
        return "—"
    lo, hi = min(vals), max(vals)
    if hi == lo:
        return "→"
    last, prev = vals[-1], vals[-2]
    return "↑" if last > prev else ("↓" if last < prev else "→")


def render_junit(results: list[KpiResult], suite_name: str) -> str:
    cases = []
    for r in results:
        d = r.definition
        # Only gated KPIs can fail the suite; non-gated become skipped tests.
        if not d.gate:
            cases.append(f'    <testcase classname="kpi.{d.category}" name="{escape(d.id)} {escape(d.name)}">'
                         f'<skipped message="non-gated ({r.status})"/></testcase>')
            continue
        if r.status == "FAIL":
            cases.append(f'    <testcase classname="kpi.{d.category}" name="{escape(d.id)} {escape(d.name)}">'
                         f'<failure message="status FAIL">actual={r.actual} warn={d.warn_th} pass={d.pass_th} '
                         f'direction={d.direction} unit={d.unit}</failure></testcase>')
        elif r.status == "MISSING":
            cases.append(f'    <testcase classname="kpi.{d.category}" name="{escape(d.id)} {escape(d.name)}">'
                         f'<error message="no measurement recorded"/></testcase>')
        else:
            cases.append(f'    <testcase classname="kpi.{d.category}" name="{escape(d.id)} {escape(d.name)}"/>')
    failures = sum(1 for r in results if r.definition.gate and r.status == "FAIL")
    errors = sum(1 for r in results if r.definition.gate and r.status == "MISSING")
    skipped = sum(1 for r in results if not r.definition.gate)
    return (f'<?xml version="1.0" encoding="UTF-8"?>\n<testsuite name="{suite_name}" tests="{len(results)}" '
            f'failures="{failures}" errors="{errors}" skipped="{skipped}">\n'
            + "\n".join(cases) + "\n</testsuite>\n")


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--definitions", type=Path, default=DEFINITIONS)
    ap.add_argument("--current", type=Path, default=ROOT / "history" / "current.json")
    ap.add_argument("--out-dir", type=Path, default=ROOT / "reports")
    ap.add_argument("--strict-warn", action="store_true", help="treat gated WARN as gate failure")
    ap.add_argument("--require-all", action="store_true", help="treat gated MISSING as gate failure")
    ap.add_argument("--fail-on-missing", action="store_true",
                    help="alias of --require-all kept for CI readability")
    args = ap.parse_args(argv)

    try:
        defs = load_definitions(args.definitions)
    except (ValueError, KeyError, yaml.YAMLError) as e:
        print(f"definition error: {e}", file=sys.stderr)
        return 2

    current = load_current(args.current)
    known = {d.id for d in defs}
    unknown = sorted(set(current) - known)
    if unknown:
        print(f"current.json references unknown KPI ids: {unknown}", file=sys.stderr)
        return 2

    history = load_history(ROOT)
    results = build_results(defs, current, history)

    args.out_dir.mkdir(parents=True, exist_ok=True)
    run_meta = {
        "run_id": datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ"),
        "pipeline": "manual", "commit": "unknown",
        "generated_utc": datetime.now(timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z"),
    }
    # CI injects richer metadata via env (see .github/workflows/kpi-review.yml).
    if os.environ.get("GITHUB_ACTIONS"):
        run_meta.update({
            "run_id": os.environ.get("GITHUB_RUN_ID", run_meta["run_id"]),
            "pipeline": os.environ.get("GITHUB_WORKFLOW", "ci"),
            "commit": os.environ.get("GITHUB_SHA", "unknown"),
        })
    ci_env = ROOT / ".ci_run_meta.json"
    if ci_env.exists():
        run_meta.update(json.loads(ci_env.read_text()))

    md = render_markdown(results, run_meta)
    (args.out_dir / "kpi_report.md").write_text(md)
    junit = render_junit(results, "kpi-gate")
    (args.out_dir / "junit_kpi.xml").write_text(junit)

    counts = {s: sum(1 for r in results if r.status == s) for s in STATUS_ORDER}
    gate_fails = [r for r in results if r.definition.gate and r.status == "FAIL"]
    gate_warns = [r for r in results if r.definition.gate and r.status == "WARN"]
    gate_missing = [r for r in results if r.definition.gate and r.status == "MISSING"]
    summary = {
        "meta": run_meta,
        "counts": counts,
        "gate": {
            "total_gated": sum(1 for r in results if r.definition.gate),
            "failed": [r.definition.id for r in gate_fails],
            "warned": [r.definition.id for r in gate_warns],
            "missing": [r.definition.id for r in gate_missing],
        },
        "kpis": [
            {"id": r.definition.id, "category": r.definition.category, "name": r.definition.name,
             "unit": r.definition.unit, "direction": r.definition.direction, "gate": r.definition.gate,
             "ideal": r.definition.ideal, "pass": r.definition.pass_th, "warn": r.definition.warn_th,
             "actual": r.actual, "status": r.status, "gap_to_pass_pct": r.gap_pct}
            for r in results
        ],
    }
    (args.out_dir / "kpi_summary.json").write_text(json.dumps(summary, indent=1) + "\n")

    require_all = args.require_all or args.fail_on_missing
    # NOTE: MISSING gated KPIs do NOT fail the gate by default so the framework
    # can land before every emitter exists; CI passes --require-all once wired.
    failed = bool(gate_fails) or (args.strict_warn and bool(gate_warns)) \
        or (require_all and bool(gate_missing))
    print(f"KPI report: {counts['PASS']} PASS / {counts['WARN']} WARN / {counts['FAIL']} FAIL / "
          f"{counts['MISSING']} MISSING over {len(results)} KPIs "
          f"({summary['gate']['total_gated']} gated). Gate: {'FAIL' if failed else 'PASS'}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
