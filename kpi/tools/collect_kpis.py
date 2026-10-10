#!/usr/bin/env python3
"""Collect KPI measurements from local artifacts into kpi/history/current.json.

Emitters already available in this repo:
  * Rust unit tests that print `KPI <id> <value>` lines (see kpi/README.md).
    Run them with:  cargo test -q ... | python kpi/tools/collect_kpis.py --stdin
  * Any pipeline job can append measurements by writing JSON of the form
    {"KPI-026": 97.4, "INFRA-001": 2.1} to a file and passing --merge FILE.

The script merges all sources over the definitions whitelist, records a
timestamped snapshot into kpi/history/<run_id>.json for regression tracking,
and refreshes kpi/history/current.json used by kpi_report.py.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from datetime import UTC, datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
from kpi_report import load_definitions  # noqa: E402

LINE_RE = re.compile(r"KPI\s+([A-Z]+-\d{3})\s+(-?\d+(?:\.\d+)?(?:e-?\d+)?)", re.I)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--stdin", action="store_true", help="parse KPI lines from stdin")
    ap.add_argument("--merge", type=Path, nargs="*", default=[], help="JSON files {kpi_id: value}")
    ap.add_argument("--run-id", default=datetime.now(UTC).strftime("%Y%m%dT%H%M%SZ"))
    args = ap.parse_args()

    known = {d.id for d in load_definitions()}
    metrics: dict[str, float] = {}
    if args.stdin:
        for line in sys.stdin:
            m = LINE_RE.search(line)
            if m and m.group(1).upper() in known:
                metrics[m.group(1).upper()] = float(m.group(2))
    for f in args.merge:
        data = json.loads(Path(f).read_text())
        bad = sorted(set(data) - known)
        if bad:
            print(f"warning: ignoring unknown ids in {f}: {bad}", file=sys.stderr)
        for k, v in data.items():
            if k in known and v is not None:
                metrics[k] = float(v)

    hist = ROOT / "history"
    hist.mkdir(exist_ok=True)
    snap = {
        "run_id": args.run_id,
        "generated_utc": datetime.now(UTC).isoformat(timespec="seconds"),
        "metrics": metrics,
    }
    (hist / f"{args.run_id}.json").write_text(json.dumps(snap, indent=1) + "\n")
    cur = hist / "current.json"
    prev = json.loads(cur.read_text()) if cur.exists() else {}
    prev.update(metrics)
    cur.write_text(json.dumps(prev, indent=1, sort_keys=True) + "\n")
    print(f"collected {len(metrics)} measurements; current.json now holds {len(prev)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
