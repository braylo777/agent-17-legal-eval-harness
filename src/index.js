import { CLAUSES, runPipeline, bm25Index } from "./core.js";
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
