// Alphabet flux series test: verifies A..Z labeling and doubling-mod-10 digit spiral

struct FluxMatrix {
    nodes: Vec<(i32, char)>,
}

impl FluxMatrix {
    fn generate_series(&mut self, _subject: &str, seed: i32, count: usize) {
        let mut current = seed;
        let mut value: char = 'A';
        for _ in 0..count {
            self.nodes.push((current, value));
            // next digit: doubling mod 10 keeps a single digit spiral 1→2→4→8→6→2→...
            current = (current * 2) % 10;
            // next letter: A..Z then wrap
            let v = value as u8;
            let next = if v == b'Z' { b'A' } else { v + 1 };
            value = next as char;
        }
    }
}

#[test]
fn test_alphabet_flux() {
    let mut matrix = FluxMatrix { nodes: vec![] };
    let subject = "test_alpha";
    let seed = 1;
    matrix.generate_series(subject, seed, 26);

    // length and first/last labels
    assert_eq!(matrix.nodes.len(), 26);
    assert_eq!(matrix.nodes[0].1, 'A');
    assert_eq!(matrix.nodes[1].1, 'B');
    assert_eq!(matrix.nodes[25].1, 'Z');

    // leading digit spiral 1,2,4,8,6
    let first5: Vec<i32> = matrix.nodes.iter().take(5).map(|(d, _)| *d).collect();
    assert_eq!(first5, vec![1, 2, 4, 8, 6]);

    // ensure the 4-cycle {2,4,8,6} repeats after the first element
    for (i, (d, _)) in matrix.nodes.iter().enumerate().skip(1) {
        let cycle = [2, 4, 8, 6];
        let expected = cycle[(i - 1) % 4];
        assert_eq!(*d, expected, "index {i} expected {expected} got {d}");
    }

    // final digit at index 25 given seed=1 should be 2
    assert_eq!(matrix.nodes[25].0, 2);
}
