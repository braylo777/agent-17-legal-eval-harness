"""Multi-agent clause extraction: extractor -> verifier -> judge. Pure Python, no model calls.
Regexes follow JavaScript semantics (ASCII \\b/\\w, `.` stops at any JS line terminator, JS whitespace set)."""
import math, re

D = r"[^\n\r  ]"  # JS `.`
WS = r"[\t\n\v\f\r    -     　﻿]"
F = re.I | re.ASCII


def _c(p):
    return re.compile(p.replace("DOT", D), F)


CLAUSES = {
    "Governing Law": {"rx": _c(r"govern(ed|ing)\bDOT*\blaws?\b|\blaws? of the (state|commonwealth|province)"), "q": "governed construed laws state jurisdiction", "cue": _c(r"law")},
    "Anti-Assignment": {"rx": _c(r"\bassign(ed|ment|s)?\bDOT*\b(without|prior written consent|not)\b|\bnot\bDOT*\bassign"), "q": "assign assignment transfer consent without prior written", "cue": _c(r"assign|transfer")},
    "Cap On Liability": {"rx": _c(r"(liabilit(y|ies))DOT*(exceed|limited to|in no event|aggregate)|in no eventDOT*liable"), "q": "liability exceed aggregate limited amount paid in no event liable", "cue": _c(r"liab")},
    "Audit Rights": {"rx": _c(r"\baudit|inspect(ion)? (the )?(books|records)"), "q": "audit inspect books records examine", "cue": _c(r"audit|inspect|records")},
    "Insurance": {"rx": _c(r"\binsurance\b|\binsured\b"), "q": "insurance maintain coverage policy insured", "cue": _c(r"insur")},
    "Termination For Convenience": {"rx": _c(r"terminat\w*DOT*(for any reason|without cause|for convenience|at any time)"), "q": "terminate any reason without cause convenience notice", "cue": _c(r"terminat")},
    "Exclusivity": {"rx": _c(r"\bexclusive(ly)?\b|\bsole (and exclusive )?(supplier|distributor|provider)"), "q": "exclusive exclusively sole distributor rights territory", "cue": _c(r"exclusiv|sole")},
    "Non-Compete": {"rx": _c(r"\bcompet(e|ing|itive)\bDOT*\b(not|shall not|refrain)|non-compet"), "q": "compete competing competitive business not engage", "cue": _c(r"compet")},
}
_TOK = re.compile(r"[a-z0-9]+")
tok = lambda s: _TOK.findall(s.lower())
_SPLIT = re.compile("(?<=[.;:])" + WS + "+(?=[A-Z(0-9])")
_TRIM = re.compile("^" + WS + "+|" + WS + "+$")


def js_len(s):
    return len(s.encode("utf-16-le")) // 2


def split_sentences(text):
    return [s for s in _SPLIT.split(text) if js_len(_TRIM.sub("", s)) > 3]


def bm25_index(sents, k1=1.2, b=0.75):
    docs = [tok(s) for s in sents]
    N = len(docs)
    df = {}
    avg = sum(len(d) for d in docs) / max(N, 1)
    for d in docs:
        for t in set(d):
            df[t] = df.get(t, 0) + 1

    def score(q):
        out = []
        for d in docs:
            tf = {}
            for t in d:
                tf[t] = tf.get(t, 0) + 1
            s = 0.0
            for t in dict.fromkeys(tok(q)):
                f = tf.get(t)
                if not f:
                    continue
                idf = math.log(1 + (N - df[t] + 0.5) / (df[t] + 0.5))
                s += idf * f * (k1 + 1) / (f + k1 * (1 - b + b * len(d) / avg))
            out.append(s)
        return out
    return score


def extract(sents, clause, mode="rule", index=None):
    """Agent 1: extractor. mode "rule" | "bm25". Returns {i, sentence, score} or None."""
    c = CLAUSES.get(clause)
    if not c:
        raise ValueError("unknown clause")
    if mode == "rule":
        for i, s in enumerate(sents):
            if c["rx"].search(s):
                return {"i": i, "sentence": s, "score": 1}
        return None
    sc = (index or bm25_index(sents))(c["q"])
    i = 0
    for j in range(1, len(sc)):
        if sc[j] > sc[i]:
            i = j
    return {"i": i, "sentence": sents[i], "score": sc[i]} if sc and sc[i] > 0 else None


def verify(ext, clause):
    """Agent 2: verifier. Rejects extractions lacking the clause's cue term or too short/long."""
    if not ext:
        return {"ok": False, "reason": "none"}
    s = ext["sentence"]
    n = js_len(s)
    if n < 20 or n > 3000:
        return {"ok": False, "reason": "length"}
    return {"ok": True} if CLAUSES[clause]["cue"].search(s) else {"ok": False, "reason": "missing cue"}


def judge_overlap(pred, golds):
    """Agent 3a: token-F1 >= 0.3 vs best gold span."""
    if not pred:
        return len(golds) == 0
    if not golds:
        return False
    p = tok(pred)
    for g in golds:
        gt = tok(g)
        gs = set(gt)
        inter = sum(1 for t in p if t in gs)
        if not inter:
            continue
        pr, rc = inter / len(p), inter / len(gt)
        if 2 * pr * rc / (pr + rc) >= 0.3:
            return True
    return False


def judge_contain(pred, golds):
    """Agent 3b: gold span (first 40 chars) appears in pred, or pred in gold."""
    if not pred:
        return len(golds) == 0
    if not golds:
        return False
    n = lambda x: " ".join(tok(x))
    p = n(pred)
    return any(p.find(n(g)[:40]) >= 0 or p in n(g) for g in golds)


def kappa(a, b):
    n = len(a)
    po = sum(1 for x, y in zip(a, b) if x == y) / n
    pa, pb = sum(a) / n, sum(b) / n
    pe = pa * pb + (1 - pa) * (1 - pb)
    return 1 if pe == 1 else (po - pe) / (1 - pe)


def run_pipeline(sents, clause, mode="rule", index=None):
    e = extract(sents, clause, mode, index)
    v = verify(e, clause)
    return {"extraction": e if v["ok"] else None, "raw": e, "verifier": v}
