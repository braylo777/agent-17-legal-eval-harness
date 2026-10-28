import test from "node:test"; import assert from "node:assert/strict"; import fs from "node:fs"; import { evaluate } from "../evals/run.mjs";
const base = JSON.parse(fs.readFileSync(new URL("../evals/baseline.json", import.meta.url)));
test("no regression vs baseline (tolerance 0.02)", () => { const now = evaluate("test"); for (const [s, m] of Object.entries(base)) for (const [k, v] of Object.entries(m)) assert.ok(now[s][k] >= v - 0.02, `${s}.${k}`); });
test("verifier improves bm25 precision", () => { const r = evaluate("test"); assert.ok(r["bm25+verifier"].precision > r.bm25.precision); });
