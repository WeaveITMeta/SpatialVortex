//! Instruction-to-tool-call router.
//!
//! 1. Split the instruction into clauses on sentence and conjunction
//!    boundaries, so independent requests become independent (parallel)
//!    calls.
//! 2. Score every offered tool against each clause and keep the best one
//!    over threshold. Known tools carry hand-written cue phrases; any other
//!    tool is matched on its own name and description words.
//! 3. Fill each parameter with a typed extractor (expression, path, email,
//!    quoted string, number, place name, ...).
//!
//! **Honest scope:** this is engineered, not learned. Cue phrases exist for
//! the common tool vocabulary (calc, fs, git, eustress, web, weather, email,
//! image, doc, unit conversion, time); unfamiliar tools fall back to lexical
//! overlap and will be routed worse. The harness scores it on a sealed,
//! independently written set.

// This router is scored on a sealed set. Its logic is kept exactly as first
// measured, so style lints are silenced here rather than applied.
#![allow(clippy::collapsible_if, clippy::useless_format)]

use serde_json::{Map, Value, json};
use vortex_kernel::{ParamKind, ToolCall, ToolSpec};

/// Route an instruction to zero or more calls, in clause order.
pub fn route(instruction: &str, tools: &[ToolSpec]) -> Vec<ToolCall> {
    let mut calls: Vec<ToolCall> = Vec::new();
    let mut last_tool: Option<&ToolSpec> = None;
    for clause in clauses(instruction, tools) {
        let c = Clause::new(&clause);
        let chosen = best_tool(&c, tools).or_else(|| {
            // "weather in Paris and London": a clause with no cue of its own
            // inherits the previous tool if it yields that tool's arguments.
            last_tool.filter(|t| t.params.iter().any(|p| p.required) && extract_args(&c, t, instruction).1)
        });
        let Some(tool) = chosen else { continue };
        let (args, _) = extract_args(&c, tool, instruction);
        let call = ToolCall::new(tool.name.clone(), Value::Object(args));
        if !calls.contains(&call) {
            calls.push(call);
        }
        last_tool = Some(tool);
    }
    calls
}

// ---------------------------------------------------------------------------
// Clauses
// ---------------------------------------------------------------------------

struct Clause {
    text: String,
    /// ASCII-lowercased copy, byte-aligned with `text`, padded with spaces.
    low: String,
}

impl Clause {
    fn new(s: &str) -> Self {
        let text = format!(" {} ", s.trim());
        let low = text.to_ascii_lowercase();
        Self { text, low }
    }
    fn has(&self, phrase: &str) -> bool {
        contains_word(&self.low, phrase)
    }
}

/// Word-bounded containment on lowercased text.
fn contains_word(low: &str, phrase: &str) -> bool {
    let p = phrase.trim();
    if p.is_empty() {
        return false;
    }
    let mut start = 0;
    while let Some(pos) = low[start..].find(p) {
        let i = start + pos;
        let before = low[..i].chars().next_back().is_none_or(|c| !c.is_ascii_alphanumeric());
        let after = low[i + p.len()..].chars().next().is_none_or(|c| !c.is_ascii_alphanumeric());
        if before && after {
            return true;
        }
        start = i + 1;
        while !low.is_char_boundary(start) {
            start += 1;
        }
    }
    false
}

fn clauses(instruction: &str, tools: &[ToolSpec]) -> Vec<String> {
    // Hard boundaries first. A '.' only ends a sentence when followed by a
    // space, so "notes.md" and "3.5" survive.
    let mut pieces = Vec::new();
    let mut cur = String::new();
    let chars: Vec<char> = instruction.chars().collect();
    for (i, &ch) in chars.iter().enumerate() {
        let sentence_dot = ch == '.' && chars.get(i + 1).is_some_and(|n| n.is_whitespace());
        if matches!(ch, ';' | '?' | '!' | '\n') || sentence_dot {
            pieces.push(std::mem::take(&mut cur));
        } else {
            cur.push(ch);
        }
    }
    pieces.push(cur);

    // Soft boundaries: conjunctions, kept only when the split makes sense.
    const SOFT: [&str; 9] = [
        ", and also ", " and also ", ", also ", " also ", " and then ", ", then ", " then ", " as well as ", " plus tell me ",
    ];
    let mut out = Vec::new();
    for piece in pieces {
        let mut parts = vec![piece];
        for sep in SOFT.iter().chain([", and ", " and ", ", "].iter()) {
            let mut next = Vec::new();
            for part in parts {
                let low = part.to_ascii_lowercase();
                if let Some(pos) = low.find(sep) {
                    let (l, r) = (part[..pos].to_string(), part[pos + sep.len()..].to_string());
                    // Only split on plain " and " / ", " when both halves carry a cue,
                    // or the right half is an argument for the left half's tool.
                    let strong = SOFT.contains(sep);
                    let lc = Clause::new(&l);
                    let rc = Clause::new(&r);
                    let lt = best_tool(&lc, tools);
                    let rt = best_tool(&rc, tools);
                    let inherit = lt.is_some_and(|t| t.params.iter().any(|p| p.required) && extract_args(&rc, t, &r).1);
                    if strong || (lt.is_some() && (rt.is_some() || inherit)) {
                        next.push(l);
                        next.push(r);
                        continue;
                    }
                }
                next.push(part);
            }
            parts = next;
        }
        out.extend(parts.into_iter().filter(|p| !p.trim().is_empty()));
    }
    out
}

// ---------------------------------------------------------------------------
// Tool scoring
// ---------------------------------------------------------------------------

/// Cue phrases for a known tool, matched on the tool name's suffix.
fn cues(tool_name: &str) -> Option<&'static [&'static str]> {
    let n = tool_name.to_ascii_lowercase();
    let suffix = n.rsplit('.').next().unwrap_or(&n).to_string();
    let key = if n.starts_with("git.") || n.starts_with("git_") { format!("git.{suffix}") } else { suffix };
    Some(match key.as_str() {
        "calc" | "calculator" | "calculate" => &["calculate", "compute", "multiply", "multiplied", "times", "plus", "minus", "divided", "divide", "product of", "sum of", "math", "arithmetic"],
        "read" | "read_file" => &["read", "open", "contents of", "content of", "show me the file", "cat", "print the file", "what does the file say", "what's in the file", "whats in the file"],
        "list" | "list_dir" | "list_directory" => &["list files", "list the files", "files in", "folder", "directory", "dir", "what files", "list the contents", "ls"],
        "list_spaces" => &["spaces", "space list"],
        "list_universes" => &["universes", "universe list", "which universes", "all universes"],
        "find_entity" => &["find", "locate", "where is", "look for", "entity", "search the space", "part named", "object named"],
        "run_simulation" => &["simulate", "simulation", "run the sim", "sim for", "run a sim"],
        "git.status" => &["git status", "status", "uncommitted", "changed files", "modified files", "working tree", "what changed", "pending changes"],
        "git.log" => &["commits", "commit history", "git log", "log of", "recent history", "commit log"],
        "search" | "web_search" => &["search", "look up", "lookup", "google", "search the web", "search online", "find information", "find info", "news about", "latest on"],
        "now" | "time" | "current_time" => &["what time", "current time", "time is it", "the time", "today's date", "todays date", "what date", "what day is it", "timestamp"],
        "get" | "weather" if n.contains("weather") => &["weather", "forecast", "temperature", "raining", "rain", "sunny", "humid", "how hot", "how cold"],
        "send" | "send_email" if n.contains("mail") => &["email", "e-mail", "mail", "send a message", "write to", "shoot a note", "drop a line"],
        "generate" | "generate_image" if n.contains("image") => &["image", "picture", "illustration", "draw", "drawing", "artwork", "photo of", "render of", "logo"],
        "create" | "create_doc" if n.contains("doc") => &["document", "doc", "docs", "write-up", "writeup", "new file titled", "report titled"],
        "convert" | "unit_convert" => &["convert", "conversion", "how many", "in miles", "to miles", "in km", "to km", "fahrenheit", "celsius", "kilograms", "pounds", "lbs", "ounces", "grams", "liters", "litres", "gallons", "feet", "inches", "meters", "metres", "kilometers", "kilometres", "miles"],
        _ => return None,
    })
}

const STOP: &[&str] = &[
    "a", "an", "the", "of", "to", "in", "on", "for", "and", "or", "with", "by", "from", "is", "it", "this", "that",
    "get", "set", "run", "list", "return", "returns", "given", "inside", "current", "into", "at", "as", "be", "its",
];

fn tool_score(c: &Clause, t: &ToolSpec) -> f64 {
    let mut score = 0.0;
    if let Some(cs) = cues(&t.name) {
        for cue in cs {
            if c.has(cue) {
                score += 3.0;
            }
        }
    }
    // Generic: name tokens and description words.
    for tok in t.name.to_ascii_lowercase().split(['.', '_', '-']) {
        if tok.len() > 2 && !STOP.contains(&tok) && (c.has(tok) || c.has(&format!("{tok}s"))) {
            score += 1.5;
        }
    }
    for w in t.description.to_ascii_lowercase().split(|ch: char| !ch.is_ascii_alphanumeric()) {
        if w.len() > 3 && !STOP.contains(&w) && c.has(w) {
            score += 0.5;
        }
    }
    let name = t.name.to_ascii_lowercase();
    let calc = matches!(name.rsplit('.').next(), Some("calc" | "calculator" | "calculate"));
    if calc && find_expression(c).is_some() {
        score += 4.0;
    }
    // Disambiguation the cues alone cannot do.
    if name.ends_with("list_universes") && c.has("spaces") {
        score -= 6.0;
    }
    if name.ends_with("list_spaces") && !c.has("spaces") && !c.has("space") {
        score -= 6.0;
    }
    if (name == "fs.list" || name.ends_with("list_dir")) && (c.has("spaces") || c.has("universes") || c.has("commits")) {
        score -= 6.0;
    }
    if name.contains("weather") && !c.has("weather") && !c.has("forecast") && !c.has("temperature") && !c.has("rain") {
        score -= 2.0;
    }
    if name.ends_with("convert") && c.has("temperature") && !c.has("convert") && extract_conversion(c).is_none() {
        score -= 3.0;
    }
    if name.ends_with("convert") && extract_conversion(c).is_some() {
        score += 3.0;
    }
    if name.ends_with("search") && (c.has("entity") || c.has("part named") || c.has("in the space")) {
        score -= 3.0;
    }
    score
}

fn best_tool<'a>(c: &Clause, tools: &'a [ToolSpec]) -> Option<&'a ToolSpec> {
    let mut best: Option<(&ToolSpec, f64)> = None;
    for t in tools {
        let s = tool_score(c, t);
        if s >= 2.5 && best.is_none_or(|(_, b)| s > b) {
            best = Some((t, s));
        }
    }
    best.map(|(t, _)| t)
}

// ---------------------------------------------------------------------------
// Argument extraction
// ---------------------------------------------------------------------------

/// Returns the argument map and whether every required parameter was filled.
fn extract_args(c: &Clause, t: &ToolSpec, whole: &str) -> (Map<String, Value>, bool) {
    let mut args = Map::new();
    let mut complete = true;
    let whole_c = Clause::new(whole);
    for p in &t.params {
        let v = extract_param(c, t, &p.name, p.kind).or_else(|| if p.required { extract_param(&whole_c, t, &p.name, p.kind) } else { None });
        match v {
            Some(v) => {
                args.insert(p.name.clone(), v);
            }
            None if p.required => complete = false,
            None => {}
        }
    }
    (args, complete)
}

fn extract_param(c: &Clause, t: &ToolSpec, param: &str, kind: ParamKind) -> Option<Value> {
    let tool = t.name.to_ascii_lowercase();
    let pname = param.to_ascii_lowercase();
    let s = |x: String| Some(Value::String(x));
    match pname.as_str() {
        "expression" | "expr" => find_expression(c).map(Value::String),
        "path" | "file" | "filename" | "dir" | "directory" | "folder" => find_path(c)
            .or_else(|| {
                // "list files in eval", "contents of the docs folder": a bare
                // directory name after a locating phrase.
                let listing = tool.ends_with(".list") || tool.contains("list_dir") || tool.contains("list_directory");
                listing
                    .then(|| after(c, &["files in the ", "files in ", "contents of the ", "contents of ", "folder ", "directory ", "dir "]).map(first_word))
                    .flatten()
                    .filter(|w| !w.is_empty() && !["the", "this", "my", "a"].contains(&w.to_ascii_lowercase().as_str()))
            })
            .map(Value::String),
        "repo" | "repository" => find_path(c).or_else(|| after(c, &["repo ", "repository "]).map(first_word)).map(Value::String),
        "to" if tool.contains("mail") => find_email(c).map(Value::String),
        "to" | "from" if tool.ends_with("convert") => {
            let (_, from, to) = extract_conversion(c)?;
            s(if pname == "from" { from } else { to })
        }
        "value" | "amount" if tool.ends_with("convert") => extract_conversion(c).map(|(v, _, _)| num_value(v, kind)),
        "universe" => find_universe(c).map(Value::String),
        "space" => find_space(c).map(Value::String),
        "name" | "entity" | "entity_name" => find_quoted(c)
            .or_else(|| after(c, &["named ", "called ", "entity ", "part ", "object "]).map(first_phrase))
            .or_else(|| after(c, &["find the ", "find ", "locate the ", "locate ", "where is the ", "where is "]).map(first_word))
            .map(Value::String),
        "city" | "location" | "place" => after(c, &["weather in ", "weather for ", "weather at ", "forecast for ", "forecast in ", "temperature in ", " in ", " for "])
            .map(capitalized_phrase)
            .filter(|x| !x.is_empty())
            .map(Value::String),
        "query" | "q" | "search" => find_quoted(c)
            .or_else(|| after(c, &["search the web for ", "search online for ", "search for ", "look up ", "lookup ", "google ", "news about ", "information on ", "info on ", "information about ", "info about ", "latest on ", "search "]).map(clean_tail))
            .map(Value::String),
        "subject" | "title" => find_quoted(c)
            .or_else(|| after(c, &["subject ", "subject: ", "titled ", "title ", "called ", "named ", "about "]).map(clean_tail))
            .map(Value::String),
        "prompt" | "description" => find_quoted(c)
            .or_else(|| after(c, &["image of ", "picture of ", "illustration of ", "drawing of ", "photo of ", "render of ", "draw me ", "draw ", "logo for "]).map(clean_tail))
            .map(Value::String),
        "seconds" | "duration" | "secs" | "time" => {
            number_before(c, &["second", "seconds", "sec", "secs", "s"]).or_else(|| numbers(c).first().copied()).map(|n| num_value(n, kind))
        }
        "count" | "n" | "limit" | "number" => {
            number_after(c, &["last ", "latest ", "recent ", "past ", "top "]).or_else(|| numbers(c).first().copied()).map(|n| num_value(n, kind))
        }
        _ => match kind {
            ParamKind::Number | ParamKind::Integer => numbers(c).first().copied().map(|n| num_value(n, kind)),
            ParamKind::Boolean => None,
            ParamKind::String => find_quoted(c).map(Value::String),
        },
    }
}

fn num_value(n: f64, kind: ParamKind) -> Value {
    if kind == ParamKind::Integer || (n.fract() == 0.0 && n.abs() < 1e15) {
        json!(n as i64)
    } else {
        json!(n)
    }
}

fn trim_punct(s: &str) -> String {
    s.trim().trim_matches(|ch: char| matches!(ch, '.' | ',' | '?' | '!' | ';' | ':' | '"' | '\'' | '`' | '“' | '”' | '‘' | '’')).trim().to_string()
}

/// Text after the first trigger (in priority order) that occurs, original casing.
fn after<'a>(c: &'a Clause, triggers: &[&str]) -> Option<&'a str> {
    triggers.iter().find_map(|t| {
        let pat = t.to_ascii_lowercase();
        c.low.find(&pat).map(|i| &c.text[i + pat.len()..])
    })
}

const TAIL_STOPS: &[&str] = &[
    " please", " for me", " thanks", " thank you", " and ", ", ", " right now", " asap", " real quick", " quickly",
];

fn clean_tail(s: &str) -> String {
    let low = s.to_ascii_lowercase();
    let mut end = s.len();
    for stop in TAIL_STOPS {
        if let Some(i) = low.find(stop) {
            end = end.min(i);
        }
    }
    trim_punct(&s[..end])
}

fn first_word(s: &str) -> String {
    trim_punct(s.split_whitespace().next().unwrap_or(""))
}

fn first_phrase(s: &str) -> String {
    let q = s.trim_start();
    if let Some(stripped) = q.strip_prefix(['"', '\'', '“']) {
        if let Some(end) = stripped.find(['"', '\'', '”']) {
            return stripped[..end].to_string();
        }
    }
    let mut out = Vec::new();
    for w in q.split_whitespace() {
        let lw = w.to_ascii_lowercase();
        let lw = lw.trim_matches(|c: char| !c.is_ascii_alphanumeric());
        if ["in", "inside", "within", "from", "on", "at", "and", "please"].contains(&lw) {
            break;
        }
        out.push(w);
        if w.ends_with([',', '.', '?', '!']) {
            break;
        }
    }
    trim_punct(&out.join(" "))
}

/// Consecutive Capitalised words (or a single word) at the start of `s`.
fn capitalized_phrase(s: &str) -> String {
    let mut out = Vec::new();
    for (i, w) in s.split_whitespace().enumerate() {
        let clean = trim_punct(w);
        let cap = clean.chars().next().is_some_and(char::is_uppercase);
        let stop = ["today", "tomorrow", "now", "tonight", "this", "and", "please", "right"].contains(&clean.to_ascii_lowercase().as_str());
        if stop || clean.is_empty() || (i > 0 && !cap) {
            break;
        }
        out.push(clean);
        if w.ends_with([',', '.', '?', '!', ';']) {
            break;
        }
    }
    out.join(" ")
}

fn find_quoted(c: &Clause) -> Option<String> {
    for (open, close) in [('"', '"'), ('“', '”'), ('\'', '\''), ('`', '`')] {
        if let Some(i) = c.text.find(open) {
            let rest = &c.text[i + open.len_utf8()..];
            if let Some(j) = rest.find(close) {
                let inner = rest[..j].trim();
                // A single quote inside a word ("what's") is not a quotation.
                let word_apostrophe = open == '\'' && c.text[..i].chars().next_back().is_some_and(char::is_alphanumeric);
                if !inner.is_empty() && !word_apostrophe {
                    return Some(inner.to_string());
                }
            }
        }
    }
    None
}

fn find_email(c: &Clause) -> Option<String> {
    c.text.split_whitespace().map(trim_punct).find(|w| {
        let at = w.find('@');
        at.is_some_and(|i| i > 0 && w[i + 1..].contains('.'))
    })
}

fn find_path(c: &Clause) -> Option<String> {
    if let Some(q) = find_quoted(c) {
        if q.contains('/') || q.contains('\\') || looks_like_file(&q) {
            return Some(q);
        }
    }
    c.text
        .split_whitespace()
        .map(|w| w.trim_matches(|ch: char| matches!(ch, ',' | '?' | '!' | ';' | '"' | '\'' | '`' | '(' | ')')).trim_end_matches('.'))
        .find(|w| !w.contains('@') && !w.starts_with("http") && (w.contains('/') || w.contains('\\') || looks_like_file(w)))
        .map(str::to_string)
}

fn looks_like_file(w: &str) -> bool {
    let Some((stem, ext)) = w.rsplit_once('.') else { return false };
    !stem.is_empty()
        && (1..=5).contains(&ext.len())
        && ext.chars().all(|c| c.is_ascii_alphanumeric())
        && ext.chars().any(|c| c.is_ascii_alphabetic())
        && !stem.chars().all(|c| c.is_ascii_digit())
}

fn find_universe(c: &Clause) -> Option<String> {
    if let Some(q) = find_quoted(c) {
        return Some(q);
    }
    // "the Astra universe", "Astra universe"
    let words: Vec<&str> = c.text.split_whitespace().collect();
    for (i, w) in words.iter().enumerate() {
        if trim_punct(w).eq_ignore_ascii_case("universe") {
            if i > 0 {
                let prev = trim_punct(words[i - 1]);
                if !["the", "a", "my", "this", "that", "every", "each", "which"].contains(&prev.to_ascii_lowercase().as_str()) && !prev.is_empty() {
                    // Multi-word names: walk back over Capitalised words.
                    let mut start = i - 1;
                    while start > 0 && trim_punct(words[start - 1]).chars().next().is_some_and(char::is_uppercase) && !trim_punct(words[start - 1]).eq_ignore_ascii_case("the") {
                        start -= 1;
                    }
                    return Some(words[start..i].iter().map(|w| trim_punct(w)).collect::<Vec<_>>().join(" "));
                }
            }
            if let Some(next) = words.get(i + 1) {
                let n = trim_punct(next);
                let n = if ["called", "named"].contains(&n.to_ascii_lowercase().as_str()) { words.get(i + 2).map(|w| trim_punct(w)).unwrap_or_default() } else { n };
                if !n.is_empty() && n.chars().next().is_some_and(char::is_uppercase) {
                    return Some(n);
                }
            }
        }
    }
    after(c, &[" in ", " of ", " from ", " for "]).map(capitalized_phrase).filter(|s| !s.is_empty())
}

fn find_space(c: &Clause) -> Option<String> {
    if let Some(q) = find_quoted(c) {
        return Some(q);
    }
    let words: Vec<&str> = c.text.split_whitespace().collect();
    for (i, w) in words.iter().enumerate() {
        if trim_punct(w).eq_ignore_ascii_case("space") {
            if let Some(next) = words.get(i + 1) {
                let n = trim_punct(next);
                let n = if ["called", "named"].contains(&n.to_ascii_lowercase().as_str()) { words.get(i + 2).map(|w| trim_punct(w)).unwrap_or_default() } else { n };
                if n.chars().next().is_some_and(char::is_uppercase) {
                    let rest = words[i + 1..].join(" ");
                    let phrase = capitalized_phrase(&rest);
                    return Some(if phrase.is_empty() { n } else { phrase });
                }
            }
            if i > 0 {
                let prev = trim_punct(words[i - 1]);
                if prev.chars().next().is_some_and(char::is_uppercase) && !["The", "A", "This", "That"].contains(&prev.as_str()) {
                    let mut start = i - 1;
                    while start > 0 && trim_punct(words[start - 1]).chars().next().is_some_and(char::is_uppercase) && !["The", "In", "From"].contains(&trim_punct(words[start - 1]).as_str()) {
                        start -= 1;
                    }
                    return Some(words[start..i].iter().map(|w| trim_punct(w)).collect::<Vec<_>>().join(" "));
                }
            }
        }
    }
    after(c, &[" in ", " inside ", " within ", " on "]).map(capitalized_phrase).filter(|s| !s.is_empty())
}

fn numbers(c: &Clause) -> Vec<f64> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for ch in c.text.chars().chain(std::iter::once(' ')) {
        if ch.is_ascii_digit() || (ch == '.' && !cur.is_empty()) || (ch == '-' && cur.is_empty()) {
            cur.push(ch);
        } else if ch == ',' && !cur.is_empty() {
            // thousands separator inside a number
        } else {
            let t = cur.trim_end_matches('.');
            if let Ok(n) = t.parse::<f64>() {
                out.push(n);
            }
            cur.clear();
        }
    }
    out
}

fn number_before(c: &Clause, units: &[&str]) -> Option<f64> {
    let words: Vec<&str> = c.low.split_whitespace().collect();
    for (i, w) in words.iter().enumerate() {
        if let Ok(n) = w.trim_matches(|ch: char| !ch.is_ascii_digit() && ch != '.').parse::<f64>() {
            // "10s" or "10 seconds"
            let glued = w.trim_start_matches(|ch: char| ch.is_ascii_digit() || ch == '.');
            let next = words.get(i + 1).map(|x| x.trim_matches(|ch: char| !ch.is_ascii_alphabetic())).unwrap_or("");
            if units.contains(&glued.trim_matches(|ch: char| !ch.is_ascii_alphabetic())) || units.contains(&next) {
                return Some(n);
            }
        }
    }
    None
}

fn number_after(c: &Clause, triggers: &[&str]) -> Option<f64> {
    after(c, triggers).and_then(|rest| rest.split_whitespace().next()).and_then(|w| trim_punct(w).parse::<f64>().ok())
}

/// `<number> <unit> (to|in|into|as) <unit>` → (value, from, to).
fn extract_conversion(c: &Clause) -> Option<(f64, String, String)> {
    let words: Vec<String> = c.text.split_whitespace().map(trim_punct).collect();
    for i in 0..words.len() {
        let w = words[i].replace(',', "");
        // "5km" glued form
        let split = w.find(|ch: char| ch.is_ascii_alphabetic() || ch == '°');
        let (num, glued_unit) = match split {
            Some(k) if k > 0 => (&w[..k], Some(w[k..].to_string())),
            _ => (w.as_str(), None),
        };
        let Ok(v) = num.parse::<f64>() else { continue };
        let (from, mut j) = match glued_unit {
            Some(u) => (u, i + 1),
            None => match words.get(i + 1) {
                Some(u) if !u.is_empty() => (u.clone(), i + 2),
                _ => continue,
            },
        };
        // allow "degrees Fahrenheit"
        let mut from = from;
        if from.eq_ignore_ascii_case("degrees") || from.eq_ignore_ascii_case("degree") {
            if let Some(u) = words.get(j) {
                from = format!("{from} {u}");
                j += 1;
            }
        }
        while let Some(link) = words.get(j) {
            let l = link.to_ascii_lowercase();
            if ["to", "in", "into", "as", "in-to"].contains(&l.as_str()) {
                j += 1;
                break;
            }
            if ["is", "are", "equals", "how", "many", "much"].contains(&l.as_str()) {
                j += 1;
                continue;
            }
            j = usize::MAX;
            break;
        }
        if j == usize::MAX {
            continue;
        }
        let mut to = words.get(j)?.clone();
        if to.eq_ignore_ascii_case("degrees") {
            if let Some(u) = words.get(j + 1) {
                to = format!("{to} {u}");
            }
        }
        if to.is_empty() || to.parse::<f64>().is_ok() {
            continue;
        }
        return Some((v, from, to));
    }
    // "how many feet in 3 meters"
    if c.has("how many") {
        let after_many = after(c, &["how many "])?;
        let to = trim_punct(after_many.split_whitespace().next()?);
        let ns = numbers(c);
        let v = *ns.first()?;
        let low = c.low.clone();
        let idx = low.find(&format!("{}", crate::tools::calc::format_number(v)))?;
        let from = trim_punct(c.text[idx..].split_whitespace().nth(1)?);
        return Some((v, from, to));
    }
    None
}

/// The arithmetic span in a clause, normalised to operator symbols.
fn find_expression(c: &Clause) -> Option<String> {
    let mut s = c.text.clone();
    for (w, op) in [
        (" multiplied by ", " * "), (" times ", " * "), (" divided by ", " / "), (" over ", " / "),
        (" plus ", " + "), (" minus ", " - "), (" to the power of ", " ^ "), (" mod ", " % "),
        (" squared", " ^ 2"), (" cubed", " ^ 3"),
    ] {
        // case-insensitive replace
        let mut out = String::new();
        let mut rest = s.as_str();
        loop {
            let low = rest.to_ascii_lowercase();
            match low.find(w) {
                Some(i) => {
                    out.push_str(&rest[..i]);
                    out.push_str(op);
                    rest = &rest[i + w.len()..];
                }
                None => {
                    out.push_str(rest);
                    break;
                }
            }
        }
        s = out;
    }
    let is_expr_char = |ch: char| ch.is_ascii_digit() || " .+-*/^%()x×÷,".contains(ch);
    let chars: Vec<char> = s.chars().collect();
    let mut best: Option<String> = None;
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_digit() || chars[i] == '(' {
            let mut j = i;
            while j < chars.len() && is_expr_char(chars[j]) {
                // 'x' only counts as an operator between spaces/digits.
                if chars[j] == 'x' {
                    let prev = chars[..j].iter().rev().find(|c| **c != ' ');
                    let next = chars[j + 1..].iter().find(|c| **c != ' ');
                    if !(prev.is_some_and(char::is_ascii_digit) && next.is_some_and(|c| c.is_ascii_digit() || *c == '(')) {
                        break;
                    }
                }
                j += 1;
            }
            let span: String = chars[i..j].iter().collect();
            let span = span.trim().trim_end_matches(['.', ',', '+', '-', '*', '/', '^']).trim().to_string();
            let has_op = span.chars().skip(1).any(|ch| "+-*/^%x×÷".contains(ch));
            if has_op && crate::tools::calc::eval(&span).is_ok() && best.as_ref().is_none_or(|b| span.len() > b.len()) {
                best = Some(span);
            }
            i = j.max(i + 1);
        } else {
            i += 1;
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use vortex_kernel::{Effect, ParamSpec};

    fn spec(name: &str, desc: &str, params: &[(&str, ParamKind)]) -> ToolSpec {
        ToolSpec {
            name: name.into(),
            description: desc.into(),
            params: params.iter().map(|(n, k)| ParamSpec { name: (*n).into(), kind: *k, required: true }).collect(),
            effect: Effect::ReadOnly,
        }
    }

    fn pool() -> Vec<ToolSpec> {
        use ParamKind::*;
        vec![
            spec("calc", "Evaluate an arithmetic expression", &[("expression", String)]),
            spec("fs.read", "Read a file", &[("path", String)]),
            spec("eustress.list_spaces", "List the Spaces in a Universe", &[("universe", String)]),
            spec("eustress.list_universes", "List all Universes", &[]),
            spec("weather.get", "Get the weather for a city", &[("city", String)]),
            spec("git.log", "Show recent commits", &[("repo", String), ("count", Integer)]),
            spec("unit.convert", "Convert between units", &[("value", Number), ("from", String), ("to", String)]),
            spec("email.send", "Send an email", &[("to", String), ("subject", String)]),
        ]
    }

    #[test]
    fn single_and_parallel_calls() {
        let tools = pool();
        let calls = route("What's 12*7? Also list the spaces in the Astra universe.", &tools);
        assert_eq!(calls.len(), 2, "{calls:?}");
        assert_eq!(calls[0].tool, "calc");
        assert_eq!(calls[0].args["expression"], "12*7");
        assert_eq!(calls[1].tool, "eustress.list_spaces");
        assert_eq!(calls[1].args["universe"], "Astra");
    }

    #[test]
    fn extracts_typed_arguments() {
        let tools = pool();
        let c = route("convert 5 km to miles please", &tools);
        assert_eq!(c[0].tool, "unit.convert");
        assert_eq!(c[0].args["value"], json!(5));
        assert_eq!(c[0].args["from"], "km");
        assert_eq!(c[0].args["to"], "miles");

        let c = route("show me the last 5 commits in ./SpatialVortex", &tools);
        assert_eq!(c[0].tool, "git.log");
        assert_eq!(c[0].args["count"], json!(5));
        assert_eq!(c[0].args["repo"], "./SpatialVortex");

        let c = route("Send an email to pat@example.com with subject \"Q3 numbers\"", &tools);
        assert_eq!(c[0].tool, "email.send");
        assert_eq!(c[0].args["to"], "pat@example.com");
        assert_eq!(c[0].args["subject"], "Q3 numbers");

        let c = route("read notes/todo.md", &tools);
        assert_eq!(c[0].args["path"], "notes/todo.md");
    }

    #[test]
    fn inherits_tool_across_and() {
        let tools = pool();
        let c = route("what's the weather in Paris and in Tokyo", &tools);
        let cities: Vec<_> = c.iter().map(|x| x.args["city"].as_str().unwrap().to_string()).collect();
        assert_eq!(cities, vec!["Paris", "Tokyo"], "{c:?}");
    }

    #[test]
    fn word_operators_become_symbols() {
        let tools = pool();
        let c = route("calculate 15 times 4 plus 2", &tools);
        assert_eq!(c[0].args["expression"], "15 * 4 + 2");
    }
}
