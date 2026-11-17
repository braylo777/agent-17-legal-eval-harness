"""Regression gate: fails CI if any tracked metric drops more than 0.02 below data/baseline.json."""
import json, sys
from pathlib import Path

root = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(root / "evals"))
from run import evaluate  # noqa: E402

base = json.loads((root / "data" / "baseline.json").read_text())
now = evaluate("test")
bad = 0
for sysname, m in base.items():
    for k, v in m.items():
        cur = now[sysname][k]
        ok = cur >= v - 0.02
        bad += 0 if ok else 1
        print(f"{'PASS' if ok else 'FAIL'} {sysname}.{k} {cur} (baseline {v})")
sys.exit(1 if bad else 0)
