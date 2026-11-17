//! Demo page (taken from the old src/index.js template string).
const TEMPLATE: &str = r####"<!doctype html><meta charset=utf-8><meta name=viewport content="width=device-width"><title>agent-17 legal eval harness</title><body style="font:15px system-ui;max-width:760px;margin:2em auto;padding:0 1em"><h1>agent-17 · Legal clause extraction harness</h1><p>Extractor → verifier → judge on CUAD clause types. Offline rule/BM25 agents (no LLM yet). <a href="https://github.com/braylo777/agent-17-legal-eval-harness">GitHub</a></p><textarea id=t rows=10 style="width:100%">This Agreement shall be governed by the laws of the State of New York. Neither party may assign this Agreement without the prior written consent of the other party.</textarea><br><select id=c>{{CLAUSE_OPTIONS}}</select> <select id=m><option>rule</option><option>bm25</option></select> <button onclick="go()">Extract</button><pre id=o style="white-space:pre-wrap;background:#f4f4f4;padding:1em"></pre><script>async function go(){const r=await fetch('/api/extract',{method:'POST',body:JSON.stringify({text:t.value,clause:c.value,mode:m.value})});o.textContent=JSON.stringify(await r.json(),null,1)}</script>"####;

/// The demo page with the clause <select> filled in (the old Worker built it with a template literal).
pub fn page() -> String {
    let opts: String = crate::core::CLAUSE_NAMES.iter().map(|c| format!("<option>{c}</option>")).collect();
    TEMPLATE.replace("{{CLAUSE_OPTIONS}}", &opts)
}
