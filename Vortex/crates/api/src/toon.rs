//! JSON -> TOON encoder (Token-Oriented Object Notation), targeting spec v4.4.
//!
//! Encode only. Pure computation: no I/O. Object keys are emitted in the order
//! `serde_json::Map` iterates (the host crate decides whether `preserve_order`
//! is enabled); tabular forms reorder cells to the header's field order, which
//! is the first element's key order (spec §9.3, §9.5).
//!
//! Notes on documented choices:
//! * `indent == 0` is treated as 1 (spec §1.3: indentSize is a positive number).
//! * Numbers outside `1e-6 <= |n| < 1e21` use JSON exponent form with a
//!   lowercase `e` and explicit sign (spec §2, SHOULD).
//! * Integers (`i64`/`u64`) are emitted as exact decimal integers; `f64` values
//!   use the shortest round-trip digits.
//! * Non-finite floats cannot be held by `serde_json::Number`, but are mapped to
//!   `null` if one ever reaches the encoder (spec §3).

use serde_json::{Map, Number, Value};

/// Document delimiter (spec §11).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Delimiter {
    #[default]
    Comma,
    Tab,
    Pipe,
}

impl Delimiter {
    fn ch(self) -> char {
        match self {
            Delimiter::Comma => ',',
            Delimiter::Tab => '\t',
            Delimiter::Pipe => '|',
        }
    }
    /// Symbol declared inside a bracket segment; absent for comma (spec §6).
    fn sym(self) -> &'static str {
        match self {
            Delimiter::Comma => "",
            Delimiter::Tab => "\t",
            Delimiter::Pipe => "|",
        }
    }
}

/// Encoder options (spec §13).
#[derive(Clone, Debug)]
pub struct Options {
    pub indent: usize,
    pub delimiter: Delimiter,
}

impl Default for Options {
    fn default() -> Self {
        Options { indent: 2, delimiter: Delimiter::Comma }
    }
}

/// Encode with default options (2-space indent, comma delimiter).
pub fn encode(value: &Value) -> String {
    encode_with(value, &Options::default())
}

/// Encode with explicit options.
pub fn encode_with(value: &Value, opts: &Options) -> String {
    let mut e = Enc { indent: opts.indent.max(1), delim: opts.delimiter, lines: Vec::new() };
    e.root(value);
    e.lines.join("\n")
}

// ---------------------------------------------------------------------------
// Shape detection (spec §9.3, §9.5)
// ---------------------------------------------------------------------------

type Obj = Map<String, Value>;

struct Field {
    key: String,
    sub: Option<Vec<Field>>,
}

fn is_prim(v: &Value) -> bool {
    !matches!(v, Value::Array(_) | Value::Object(_))
}

/// Column rules of §9.3: every column is uniform-primitive or nested-uniform.
/// Returns the field list in the first object's key order.
fn uniform_columns(objs: &[&Obj]) -> Option<Vec<Field>> {
    let first = *objs.first()?;
    if first.is_empty() {
        return None;
    }
    for o in objs {
        if o.len() != first.len() || !first.keys().all(|k| o.contains_key(k)) {
            return None;
        }
    }
    let mut out = Vec::with_capacity(first.len());
    for k in first.keys() {
        let col: Vec<&Value> = objs.iter().map(|o| &o[k.as_str()]).collect();
        if col.iter().all(|v| is_prim(v)) {
            out.push(Field { key: k.clone(), sub: None });
        } else if col.iter().all(|v| v.is_object()) {
            let subs: Vec<&Obj> = col.iter().filter_map(|v| v.as_object()).collect();
            out.push(Field { key: k.clone(), sub: Some(uniform_columns(&subs)?) });
        } else {
            return None;
        }
    }
    Some(out)
}

/// Tabular detection for an array (§9.3).
fn tabular_fields(arr: &[Value]) -> Option<Vec<Field>> {
    let objs: Vec<&Obj> = arr.iter().filter_map(|v| v.as_object()).collect();
    if objs.len() != arr.len() {
        return None;
    }
    uniform_columns(&objs)
}

/// Keyed tabular detection for an object (§9.5).
fn keyed_fields(m: &Obj) -> Option<Vec<Field>> {
    if m.len() < 2 {
        return None;
    }
    let objs: Vec<&Obj> = m.values().filter_map(|v| v.as_object()).collect();
    if objs.len() != m.len() {
        return None;
    }
    uniform_columns(&objs)
}

// ---------------------------------------------------------------------------
// Primitives (spec §2, §7)
// ---------------------------------------------------------------------------

fn fmt_f64(x: f64) -> String {
    if !x.is_finite() {
        return "null".to_string();
    }
    if x == 0.0 {
        return "0".to_string(); // also normalises -0
    }
    // `{:e}` on f64 yields the shortest digits that round-trip.
    let s = format!("{:e}", x.abs());
    let (mant, exp) = s.split_once('e').expect("LowerExp always has an exponent");
    let exp: i32 = exp.parse().expect("integer exponent");
    let digits: String = mant.chars().filter(|c| *c != '.').collect();
    let sign = if x < 0.0 { "-" } else { "" };
    if (-6..21).contains(&exp) {
        let n = digits.len() as i32;
        let body = if exp >= 0 {
            if n <= exp + 1 {
                format!("{}{}", digits, "0".repeat((exp + 1 - n) as usize))
            } else {
                let (i, f) = digits.split_at((exp + 1) as usize);
                format!("{i}.{f}")
            }
        } else {
            format!("0.{}{}", "0".repeat((-exp - 1) as usize), digits)
        };
        format!("{sign}{body}")
    } else {
        let mut m = digits[..1].to_string();
        if digits.len() > 1 {
            m.push('.');
            m.push_str(&digits[1..]);
        }
        let es = if exp < 0 { '-' } else { '+' };
        format!("{sign}{m}e{es}{}", exp.abs())
    }
}

fn fmt_number(n: &Number) -> String {
    if let Some(i) = n.as_i64() {
        return i.to_string();
    }
    if let Some(u) = n.as_u64() {
        return u.to_string();
    }
    fmt_f64(n.as_f64().unwrap_or(f64::NAN))
}

/// `/^[+-]?[0-9]+(?:\.[0-9]+)?(?:e[+-]?[0-9]+)?$/i` (spec §7.2), ASCII digits only.
fn is_numeric_like(s: &str) -> bool {
    let b = s.as_bytes();
    let mut i = 0;
    if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
        i += 1;
    }
    let start = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    if i == start {
        return false;
    }
    if i < b.len() && b[i] == b'.' {
        i += 1;
        let f = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i == f {
            return false;
        }
    }
    if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
        i += 1;
        if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
            i += 1;
        }
        let e = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i == e {
            return false;
        }
    }
    i == b.len()
}

/// Quoting rule for string values (spec §7.2).
fn needs_quote(s: &str, delim: char, root: bool) -> bool {
    if s.is_empty() {
        return true;
    }
    let first = s.chars().next().unwrap_or(' ');
    let last = s.chars().next_back().unwrap_or(' ');
    if first == ' ' || first == '\t' || last == ' ' || last == '\t' {
        return true;
    }
    if s == "true" || s == "false" || s == "null" {
        return true;
    }
    if is_numeric_like(s) {
        return true;
    }
    if s.chars().any(|c| {
        matches!(c, ':' | '"' | '\\' | '[' | ']' | '{' | '}') || (c as u32) < 0x20 || c == delim
    }) {
        return true;
    }
    if first == '-' || first == '#' {
        return true;
    }
    root && first == '\u{FEFF}'
}

/// Quote and escape per §7.1 (only `\\ \" \n \r \t` and `\u00xx` for other controls).
fn quote(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    o.push('"');
    for c in s.chars() {
        match c {
            '\\' => o.push_str("\\\\"),
            '"' => o.push_str("\\\""),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

/// Key encoding (spec §7.3): `^[A-Za-z_][A-Za-z0-9_.]*$` unquoted, else quoted.
fn enc_key(k: &str) -> String {
    let mut ch = k.chars();
    let ok = match ch.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {
            ch.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
        }
        _ => false,
    };
    if ok { k.to_string() } else { quote(k) }
}

// ---------------------------------------------------------------------------
// Encoder
// ---------------------------------------------------------------------------

struct Enc {
    indent: usize,
    delim: Delimiter,
    lines: Vec<String>,
}

impl Enc {
    fn pad(&self, depth: usize) -> String {
        " ".repeat(depth * self.indent)
    }

    fn prim(&self, v: &Value, root: bool) -> String {
        match v {
            Value::Null => "null".to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => fmt_number(n),
            Value::String(s) => {
                if needs_quote(s, self.delim.ch(), root) { quote(s) } else { s.clone() }
            }
            // Containers never reach here; keep total anyway.
            _ => "null".to_string(),
        }
    }

    fn join_prims(&self, arr: &[Value]) -> String {
        let d = self.delim.ch().to_string();
        arr.iter().map(|v| self.prim(v, false)).collect::<Vec<_>>().join(&d)
    }

    fn fields_str(&self, fields: &[Field]) -> String {
        let d = self.delim.ch().to_string();
        fields
            .iter()
            .map(|f| match &f.sub {
                None => enc_key(&f.key),
                Some(s) => format!("{}{{{}}}", enc_key(&f.key), self.fields_str(s)),
            })
            .collect::<Vec<_>>()
            .join(&d)
    }

    /// Cells of one row, depth-first pre-order over the field list (§9.3).
    fn cells(&self, m: &Obj, fields: &[Field], out: &mut Vec<String>) {
        for f in fields {
            let v = &m[f.key.as_str()];
            match (&f.sub, v) {
                (Some(sub), Value::Object(inner)) => self.cells(inner, sub, out),
                _ => out.push(self.prim(v, false)),
            }
        }
    }

    fn row(&self, m: &Obj, fields: &[Field]) -> String {
        let mut c = Vec::new();
        self.cells(m, fields, &mut c);
        c.join(&self.delim.ch().to_string())
    }

    fn root(&mut self, v: &Value) {
        match v {
            Value::Object(m) => {
                if m.is_empty() {
                    return;
                }
                match keyed_fields(m) {
                    Some(f) => self.keyed("", "", m, &f, 1),
                    None => self.fields(m, 0),
                }
            }
            Value::Array(a) => self.array("", "", a, 1),
            p => {
                let s = self.prim(p, true);
                self.lines.push(s);
            }
        }
    }

    fn fields(&mut self, m: &Obj, depth: usize) {
        let lead = self.pad(depth);
        for (k, v) in m {
            self.value(&lead, &enc_key(k), v, depth + 1);
        }
    }

    /// Emit `key` + value starting a line with `lead`; scope content goes at `cd`.
    fn value(&mut self, lead: &str, key: &str, v: &Value, cd: usize) {
        match v {
            Value::Object(m) => {
                if m.is_empty() {
                    self.lines.push(format!("{lead}{key}:"));
                } else if let Some(f) = keyed_fields(m) {
                    self.keyed(lead, key, m, &f, cd);
                } else {
                    self.lines.push(format!("{lead}{key}:"));
                    self.fields(m, cd);
                }
            }
            Value::Array(a) => self.array(lead, key, a, cd),
            p => {
                let s = self.prim(p, false);
                self.lines.push(format!("{lead}{key}: {s}"));
            }
        }
    }

    /// Keyed tabular object (§9.5). `key` is empty at the root.
    fn keyed(&mut self, lead: &str, key: &str, m: &Obj, fields: &[Field], cd: usize) {
        let hdr = format!(
            "{lead}{key}[{}:{}]{{{}}}:",
            m.len(),
            self.delim.sym(),
            self.fields_str(fields)
        );
        self.lines.push(hdr);
        let p = self.pad(cd);
        for (k, v) in m {
            if let Some(inner) = v.as_object() {
                let r = self.row(inner, fields);
                self.lines.push(format!("{p}{}: {r}", enc_key(k)));
            }
        }
    }

    /// Array in object-field or root position (§9.1, §9.3, §9.4). `key` is empty at the root.
    fn array(&mut self, lead: &str, key: &str, a: &[Value], cd: usize) {
        let n = a.len();
        let sym = self.delim.sym();
        if a.is_empty() {
            if key.is_empty() {
                self.lines.push(format!("{lead}[]"));
            } else {
                self.lines.push(format!("{lead}{key}: []"));
            }
        } else if a.iter().all(is_prim) {
            let j = self.join_prims(a);
            self.lines.push(format!("{lead}{key}[{n}{sym}]: {j}"));
        } else if let Some(f) = tabular_fields(a) {
            let hdr = format!("{lead}{key}[{n}{sym}]{{{}}}:", self.fields_str(&f));
            self.lines.push(hdr);
            let p = self.pad(cd);
            for v in a {
                if let Some(m) = v.as_object() {
                    let r = self.row(m, &f);
                    self.lines.push(format!("{p}{r}"));
                }
            }
        } else {
            self.lines.push(format!("{lead}{key}[{n}{sym}]:"));
            for v in a {
                self.item(v, cd);
            }
        }
    }

    /// One list item whose hyphen stands at depth `i` (§9.2, §9.4, §10).
    fn item(&mut self, v: &Value, i: usize) {
        let p = self.pad(i);
        let sym = self.delim.sym();
        match v {
            Value::Array(a) => {
                let n = a.len();
                if a.is_empty() {
                    self.lines.push(format!("{p}- [0{sym}]:"));
                } else if a.iter().all(is_prim) {
                    let j = self.join_prims(a);
                    self.lines.push(format!("{p}- [{n}{sym}]: {j}"));
                } else {
                    // Tabular form is unavailable here (§9.4): list form only.
                    self.lines.push(format!("{p}- [{n}{sym}]:"));
                    for x in a {
                        self.item(x, i + 1);
                    }
                }
            }
            Value::Object(m) => {
                if m.is_empty() {
                    self.lines.push(format!("{p}-"));
                    return;
                }
                let hyphen = format!("{p}- ");
                let next = self.pad(i + 1);
                for (idx, (k, val)) in m.iter().enumerate() {
                    let lead = if idx == 0 { hyphen.as_str() } else { next.as_str() };
                    self.value(lead, &enc_key(k), val, i + 2);
                }
            }
            prim => {
                let s = self.prim(prim, false);
                self.lines.push(format!("{p}- {s}"));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Unit tests. Section numbers refer to TOON spec v4.4.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn e(v: Value) -> String {
        encode(&v)
    }
    fn ed(v: Value, d: Delimiter) -> String {
        encode_with(&v, &Options { indent: 2, delimiter: d })
    }
    fn s(x: &str) -> String {
        e(json!({ "k": x }))
    }
    /// True when `serde_json::Map` keeps insertion order in this build (feature
    /// `preserve_order`). Tests whose spec examples use non-alphabetical keys only
    /// make sense then; without it the map sorts keys and they return early.
    fn ordered() -> bool {
        serde_json::from_str::<Value>(r#"{"b":1,"a":2}"#)
            .ok()
            .and_then(|v| v.as_object().and_then(|m| m.keys().next().cloned()))
            .as_deref()
            == Some("b")
    }
    macro_rules! need_order {
        () => {
            if !ordered() {
                eprintln!("skipped: serde_json preserve_order is off in this build");
                return;
            }
        };
    }

    // ---- App. A / §8 objects -------------------------------------------
    #[test]
    fn s8_flat_object_appendix_a() {
        need_order!();
        assert_eq!(e(json!({"id":123,"name":"Ada","active":true})), "id: 123\nname: Ada\nactive: true");
    }
    #[test]
    fn s8_nested_object_appendix_a() {
        assert_eq!(e(json!({"user":{"id":123,"name":"Ada"}})), "user:\n  id: 123\n  name: Ada");
    }
    #[test]
    fn s8_empty_root_object_is_empty_document() {
        assert_eq!(e(json!({})), "");
    }
    #[test]
    fn s8_empty_nested_object_is_bare_key() {
        assert_eq!(e(json!({"a":{},"b":1})), "a:\nb: 1");
    }
    #[test]
    fn s8_deeply_nested_appendix_a_edge() {
        let v = json!({"root":{"level1":{"level2":{"level3":{"items":[{"id":1,"val":"a"},{"id":2,"val":"b"}]}}}}});
        assert_eq!(
            e(v),
            "root:\n  level1:\n    level2:\n      level3:\n        items[2]{id,val}:\n          1,a\n          2,b"
        );
    }

    // ---- §7.2 quoting, one test per condition ----------------------------
    #[test]
    fn s7_2_empty_string_quoted() {
        assert_eq!(s(""), "k: \"\"");
    }
    #[test]
    fn s7_2_leading_trailing_space_or_tab_quoted() {
        assert_eq!(s(" a"), "k: \" a\"");
        assert_eq!(s("a "), "k: \"a \"");
        assert_eq!(s("a\t"), "k: \"a\\t\"");
        assert_eq!(s("\ta"), "k: \"\\ta\"");
        assert_eq!(s("a b"), "k: a b"); // internal space is safe
    }
    #[test]
    fn s7_2_keywords_quoted_case_sensitive() {
        assert_eq!(s("true"), "k: \"true\"");
        assert_eq!(s("false"), "k: \"false\"");
        assert_eq!(s("null"), "k: \"null\"");
        assert_eq!(s("True"), "k: True");
        assert_eq!(s("NULL"), "k: NULL");
    }
    #[test]
    fn s7_2_numeric_like_quoted() {
        for x in ["42", "-3.14", "05", "+1", "1e-6", "1E5", "0", "-0", "1.5e+10", "007.5"] {
            assert_eq!(s(x), format!("k: \"{x}\""), "{x}");
        }
        // not numeric-like by the §7.2 regex: stay unquoted
        for x in ["1.", ".5", "0x10", "1_000", "Infinity", "NaN", "1e", "e5", "1.2.3", "٣"] {
            assert_eq!(s(x), format!("k: {x}"), "{x}");
        }
    }
    #[test]
    fn s7_2_colon_quote_backslash_quoted() {
        assert_eq!(s("a:b"), "k: \"a:b\"");
        assert_eq!(s("a\"b"), "k: \"a\\\"b\"");
        assert_eq!(s("a\\b"), "k: \"a\\\\b\"");
    }
    #[test]
    fn s7_2_brackets_and_braces_quoted() {
        for x in ["[", "]", "{", "}", "a[0]", "x{y}"] {
            assert!(s(x).starts_with("k: \""), "{x}");
        }
    }
    #[test]
    fn s7_2_control_chars_quoted_and_s7_1_escapes() {
        assert_eq!(s("a\nb"), "k: \"a\\nb\"");
        assert_eq!(s("a\rb"), "k: \"a\\rb\"");
        assert_eq!(s("a\tb"), "k: \"a\\tb\"");
        assert_eq!(s("a\u{0}b"), "k: \"a\\u0000b\"");
        assert_eq!(s("a\u{1f}b"), "k: \"a\\u001fb\"");
        assert_eq!(s("a\u{8}b"), "k: \"a\\u0008b\"");
        assert_eq!(s("a\u{b}b"), "k: \"a\\u000bb\"");
    }
    #[test]
    fn s7_1_nothing_else_escaped() {
        // DEL, NBSP, non-ASCII, emoji, supplementary plane stay literal.
        assert_eq!(s("a\u{7f}b"), "k: a\u{7f}b");
        assert_eq!(s("a\u{a0}b"), "k: a\u{a0}b");
        assert_eq!(s("Hello 世界 👋"), "k: Hello 世界 👋"); // App. A edge cases
        assert_eq!(s("\u{1F389}"), "k: \u{1F389}");
        assert_eq!(s("a/b"), "k: a/b");
    }
    #[test]
    fn s7_2_delimiter_rule_depends_on_document_delimiter() {
        let v = json!({"k":"a,b","p":"a|b","t":"a b"});
        assert_eq!(ed(v.clone(), Delimiter::Comma), "k: \"a,b\"\np: a|b\nt: a b");
        assert_eq!(ed(v.clone(), Delimiter::Pipe), "k: a,b\np: \"a|b\"\nt: a b");
        assert_eq!(ed(v, Delimiter::Tab), "k: a,b\np: a|b\nt: a b");
    }
    #[test]
    fn s7_2_starts_with_hyphen_or_hash_quoted() {
        assert_eq!(s("-"), "k: \"-\"");
        assert_eq!(s("-x"), "k: \"-x\"");
        assert_eq!(s("- x"), "k: \"- x\"");
        assert_eq!(s("#x"), "k: \"#x\"");
        assert_eq!(s("#"), "k: \"#\"");
        assert_eq!(s("a-b"), "k: a-b");
        assert_eq!(s("a#b"), "k: a#b");
    }
    #[test]
    fn s7_2_bom_quoted_only_as_root_primitive() {
        assert_eq!(e(json!("\u{FEFF}x")), "\"\u{FEFF}x\"");
        assert_eq!(s("\u{FEFF}x"), "k: \u{FEFF}x");
        assert_eq!(e(json!(["\u{FEFF}x"])), "[1]: \u{FEFF}x");
    }
    #[test]
    fn s5_root_primitives() {
        assert_eq!(e(json!("hello")), "hello");
        assert_eq!(e(json!(42)), "42");
        assert_eq!(e(json!(true)), "true");
        assert_eq!(e(json!(null)), "null");
        assert_eq!(e(json!("")), "\"\"");
        assert_eq!(e(json!("a:b")), "\"a:b\"");
        assert_eq!(e(json!("42")), "\"42\"");
    }
    #[test]
    fn s7_2_edge_cases_appendix_a() {
        assert_eq!(e(json!({"name":""})), "name: \"\"");
        assert_eq!(e(json!({"tags":[]})), "tags: []");
        assert_eq!(e(json!({"version":"123"})), "version: \"123\"");
        assert_eq!(e(json!({"enabled":"true"})), "enabled: \"true\"");
        assert_eq!(e(json!({"message":"Hello 世界 👋"})), "message: Hello 世界 👋");
        assert_eq!(e(json!({"tags":["🎉","🎊","🎈"]})), "tags[3]: 🎉,🎊,🎈");
    }

    // ---- §7.3 keys --------------------------------------------------------
    #[test]
    fn s7_3_key_quoting() {
        assert_eq!(e(json!({"a_b.c1":1})), "a_b.c1: 1");
        assert_eq!(e(json!({"_x":1})), "_x: 1");
        assert_eq!(e(json!({"user.name":1})), "user.name: 1"); // §8 dotted keys literal
        assert_eq!(e(json!({"my-key":1})), "\"my-key\": 1");
        assert_eq!(e(json!({"1a":1})), "\"1a\": 1");
        assert_eq!(e(json!({"":1})), "\"\": 1");
        assert_eq!(e(json!({"a b":1})), "\"a b\": 1");
        assert_eq!(e(json!({"a:b":1})), "\"a:b\": 1");
        assert_eq!(e(json!({"q\"k":1})), "\"q\\\"k\": 1");
        assert_eq!(e(json!({"nl\nk":1})), "\"nl\\nk\": 1");
        assert_eq!(e(json!({"é":1})), "\"é\": 1");
    }
    #[test]
    fn s7_3_quoted_keys_in_headers_appendix_a() {
        assert_eq!(e(json!({"my-key":[1,2,3]})), "\"my-key\"[3]: 1,2,3");
        assert_eq!(
            e(json!({"x-items":[{"id":1,"name":"Ada"},{"id":2,"name":"Bob"}]})),
            "\"x-items\"[2]{id,name}:\n  1,Ada\n  2,Bob"
        );
        assert_eq!(
            e(json!({"x-items":[{"id":1},{"id":2,"label":"archived"}]})),
            "\"x-items\"[2]:\n  - id: 1\n  - id: 2\n    label: archived"
        );
        // field names quoted inside the field list
        assert_eq!(
            e(json!({"r":[{"a-b":1,"c":2},{"a-b":3,"c":4}]})),
            "r[2]{\"a-b\",c}:\n  1,2\n  3,4"
        );
    }

    // ---- §2 numbers -------------------------------------------------------
    #[test]
    fn s2_number_canonical_decimal() {
        assert_eq!(e(json!(0)), "0");
        assert_eq!(e(json!(-0.0)), "0");
        assert_eq!(e(json!(1.0)), "1");
        assert_eq!(e(json!(1.5000)), "1.5");
        assert_eq!(e(json!(-1.5)), "-1.5");
        assert_eq!(e(json!(1e6)), "1000000");
        assert_eq!(e(json!(1e-6)), "0.000001");
        assert_eq!(e(json!(123456.789)), "123456.789");
        assert_eq!(e(json!(0.3333333333333333)), "0.3333333333333333"); // App. A
        assert_eq!(e(json!(9007199254740992_i64)), "9007199254740992"); // App. A
        assert_eq!(e(json!(0.1 + 0.2)), "0.30000000000000004");
        assert_eq!(e(json!(u64::MAX)), "18446744073709551615");
        assert_eq!(e(json!(i64::MIN)), "-9223372036854775808");
        assert_eq!(e(json!(9.99)), "9.99");
        assert_eq!(e(json!(14.5)), "14.5");
    }
    #[test]
    fn s2_number_range_edges() {
        assert_eq!(e(json!(1e20)), "100000000000000000000");
        assert_eq!(e(json!(1.5e20)), "150000000000000000000");
        assert_eq!(e(json!(1e21)), "1e+21");
        assert_eq!(e(json!(-1e21)), "-1e+21");
        assert_eq!(e(json!(1.2345e25)), "1.2345e+25");
        assert_eq!(e(json!(999999.9999999)), "999999.9999999");
        assert_eq!(e(json!(0.000001234)), "0.000001234");
        assert_eq!(e(json!(9.99e-7)), "9.99e-7");
        assert_eq!(e(json!(1e-7)), "1e-7");
        assert_eq!(e(json!(-1.5e-7)), "-1.5e-7");
        assert_eq!(e(json!(5e-324)), "5e-324");
        assert_eq!(e(json!(1.7976931348623157e308)), "1.7976931348623157e+308");
    }
    #[test]
    fn s3_non_finite_is_null() {
        assert_eq!(e(Value::from(f64::NAN)), "null");
        assert_eq!(e(Value::from(f64::INFINITY)), "null");
        assert_eq!(e(Value::from(f64::NEG_INFINITY)), "null");
        assert_eq!(e(json!({"a":[f64::NAN]})), "a[1]: null");
    }

    // ---- §9.1 inline arrays ----------------------------------------------
    #[test]
    fn s9_1_inline_primitive_array() {
        assert_eq!(e(json!({"tags":["admin","ops","dev"]})), "tags[3]: admin,ops,dev");
        assert_eq!(e(json!({"m":[1,"a",true,null,2.5]})), "m[5]: 1,a,true,null,2.5");
        assert_eq!(e(json!({"q":["a,b","","x y"," z"]})), "q[4]: \"a,b\",\"\",x y,\" z\"");
    }
    #[test]
    fn s9_1_empty_arrays() {
        assert_eq!(e(json!({"tags":[]})), "tags: []");
        assert_eq!(e(json!([])), "[]");
        assert_eq!(e(json!({"a":{"b":[]}})), "a:\n  b: []");
    }
    #[test]
    fn s9_1_root_arrays() {
        assert_eq!(e(json!([1,2,3])), "[3]: 1,2,3");
        assert_eq!(e(json!(["x"])), "[1]: x");
        assert_eq!(e(json!([""])), "[1]: \"\"");
    }

    // ---- §9.2 arrays of primitive arrays ---------------------------------
    #[test]
    fn s9_2_pairs_appendix_a() {
        assert_eq!(e(json!({"pairs":[[1,2],[3,4]]})), "pairs[2]:\n  - [2]: 1,2\n  - [2]: 3,4");
    }
    #[test]
    fn s9_2_empty_inner_array_is_hyphen_zero() {
        assert_eq!(e(json!({"p":[[],[1]]})), "p[2]:\n  - [0]:\n  - [1]: 1");
        assert_eq!(e(json!([[]])), "[1]:\n  - [0]:");
        assert_eq!(e(json!([[],[]])), "[2]:\n  - [0]:\n  - [0]:");
    }

    // ---- §9.3 tabular -----------------------------------------------------
    #[test]
    fn s9_3_tabular_appendix_a() {
        need_order!();
        assert_eq!(
            e(json!({"items":[{"sku":"A1","qty":2,"price":9.99},{"sku":"B2","qty":1,"price":14.5}]})),
            "items[2]{sku,qty,price}:\n  A1,2,9.99\n  B2,1,14.5"
        );
    }
    #[test]
    fn s9_3_intro_example() {
        assert_eq!(
            e(json!({"users":[{"id":1,"name":"Ada","role":"admin"},{"id":2,"name":"Bob","role":"user"}]})),
            "users[2]{id,name,role}:\n  1,Ada,admin\n  2,Bob,user"
        );
    }
    #[test]
    fn s9_3_root_tabular() {
        assert_eq!(e(json!([{"a":1,"b":"x"},{"a":2,"b":"y"}])), "[2]{a,b}:\n  1,x\n  2,y");
    }
    #[test]
    fn s9_3_field_order_is_first_objects_order() {
        need_order!();
        let v: Value = serde_json::from_str(r#"[{"b":1,"a":2},{"a":4,"b":3}]"#).unwrap();
        assert_eq!(e(v), "[2]{b,a}:\n  1,2\n  3,4");
    }
    #[test]
    fn s9_3_cells_quoted_and_null_ok() {
        assert_eq!(
            e(json!({"links":[{"id":1,"url":"http://a:b"},{"id":2,"url":"https://example.com?q=a:b"}]})),
            "links[2]{id,url}:\n  1,\"http://a:b\"\n  2,\"https://example.com?q=a:b\""
        );
        assert_eq!(e(json!([{"a":null},{"a":"x,y"}])), "[2]{a}:\n  null\n  \"x,y\"");
    }
    #[test]
    fn s9_3_nested_field_groups_appendix_a() {
        need_order!();
        assert_eq!(
            e(json!({"orders":[
                {"id":1,"customer":{"name":"Ada","country":"DK"},"total":99},
                {"id":2,"customer":{"name":"Bob","country":"UK"},"total":149}]})),
            "orders[2]{id,customer{name,country},total}:\n  1,Ada,DK,99\n  2,Bob,UK,149"
        );
    }
    #[test]
    fn s9_3_nested_field_groups_recursive() {
        // `serde_json` w/o preserve_order sorts keys alphabetically: a < b < c.
        let v = json!([
            {"a":{"b":{"c":1,"d":2},"e":3},"f":4},
            {"a":{"b":{"c":5,"d":6},"e":7},"f":8}]);
        assert_eq!(e(v), "[2]{a{b{c,d},e},f}:\n  1,2,3,4\n  5,6,7,8");
    }
    #[test]
    fn s9_3_disqualified_arrays_use_list_form() {
        // empty object element
        assert_eq!(e(json!({"x":[{"a":1},{}]})), "x[2]:\n  - a: 1\n  -");
        // different key sets
        assert_eq!(e(json!({"x":[{"a":1},{"b":1}]})), "x[2]:\n  - a: 1\n  - b: 1");
        assert_eq!(e(json!({"x":[{"a":1},{"a":1,"b":2}]})), "x[2]:\n  - a: 1\n  - a: 1\n    b: 2");
        // column with an array value
        assert_eq!(e(json!({"x":[{"a":[1]},{"a":[2]}]})), "x[2]:\n  - a[1]: 1\n  - a[1]: 2");
        // column mixing null with objects
        assert_eq!(
            e(json!({"x":[{"a":null},{"a":{"b":1}}]})),
            "x[2]:\n  - a: null\n  - a:\n      b: 1"
        );
        // column holding an empty object
        assert_eq!(e(json!({"x":[{"a":{}},{"a":{}}]})), "x[2]:\n  - a:\n  - a:");
        // nested group whose sub-objects have different key sets
        assert_eq!(
            e(json!({"x":[{"a":{"b":1}},{"a":{"c":1}}]})),
            "x[2]:\n  - a:\n      b: 1\n  - a:\n      c: 1"
        );
        // nested group with a non-uniform sub-column (array inside)
        assert_eq!(
            e(json!({"x":[{"a":{"b":[1]}},{"a":{"b":[2]}}]})),
            "x[2]:\n  - a:\n      b[1]: 1\n  - a:\n      b[1]: 2"
        );
    }

    // ---- §9.4 mixed / list form -------------------------------------------
    #[test]
    fn s9_4_mixed_appendix_a() {
        assert_eq!(e(json!({"items":[1,{"a":1},"text"]})), "items[3]:\n  - 1\n  - a: 1\n  - text");
    }
    #[test]
    fn s9_4_list_of_objects_appendix_a() {
        // key sets differ (extra), so not tabular under §9.3; list form per §9.4/§10.
        need_order!();
        assert_eq!(
            e(json!({"items":[{"id":1,"name":"First"},{"id":2,"name":"Second","extra":true}]})),
            "items[2]:\n  - id: 1\n    name: First\n  - id: 2\n    name: Second\n    extra: true"
        );
    }
    #[test]
    fn s9_4_nested_arrays_use_list_form_under_hyphen() {
        // array of objects inside an array: hyphen line then items at +1 (no tabular here)
        assert_eq!(e(json!({"x":[[{"a":1}]]})), "x[1]:\n  - [1]:\n    - a: 1");
        // array of arrays inside an array
        assert_eq!(e(json!({"x":[[[1],[2,3]]]})), "x[1]:\n  - [2]:\n    - [1]: 1\n    - [2]: 2,3");
        // mixed inner array
        assert_eq!(e(json!([[1,[2]]])), "[1]:\n  - [2]:\n    - 1\n    - [1]: 2");
        // root list form
        assert_eq!(e(json!([1,{"a":1}])), "[2]:\n  - 1\n  - a: 1");
    }

    // ---- §9.5 keyed tabular ----------------------------------------------
    #[test]
    fn s9_5_keyed_appendix_a() {
        assert_eq!(
            e(json!({"users":{"alice":{"age":30,"city":"Berlin"},"bob":{"age":25,"city":"Oslo"}}})),
            "users[2:]{age,city}:\n  alice: 30,Berlin\n  bob: 25,Oslo"
        );
    }
    #[test]
    fn s9_5_keyed_at_root() {
        assert_eq!(
            e(json!({"alice":{"age":30,"city":"Berlin"},"bob":{"age":25,"city":"Oslo"}})),
            "[2:]{age,city}:\n  alice: 30,Berlin\n  bob: 25,Oslo"
        );
    }
    #[test]
    fn s9_5_not_collapsed_when_detection_fails() {
        // single entry nests
        assert_eq!(e(json!({"u":{"a":{"x":1}}})), "u:\n  a:\n    x: 1");
        // empty entry value
        assert_eq!(e(json!({"u":{"a":{},"b":{"x":1}}})), "u:\n  a:\n  b:\n    x: 1");
        // different key sets
        assert_eq!(e(json!({"u":{"a":{"x":1},"b":{"y":1}}})), "u:\n  a:\n    x: 1\n  b:\n    y: 1");
        // a non-object entry
        assert_eq!(e(json!({"u":{"a":{"x":1},"b":2}})), "u:\n  a:\n    x: 1\n  b: 2");
        // array-valued column
        assert_eq!(
            e(json!({"u":{"a":{"x":[1]},"b":{"x":[2]}}})),
            "u:\n  a:\n    x[1]: 1\n  b:\n    x[1]: 2"
        );
    }
    #[test]
    fn s9_5_entry_key_quoting_and_nested_group() {
        assert_eq!(
            e(json!({"u":{"a-b":{"x":1},"c d":{"x":2}}})),
            "u[2:]{x}:\n  \"a-b\": 1\n  \"c d\": 2"
        );
        assert_eq!(
            e(json!({"u":{"a":{"p":{"q":1,"r":2}},"b":{"p":{"q":3,"r":4}}}})),
            "u[2:]{p{q,r}}:\n  a: 1,2\n  b: 3,4"
        );
    }
    #[test]
    fn s9_5_array_elements_are_never_keyed() {
        // elements {"a":{...},"b":{...}} are anonymous objects; the column values collapse as groups
        assert_eq!(
            e(json!([{"a":{"x":1},"b":{"x":2}},{"a":{"x":3},"b":{"x":4}}])),
            "[2]{a{x},b{x}}:\n  1,2\n  3,4"
        );
        // an element that is itself a would-be keyed object uses list-item layout, first field on hyphen
        assert_eq!(
            e(json!([{"a":{"x":1},"b":{"x":2}}, 5])),
            "[2]:\n  - a:\n      x: 1\n    b:\n      x: 2\n  - 5"
        );
    }
    #[test]
    fn s9_5_keyed_tabular_nested_in_object_value() {
        assert_eq!(
            e(json!({"o":{"inner":{"u":{"a":{"x":1},"b":{"x":2}}}}})),
            "o:\n  inner:\n    u[2:]{x}:\n      a: 1\n      b: 2"
        );
    }

    // ---- §10 objects as list items -----------------------------------------
    #[test]
    fn s10_tabular_first_field_appendix_a() {
        need_order!();
        assert_eq!(
            e(json!({"items":[{"users":[{"id":1,"name":"Ada"},{"id":2,"name":"Bob"}],"status":"active"}]})),
            "items[1]:\n  - users[2]{id,name}:\n      1,Ada\n      2,Bob\n    status: active"
        );
    }
    #[test]
    fn s10_nested_object_first_field() {
        // a uniform single-element array would be tabular (§9.3); a second element with a
        // different key set forces list form.
        assert_eq!(
            e(json!({"items":[{"a":{"b":1},"c":2},{"a":{"b":1},"c":2,"d":0}]})),
            "items[2]:\n  - a:\n      b: 1\n    c: 2\n  - a:\n      b: 1\n    c: 2\n    d: 0"
        );
    }
    #[test]
    fn s10_list_form_array_first_field() {
        assert_eq!(
            e(json!({"items":[{"a":[1,{"z":1}],"b":2}]})),
            "items[1]:\n  - a[2]:\n      - 1\n      - z: 1\n    b: 2"
        );
        assert_eq!(e(json!({"items":[{"a":[],"b":2}]})), "items[1]:\n  - a: []\n    b: 2");
    }
    #[test]
    fn s10_keyed_first_field() {
        need_order!();
        assert_eq!(
            e(json!({"items":[{"u":{"a":{"x":1},"b":{"x":2}},"s":1},{"s":2}]})),
            "items[2]:\n  - u[2:]{x}:\n      a: 1\n      b: 2\n    s: 1\n  - s: 2"
        );
    }
    #[test]
    fn s10_empty_object_item_is_bare_hyphen() {
        assert_eq!(e(json!({"x":[{},1]})), "x[2]:\n  -\n  - 1");
        assert_eq!(e(json!([{}])), "[1]:\n  -");
    }
    #[test]
    fn s10_item_with_primitive_first_field_and_nested_later_field() {
        assert_eq!(
            e(json!({"x":[{"a":1,"b":{"c":2}},{"a":3}]})),
            "x[2]:\n  - a: 1\n    b:\n      c: 2\n  - a: 3"
        );
    }

    // ---- §11 delimiters ----------------------------------------------------
    #[test]
    fn s11_tab_delimiter_appendix_a() {
        need_order!();
        assert_eq!(
            ed(json!({"items":[{"sku":"A1","name":"Widget","qty":2,"price":9.99},{"sku":"B2","name":"Gadget","qty":1,"price":14.5}]}), Delimiter::Tab),
            "items[2\t]{sku\tname\tqty\tprice}:\n  A1\tWidget\t2\t9.99\n  B2\tGadget\t1\t14.5"
        );
    }
    #[test]
    fn s11_pipe_delimiter_appendix_a() {
        assert_eq!(
            ed(json!({"tags":["reading","gaming","coding"]}), Delimiter::Pipe),
            "tags[3|]: reading|gaming|coding"
        );
    }
    #[test]
    fn s11_pipe_everywhere() {
        let v = json!({
            "g":[{"c":{"d":1,"e":2}},{"c":{"d":3,"e":4}}],
            "k":{"p":{"x":1},"q":{"x":2}},
            "m":[{"a":[1,2]}],
            "n":[[1,2],[]],
            "t":[{"a":1,"b":"x|y"},{"a":2,"b":"z"}]
        });
        // `ordered()` builds keep the literal order above; sorted builds give the same order.
        assert_eq!(
            ed(v, Delimiter::Pipe),
            "g[2|]{c{d|e}}:\n  1|2\n  3|4\nk[2:|]{x}:\n  p: 1\n  q: 2\nm[1|]:\n  - a[2|]: 1|2\nn[2|]:\n  - [2|]: 1|2\n  - [0|]:\nt[2|]{a|b}:\n  1|\"x|y\"\n  2|z"
        );
    }
    #[test]
    fn s11_tab_everywhere() {
        let v = json!({"k":{"p":{"x":1},"q":{"x":2}},"n":[[1,2],[]],"t":[1,"a b"]});
        assert_eq!(
            ed(v, Delimiter::Tab),
            "k[2:\t]{x}:\n  p: 1\n  q: 2\nn[2\t]:\n  - [2\t]: 1\t2\n  - [0\t]:\nt[2\t]: 1\ta b"
        );
        assert_eq!(ed(json!([1,2]), Delimiter::Tab), "[2\t]: 1\t2");
        assert_eq!(ed(json!([{"a":1},{"a":2}]), Delimiter::Pipe), "[2|]{a}:\n  1\n  2");
    }
    #[test]
    fn s11_delimiter_only_changes_quoting_of_that_delimiter() {
        // comma is safe under pipe; pipe is safe under comma
        assert_eq!(ed(json!(["a,b"]), Delimiter::Pipe), "[1|]: a,b");
        assert_eq!(ed(json!(["a|b"]), Delimiter::Comma), "[1]: a|b");
        assert_eq!(ed(json!(["a|b"]), Delimiter::Pipe), "[1|]: \"a|b\"");
    }

    // ---- §12 whitespace / layout ----------------------------------------------
    #[test]
    fn s12_no_trailing_newline_spaces_blank_lines_or_bom() {
        let v = json!({"a":{"b":[1,{"c":[]}],"d":{}},"e":[[{"x":1}]],"f":""});
        let out = e(v);
        assert!(!out.ends_with('\n'));
        assert!(!out.starts_with('\u{FEFF}'));
        assert!(!out.contains('\r'));
        for l in out.split('\n') {
            assert!(!l.is_empty(), "blank line in {out:?}");
            assert!(!l.ends_with(' '), "trailing space in {l:?}");
            assert!(!l.contains('\t'), "tab in {l:?}");
        }
    }
    #[test]
    fn s12_custom_indent() {
        let v = json!({"a":{"b":1},"items":[{"id":1,"n":{"z":1}},{"id":2}]});
        assert_eq!(
            encode_with(&v, &Options { indent: 4, delimiter: Delimiter::Comma }),
            "a:\n    b: 1\nitems[2]:\n    - id: 1\n        n:\n            z: 1\n    - id: 2"
        );
        let t = json!({"a":[{"x":1},{"x":2}]});
        assert_eq!(
            encode_with(&t, &Options { indent: 4, delimiter: Delimiter::Comma }),
            "a[2]{x}:\n    1\n    2"
        );
        assert_eq!(
            encode_with(&json!({"a":{"b":1}}), &Options { indent: 1, ..Options::default() }),
            "a:\n b: 1"
        );
    }
    #[test]
    fn s12_options_default() {
        let o = Options::default();
        assert_eq!(o.indent, 2);
        assert_eq!(o.delimiter, Delimiter::Comma);
    }
}
