use spatial_vortex::grammar_graph::GrammarGraph;

fn contains_cjk(s: &str) -> bool {
    s.chars().any(|ch| {
        let u = ch as u32;
        (0x4E00..=0x9FFF).contains(&u) || (0x3040..=0x309F).contains(&u) || (0x30A0..=0x30FF).contains(&u)
    })
}

#[test]
fn dog_runs_park_multilingual_concepts() {
    let mut g = GrammarGraph::new();

    // Teach graph with three languages (surface forms)
    g.say("dog runs park");
    g.say("perro corre parque");
    g.say("犬 走る 公園");

    // Link aliases to unify concept anchors
    g.link_aliases(&["dog", "perro", "犬"]);
    g.link_aliases(&["park", "parque", "公園"]);

    // Ask in Japanese (CJK) → expect CJK surface back
    let ans_cjk = g.ask("犬 走る 公園");
    assert!(ans_cjk.yes, "Expected path exists for CJK");
    let surface = ans_cjk.answer_surface.as_deref().unwrap_or("");
    assert!(contains_cjk(surface), "Expected CJK surface, got: {}", surface);

    // Ask in Spanish (Latin) → expect Latin surface back
    let ans_lat = g.ask("perro corre parque");
    assert!(ans_lat.yes, "Expected path exists for Latin");
    let surface_lat = ans_lat.answer_surface.as_deref().unwrap_or("");
    assert!(!contains_cjk(surface_lat), "Expected Latin surface, got CJK: {}", surface_lat);
}
