// Regression gate: fails CI if any tracked metric drops more than 0.02 below evals/baseline.json.
import fs from "node:fs"; import { evaluate } from "./run.mjs";
const base = JSON.parse(fs.readFileSync(new URL("./baseline.json", import.meta.url))); const now = evaluate("test"); let bad = 0;
for (const [sys, m] of Object.entries(base)) for (const [k, v] of Object.entries(m)) { const cur = now[sys][k]; const ok = cur >= v - 0.02; if (!ok) bad++; console.log(`${ok ? "PASS" : "FAIL"} ${sys}.${k} ${cur} (baseline ${v})`); }
process.exit(bad ? 1 : 0);
