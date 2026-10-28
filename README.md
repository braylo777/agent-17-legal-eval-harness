# agent-17 · Legal clause-extraction eval harness

**Live demo:** https://agent-17-legal-eval-harness.braylo777.workers.dev  ·  API: `POST /api/extract {text, clause, mode: "rule"|"bm25"}`

## What
A multi-agent eval harness for contract clause extraction on a public CUAD subset:
1. **Extractor**: finds the sentence holding a clause (rule baseline or BM25 baseline; LLM extractor after quota reset).
2. **Verifier**: rejects extractions missing the clause's cue term or with a bad length, so the system abstains instead of guessing.
3. **Judge**: two independent judges (token-F1 ≥ 0.3 overlap judge *A*, normalized-containment judge *B*) score each output against CUAD gold spans. Their agreement and Cohen's κ are reported, so judge reliability is measured, not assumed.
4. **Regression gate**: `npm run gate` (also wired as a GitHub Actions job in `ci/github-actions-ci.yml`; copy it to `.github/workflows/` to enable, since the box token lacks the `workflow` scope) fails the build if any tracked metric drops more than 0.02 below `evals/baseline.json`.

## Data
CUAD v1 test split (Atticus Project, CC BY 4.0) via the `chenghao/cuad_qa` HF mirror. The first 40 contracts (alphabetical) were split into **10 dev** (used to write the rules) and **30 test** contracts. 8 clause types: Governing Law, Anti-Assignment, Cap On Liability, Audit Rights, Insurance, Termination For Convenience, Exclusivity, Non-Compete. A (contract, clause) pair with no CUAD answer counts as absent, so precision counts false alarms.

## Results (real, from `evals/results.json`, measured 2026-10-27)
| system (test, 30 contracts × 8 clauses = 240 cases) | precision | recall | F1 | judge accuracy | judge agreement | Cohen κ (judge A vs B) |
|---|---|---|---|---|---|---|
| rule | 0.585 | 0.65 | 0.615 | 0.729 | 0.883 | 0.735 |
| rule+verifier | 0.661 | 0.65 | 0.655 | 0.729 | 0.892 | 0.756 |
| bm25 | 0.464 | 0.821 | 0.593 | 0.538 | 0.917 | 0.834 |
| bm25+verifier | 0.614 | 0.735 | 0.669 | 0.692 | 0.958 | 0.906 |

Per-clause F1, bm25+verifier (test): Governing Law 0.962 · Anti-Assignment 0.809 · Cap On Liability 0.552 · Audit Rights 0.667 · Insurance 0.857 · Termination For Convenience 0.563 · Exclusivity 0.286 · Non-Compete 0

Dev split (10 contracts, used to write the rules): bm25+verifier F1 0.69, rule+verifier F1 0.605.

What this shows: the verifier adds **+0.15 precision** to BM25 (0.464 → 0.614) and lifts F1 to 0.669. The two judges agree on 89–96% of cases (κ 0.74–0.91). Weakest clauses are Exclusivity and Non-Compete, where keyword cues show up in unrelated sentences.

## How to run
```bash
npm test          # 19 offline tests (unit + regression gate), no model calls
npm run eval      # recompute evals/results.json
npm run gate      # regression gate vs evals/baseline.json (also runs in CI)
npm run build     # bundle dist/worker.js;  or: npx wrangler deploy
```
No dependencies. Free tier only. Rollback tag: `v0-rollback`.

## Pending (after the Workers AI quota reset, 7 PM CT Oct 28 2026)
- LLM extractor (`@cf/meta/llama-3.1-8b-instruct`) on the same 240 test cases, gated by the same verifier.
- LLM-as-judge, plus its agreement and κ against judges A and B and against gold.
- Add the LLM rows to the table and baseline.json so the gate covers them.
Not yet run, so no numbers are reported for them.
