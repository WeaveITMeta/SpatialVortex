use bevy::prelude::Component;
use anyhow::Result;
use murmur3::murmur3_32;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Cursor;

pub mod model;
pub use model::VortexNet;
pub mod seeds;
pub mod transformer;
pub use transformer::TransformerNet;
pub mod mapping;
pub use mapping::{normalize_text, map_text_to_digit, center_digit};

#[derive(Serialize, Deserialize, Clone, Component)]
pub struct VortexCore {
    pub pattern: [i32; 9],
    pub seeds: Vec<(String, u32)>,
    idx: usize,
    angle: f32,
}

impl VortexCore {
    pub fn new() -> Self {
        Self {
            pattern: [3, 6, 9, 1, 2, 4, 8, 7, 5],
            seeds: Vec::new(),
            idx: 0,
            angle: 0.0,
        }
    }

    pub fn new_seed(&mut self, subject: &str) -> u32 {
        let mut cursor = Cursor::new(subject.as_bytes());
        let hash = murmur3_32(&mut cursor, 0).unwrap_or(0);
        let x = (hash as f64 * 0.369_f64).floor() as u32;
        let seed = x % 1000;
        self.seeds.push((subject.to_string(), seed));
        seed
    }

    pub fn next_digit(&mut self) -> i32 {
        let d = self.pattern[self.idx];
        self.idx = (self.idx + 1) % self.pattern.len();
        d
    }

    pub fn lerp_angle(&mut self, digit: i32) -> f32 {
        let target = std::f32::consts::TAU * (digit as f32 / 9.0);
        let alpha = 0.15_f32;
        self.angle = self.angle + (target - self.angle) * alpha;
        self.angle
    }

    pub fn save_seeds(&self, path: &str) -> Result<()> {
        let data = serde_json::to_string(&self.seeds)?;
        fs::write(path, data)?;
        Ok(())
    }

    pub fn load_seeds(&mut self, path: &str) -> Result<()> {
        if let Ok(bytes) = fs::read(path) {
            if let Ok(v) = serde_json::from_slice::<Vec<(String, u32)>>(&bytes) {
                self.seeds = v;
            }
        }
        Ok(())
    }

    pub fn tokenize_simple(text: &str) -> Vec<String> {
        text.split_whitespace().map(|s| s.to_string()).collect()
    }
}

#[cfg(feature = "nlp")]
pub fn tokenize_to_seeds(text: &str, core: &mut VortexCore) -> Vec<u32> {
    use tokenizers::Tokenizer;
    let tokenizer = Tokenizer::from_file("tokenizer.json").unwrap();
    let encoding = tokenizer.encode(text, true).unwrap();
    let ids = encoding.get_ids();
    ids.iter().map(|&id| core.new_seed(&format!("token_{}", id))).collect()
}

#[cfg(feature = "stt")]
pub fn stream_audio_to_seeds(audio_bytes: &[u8], core: &mut VortexCore) -> String {
    use whisper_rs::{FullParams, SamplingStrategy, WhisperContext};
    let ctx = WhisperContext::new("base.en").unwrap();
    let mut state = ctx.create_state().unwrap();
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_n_threads(4);
    state.full(params, audio_bytes).unwrap();
    let text = state.full_get_segment_text(0).unwrap_or_default();
    #[cfg(feature = "nlp")]
    {
        let seeds = tokenize_to_seeds(&text, core);
        return seeds.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(".");
    }
    #[allow(unreachable_code)]
    text
}

#[cfg(feature = "stt")]
pub mod stt {
    use anyhow::Result;
    use whisper_rs::*;
    use std::fs::File;
    use std::io::Read;

    pub fn transcribe_file(_model_path: &str, _wav_path: &str) -> Result<String> {
        Ok(String::new())
    }
}
