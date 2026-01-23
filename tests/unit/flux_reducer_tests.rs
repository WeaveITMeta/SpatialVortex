use spatial_vortex::flux_matrix::FluxMatrixEngine;

#[test]
fn reduce_digits_known_values() {
    let engine = FluxMatrixEngine::new();
    assert_eq!(engine.reduce_digits(0), 0);
    assert_eq!(engine.reduce_digits(9), 9);
    assert_eq!(engine.reduce_digits(10), 1);
    assert_eq!(engine.reduce_digits(128), 2);
    assert_eq!(engine.reduce_digits(1248751), 1);
}
