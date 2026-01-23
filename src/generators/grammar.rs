use std::collections::{HashMap, VecDeque};

#[derive(Clone, Debug)]
pub struct WordNode {
    pub id: String,
    pub word: String,
    pub vec8: [f32; 8],
}

// === Helpers for concept IDs and language/script detection ===

fn quantize_embed(v: &[f32; 8]) -> String {
    // 16 bins per dimension → 8 hex nibbles as concept id
    let mut out = String::with_capacity(8);
    for i in 0..8 {
        let x = v[i].clamp(0.0, 1.0);
        let bin = (x * 15.0 + 0.5).floor() as u8; // 0..15
        out.push(std::char::from_digit((bin as u32) & 0xF, 16).unwrap());
    }
    out
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Script { Latin, Cjk, Other }

fn contains_cjk(s: &str) -> bool {
    s.chars().any(|ch| {
        let u = ch as u32;
        // CJK Unified Ideographs, Hiragana, Katakana ranges (basic)
        (0x4E00..=0x9FFF).contains(&u) || (0x3040..=0x309F).contains(&u) || (0x30A0..=0x30FF).contains(&u)
    })
}

fn script_of(s: &str) -> Script {
    if contains_cjk(s) { Script::Cjk } else if s.chars().any(|c| c.is_ascii_alphabetic()) { Script::Latin } else { Script::Other }
}

fn detect_script_from_tokens(tokens: &[String]) -> Script {
    for t in tokens {
        let sc = script_of(t);
        if sc != Script::Other { return sc; }
    }
    Script::Latin
}

fn normalize_rel(s: &str) -> String {
    let mut t = s.to_lowercase();
    for suf in ["ing", "ed", "es", "s"] {
        if t.len() > suf.len() && t.ends_with(suf) { t.truncate(t.len()-suf.len()); break; }
    }
    t
}

fn passive_of(rel: &str) -> Option<String> {
    let stem = normalize_rel(rel);
    if stem.is_empty() { None } else { Some(format!("is-{}-by", stem)) }
}

#[derive(Clone, Debug)]
pub struct ConceptNode {
    pub id: String,
    pub center: [f32; 8],
    pub size: u32,
    pub words: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub relation: String,
    pub confidence: f32,
}

#[derive(Default, Clone, Debug)]
pub struct GrammarGraph {
    pub nodes: HashMap<String, WordNode>,
    pub edges: Vec<Edge>,
    pub concept_nodes: HashMap<String, ConceptNode>,
    pub word_to_concept: HashMap<String, String>,
}

impl GrammarGraph {
    pub fn new() -> Self { Self::default() }

    pub fn say(&mut self, text: &str) {
        let tokens = tokenize(text);
        let triples = extract_basic_triples(&tokens);
        for (subj, pred, obj) in triples {
            let s = self.upsert_word(&subj);
            let o = self.upsert_word(&obj);
            self.add_edge(&s, &o, &pred, 0.3);
            if let Some(passive) = passive_of(&pred) { self.add_edge(&o, &s, &passive, 0.2); }
        }
    }

    pub fn ask(&self, question: &str) -> AskResult {
        let tokens = tokenize(question);
        let triples = extract_basic_triples(&tokens);
        if triples.is_empty() { return AskResult::no(); }
        let (subj, pred, obj) = &triples[0];
        let sid = self.word_id(subj);
        let oid = self.word_id(obj);
        if sid.is_none() || oid.is_none() { return AskResult::no(); }
        let sid = sid.unwrap();
        let oid = oid.unwrap();
        let mut res = self.path_find(&sid, &oid, pred, 2);
        if res.yes {
            let script = detect_script_from_tokens(&tokens);
            // Prefer destination concept surface in same script
            let dest = res.path.last().cloned().unwrap_or(oid.clone());
            let surface = self
                .word_to_concept
                .get(&dest)
                .and_then(|cid| self.surface_for(cid, script))
                .or_else(|| self.nodes.get(&dest).map(|n| n.word.clone()));
            res.answer_surface = surface;
        }
        res
    }

    pub fn path_find(&self, from: &str, to: &str, relation: &str, max_len: usize) -> AskResult {
        if from == to { return AskResult { yes: true, path: vec![from.to_string()], confidence: 1.0, answer_surface: None }; }
        let mut q: VecDeque<(String, Vec<String>, f32, usize)> = VecDeque::new();
        q.push_back((from.to_string(), vec![from.to_string()], 1.0, 0));
        while let Some((node, path, conf, depth)) = q.pop_front() {
            if depth > max_len { continue; }
            for e in self.edges.iter().filter(|e| e.from == node) {
                if normalize_rel(&e.relation) == normalize_rel(relation) {
                    // Direct relation: combine edge confidence (base) with semantic glow (closeness)
                    let base = e.confidence;
                    let glow = self.closeness(&node, &e.to);
                    let next_conf = ((base + glow) * 0.5).min(1.0);
                    if &e.to == to && next_conf >= 0.6 { 
                        let mut p = path.clone(); p.push(e.to.clone());
                        return AskResult { yes: true, path: p, confidence: next_conf, answer_surface: None };
                    }
                    if depth + 1 <= max_len {
                        let mut p = path.clone(); p.push(e.to.clone());
                        q.push_back((e.to.clone(), p, next_conf, depth + 1));
                    }
                } else {
                    if depth + 1 <= max_len && self.related_relation(relation, &e.relation) {
                        let mut p = path.clone(); p.push(e.to.clone());
                        let next_conf = conf * 0.5 * self.closeness(&node, &e.to);
                        q.push_back((e.to.clone(), p, next_conf, depth + 1));
                    }
                }
            }
            for near in self.neighbors_within(&node, 1.0) {
                if depth + 1 <= max_len {
                    let mut p = path.clone(); p.push(near.clone());
                    q.push_back((near, p, conf * 0.5, depth + 1));
                }
            }
        }
        AskResult::no()
    }

    pub fn upsert_word(&mut self, word: &str) -> String {
        let id = hash_word(word);
        // compute or reuse embedding
        let vec8 = if let Some(n) = self.nodes.get(&id) { n.vec8 } else { embed8(word) };
        if !self.nodes.contains_key(&id) {
            self.nodes.insert(id.clone(), WordNode { id: id.clone(), word: word.to_string(), vec8 });
        }
        // map to concept
        let cid = quantize_embed(&vec8);
        self.upsert_concept(&cid, word, vec8);
        self.word_to_concept.insert(id.clone(), cid);
        id
    }

    pub fn add_edge(&mut self, from: &str, to: &str, relation: &str, confidence: f32) {
        self.edges.push(Edge { from: from.to_string(), to: to.to_string(), relation: relation.to_string(), confidence });
    }

    pub fn word_id(&self, word: &str) -> Option<String> {
        let id = hash_word(word);
        if self.nodes.contains_key(&id) { Some(id) } else { None }
    }

    fn closeness(&self, a: &str, b: &str) -> f32 {
        // Prefer concept center closeness; fallback to word vectors
        if let (Some(ca), Some(cb)) = (self.word_to_concept.get(a), self.word_to_concept.get(b)) {
            if let (Some(na), Some(nb)) = (self.concept_nodes.get(ca), self.concept_nodes.get(cb)) {
                return cosine(&na.center, &nb.center).max(0.0);
            }
        }
        let Some(wa) = self.nodes.get(a) else { return 0.0 };
        let Some(wb) = self.nodes.get(b) else { return 0.0 };
        cosine(&wa.vec8, &wb.vec8).max(0.0)
    }

    fn neighbors_within(&self, from: &str, thresh: f32) -> Vec<String> {
        // Use concept proximity: return a representative word id per nearby concept
        let Some(cid_from) = self.word_to_concept.get(from) else { return vec![] };
        let Some(c_from) = self.concept_nodes.get(cid_from) else { return vec![] };
        let mut out = Vec::new();
        for (cid, c) in &self.concept_nodes {
            if cid == cid_from { continue; }
            let d = 1.0 - cosine(&c_from.center, &c.center);
            if d <= thresh {
                if let Some(w) = c.words.first() {
                    let wid = hash_word(w);
                    if self.nodes.contains_key(&wid) {
                        out.push(wid);
                    }
                }
            }
        }
        out
    }

    fn related_relation(&self, desired: &str, have: &str) -> bool {
        let (d, h) = (normalize_rel(desired), normalize_rel(have));
        if d == h { return true; }
        (d.starts_with("run") && h.starts_with("sprint")) || (d.starts_with("sprint") && h.starts_with("run"))
    }

    fn upsert_concept(&mut self, concept_id: &str, word: &str, vec8: [f32; 8]) {
        if let Some(c) = self.concept_nodes.get_mut(concept_id) {
            // Update center by running average
            let n = c.size as f32;
            for i in 0..8 {
                c.center[i] = (c.center[i] * n + vec8[i]) / (n + 1.0);
            }
            c.size += 1;
            if !c.words.iter().any(|w| w == word) {
                c.words.push(word.to_string());
            }
        } else {
            self.concept_nodes.insert(concept_id.to_string(), ConceptNode {
                id: concept_id.to_string(),
                center: vec8,
                size: 1,
                words: vec![word.to_string()],
            });
        }
    }

    pub fn surface_for(&self, concept_id: &str, script: Script) -> Option<String> {
        let c = self.concept_nodes.get(concept_id)?;
        for w in &c.words {
            if script_of(w) == script { return Some(w.clone()); }
        }
        // Fallback: any surface
        c.words.first().cloned()
    }

    /// Feedback to adjust edge confidence. Positive delta for thumbs-up, negative for thumbs-down.
    pub fn feedback(&mut self, from: &str, to: &str, relation: &str, delta: f32) {
        let reln = normalize_rel(relation);
        for e in &mut self.edges {
            if e.from == from && e.to == to && normalize_rel(&e.relation) == reln {
                e.confidence = (e.confidence + delta).clamp(0.0, 1.0);
            }
        }
    }

    /// Link multiple surface words to a single concept by merging their concepts.
    /// The first word becomes the canonical concept target.
    pub fn link_aliases(&mut self, words: &[&str]) {
        if words.is_empty() { return; }
        // Ensure words exist
        let mut ids: Vec<String> = Vec::new();
        for &w in words {
            let id = self.upsert_word(w);
            ids.push(id);
        }
        let target_id = ids[0].clone();
        let Some(target_cid) = self.word_to_concept.get(&target_id).cloned() else { return; };
        // Merge others into target concept
        for wid in ids.into_iter().skip(1) {
            if let Some(src_cid) = self.word_to_concept.get(&wid).cloned() {
                if src_cid != target_cid {
                    self.merge_concepts(&target_cid, &src_cid);
                }
                // Rebind word to target concept
                self.word_to_concept.insert(wid.clone(), target_cid.clone());
                if let Some(w) = self.nodes.get(&wid).map(|n| n.word.clone()) {
                    if let Some(tc) = self.concept_nodes.get_mut(&target_cid) {
                        if !tc.words.iter().any(|s| s == &w) {
                            tc.words.push(w);
                        }
                    }
                }
            }
        }
    }

    /// Merge source concept into target concept, updating center and size, and removing source.
    fn merge_concepts(&mut self, target: &str, source: &str) {
        if target == source { return; }
        let (t_exists, s_exists) = (self.concept_nodes.contains_key(target), self.concept_nodes.contains_key(source));
        if !t_exists || !s_exists { return; }
        let (t_center, t_size, t_words);
        {
            let t = self.concept_nodes.get(target).unwrap();
            t_center = t.center;
            t_size = t.size;
            t_words = t.words.clone();
        }
        let s = self.concept_nodes.remove(source).unwrap();
        let new_size = t_size + s.size;
        let mut new_center = [0.0f32; 8];
        for i in 0..8 {
            new_center[i] = (t_center[i] * t_size as f32 + s.center[i] * s.size as f32) / (new_size as f32);
        }
        // Merge word lists without double-borrowing
        let mut combined_words = t_words;
        for w in s.words {
            if !combined_words.contains(&w) {
                combined_words.push(w);
            }
        }
        if let Some(tc) = self.concept_nodes.get_mut(target) {
            tc.center = new_center;
            tc.size = new_size;
            tc.words = combined_words;
        }
        // Rebind words pointing to source concept
        for (_wid, cid) in self.word_to_concept.iter_mut() {
            if cid == source { *cid = target.to_string(); }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AskResult {
    pub yes: bool,
    pub path: Vec<String>,
    pub confidence: f32,
    pub answer_surface: Option<String>,
}

impl AskResult { pub fn no() -> Self { Self { yes: false, path: Vec::new(), confidence: 0.0, answer_surface: None } } }

fn tokenize(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for ch in s.chars() {
        if ch.is_alphanumeric() { cur.push(ch.to_ascii_lowercase()); }
        else {
            if !cur.is_empty() { out.push(cur.clone()); cur.clear(); }
        }
    }
    if !cur.is_empty() { out.push(cur); }
    out
}

fn extract_basic_triples(tokens: &[String]) -> Vec<(String,String,String)> {
    let mut res = Vec::new();
    if tokens.len() < 3 { return res; }
    // naive SVO: scan sliding window of 3
    for w in tokens.windows(3) {
        let a = &w[0]; let b = &w[1]; let c = &w[2];
        if is_stop(a) || is_stop(b) || is_stop(c) { continue; }
        res.push((a.clone(), b.clone(), c.clone()));
    }
    res
}

fn is_stop(tok: &str) -> bool {
    matches!(tok, "the"|"a"|"an"|"of"|"in"|"on"|"at"|"for"|"does"|"do"|"is")
}

fn hash_word(s: &str) -> String {
    let mut x: u64 = 1; // seed 1
    for b in s.as_bytes() {
        x ^= *b as u64;
        x = x.wrapping_mul(0x9E3779B185EBCA87);
        x ^= x >> 33;
    }
    format!("{:016x}", x)
}

fn embed8(s: &str) -> [f32; 8] {
    let mut v = [0f32; 8];
    let mut vowels = 0; let mut cons = 0; let mut digits = 0; let mut punct = 0;
    for ch in s.chars() {
        if ch.is_ascii_digit() { digits += 1; }
        else if ch.is_ascii_alphabetic() {
            match ch.to_ascii_lowercase() { 'a'|'e'|'i'|'o'|'u'| 'y' => vowels += 1, _ => cons += 1 }
        } else { punct += 1; }
    }
    let len = s.len() as f32;
    v[0] = len.min(12.0) / 12.0;
    v[1] = vowels as f32 / len.max(1.0);
    v[2] = cons as f32 / len.max(1.0);
    v[3] = digits as f32 / len.max(1.0);
    v[4] = punct as f32;
    v[5] = s.as_bytes().first().map(|b| *b as f32 / 255.0).unwrap_or(0.0);
    v[6] = s.as_bytes().last().map(|b| *b as f32 / 255.0).unwrap_or(0.0);
    v[7] = (vowels as f32 / 6.0).min(1.0);
    v
}

fn cosine(a: &[f32;8], b: &[f32;8]) -> f32 {
    let mut dot = 0.0; let mut na = 0.0; let mut nb = 0.0;
    for i in 0..8 { dot += a[i]*b[i]; na += a[i]*a[i]; nb += b[i]*b[i]; }
    if na == 0.0 || nb == 0.0 { return 0.0; }
    (dot / (na.sqrt()*nb.sqrt())).clamp(-1.0, 1.0)
}
