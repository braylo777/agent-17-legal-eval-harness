import json, sys
from pathlib import Path
import pytest

root = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(root / "evals"))
from agent17.core import CLAUSES, extract, verify, judge_overlap, judge_contain, kappa, bm25_index, run_pipeline, split_sentences
from agent17.api import handle
from run import evaluate

S = ["This Agreement shall be governed by the laws of the State of Delaware.", "Neither party may assign this Agreement without prior written consent.", "Supplier shall maintain general liability insurance of $1,000,000.", "The weather is nice."]


def test_clauses_and_extractors():
    assert len(CLAUSES) == 8
    assert extract(S, "Governing Law")["i"] == 0 and extract(S, "Anti-Assignment")["i"] == 1 and extract(S, "Insurance", "bm25")["i"] == 2
    assert extract(S, "Audit Rights") is None
    with pytest.raises(Exception):
        extract(S, "Foo")


def test_verifier():
    assert verify({"sentence": S[3] + " padding padding padding"}, "Insurance")["ok"] is False
    assert verify({"sentence": S[2]}, "Insurance")["ok"] is True and verify(None, "Insurance")["reason"] == "none"
    assert run_pipeline(S, "Audit Rights", "rule")["extraction"] is None


def test_judges_and_kappa():
    assert judge_overlap(S[0], ["governed by the laws of the State of Delaware"]) and not judge_overlap(S[3], ["governed by the laws of Delaware"])
    assert judge_overlap(None, []) and judge_contain(None, []) and judge_contain(S[1], ["Neither party may assign this Agreement"])
    assert kappa([1, 0, 1], [1, 0, 1]) == 1 and kappa([1, 0, 1, 0], [0, 1, 1, 0]) <= 0
    assert all(x >= 0 for x in bm25_index(S)("law"))


def test_no_regression_vs_baseline_and_verifier_helps():
    base = json.loads((root / "data" / "baseline.json").read_text())
    now = evaluate("test")
    for s, m in base.items():
        for k, v in m.items():
            assert now[s][k] >= v - 0.02, f"{s}.{k}"
    assert now["bm25+verifier"]["precision"] > now["bm25"]["precision"]


def test_http_routes_and_splitter():
    assert handle("GET", "/health")[1] == {"ok": True} and len(handle("GET", "/api/clauses")[1]) == 8 and handle("GET", "/")[0] == 0
    assert handle("POST", "/api/extract", "{nope")[0] == 400 and handle("POST", "/api/extract", json.dumps({"clause": "Foo", "text": "x"}))[0] == 400
    s, r = handle("POST", "/api/extract", json.dumps({"clause": "Insurance", "text": " ".join(S)}))
    assert s == 200 and r["extraction"]["i"] == 2 and r["mode"] == "rule"
    assert split_sentences("Alpha beta. Gamma delta; (Epsilon) zeta: 7 eta. lower case x") == ["Alpha beta.", "Gamma delta;", "(Epsilon) zeta:", "7 eta. lower case x"]
