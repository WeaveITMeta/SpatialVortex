use murmur3::murmur3_32;
use std::io::Cursor;

pub fn normalize_text(s: &str) -> String {
    let t = s.to_lowercase();
    if t.contains("wendsurf") { return t.replace("wendsurf", "windsurf"); }
    t
}

pub fn map_text_to_digit(s: &str) -> u8 {
    if s.chars().all(|c| c.is_ascii_digit()) {
        let d = s.as_bytes().last().map(|b| b - b'0').unwrap_or(9);
        return if d == 0 { 9 } else { d as u8 };
    }
    let mut cursor = Cursor::new(s.as_bytes());
    let h = murmur3_32(&mut cursor, 0).unwrap_or(0) as u64;
    let scaled = ((h as f64) * 0.369_f64).floor() as u64;
    let d = (scaled % 9) as u8 + 1;
    d
}

pub fn center_digit(d: u8) -> i8 { d as i8 - 5 }
