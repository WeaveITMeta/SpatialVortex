//! Cross-check of `vortex_api::toon::encode*` against an independent implementation.
//!
//! Reference: `toon-format` 0.7.0 (official toon-format org crate, declares
//! `toon-spec: 4.4`, spec fixtures in its own suite). Used ONLY as a decoder here:
//! `decode(encode(v)) == v` under the spec's JSON-model equality (§2), for all
//! three delimiters. Cases the reference decoder rejects although spec 4.4 requires
//! the form are listed in `EXCLUDED` with the section number; the encoder is not
//! bent to the decoder.

use serde_json::{Value, json};
use vortex_api::toon::{Delimiter, Options, encode, encode_with};
use toon_format::{DecodeOptions, Indent, decode};

// ---------------------------------------------------------------------------
// JSON-model equality (spec §2): numbers by mathematical value, objects
// order-insensitively (key order is checked separately where it is guaranteed).
// ---------------------------------------------------------------------------
fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Null, Value::Null) => true,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::String(x), Value::String(y)) => x == y,
        (Value::Number(x), Value::Number(y)) => {
            if let (Some(i), Some(j)) = (x.as_i64(), y.as_i64()) {
                return i == j;
            }
            if let (Some(i), Some(j)) = (x.as_u64(), y.as_u64()) {
                return i == j;
            }
            match (x.as_f64(), y.as_f64()) {
                (Some(p), Some(q)) => p == q,
                _ => false,
            }
        }
        (Value::Array(x), Value::Array(y)) => x.len() == y.len() && x.iter().zip(y).all(|(p, q)| same(p, q)),
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len() && x.iter().all(|(k, v)| y.get(k).is_some_and(|w| same(v, w)))
        }
        _ => false,
    }
}

fn dec(s: &str, indent: usize) -> Result<Value, String> {
    let o = DecodeOptions::new().with_indent(Indent::Spaces(indent));
    decode::<Value>(s, &o).map_err(|e| e.to_string())
}

const DELIMS: [Delimiter; 3] = [Delimiter::Comma, Delimiter::Tab, Delimiter::Pipe];

/// Returns Err(description) on any mismatch or decoder rejection.
fn roundtrip(v: &Value) -> Result<(), String> {
    for d in DELIMS {
        for indent in [2usize, 4] {
            let t = encode_with(v, &Options { indent, delimiter: d });
            // format invariants (spec §12)
            if t.ends_with('\n') || t.contains('\r') || t.starts_with('\u{FEFF}' ) && !v.is_string() {
                return Err(format!("{d:?}/{indent}: bad framing in {t:?}"));
            }
            for l in t.split('\n') {
                if t.contains('\n') && l.is_empty() {
                    return Err(format!("{d:?}/{indent}: blank line in {t:?}"));
                }
                if l.ends_with(' ') {
                    return Err(format!("{d:?}/{indent}: trailing space in {t:?}"));
                }
            }
            let back = dec(&t, indent).map_err(|e| format!("{d:?}/{indent}: decode error {e}\n--- toon ---\n{t}\n---"))?;
            if !same(v, &back) {
                return Err(format!("{d:?}/{indent}: mismatch\n--- toon ---\n{t}\n--- got ---\n{back}\n--- want ---\n{v}"));
            }
        }
    }
    Ok(())
}

/// Spec-4.4-required forms that the 0.7.0 reference decoder rejects; each is skipped
/// from the round trip. Empty on this run: see `excluded_cases_are_still_excluded`.
const EXCLUDED: &[(&str, &str)] = &[];

// ---------------------------------------------------------------------------
// Named corpus (>= 40 values)
// ---------------------------------------------------------------------------
fn corpus() -> Vec<(&'static str, Value)> {
    let parse = |s: &str| serde_json::from_str::<Value>(s).unwrap();
    vec![
        ("root_null", json!(null)),
        ("root_true", json!(true)),
        ("root_int", json!(-42)),
        ("root_string_plain", json!("hello world")),
        ("root_string_numeric_like", json!("0042")),
        ("root_string_bom", json!("\u{FEFF}lead")),
        ("root_string_empty", json!("")),
        ("root_empty_object", json!({})),
        ("root_empty_array", json!([])),
        ("flat_object", json!({"id":123,"name":"Ada","active":true,"nick":null})),
        ("nested_objects", json!({"a":{"b":{"c":{"d":1}}},"e":{}})),
        ("empties_everywhere", json!({"o":{},"a":[],"s":"","n":null,"x":[[],[[]],{}]})),
        ("inline_primitive_array", json!({"tags":["admin","ops","dev"]})),
        ("inline_mixed_primitives", json!({"m":[1,"a",true,null,2.5,"",  " "]})),
        ("array_of_arrays", json!({"pairs":[[1,2],[3,4],[]]})),
        ("array_of_arrays_deep", json!([[[1],[2,[3]]],[{"a":1}],[[{"b":[1,2]}]]])),
        ("tabular_basic", json!({"items":[{"sku":"A1","qty":2,"price":9.99},{"sku":"B2","qty":1,"price":14.5}]})),
        ("tabular_root", json!([{"a":1,"b":"x"},{"a":2,"b":"y"},{"a":3,"b":"z"}])),
        ("tabular_single_row", json!({"r":[{"a":1}]})),
        ("tabular_with_nulls_and_bools", json!({"r":[{"a":null,"b":true},{"a":"x","b":false}]})),
        ("tabular_nested_groups", json!({"orders":[
            {"id":1,"customer":{"name":"Ada","country":"DK"},"total":99},
            {"id":2,"customer":{"name":"Bob","country":"UK"},"total":149}]})),
        ("tabular_nested_groups_deep", json!([
            {"a":{"b":{"c":1,"d":"x"},"e":2},"f":3},
            {"a":{"b":{"c":4,"d":"y"},"e":5},"f":6}])),
        ("tabular_reordered_keys", parse(r#"[{"b":1,"a":2},{"a":4,"b":3},{"a":6,"b":5}]"#)),
        ("tabular_awkward_cells", json!({"t":[
            {"a":"x,y","b":"p|q","c":"t\tab"},
            {"a":"-","b":"#h","c":"a:b"}]})),
        ("tabular_quoted_field_names", json!({"t":[{"a-b":1,"c d":2,"":3},{"a-b":4,"c d":5,"":6}]})),
        ("list_form_nonuniform_objects", json!({"x":[{"a":1},{"b":2},{"a":1,"b":2}]})),
        ("list_form_mixed", json!({"items":[1,{"a":1},"text",[1,2],[],{}, null]})),
        ("list_item_first_field_object", json!({"x":[{"a":{"b":1},"c":2},{"a":{"b":1},"c":2,"d":0}]})),
        ("list_item_first_field_array_list", json!({"x":[{"a":[1,{"z":1}],"b":2},{"q":0}]})),
        ("list_item_first_field_tabular", json!({"items":[{"users":[{"id":1,"name":"Ada"},{"id":2,"name":"Bob"}],"status":"active"},{"z":1}]})),
        ("list_item_first_field_empty_array", json!({"x":[{"a":[],"b":2},{"c":1}]})),
        ("list_item_empty_object", json!({"x":[{},{"a":1},{}]})),
        ("keyed_basic", json!({"users":{"alice":{"age":30,"city":"Berlin"},"bob":{"age":25,"city":"Oslo"}}})),
        ("keyed_root", json!({"alice":{"age":30,"city":"Berlin"},"bob":{"age":25,"city":"Oslo"}})),
        ("keyed_nested_group", json!({"u":{"a":{"p":{"q":1,"r":2}},"b":{"p":{"q":3,"r":4}}}})),
        ("keyed_awkward_entry_keys", json!({"u":{"a-b":{"x":1},"c d":{"x":2},"":{"x":3},"q\"k":{"x":4}}})),
        ("keyed_inside_list_item", json!({"items":[{"u":{"a":{"x":1},"b":{"x":2}},"s":1},{"s":2}]})),
        ("keyed_not_collapsed_single", json!({"u":{"a":{"x":1}}})),
        ("keyed_not_collapsed_mixed", json!({"u":{"a":{"x":1},"b":{"y":1}},"v":{"a":{"x":[1]},"b":{"x":[2]}}})),
        ("awkward_string_values", json!({
            "a":"", "b":" lead", "c":"trail ", "d":"true", "e":"false", "f":"null",
            "g":"42", "h":"-3.14", "i":"05", "j":"+1", "k":"1e-6", "l":"1E5", "m":"a:b",
            "n":"q\"uote", "o":"back\\slash", "p":"[x]", "q":"{x}", "r":"a\nb", "s":"a\rb",
            "t":"a\tb", "u":"a\u{0}b", "v":"a,b", "w":"a|b", "x":"-lead", "y":"#hash",
            "z":"- x"})),
        ("awkward_string_values_2", json!({
            "a":"1.", "b":".5", "c":"0x10", "d":"Infinity", "e":"NaN", "f":"[]", "g":"{}",
            "h":"[1]", "i":"key: v", "j":"日本語", "k":"😀", "l":"\u{7f}", "m":"\u{a0}nbsp",
            "n":"a\u{1f}b", "o":"True", "p":"NULL", "q":"x\u{FEFF}y", "r":"\u{FEFF}"})),
        ("awkward_inline_strings", json!(["a,b","a|b","a\tb","",  "-", "#", "x y", "1", "true", "[]", "\"" ])),
        ("awkward_keys", json!({
            "my-key":1,"":2,"a b":3,"1x":4,"x.y":5,"q\"k":6,"日本":7,"a:b":8,"[x]":9,
            "-":10,"#":11,"null":12,"true":13,"a,b":14,"a|b":15,"_u":16,"A.b_c9":17,"a\nb":18,"__proto__":19})),
        ("awkward_keys_in_headers", json!({
            "my-key":[1,2,3], "x-items":[{"id":1,"name":"Ada"},{"id":2,"name":"Bob"}],
            "k y":[[1],[2]], "":[{"a":1},{"b":2}], "[b]":{"x":[]}})),
        ("numbers_basic", json!([0, -0.0, 1.0, 1.5, -1.5, 100, 1e6, 123456.789, 0.1 + 0.2, 0.3333333333333333])),
        ("numbers_range_edges", parse("[1e20, 1e21, -1e21, 1.5e20, 999999.9999999, 1e-6, 1e-7, 9.99e-7, 0.000001234, 5e-324, 1.7976931348623157e308, 123456789012345680000]")),
        ("numbers_int_extremes", parse("[9007199254740992, 9007199254740993, -9223372036854775808, 9223372036854775807, 18446744073709551615]")),
        ("numbers_in_tabular", parse(r#"{"t":[{"a":1e21,"b":1e-7},{"a":-1e21,"b":5e-324}]}"#)),
        ("deep_config", json!({
            "server":{"host":"localhost","port":8080,"tls":{"enabled":true,"cert":"/etc/ssl/c.pem"}},
            "db":{"pool":{"min":1,"max":10},"replicas":[{"host":"r1","port":5432},{"host":"r2","port":5432}]},
            "features":{"flags":["a","b"],"limits":{"rate":{"per_sec":100,"burst":[1,2,3]}}}})),
        ("tool_result_like", json!({
            "ok":true,"status":200,"elapsed_ms":12.5,
            "results":[{"id":"r1","score":0.91,"tags":["x","y"]},{"id":"r2","score":0.5,"tags":[]}],
            "next":null,"error":"",
            "meta":{"query":"find: all things","filters":[],"page":{"n":1,"of":3}}})),
        ("array_of_empty_objects", json!({"x":[{},{}]})),
        ("array_of_empty_arrays", json!({"x":[[],[]]})),
        ("object_values_all_kinds", json!({"a":{"b":[{"c":1},{"c":2}],"d":[1,[2,3]],"e":{"f":{}}}})),
    ]
}

#[test]
fn corpus_has_at_least_40_cases() {
    assert!(corpus().len() >= 40, "{}", corpus().len());
}

#[test]
fn corpus_round_trips_through_reference_decoder() {
    let mut failures = Vec::new();
    let mut n = 0;
    for (name, v) in corpus() {
        if EXCLUDED.iter().any(|(x, _)| *x == name) {
            continue;
        }
        n += 1;
        if let Err(e) = roundtrip(&v) {
            failures.push(format!("[{name}] {e}"));
        }
    }
    assert!(failures.is_empty(), "{} of {n} failed:\n{}", failures.len(), failures.join("\n\n"));
    println!("corpus round-tripped: {n} cases x 3 delimiters x 2 indents");
}

#[test]
fn excluded_cases_are_still_excluded() {
    // Guard: an exclusion must name a real corpus case and spec section.
    for (name, why) in EXCLUDED {
        assert!(corpus().iter().any(|(n, _)| n == name), "{name} not in corpus");
        assert!(why.contains('§'), "{name}: exclusion must cite a spec section");
    }
}

// ---------------------------------------------------------------------------
// Generated corpus: deterministic xorshift, awkward strings, uniform arrays.
// ---------------------------------------------------------------------------
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
    fn pick<'a, T>(&mut self, xs: &'a [T]) -> &'a T {
        &xs[self.below(xs.len())]
    }
}

const STRS: &[&str] = &[
    "", " ", "a", "a b", " a", "a ", "true", "false", "null", "42", "-1", "1e5", "05", "+3", "1.", ".5",
    "a,b", "a|b", "a\tb", "a:b", "\"", "\\", "[x]", "{x}", "#c", "-", "- x", "\u{FEFF}x", "日本語", "😀",
    "\n", "line1\nline2", "\u{7f}", "x\u{0}y", "Infinity", "[]", "{}", "[1]", "key: v", "plain", "Hello World",
    "https://example.com?q=a:b", "0x1F", "e5", "1e", "-", "--", "x#y", "x-y",
];
const KEYS: &[&str] = &[
    "a", "b", "c", "id", "name", "my-key", "", "a b", "1x", "x.y", "q\"k", "日本", "a:b", "[x]", "-", "#",
    "null", "true", "a,b", "a|b", "_u", "A.b_c9", "k\tt",
];
const NUMS: &[f64] = &[
    0.0, -0.0, 1.0, -1.0, 0.5, 1.5, 100.0, 1e6, 1e20, 1e21, -1e21, 1e-6, 1e-7, 9.99e-7, 123456.789,
    0.1, 0.30000000000000004, 5e-324, 1.7976931348623157e308, 9007199254740992.0, 2.5e-10, 4.2e22,
];

fn prim(r: &mut Rng) -> Value {
    match r.below(7) {
        0 => Value::Null,
        1 => Value::Bool(r.below(2) == 0),
        2 => json!(r.next() as i64 % 100_000),
        3 => json!(*r.pick(NUMS)),
        4 => json!(r.next() as i64),
        _ => Value::String(r.pick(STRS).to_string()),
    }
}

fn keyset(r: &mut Rng, n: usize) -> Vec<String> {
    let mut ks: Vec<String> = Vec::new();
    while ks.len() < n {
        let k = r.pick(KEYS).to_string();
        if !ks.contains(&k) {
            ks.push(k);
        }
    }
    ks
}

/// Object with a fixed key list; some columns are nested-uniform when `nest`>0.
fn uniform_obj(r: &mut Rng, shape: &Shape) -> Value {
    let mut m = serde_json::Map::new();
    for (k, sub) in &shape.cols {
        match sub {
            None => {
                m.insert(k.clone(), prim(r));
            }
            Some(s) => {
                m.insert(k.clone(), uniform_obj(r, s));
            }
        }
    }
    Value::Object(m)
}

struct Shape {
    cols: Vec<(String, Option<Shape>)>,
}

fn shape(r: &mut Rng, nest: usize) -> Shape {
    let n = 1 + r.below(4);
    let ks = keyset(r, n);
    Shape {
        cols: ks
            .into_iter()
            .map(|k| {
                let sub = if nest > 0 && r.below(4) == 0 { Some(shape(r, nest - 1)) } else { None };
                (k, sub)
            })
            .collect(),
    }
}

fn gen_value(r: &mut Rng, depth: usize) -> Value {
    let top = if depth == 0 { 3 } else { 9 };
    match r.below(top) {
        0..=2 => prim(r),
        3 => {
            // heterogeneous object
            let n = r.below(5);
            let mut m = serde_json::Map::new();
            for _ in 0..n {
                m.insert(r.pick(KEYS).to_string(), gen_value(r, depth - 1));
            }
            Value::Object(m)
        }
        4 => {
            // heterogeneous array
            let n = r.below(5);
            Value::Array((0..n).map(|_| gen_value(r, depth - 1)).collect())
        }
        5 => {
            // uniform array of objects (tabular candidate, sometimes with nested groups)
            let sh = shape(r, 2);
            let n = 1 + r.below(4);
            Value::Array((0..n).map(|_| uniform_obj(r, &sh)).collect())
        }
        6 => {
            // keyed tabular candidate
            let sh = shape(r, 2);
            let n = 1 + r.below(4);
            let ks = keyset(r, n);
            let mut m = serde_json::Map::new();
            for k in ks {
                m.insert(k, uniform_obj(r, &sh));
            }
            Value::Object(m)
        }
        7 => {
            // primitive array
            let n = r.below(6);
            Value::Array((0..n).map(|_| prim(r)).collect())
        }
        _ => {
            // array of arrays / objects wrapping the above
            let n = 1 + r.below(3);
            Value::Array((0..n).map(|_| gen_value(r, depth - 1)).collect())
        }
    }
}

#[test]
fn generated_values_round_trip() {
    let mut r = Rng(0x9E37_79B9_7F4A_7C15);
    let mut failures = Vec::new();
    let total = 3000;
    // coverage of the generator: how many encodings exercise each form
    let (mut tab, mut keyed, mut nested, mut list, mut empty_item) = (0, 0, 0, 0, 0);
    for i in 0..total {
        let v = gen_value(&mut r, 4);
        let t = encode(&v);
        tab += t.contains("]{") as u32;
        keyed += t.contains(":]{") as u32;
        nested += t.lines().any(|l| l.contains("]{") && l.matches('{').count() > 1) as u32;
        list += t.contains("- ") as u32;
        empty_item += t.lines().any(|l| l.trim() == "-") as u32;
        if let Err(e) = roundtrip(&v) {
            failures.push(format!("[gen #{i}] {e}"));
            if failures.len() >= 5 {
                break;
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
    println!("coverage: tabular={tab} keyed={keyed} nested_groups={nested} list_form={list} empty_obj_item={empty_item}");
    assert!(tab > 50 && keyed > 20 && nested > 20 && list > 50, "generator is not reaching every form");
    println!("generated: {total} values x 3 delimiters x 2 indents round-tripped");
}

// ---------------------------------------------------------------------------
// Direct cross-check of byte output against the reference ENCODER is not done
// (the reference may legitimately differ in non-normative choices); instead the
// reference decoder is the oracle, plus the exact-output unit tests in src/lib.rs.
// A few structural expectations on the default encoding:
// ---------------------------------------------------------------------------
#[test]
fn default_encode_matches_spec_headline_example() {
    let v = json!({"users":[{"id":1,"name":"Ada","role":"admin"},{"id":2,"name":"Bob","role":"user"}]});
    assert_eq!(encode(&v), "users[2]{id,name,role}:\n  1,Ada,admin\n  2,Bob,user");
}

#[test]
fn key_order_survives_round_trip_for_plain_objects() {
    // Only meaningful when serde_json keeps insertion order (reference crate enables
    // `preserve_order` for this build); tabular forms reorder to header order by design.
    let v: Value = serde_json::from_str(r#"{"z":1,"a":{"y":2,"b":3},"m":[1,2]}"#).unwrap();
    let back = dec(&encode(&v), 2).unwrap();
    let keys = |v: &Value| v.as_object().unwrap().keys().cloned().collect::<Vec<_>>();
    assert_eq!(keys(&v), keys(&back));
    assert_eq!(keys(&v["a"]), keys(&back["a"]));
}

// ---------------------------------------------------------------------------
// Differential check against the reference ENCODER (gating).
// Byte equality is required everywhere EXCEPT numbers outside 1e-6 <= |n| < 1e21:
// spec §2 lets encoders choose (MAY exponent; SHOULD lowercase `e` + sign), the
// reference writes long plain decimals, this encoder writes exponent form. Values
// containing such numbers are skipped here and still covered by the round trip.
// ---------------------------------------------------------------------------
fn has_out_of_range_number(v: &Value) -> bool {
    match v {
        Value::Number(n) => n.as_i64().is_none() && n.as_u64().is_none() && {
            let a = n.as_f64().unwrap_or(0.0).abs();
            a != 0.0 && !(1e-6..1e21).contains(&a)
        },
        Value::Array(a) => a.iter().any(has_out_of_range_number),
        Value::Object(m) => m.values().any(has_out_of_range_number),
        _ => false,
    }
}

#[test]
fn reference_encoder_byte_diff_on_corpus_and_generated() {
    use toon_format::{Delimiter as RD, EncodeOptions, encode as ref_encode};
    let mut r = Rng(0xD1B5_4A32_D192_ED03);
    let mut cases: Vec<(String, Value)> = corpus().into_iter().map(|(n, v)| (n.to_string(), v)).collect();
    for i in 0..1500 {
        cases.push((format!("gen#{i}"), gen_value(&mut r, 4)));
    }
    let mut diffs = Vec::new();
    let mut skipped = 0;
    for (name, v) in &cases {
        if has_out_of_range_number(v) {
            skipped += 1;
            continue;
        }
        for (d, rd) in [(Delimiter::Comma, RD::Comma), (Delimiter::Tab, RD::Tab), (Delimiter::Pipe, RD::Pipe)] {
            let ours = encode_with(v, &Options { indent: 2, delimiter: d });
            let theirs = ref_encode(v, &EncodeOptions::new().with_delimiter(rd)).map_err(|e| e.to_string());
            match theirs {
                Ok(t) if t == ours => {}
                Ok(t) => diffs.push(format!("[{name}/{d:?}]\n--- ours ---\n{ours}\n--- reference ---\n{t}")),
                Err(e) => diffs.push(format!("[{name}/{d:?}] reference encoder error: {e}")),
            }
        }
    }
    println!(
        "reference byte diff: {} differ among {} compared values x 3 delimiters ({} out-of-range-number values skipped)",
        diffs.len(),
        cases.len() - skipped,
        skipped
    );
    assert!(
        diffs.is_empty(),
        "{} divergences, first:\n{}",
        diffs.len(),
        diffs.iter().take(5).cloned().collect::<Vec<_>>().join("\n\n")
    );
}

// ---------------------------------------------------------------------------
// Size evidence (bytes only; no token claims). Run with `-- --nocapture`.
// ---------------------------------------------------------------------------
#[test]
fn size_evidence_bytes() {
    let names = ["Ada Lovelace", "Bob Stone", "Chen Wei", "Dara Okafor", "Eli Park"];
    let roles = ["admin", "editor", "viewer", "ops"];
    let users: Vec<Value> = (1..=20)
        .map(|i| {
            json!({
                "id": i,
                "name": names[i % names.len()],
                "email": format!("user{i}@example.com"),
                "role": roles[i % roles.len()],
                "active": i % 3 != 0
            })
        })
        .collect();
    let uniform = json!({ "users": users });
    let config = json!({
        "server": {"host":"0.0.0.0","port":8080,"tls":{"enabled":true,"cert":"/etc/ssl/cert.pem","key":"/etc/ssl/key.pem","protocols":["TLSv1.2","TLSv1.3"]}},
        "database": {"primary":{"host":"db1.internal","port":5432,"pool":{"min":2,"max":20,"idle_timeout_s":300}},
                     "replica":{"host":"db2.internal","port":5432,"pool":{"min":1,"max":10,"idle_timeout_s":300}}},
        "logging": {"level":"info","sinks":{"stdout":{"format":"json","color":false},"file":{"path":"/var/log/app.log","rotate":{"max_mb":100,"keep":7}}}},
        "features": {"search":{"enabled":true,"backends":{"primary":"es","fallback":"pg"}},"billing":{"enabled":false,"currency":"USD","tax":{"rate":0.0825,"inclusive":false}}},
        "limits": {"rate":{"per_sec":100,"burst":250},"body_kb":512}
    });
    let tool = json!({
        "ok": true, "status": 200, "elapsed_ms": 41.7,
        "query": "find: open issues labeled bug",
        "results": [
            {"id":"ISS-101","title":"Crash on save","labels":["bug","p1"],"assignee":{"name":"Ada","team":"core"},"score":0.93},
            {"id":"ISS-117","title":"Timeout in sync","labels":["bug"],"assignee":null,"score":0.88},
            {"id":"ISS-120","title":"Wrong total","labels":[],"assignee":{"name":"Bob","team":"billing"},"score":0.71}
        ],
        "facets": {"label":{"bug":3,"p1":1},"team":{"core":1,"billing":1}},
        "next_cursor": null, "warnings": ["partial index", "stale cache: 12s"], "debug": {}
    });
    println!("{:<28}{:>10}{:>16}{:>10}{:>10}", "payload", "json", "json_pretty", "toon", "toon/json");
    for (name, v) in [("uniform: 20 objs x 5 fields", &uniform), ("deeply nested config", &config), ("mixed tool result", &tool)] {
        let c = serde_json::to_string(v).unwrap().len();
        let p = serde_json::to_string_pretty(v).unwrap().len();
        let t = encode(v);
        assert!(same(v, &dec(&t, 2).unwrap()));
        println!("{:<28}{:>10}{:>16}{:>10}{:>9.0}%", name, c, p, t.len(), 100.0 * t.len() as f64 / c as f64);
    }
}
