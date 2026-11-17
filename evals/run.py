"""CUAD clause-extraction eval: rule vs BM25, with and without the verifier. Usage: python3 evals/run.py [--write]"""
import json, math, sys
from pathlib import Path

root = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(root / "src"))
from agent17.core import CLAUSES, bm25_index, judge_contain, judge_overlap, kappa, run_pipeline  # noqa: E402

data = json.loads((root / "data" / "cuad_subset.json").read_text(encoding="utf-8"))
r3 = lambda x: math.floor(x * 1000 + 0.5) / 1000


def evaluate(split="test"):
    res = {}
    for mode in ("rule", "bm25"):
        for verified in (False, True):
            tp = fp = fn = tn = correct = n = 0
            j1, j2, per = [], [], {}
            for d in (x for x in data["docs"] if x["split"] == split):
                idx = bm25_index(d["sentences"])
                for c in CLAUSES:
                    g = d["gold"].get(c) or []
                    r = run_pipeline(d["sentences"], c, mode, idx)
                    e = r["extraction"] if verified else r["raw"]
                    pred = e["sentence"] if e else None
                    a, b = judge_overlap(pred, g), judge_contain(pred, g)
                    j1.append(int(a)); j2.append(int(b)); n += 1; correct += int(a)
                    p = per.setdefault(c, {"tp": 0, "fp": 0, "fn": 0})
                    if pred and g:
                        if a: tp += 1; p["tp"] += 1
                        else: fp += 1; fn += 1; p["fp"] += 1; p["fn"] += 1
                    elif pred: fp += 1; p["fp"] += 1
                    elif g: fn += 1; p["fn"] += 1
                    else: tn += 1
            P = tp / ((tp + fp) or 1); R = tp / ((tp + fn) or 1)
            def f1(pp, rr):
                return 2 * pp * rr / ((pp + rr) or 1)
            res[f"{mode}{'+verifier' if verified else ''}"] = {
                "n": n, "precision": r3(P), "recall": r3(R), "f1": r3(f1(P, R)), "accuracy": r3(correct / n),
                "judge_agreement": r3(sum(1 for x, y in zip(j1, j2) if x == y) / n), "judge_kappa": r3(kappa(j1, j2)),
                "per_clause_f1": {k: r3(f1(v["tp"] / ((v["tp"] + v["fp"]) or 1), v["tp"] / ((v["tp"] + v["fn"]) or 1))) for k, v in per.items()}}
    return res


if __name__ == "__main__":
    out = {"dataset": "CUAD v1 test subset (chenghao/cuad_qa), 40 contracts: 10 dev / 30 test, 8 clause types", "test": evaluate("test"), "dev": evaluate("dev")}
    (root / "evals" / ("results.json" if "--write" in sys.argv else "results.check.json")).write_text(json.dumps(out, indent=1))
    for k, v in out["test"].items():
        print(k, v["precision"], v["recall"], v["f1"], v["accuracy"], v["judge_kappa"])
