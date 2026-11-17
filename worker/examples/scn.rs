// Parity helper: all dataset predictions + splitter cases; prints JSON.
use agent_17_legal_eval_harness::core::*;
use serde_json::{json, Value};
fn main() {
    let data: Value = serde_json::from_str(&std::fs::read_to_string("../data/cuad_subset.json").unwrap()).unwrap();
    let mut preds: Vec<Value> = vec![];
    for d in data["docs"].as_array().unwrap() {
        let sents: Vec<String> = d["sentences"].as_array().unwrap().iter().map(|s| s.as_str().unwrap().to_string()).collect();
        for c in CLAUSE_NAMES {
            for mode in ["rule", "bm25"] {
                let r = run_pipeline(&sents, c, mode).unwrap();
                let (raw, ex) = (&r["raw"], &r["extraction"]);
                let score = if raw.is_null() { Value::Null } else { json!((raw["score"].as_f64().unwrap() * 1e9).round() / 1e9) };
                preds.push(json!([&d["id"].as_str().unwrap()[..12], c, mode, raw.get("i").cloned().unwrap_or(Value::Null), ex.get("i").cloned().unwrap_or(Value::Null), score, r["verifier"]["ok"], r["verifier"].get("reason").cloned().unwrap_or(Value::Null)]));
            }
        }
    }
    let texts = ["Alpha beta. Gamma delta; (Epsilon) zeta: 7 eta. lower case x", "A\n\n  B. C\u{a0}D. E\u{2003}F.  G;H: I", "Short. Ok? This one is OK. And then 12 things; Fine.", "Line one.\r\nLine two. Line three", "No terminator here at all", "x. y. Z. 1. (a) b"];
    println!("{}", serde_json::to_string(&json!({"preds": preds, "split": texts.iter().map(|t| split_sentences(t)).collect::<Vec<_>>(), "texts": texts})).unwrap());
}
