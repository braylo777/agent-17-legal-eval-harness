"""HTTP routes."""
import json
from .core import CLAUSES, bm25_index, run_pipeline, split_sentences


def handle(method, path, body_text="", now=lambda: 0):
    """Pure router: (status, json); status 0 = serve the page. All JSON responses carry CORS allow-origin *."""
    if path == "/api/extract" and method == "POST":
        try:
            b = json.loads(body_text)
        except ValueError:
            return 400, {"error": "bad json"}
        raw = b.get("text") if isinstance(b, dict) else None
        text = ("" if not raw else raw if isinstance(raw, str) else str(raw))[:200000]
        clause = b.get("clause") if isinstance(b, dict) else None
        if not isinstance(clause, str) or clause not in CLAUSES:
            return 400, {"error": "unknown clause"}
        sents = split_sentences(text)
        t0 = now()
        mode = b.get("mode")
        r = run_pipeline(sents, clause, "bm25" if mode == "bm25" else "rule", bm25_index(sents))
        return 200, {"clause": clause, "mode": mode if mode else "rule", **r, "ms": now() - t0}
    if path == "/api/clauses":
        return 200, list(CLAUSES)
    if path == "/health":
        return 200, {"ok": True}
    return 0, None
