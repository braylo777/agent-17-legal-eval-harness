//! Multi-agent clause extraction: extractor -> verifier -> judge. No model calls.
//! Regexes follow JavaScript semantics (ASCII \b/\w, `.` stops at any JS line terminator, JS whitespace set).
use regex::Regex;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::OnceLock;

const DOT: &str = r"[^\n\r\x{2028}\x{2029}]";
pub const CLAUSE_NAMES: [&str; 8] = ["Governing Law", "Anti-Assignment", "Cap On Liability", "Audit Rights", "Insurance", "Termination For Convenience", "Exclusivity", "Non-Compete"];

pub struct Clause {
    pub rx: Regex,
    pub q: &'static str,
    pub cue: Regex,
}

fn c(p: &str) -> Regex {
    Regex::new(&format!("(?i){}", p.replace("DOT", DOT).replace(r"\b", r"(?-u:\b)").replace(r"\w", "[A-Za-z0-9_]"))).expect("clause regex")
}

pub fn clauses() -> &'static HashMap<&'static str, Clause> {
    static C: OnceLock<HashMap<&'static str, Clause>> = OnceLock::new();
    C.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("Governing Law", Clause { rx: c(r"govern(ed|ing)\bDOT*\blaws?\b|\blaws? of the (state|commonwealth|province)"), q: "governed construed laws state jurisdiction", cue: c("law") });
        m.insert("Anti-Assignment", Clause { rx: c(r"\bassign(ed|ment|s)?\bDOT*\b(without|prior written consent|not)\b|\bnot\bDOT*\bassign"), q: "assign assignment transfer consent without prior written", cue: c("assign|transfer") });
        m.insert("Cap On Liability", Clause { rx: c(r"(liabilit(y|ies))DOT*(exceed|limited to|in no event|aggregate)|in no eventDOT*liable"), q: "liability exceed aggregate limited amount paid in no event liable", cue: c("liab") });
        m.insert("Audit Rights", Clause { rx: c(r"\baudit|inspect(ion)? (the )?(books|records)"), q: "audit inspect books records examine", cue: c("audit|inspect|records") });
        m.insert("Insurance", Clause { rx: c(r"\binsurance\b|\binsured\b"), q: "insurance maintain coverage policy insured", cue: c("insur") });
        m.insert("Termination For Convenience", Clause { rx: c(r"terminat\wDOT*(for any reason|without cause|for convenience|at any time)".replace(r"\wDOT", r"\w*DOT").as_str()), q: "terminate any reason without cause convenience notice", cue: c("terminat") });
        m.insert("Exclusivity", Clause { rx: c(r"\bexclusive(ly)?\b|\bsole (and exclusive )?(supplier|distributor|provider)"), q: "exclusive exclusively sole distributor rights territory", cue: c("exclusiv|sole") });
        m.insert("Non-Compete", Clause { rx: c(r"\bcompet(e|ing|itive)\bDOT*\b(not|shall not|refrain)|non-compet"), q: "compete competing competitive business not engage", cue: c("compet") });
        m
    })
}

pub fn tok(s: &str) -> Vec<String> {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new("[a-z0-9]+").unwrap()).find_iter(&s.to_lowercase()).map(|m| m.as_str().to_string()).collect()
}

fn is_js_ws(ch: char) -> bool {
    matches!(ch, '\t' | '\n' | '\u{0B}' | '\u{0C}' | '\r' | ' ' | '\u{a0}' | '\u{1680}' | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}')
}

/// Same as JS `text.split(/(?<=[.;:])\s+(?=[A-Z(0-9])/).filter(s => s.trim().length > 3)`.
pub fn split_sentences(text: &str) -> Vec<String> {
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let (mut parts, mut start, mut i) = (vec![], 0usize, 0usize);
    while i < chars.len() {
        let prev_ok = i > 0 && matches!(chars[i - 1].1, '.' | ';' | ':');
        if prev_ok && is_js_ws(chars[i].1) {
            let mut j = i;
            while j < chars.len() && is_js_ws(chars[j].1) {
                j += 1;
            }
            if j < chars.len() && (chars[j].1.is_ascii_uppercase() || chars[j].1 == '(' || chars[j].1.is_ascii_digit()) {
                parts.push(&text[start..chars[i].0]);
                start = chars[j].0;
                i = j;
                continue;
            }
            i = j;
            continue;
        }
        i += 1;
    }
    parts.push(&text[start..]);
    parts.into_iter().filter(|s| s.trim_matches(is_js_ws).encode_utf16().count() > 3).map(String::from).collect()
}

pub fn bm25_scores(sents: &[String], q: &str) -> Vec<f64> {
    let (k1, b) = (1.2f64, 0.75f64);
    let docs: Vec<Vec<String>> = sents.iter().map(|s| tok(s)).collect();
    let n = docs.len() as f64;
    let avg = docs.iter().map(|d| d.len()).sum::<usize>() as f64 / n.max(1.0);
    let mut df: HashMap<&str, f64> = HashMap::new();
    for d in &docs {
        let mut seen = std::collections::HashSet::new();
        for t in d {
            if seen.insert(t.as_str()) {
                *df.entry(t.as_str()).or_insert(0.0) += 1.0;
            }
        }
    }
    let mut qt: Vec<String> = vec![];
    for t in tok(q) {
        if !qt.contains(&t) {
            qt.push(t);
        }
    }
    docs.iter()
        .map(|d| {
            let mut tf: HashMap<&str, f64> = HashMap::new();
            for t in d {
                *tf.entry(t.as_str()).or_insert(0.0) += 1.0;
            }
            let mut s = 0.0;
            for t in &qt {
                let Some(&f) = tf.get(t.as_str()) else { continue };
                let dfv = df[t.as_str()];
                let idf = (1.0 + (n - dfv + 0.5) / (dfv + 0.5)).ln();
                s += idf * f * (k1 + 1.0) / (f + k1 * (1.0 - b + b * d.len() as f64 / avg));
            }
            s
        })
        .collect()
}

/// Agent 1: extractor. mode "rule" | "bm25". Returns {i, sentence, score} or null.
pub fn extract(sents: &[String], clause: &str, mode: &str) -> Result<Value, String> {
    let cl = clauses().get(clause).ok_or("unknown clause")?;
    if mode == "rule" {
        return Ok(sents.iter().position(|s| cl.rx.is_match(s)).map_or(Value::Null, |i| json!({"i": i, "sentence": sents[i], "score": 1})));
    }
    let sc = bm25_scores(sents, cl.q);
    let mut i = 0;
    for j in 1..sc.len() {
        if sc[j] > sc[i] {
            i = j;
        }
    }
    Ok(if !sc.is_empty() && sc[i] > 0.0 { json!({"i": i, "sentence": sents[i], "score": sc[i]}) } else { Value::Null })
}

/// Agent 2: verifier. Rejects extractions lacking the clause's cue term or too short/long.
pub fn verify(ext: &Value, clause: &str) -> Value {
    if ext.is_null() {
        return json!({"ok": false, "reason": "none"});
    }
    let s = ext["sentence"].as_str().unwrap_or("");
    let n = s.encode_utf16().count();
    if n < 20 || n > 3000 {
        return json!({"ok": false, "reason": "length"});
    }
    if clauses()[clause].cue.is_match(s) { json!({"ok": true}) } else { json!({"ok": false, "reason": "missing cue"}) }
}

/// Agent 3a: token-F1 >= 0.3 vs best gold span.
pub fn judge_overlap(pred: Option<&str>, golds: &[String]) -> bool {
    let Some(pred) = pred.filter(|p| !p.is_empty()) else { return golds.is_empty() };
    if golds.is_empty() {
        return false;
    }
    let p = tok(pred);
    golds.iter().any(|g| {
        let gt = tok(g);
        let inter = p.iter().filter(|t| gt.contains(t)).count() as f64;
        if inter == 0.0 {
            return false;
        }
        let (pr, rc) = (inter / p.len() as f64, inter / gt.len() as f64);
        2.0 * pr * rc / (pr + rc) >= 0.3
    })
}

/// Agent 3b: gold span (first 40 chars) appears in pred, or pred in gold.
pub fn judge_contain(pred: Option<&str>, golds: &[String]) -> bool {
    let Some(pred) = pred.filter(|p| !p.is_empty()) else { return golds.is_empty() };
    if golds.is_empty() {
        return false;
    }
    let n = |x: &str| tok(x).join(" ");
    let p = n(pred);
    golds.iter().any(|g| {
        let gg = n(g);
        p.contains(&gg.chars().take(40).collect::<String>()) || gg.contains(&p)
    })
}

pub fn kappa(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len() as f64;
    let po = a.iter().zip(b).filter(|(x, y)| x == y).count() as f64 / n;
    let (pa, pb) = (a.iter().sum::<f64>() / n, b.iter().sum::<f64>() / n);
    let pe = pa * pb + (1.0 - pa) * (1.0 - pb);
    if pe == 1.0 { 1.0 } else { (po - pe) / (1.0 - pe) }
}

pub fn run_pipeline(sents: &[String], clause: &str, mode: &str) -> Result<Value, String> {
    let e = extract(sents, clause, mode)?;
    let v = verify(&e, clause);
    Ok(json!({"extraction": if v["ok"] == true { e.clone() } else { Value::Null }, "raw": e, "verifier": v}))
}

/// Pure router: (status, json); status 0 = serve the page. All JSON responses carry CORS allow-origin *.
pub fn handle(method: &str, path: &str, body_text: &str, now_ms: &dyn Fn() -> i64) -> (u16, Value) {
    if path == "/api/extract" && method == "POST" {
        let Ok(b) = serde_json::from_str::<Value>(body_text) else { return (400, json!({"error": "bad json"})) };
        let text: String = match b.get("text") {
            Some(Value::String(s)) if !s.is_empty() => s.clone(),
            Some(Value::Null) | Some(Value::Bool(false)) | None => String::new(),
            Some(Value::String(_)) => String::new(),
            Some(o) => o.to_string(),
        };
        let text: String = String::from_utf16_lossy(&text.encode_utf16().take(200000).collect::<Vec<u16>>());
        let clause = b.get("clause").and_then(|c| c.as_str()).filter(|c| clauses().contains_key(c));
        let Some(clause) = clause else { return (400, json!({"error": "unknown clause"})) };
        let sents = split_sentences(&text);
        let t0 = now_ms();
        let mode = b.get("mode");
        let m = if mode.and_then(|m| m.as_str()) == Some("bm25") { "bm25" } else { "rule" };
        let r = run_pipeline(&sents, clause, m).unwrap();
        let mode_out = match mode { Some(Value::String(s)) if !s.is_empty() => json!(s), Some(v) if !v.is_null() && *v != json!(false) && *v != json!(0) => v.clone(), _ => json!("rule") };
        return (200, json!({"clause": clause, "mode": mode_out, "extraction": r["extraction"], "raw": r["raw"], "verifier": r["verifier"], "ms": now_ms() - t0}));
    }
    if path == "/api/clauses" {
        return (200, json!(CLAUSE_NAMES));
    }
    if path == "/health" {
        return (200, json!({"ok": true}));
    }
    (0, Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn s() -> Vec<String> {
        ["This Agreement shall be governed by the laws of the State of Delaware.", "Neither party may assign this Agreement without prior written consent.", "Supplier shall maintain general liability insurance of $1,000,000.", "The weather is nice."].iter().map(|x| x.to_string()).collect()
    }
    #[test]
    fn extractors_verifier_judges() {
        let s = s();
        assert_eq!(clauses().len(), 8);
        assert_eq!(extract(&s, "Governing Law", "rule").unwrap()["i"], 0);
        assert_eq!(extract(&s, "Anti-Assignment", "rule").unwrap()["i"], 1);
        assert_eq!(extract(&s, "Insurance", "bm25").unwrap()["i"], 2);
        assert!(extract(&s, "Audit Rights", "rule").unwrap().is_null());
        assert!(extract(&s, "Foo", "rule").is_err());
        assert_eq!(verify(&json!({"sentence": format!("{} padding padding padding", s[3])}), "Insurance")["ok"], false);
        assert_eq!(verify(&json!({"sentence": s[2]}), "Insurance")["ok"], true);
        assert_eq!(verify(&Value::Null, "Insurance")["reason"], "none");
        assert!(run_pipeline(&s, "Audit Rights", "rule").unwrap()["extraction"].is_null());
        assert!(judge_overlap(Some(&s[0]), &["governed by the laws of the State of Delaware".to_string()]));
        assert!(!judge_overlap(Some(&s[3]), &["governed by the laws of Delaware".to_string()]));
        assert!(judge_overlap(None, &[]) && judge_contain(None, &[]));
        assert!(judge_contain(Some(&s[1]), &["Neither party may assign this Agreement".to_string()]));
        assert_eq!(kappa(&[1.0, 0.0, 1.0], &[1.0, 0.0, 1.0]), 1.0);
        assert!(kappa(&[1.0, 0.0, 1.0, 0.0], &[0.0, 1.0, 1.0, 0.0]) <= 0.0);
        assert!(bm25_scores(&s, "law").iter().all(|x| *x >= 0.0));
    }
    #[test]
    fn splitter_and_routes() {
        assert_eq!(split_sentences("Alpha beta. Gamma delta; (Epsilon) zeta: 7 eta. lower case x"), ["Alpha beta.", "Gamma delta;", "(Epsilon) zeta:", "7 eta. lower case x"]);
        let now = || 0i64;
        assert_eq!(handle("GET", "/health", "", &now).1, json!({"ok": true}));
        assert_eq!(handle("GET", "/api/clauses", "", &now).1.as_array().unwrap().len(), 8);
        assert_eq!(handle("POST", "/api/extract", "{nope", &now).0, 400);
        assert_eq!(handle("POST", "/api/extract", &json!({"clause": "Foo", "text": "x"}).to_string(), &now).0, 400);
        let (st, r) = handle("POST", "/api/extract", &json!({"clause": "Insurance", "text": s().join(" ")}).to_string(), &now);
        assert_eq!((st, r["extraction"]["i"].as_i64(), r["mode"].as_str()), (200, Some(2), Some("rule")));
        assert_eq!(handle("GET", "/", "", &now).0, 0);
    }
}

#[cfg(test)]
mod page_tests {
    #[test]
    fn page_has_all_clause_options() {
        let p = crate::page::page();
        assert!(!p.contains("{{CLAUSE_OPTIONS}}") && p.contains("<option>Non-Compete</option>") && p.matches("<option>").count() == 8 + 2);
    }
}
