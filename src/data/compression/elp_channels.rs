/// Compression hash system for 12-byte thought encoding
/// Based on the 833x compression specification from COMPRESSION_HASHING.md
use serde::{Deserialize, Serialize};
use std::fmt;

/// 12-byte compression hash structure
/// Total: 12 bytes (24 hex characters)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompressionHash {
    /// WHO: Entity/Agent (2 bytes)
    pub who: [u8; 2],
    /// WHAT: Action/Concept (4 bytes)
    pub what: [u8; 4],
    /// WHERE: Position in flux (2 bytes)
    pub where_pos: [u8; 2],
    /// TENSOR: ELP channels (2 bytes)
    pub tensor: [u8; 2],
    /// COLOR: RGB blend (1 byte)
    pub color: u8,
    /// ATTRS: Metadata flags (1 byte)
    pub attrs: u8,
}

impl CompressionHash {
    /// Create from 12-byte array
    pub fn from_bytes(bytes: [u8; 12]) -> Self {
        Self {
            who: [bytes[0], bytes[1]],
            what: [bytes[2], bytes[3], bytes[4], bytes[5]],
            where_pos: [bytes[6], bytes[7]],
            tensor: [bytes[8], bytes[9]],
            color: bytes[10],
            attrs: bytes[11],
        }
    }

    /// Convert to 12-byte array
    pub fn to_bytes(&self) -> [u8; 12] {
        [
            self.who[0],
            self.who[1],
            self.what[0],
            self.what[1],
            self.what[2],
            self.what[3],
            self.where_pos[0],
            self.where_pos[1],
            self.tensor[0],
            self.tensor[1],
            self.color,
            self.attrs,
        ]
    }

    /// Create from hex string (24 characters)
    pub fn from_hex(hex: &str) -> Result<Self, String> {
        if hex.len() != 24 {
            return Err(format!("Invalid hex length: {} (expected 24)", hex.len()));
        }

        let bytes = hex_to_bytes(hex)?;
        Ok(Self::from_bytes(bytes))
    }

    /// Convert to hex string (24 characters)
    pub fn to_hex(&self) -> String {
        self.to_bytes()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }

    /// Extract flux position (0-9) from WHERE field
    pub fn flux_position(&self) -> u8 {
        // Use first byte of WHERE, modulo 10
        self.where_pos[0] % 10
    }

    /// Extract ELP channels from TENSOR field
    pub fn elp_channels(&self) -> ELPChannels {
        // Tensor byte 1: Ethos (0-9)
        // Tensor byte 2: Logos high nibble (0-9), Pathos low nibble (0-9)
        let ethos = (self.tensor[0] % 10) as f32;
        let logos = ((self.tensor[1] >> 4) % 10) as f32;
        let pathos = ((self.tensor[1] & 0x0F) % 10) as f32;

        ELPChannels {
            ethos,
            logos,
            pathos,
        }
    }

    /// Extract RGB color components
    pub fn rgb_color(&self) -> (u8, u8, u8) {
        let elp = self.elp_channels();
        (
            (elp.pathos * 28.0) as u8, // Red
            (elp.logos * 28.0) as u8,  // Green
            (elp.ethos * 28.0) as u8,  // Blue
        )
    }

    /// Check if this is a sacred position (3, 6, 9)
    pub fn is_sacred(&self) -> bool {
        let pos = self.flux_position();
        pos == 3 || pos == 6 || pos == 9
    }

    /// Get confidence score from ATTRS
    pub fn confidence(&self) -> f32 {
        // Attrs high nibble = confidence (0-15) mapped to 0.0-1.0
        ((self.attrs >> 4) as f32) / 15.0
    }
}

impl fmt::Display for CompressionHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// ELP (Ethos, Logos, Pathos) channels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ELPChannels {
    pub ethos: f32,  // Ethics/morality (0-9)
    pub logos: f32,  // Logic/reason (0-9)
    pub pathos: f32, // Emotion/feeling (0-9)
}

impl ELPChannels {
    /// Create from individual values
    pub fn new(ethos: f32, logos: f32, pathos: f32) -> Self {
        Self {
            ethos: ethos.clamp(0.0, 9.0),
            logos: logos.clamp(0.0, 9.0),
            pathos: pathos.clamp(0.0, 9.0),
        }
    }

    /// Encode into 2-byte tensor field
    pub fn to_tensor_bytes(&self) -> [u8; 2] {
        let ethos_byte = (self.ethos as u8).min(9);
        let logos_nibble = ((self.logos as u8).min(9)) << 4;
        let pathos_nibble = (self.pathos as u8).min(9);
        [ethos_byte, logos_nibble | pathos_nibble]
    }

    /// Calculate overall intensity
    pub fn intensity(&self) -> f32 {
        (self.ethos + self.logos + self.pathos) / 27.0 // Normalized to 0-1
    }

    /// Dominant channel
    pub fn dominant_channel(&self) -> &'static str {
        if self.ethos >= self.logos && self.ethos >= self.pathos {
            "ethos"
        } else if self.logos >= self.pathos {
            "logos"
        } else {
            "pathos"
        }
    }
}

/// Compress text into 12-byte hash
pub fn compress_text(text: &str, who: u16, position: u8, elp: ELPChannels) -> CompressionHash {
    // Simple deterministic compression
    // In production, this would use sophisticated NLP and embedding

    // WHO: User ID or entity (2 bytes)
    let who_bytes = who.to_be_bytes();

    // WHAT: Hash of text content (4 bytes)
    let mut what_hash: u32 = 0;
    for (i, c) in text.chars().enumerate() {
        what_hash = what_hash.wrapping_add((c as u32).wrapping_mul((i + 1) as u32));
    }
    let what_bytes = what_hash.to_be_bytes();

    // WHERE: Flux position + secondary location (2 bytes)
    let where_bytes = [position % 10, (text.len() % 256) as u8];

    // TENSOR: ELP channels (2 bytes)
    let tensor_bytes = elp.to_tensor_bytes();

    // COLOR: Derived from ELP
    let color = ((elp.ethos + elp.logos + elp.pathos) / 3.0 * 28.0) as u8;

    // ATTRS: Confidence + flags
    let confidence_nibble = 0xE0; // High confidence for direct input
    let flags_nibble = if text.len() > 100 { 0x01 } else { 0x00 };
    let attrs = confidence_nibble | flags_nibble;

    CompressionHash {
        who: who_bytes,
        what: what_bytes,
        where_pos: where_bytes,
        tensor: tensor_bytes,
        color,
        attrs,
    }
}

/// Decompress hash (currently returns metadata, full decompression would need Confidence Lake)
pub fn decompress_hash(hash: &CompressionHash) -> DecompressionResult {
    DecompressionResult {
        hash: hash.clone(),
        flux_position: hash.flux_position(),
        elp_channels: hash.elp_channels(),
        rgb_color: hash.rgb_color(),
        is_sacred: hash.is_sacred(),
        confidence: hash.confidence(),
        // Full text recovery would require Confidence Lake lookup
        recovered_text: None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecompressionResult {
    pub hash: CompressionHash,
    pub flux_position: u8,
    pub elp_channels: ELPChannels,
    pub rgb_color: (u8, u8, u8),
    pub is_sacred: bool,
    pub confidence: f32,
    pub recovered_text: Option<String>,
}

/// Helper: Convert hex string to 12-byte array
fn hex_to_bytes(hex: &str) -> Result<[u8; 12], String> {
    if hex.len() != 24 {
        return Err("Hex string must be exactly 24 characters".to_string());
    }

    let mut bytes = [0u8; 12];
    for i in 0..12 {
        let byte_str = &hex[i * 2..i * 2 + 2];
        bytes[i] = u8::from_str_radix(byte_str, 16)
            .map_err(|e| format!("Invalid hex at position {}: {}", i * 2, e))?;
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_round_trip() {
        let text = "What is consciousness?";
        let elp = ELPChannels::new(8.5, 8.0, 7.0);
        let hash = compress_text(text, 1001, 9, elp);

        assert_eq!(hash.flux_position(), 9);
        assert!(hash.is_sacred());

        let hex = hash.to_hex();
        assert_eq!(hex.len(), 24);

        let restored = CompressionHash::from_hex(&hex).unwrap();
        assert_eq!(restored, hash);
    }

    #[test]
    fn test_elp_channels() {
        let elp = ELPChannels::new(9.0, 8.5, 7.0);
        let bytes = elp.to_tensor_bytes();

        let hash = CompressionHash {
            who: [0, 1],
            what: [0, 0, 0, 0],
            where_pos: [9, 0],
            tensor: bytes,
            color: 0,
            attrs: 0,
        };

        let recovered = hash.elp_channels();
        assert_eq!(recovered.ethos, 9.0);
        assert_eq!(recovered.dominant_channel(), "ethos");
    }

    #[test]
    fn test_sacred_positions() {
        let hash = CompressionHash {
            who: [0, 0],
            what: [0, 0, 0, 0],
            where_pos: [3, 0],
            tensor: [0, 0],
            color: 0,
            attrs: 0,
        };
        assert!(hash.is_sacred());

        let hash2 = CompressionHash {
            who: [0, 0],
            what: [0, 0, 0, 0],
            where_pos: [5, 0],
            tensor: [0, 0],
            color: 0,
            attrs: 0,
        };
        assert!(!hash2.is_sacred());
    }
}
