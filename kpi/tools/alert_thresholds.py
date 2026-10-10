#!/usr/bin/env python3
"""Alert rules derived from the KPI definitions (single source of truth).

Reads kpi/reports/kpi_summary.json (produced by kpi_report.py) and emits an
alert list for Slack/PagerDuty/email hooks or dashboard annotation. Exit code
is the number of alerts capped at 1 (nonzero => alerting state), so CI can
`|| true` it for informational runs or fail on it for release pipelines.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SUMMARY = ROOT / "reports" / "kpi_summary.json"


def main() -> int:
    path = Path(sys.argv[1]) if len(sys.argv) > 1 else SUMMARY
    if not path.exists():
        print("no kpi_summary.json found — run kpi_report.py first", file=sys.stderr)
        return 2
    s = json.loads(path.read_text())
    alerts = []
    for kid in s["gate"]["failed"]:
        k = next(x for x in s["kpis"] if x["id"] == kid)
        alerts.append(
            f"[CRITICAL] {kid} {k['name']}: actual {k['actual']}{k['unit']} "
            f"worse than warn {k['warn']}{k['unit']} (ideal {k['ideal']}{k['unit']})"
        )
    for kid in s["gate"]["missing"]:
        k = next(x for x in s["kpis"] if x["id"] == kid)
        alerts.append(
            f"[WARNING] gated KPI has no measurement yet: {kid} {k['name']} (source {k['category']})"
        )
    non_gate_fail = [k for k in s["kpis"] if not k["gate"] and k["status"] == "FAIL"]
    for k in non_gate_fail:
        alerts.append(
            f"[INFO] non-gated KPI failing (review): {k['id']} {k['name']} "
            f"actual {k['actual']}{k['unit']} vs ideal {k['ideal']}{k['unit']}"
        )
    for a in alerts:
        print(a)
    print(
        f"-- {len(alerts)} alert(s); gate status: {'FAIL' if s['gate']['failed'] else 'PASS'}",
        file=sys.stderr,
    )
    return 1 if any(a.startswith("[CRITICAL]") for a in alerts) else 0


if __name__ == "__main__":
    sys.exit(main())
