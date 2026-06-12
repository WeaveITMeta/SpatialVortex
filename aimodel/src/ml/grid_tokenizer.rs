//! RGBA-Factorized Grid Tokenization
//!
//! ARC grids are 2D arrays of color indices 0-15. The engine palette maps each
//! index to RGBA; each cell is factorized into 4 byte-tokens (R, G, B, A), so the
//! token vocabulary is 256 byte values plus a handful of special tokens. This lets
//! a grid-diffusion world model share one byte-level vocabulary across grids and
//! actions, and detokenize unseen RGBA values via nearest-palette match.

/// Byte vocabulary size: tokens 0..=255 are raw channel bytes.
pub const BYTE_VOCAB: u16 = 256;
/// Masked-cell token for diffusion denoising.
pub const TOK_MASK: u16 = 256;
/// Padding token.
pub const TOK_PAD: u16 = 257;
/// Separator between grid and action segments.
pub const TOK_SEP: u16 = 258;
/// Base token for action ids: action `a` (0=RESET, 1..=7=ACTION1..7) maps to `ACT_BASE + a`.
pub const TOK_ACT_BASE: u16 = 259;
/// Number of action tokens (RESET + ACTION1..ACTION7).
pub const NUM_ACTIONS: u16 = 8;
/// Total vocabulary size including special and action tokens.
pub const VOCAB_SIZE: u16 = TOK_ACT_BASE + NUM_ACTIONS;

/// Standard ARC palette (indices 0-9) plus 6 reserved distinct colors (10-15).
const PALETTE: [[u8; 4]; 16] = [
    [0, 0, 0, 255],       // 0 black
    [0, 116, 217, 255],   // 1 blue
    [255, 65, 54, 255],   // 2 red
    [46, 204, 64, 255],   // 3 green
    [255, 220, 0, 255],   // 4 yellow
    [170, 170, 170, 255], // 5 gray
    [240, 18, 190, 255],  // 6 magenta
    [255, 133, 27, 255],  // 7 orange
    [127, 219, 255, 255], // 8 cyan
    [135, 12, 37, 255],   // 9 maroon
    [255, 255, 255, 255], // 10 white (reserved)
    [85, 85, 85, 255],    // 11 dark gray (reserved)
    [0, 68, 0, 255],      // 12 dark green (reserved)
    [120, 60, 180, 255],  // 13 purple (reserved)
    [255, 170, 200, 255], // 14 pink (reserved)
    [60, 30, 10, 255],    // 15 brown (reserved)
];

/// Tokenizes ARC color grids into RGBA byte-token sequences and back.
#[derive(Debug, Clone, Default)]
pub struct GridTokenizer;

impl GridTokenizer {
    pub fn new() -> Self {
        Self
    }

    /// RGBA for a color index (indices >15 clamp to 15).
    pub fn rgba(&self, color: u8) -> [u8; 4] {
        PALETTE[(color as usize).min(15)]
    }

    /// Nearest palette index to an arbitrary RGBA by squared Euclidean distance.
    pub fn nearest_color(&self, rgba: [u8; 4]) -> u8 {
        let dist = |p: &[u8; 4]| -> u32 {
            p.iter()
                .zip(rgba.iter())
                .map(|(&a, &b)| {
                    let d = a as i32 - b as i32;
                    (d * d) as u32
                })
                .sum()
        };
        PALETTE
            .iter()
            .enumerate()
            .min_by_key(|(_, p)| dist(p))
            .map(|(i, _)| i as u8)
            .unwrap_or(0)
    }

    /// Token for an action id (0=RESET, 1..=7=ACTION1..7).
    pub fn action_token(&self, action_id: u8) -> u16 {
        TOK_ACT_BASE + (action_id as u16).min(NUM_ACTIONS - 1)
    }

    /// Each cell becomes 4 byte-tokens (R, G, B, A), row-major.
    pub fn tokenize(&self, grid: &[Vec<u8>]) -> Vec<u16> {
        let mut tokens = Vec::with_capacity(grid.iter().map(|r| r.len() * 4).sum());
        for row in grid {
            for &cell in row {
                for byte in self.rgba(cell) {
                    tokens.push(byte as u16);
                }
            }
        }
        tokens
    }

    /// Inverse of `tokenize` via nearest-palette match.
    ///
    /// Special tokens (>= 256) decode their low byte; short inputs pad with
    /// color 0. Extra tokens beyond `width * height * 4` are ignored.
    pub fn detokenize(&self, tokens: &[u16], width: usize, height: usize) -> Vec<Vec<u8>> {
        let mut grid = vec![vec![0u8; width]; height];
        for (i, cell) in grid.iter_mut().flatten().enumerate() {
            let chunk = &tokens.get(i * 4..i * 4 + 4);
            if let Some(chunk) = chunk {
                let rgba = [
                    (chunk[0] & 0xFF) as u8,
                    (chunk[1] & 0xFF) as u8,
                    (chunk[2] & 0xFF) as u8,
                    (chunk[3] & 0xFF) as u8,
                ];
                *cell = self.nearest_color(rgba);
            }
        }
        grid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_all_colors() {
        let tok = GridTokenizer::new();
        // 4x4 grid covering all 16 palette indices
        let grid: Vec<Vec<u8>> = (0..4).map(|r| (0..4).map(|c| r * 4 + c).collect()).collect();
        let tokens = tok.tokenize(&grid);
        assert_eq!(tokens.len(), 4 * 4 * 4);
        assert!(tokens.iter().all(|&t| t < BYTE_VOCAB));
        assert_eq!(tok.detokenize(&tokens, 4, 4), grid);
    }

    #[test]
    fn round_trip_rectangular() {
        let tok = GridTokenizer::new();
        let grid = vec![vec![9, 10, 10, 5, 11], vec![0, 4, 4, 4, 9]];
        let tokens = tok.tokenize(&grid);
        assert_eq!(tok.detokenize(&tokens, 5, 2), grid);
    }

    #[test]
    fn unknown_rgba_maps_to_nearest() {
        let tok = GridTokenizer::new();
        // Slightly-off blue should snap to index 1 (0,116,217)
        assert_eq!(tok.nearest_color([10, 110, 210, 255]), 1);
        // Near-black snaps to 0
        assert_eq!(tok.nearest_color([5, 5, 5, 250]), 0);
        // Detokenize a hand-built token stream with perturbed bytes
        let tokens = vec![250u16, 215, 5, 255]; // ~yellow (255,220,0)
        assert_eq!(tok.detokenize(&tokens, 1, 1), vec![vec![4]]);
    }

    #[test]
    fn special_tokens_distinct() {
        let tok = GridTokenizer::new();
        assert_eq!(TOK_MASK, 256);
        assert_eq!(TOK_PAD, 257);
        assert_eq!(TOK_SEP, 258);
        assert_eq!(tok.action_token(0), 259);
        assert_eq!(tok.action_token(7), 266);
        assert_eq!(VOCAB_SIZE, 267);
    }

    #[test]
    fn palette_entries_are_distinct() {
        let tok = GridTokenizer::new();
        // Every palette color must round-trip exactly through nearest match
        for i in 0u8..16 {
            assert_eq!(tok.nearest_color(tok.rgba(i)), i, "palette index {i} not nearest to itself");
        }
    }
}
