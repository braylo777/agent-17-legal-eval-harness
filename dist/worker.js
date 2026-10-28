// Multi-agent clause extraction: extractor -> verifier -> judge. Pure JS, no model calls.
const CLAUSES = {
  "Governing Law": { rx: /govern(ed|ing)\b.*\blaws?\b|\blaws? of the (state|commonwealth|province)/i, q: "governed construed laws state jurisdiction", cue: /law/i },
  "Anti-Assignment": { rx: /\bassign(ed|ment|s)?\b.*\b(without|prior written consent|not)\b|\bnot\b.*\bassign/i, q: "assign assignment transfer consent without prior written", cue: /assign|transfer/i },
  "Cap On Liability": { rx: /(liabilit(y|ies)).*(exceed|limited to|in no event|aggregate)|in no event.*liable/i, q: "liability exceed aggregate limited amount paid in no event liable", cue: /liab/i },
  "Audit Rights": { rx: /\baudit|inspect(ion)? (the )?(books|records)/i, q: "audit inspect books records examine", cue: /audit|inspect|records/i },
  "Insurance": { rx: /\binsurance\b|\binsured\b/i, q: "insurance maintain coverage policy insured", cue: /insur/i },
  "Termination For Convenience": { rx: /terminat\w*.*(for any reason|without cause|for convenience|at any time)/i, q: "terminate any reason without cause convenience notice", cue: /terminat/i },
  "Exclusivity": { rx: /\bexclusive(ly)?\b|\bsole (and exclusive )?(supplier|distributor|provider)/i, q: "exclusive exclusively sole distributor rights territory", cue: /exclusiv|sole/i },
  "Non-Compete": { rx: /\bcompet(e|ing|itive)\b.*\b(not|shall not|refrain)|non-compet/i, q: "compete competing competitive business not engage", cue: /compet/i },
};
const tok = (s) => (s.toLowerCase().match(/[a-z0-9]+/g) || []);
function bm25Index(sents, k1 = 1.2, b = 0.75) {
  const docs = sents.map(tok), N = docs.length, df = new Map();
  const avg = docs.reduce((a, d) => a + d.length, 0) / Math.max(N, 1);
  for (const d of docs) for (const t of new Set(d)) df.set(t, (df.get(t) || 0) + 1);
  return (q) => docs.map((d) => { const tf = new Map(); for (const t of d) tf.set(t, (tf.get(t) || 0) + 1); let s = 0;
    for (const t of new Set(tok(q))) { const f = tf.get(t); if (!f) continue; const idf = Math.log(1 + (N - df.get(t) + 0.5) / (df.get(t) + 0.5)); s += idf * f * (k1 + 1) / (f + k1 * (1 - b + b * d.length / avg)); } return s; });
}
// Agent 1: extractor. mode "rule" | "bm25". Returns {sentence, score} or null.
function extract(sents, clause, mode = "rule", index) {
  const c = CLAUSES[clause]; if (!c) throw new Error("unknown clause");
  if (mode === "rule") { const i = sents.findIndex((s) => c.rx.test(s)); return i < 0 ? null : { i, sentence: sents[i], score: 1 }; }
  const sc = (index || bm25Index(sents))(c.q); let i = 0; for (let j = 1; j < sc.length; j++) if (sc[j] > sc[i]) i = j;
  return sc[i] > 0 ? { i, sentence: sents[i], score: sc[i] } : null;
}
// Agent 2: verifier. Rejects extractions lacking the clause's cue term or too short/long.
function verify(ext, clause) {
  if (!ext) return { ok: false, reason: "none" };
  const s = ext.sentence; if (s.length < 20 || s.length > 3000) return { ok: false, reason: "length" };
  return CLAUSES[clause].cue.test(s) ? { ok: true } : { ok: false, reason: "missing cue" };
}
// Agent 3: judges. Two independent deterministic judges; LLM judge after quota reset.
function judgeOverlap(pred, golds) { // token-F1 >= 0.3 vs best gold span
  if (!pred) return golds.length === 0; if (!golds.length) return false;
  const p = tok(pred); return golds.some((g) => { const gt = tok(g), gs = new Set(gt); const inter = p.filter((t) => gs.has(t)).length; if (!inter) return false; const pr = inter / p.length, rc = inter / gt.length; return 2 * pr * rc / (pr + rc) >= 0.3; });
}
function judgeContain(pred, golds) { // gold span (first 40 chars) appears in pred, or pred in gold
  if (!pred) return golds.length === 0; if (!golds.length) return false;
  const n = (x) => tok(x).join(" "), p = n(pred); return golds.some((g) => { const gg = n(g); return p.includes(gg.slice(0, 40)) || gg.includes(p); });
}
function kappa(a, b) { const n = a.length; let po = 0, pa = 0, pb = 0; for (let i = 0; i < n; i++) { po += a[i] === b[i]; pa += a[i]; pb += b[i]; } po /= n; pa /= n; pb /= n; const pe = pa * pb + (1 - pa) * (1 - pb); return pe === 1 ? 1 : (po - pe) / (1 - pe); }
function runPipeline(sents, clause, mode, index) { const e = extract(sents, clause, mode, index); const v = verify(e, clause); return { extraction: v.ok ? e : null, raw: e, verifier: v }; }

const H = { "content-type": "application/json", "access-control-allow-origin": "*" };
const page = `<!doctype html><meta charset=utf-8><meta name=viewport content="width=device-width"><title>agent-17 legal eval harness</title><body style="font:15px system-ui;max-width:760px;margin:2em auto;padding:0 1em"><h1>agent-17 · Legal clause extraction harness</h1><p>Extractor → verifier → judge on CUAD clause types. Offline rule/BM25 agents (no LLM yet). <a href="https://github.com/braylo777/agent-17-legal-eval-harness">GitHub</a></p><textarea id=t rows=10 style="width:100%">This Agreement shall be governed by the laws of the State of New York. Neither party may assign this Agreement without the prior written consent of the other party.</textarea><br><select id=c>${Object.keys(CLAUSES).map((c) => `<option>${c}</option>`).join("")}</select> <select id=m><option>rule</option><option>bm25</option></select> <button onclick="go()">Extract</button><pre id=o style="white-space:pre-wrap;background:#f4f4f4;padding:1em"></pre><script>async function go(){const r=await fetch('/api/extract',{method:'POST',body:JSON.stringify({text:t.value,clause:c.value,mode:m.value})});o.textContent=JSON.stringify(await r.json(),null,1)}</script>`;
export default { async fetch(req) {
  const u = new URL(req.url);
  if (u.pathname === "/api/extract" && req.method === "POST") { let b; try { b = await req.json(); } catch { return new Response('{"error":"bad json"}', { status: 400, headers: H }); }
    const text = String(b.text || "").slice(0, 200000); if (!CLAUSES[b.clause]) return new Response('{"error":"unknown clause"}', { status: 400, headers: H });
    const sents = text.split(/(?<=[.;:])\s+(?=[A-Z(0-9])/).filter((s) => s.trim().length > 3); const t0 = Date.now();
    const r = runPipeline(sents, b.clause, b.mode === "bm25" ? "bm25" : "rule", bm25Index(sents)); return new Response(JSON.stringify({ clause: b.clause, mode: b.mode || "rule", ...r, ms: Date.now() - t0 }), { headers: H }); }
  if (u.pathname === "/api/clauses") return new Response(JSON.stringify(Object.keys(CLAUSES)), { headers: H });
  if (u.pathname === "/health") return new Response('{"ok":true}', { headers: H });
  return new Response(page, { headers: { "content-type": "text/html; charset=utf-8" } }); } };
