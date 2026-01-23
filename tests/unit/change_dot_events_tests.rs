use spatial_vortex::change_dot::{ChangeDotEvent, ChangeDotIter};
use spatial_vortex::flux_matrix::FluxMatrixEngine;

#[test]
fn iterator_survives_24_steps_and_emits_loop() {
    let engine = FluxMatrixEngine::new();
    let mut it = ChangeDotIter::from_value(1, &engine);
    let mut events = Vec::new();
    for _ in 0..24 {
        let ev = it.next().expect("event");
        events.push(ev);
    }
    let has_loop = events.iter().any(|e| matches!(e, ChangeDotEvent::Loop { .. }));
    assert!(has_loop, "iterator should emit Loop event when returning to 1");
}

#[test]
fn sacred_event_every_third_yield() {
    let engine = FluxMatrixEngine::new();
    let mut it = ChangeDotIter::from_value(1, &engine);
    // Collect first 12 yields
    let mut sacred_positions = Vec::new();
    for i in 1..=12 {
        let ev = it.next().unwrap();
        if let ChangeDotEvent::SacredHit { position } = ev { sacred_positions.push((i, position)); }
    }
    // Expect at least 3 sacred hits and all at position 3 per current rule
    assert!(sacred_positions.len() >= 3);
    for (_, pos) in sacred_positions { assert_eq!(pos, 3); }
}
