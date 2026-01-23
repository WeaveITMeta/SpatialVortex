use spatial_vortex::angle::compute_next;

#[test]
fn angle_delta_scales_with_digit_magnitude() {
    let alpha = 0.15;
    let a0 = 0.0;
    let d_small = 2u8;
    let d_large = 8u8;
    let a1_small = compute_next(a0, d_small, alpha);
    let a1_large = compute_next(a0, d_large, alpha);
    assert!(a1_large - a0 > a1_small - a0);
}
