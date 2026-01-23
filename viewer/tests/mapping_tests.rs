use windsurf::{map_text_to_digit, center_digit, normalize_text};
use rand::{Rng, distributions::Alphanumeric};
use rand::distributions::Distribution;

#[test]
fn numeric_string_128_maps_to_8() {
    let d = map_text_to_digit("128");
    assert_eq!(d, 8);
}

#[test]
fn normalize_wendsurf_to_windsurf() {
    let s = normalize_text("I love wendsurf");
    assert!(s.contains("windsurf"));
}

#[test]
fn random_strings_center_within_range() {
    let mut rng = rand::thread_rng();
    for _ in 0..100 {
        let len = rng.gen_range(3..32);
        let s: String = rand::thread_rng().sample_iter(&Alphanumeric).take(len).map(char::from).collect();
        let d = map_text_to_digit(&s);
        let c = center_digit(d);
        assert!(c >= -4 && c <= 4, "centered digit out of range: {} from {}", c, d);
    }
}
