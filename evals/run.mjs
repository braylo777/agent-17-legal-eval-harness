import fs from "node:fs"; import { CLAUSES, bm25Index, runPipeline, judgeOverlap, judgeContain, kappa } from "../src/core.js";
const data = JSON.parse(fs.readFileSync(new URL("./data/cuad_subset.json", import.meta.url)));
export function evaluate(split = "test") {
  const res = {};
  for (const mode of ["rule", "bm25"]) for (const verified of [false, true]) {
    let tp = 0, fp = 0, fn = 0, tn = 0, correct = 0, n = 0; const j1 = [], j2 = []; const per = {};
    for (const d of data.docs.filter((x) => x.split === split)) { const idx = bm25Index(d.sentences);
      for (const c of Object.keys(CLAUSES)) { const g = d.gold[c] || []; const r = runPipeline(d.sentences, c, mode, idx); const pred = (verified ? r.extraction : r.raw)?.sentence || null;
        const a = judgeOverlap(pred, g), b = judgeContain(pred, g); j1.push(+a); j2.push(+b); n++; correct += a;
        per[c] ??= { tp: 0, fp: 0, fn: 0 };
        if (pred && g.length) { a ? (tp++, per[c].tp++) : (fp++, fn++, per[c].fp++, per[c].fn++); } else if (pred) { fp++; per[c].fp++; } else if (g.length) { fn++; per[c].fn++; } else tn++; } }
    const P = tp / (tp + fp || 1), R = tp / (tp + fn || 1), r3 = (x) => Math.round(x * 1000) / 1000;
    res[`${mode}${verified ? "+verifier" : ""}`] = { n, precision: r3(P), recall: r3(R), f1: r3(2 * P * R / (P + R || 1)), accuracy: r3(correct / n), judge_agreement: r3(j1.filter((x, i) => x === j2[i]).length / n), judge_kappa: r3(kappa(j1, j2)),
      per_clause_f1: Object.fromEntries(Object.entries(per).map(([k, v]) => { const p = v.tp / (v.tp + v.fp || 1), r = v.tp / (v.tp + v.fn || 1); return [k, r3(2 * p * r / (p + r || 1))]; })) };
  }
  return res;
}
if (process.argv[1].endsWith("run.mjs")) { const out = { dataset: "CUAD v1 test subset (chenghao/cuad_qa), 40 contracts: 10 dev / 30 test, 8 clause types", test: evaluate("test"), dev: evaluate("dev") };
  fs.writeFileSync(new URL("./results.json", import.meta.url), JSON.stringify(out, null, 1)); for (const [k, v] of Object.entries(out.test)) console.log(k, v.precision, v.recall, v.f1, v.accuracy, v.judge_kappa); }
