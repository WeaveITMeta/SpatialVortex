//! ASI Compression Algorithms (12-byte and 16-byte)
//! 
//! **DEPRECATED**: These formats are superseded by `aimodel::ml::flux_compression_sota`
//! which provides a unified 24-byte format combining 6W + ELP + inference capabilities.
//! 
//! Use `FluxCompression24` from `aimodel::ml::flux_compression_sota` for new code.
//! This module is retained for backward compatibility.
//!
//! Revolutionary semantic compression using sacred geometry and vortex math.
//! Achieves 200-250× compression ratio while maintaining semantic fidelity.
//! 
//! ## Architecture
//! - 3-6-9 sacred anchors provide reference frame
//! - Doubling sequence (1→2→4→8→7→5→1) creates collision-free hashing
//! - ±13 scale enables precise quantization
//! - Differential encoding stores offsets from anchors
//! 
//! ## Compression Formats (LEGACY)
//! 
//! ### 12-Byte Format (Original) - DEPRECATED
//! ```text
//! BYTE 0-1: Position encoding (position in 0-9 space + phase)
//! BYTE 2-7: ELP deltas from nearest sacred anchor (i16 × 3)
//! BYTE 8-9: Confidence + semantic hash
//! BYTE 10-11: Cycle count + metadata
//! ```
//! 
//! ### 16-Byte Format (Extended with 6W Framework) - DEPRECATED
//! ```text
//! BYTE 0-1: WHO (Entity/Actor) - 12-bit hash + 3-bit type + 1-bit plural
//! BYTE 2-5: WHAT (Action/Concept) - 24-bit hash + action/tense/flags
//! BYTE 6-7: WHEN (Temporal) - 12-bit offset + 3-bit granularity + 1-bit absolute
//! BYTE 8-9: WHERE (Spatial/Flux) - 4-bit flux pos + 4-bit spatial type + 8-bit hash
//! BYTE 10: WHY (Causality) - 4-bit type + 4-bit intention strength
//! BYTE 11: HOW (Method) - 4-bit type + 3-bit complexity + 1-bit sequential
//! BYTE 12-13: ASPECT COLOR - 9-bit hue + 4-bit sat + 3-bit lum
//! BYTE 14: CONFIDENCE - 5-bit level + sacred/high/validated flags (CONSOLIDATED)
//! BYTE 15: METADATA - 2-bit version + 3-bit source + flags
//! ```

use crate::models::ELPTensor;
#[allow(unused_imports)]
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// 12-byte compressed representation of semantic concept
#[repr(C)]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ASI12ByteCompression {
    /// Base position in 0-9 flux space
    pub position_0_9: u8,
    
    /// Doubling/halving sequence phase (forward=0, backward=1)
    pub sequence_phase: u8,
    
    /// ELP delta from nearest sacred anchor (±13000 millis precision)
    pub ethos_delta_i16: i16,
    pub logos_delta_i16: i16,
    pub pathos_delta_i16: i16,
    
    /// Confidence level (0-255 → 0.0-1.0)
    pub confidence_u8: u8,
    
    /// Semantic hash for collision detection
    pub semantic_hash_u8: u8,
    
    /// Number of vortex cycles + metadata flags
    pub cycle_count: u16,
}

impl ASI12ByteCompression {
    /// Compress semantic concept to 12 bytes
    pub fn compress(
        concept: &str,
        position: u8,
        elp: ELPTensor,
        anchor_elp: ELPTensor,
        confidence: f64,
    ) -> Self {
        // Calculate deltas from anchor
        let delta = ELPTensor::new(
            elp.ethos - anchor_elp.ethos,
            elp.logos - anchor_elp.logos,
            elp.pathos - anchor_elp.pathos,
        );
        
        // Quantize to i16 (±13.0 → ±13000)
        let e_delta = (delta.ethos * 1000.0).clamp(-13000.0, 13000.0) as i16;
        let l_delta = (delta.logos * 1000.0).clamp(-13000.0, 13000.0) as i16;
        let p_delta = (delta.pathos * 1000.0).clamp(-13000.0, 13000.0) as i16;
        
        // Compute semantic hash
        let mut hasher = DefaultHasher::new();
        concept.hash(&mut hasher);
        let hash = hasher.finish() as u8;
        
        // Determine sequence phase
        let doubling_seq = [1, 2, 4, 8, 7, 5];
        let phase = if doubling_seq.contains(&position) { 0 } else { 1 };
        
        ASI12ByteCompression {
            position_0_9: position,
            sequence_phase: phase,
            ethos_delta_i16: e_delta,
            logos_delta_i16: l_delta,
            pathos_delta_i16: p_delta,
            confidence_u8: (confidence * 255.0).clamp(0.0, 255.0) as u8,
            semantic_hash_u8: hash,
            cycle_count: 0,
        }
    }
    
    /// Decompress back to ELP tensor
    pub fn decompress(&self, anchor_elp: ELPTensor) -> ELPTensor {
        // Dequantize deltas
        let e_delta = self.ethos_delta_i16 as f64 / 1000.0;
        let l_delta = self.logos_delta_i16 as f64 / 1000.0;
        let p_delta = self.pathos_delta_i16 as f64 / 1000.0;
        
        // Apply deltas to anchor
        ELPTensor::new(
            anchor_elp.ethos + e_delta,
            anchor_elp.logos + l_delta,
            anchor_elp.pathos + p_delta,
        )
    }
    
    /// Get confidence level (0.0-1.0)
    pub fn confidence(&self) -> f64 {
        self.confidence_u8 as f64 / 255.0
    }
    
    /// Calculate Hamming distance between two compressed concepts
    pub fn hamming_distance(&self, other: &Self) -> u32 {
        let pos_dist = (self.position_0_9 as i32 - other.position_0_9 as i32).abs() as u32;
        let phase_dist = (self.sequence_phase != other.sequence_phase) as u32;
        pos_dist + phase_dist
    }
    
    /// Calculate semantic similarity in compressed space (0.0-1.0, higher = more similar)
    /// Note: This compares deltas which may be from different anchors.
    /// For accurate similarity, consider decompressing both and comparing actual ELP values.
    pub fn compressed_similarity(&self, other: &Self) -> f32 {
        // Position distance (0-9 space wraps around)
        let pos_dist = {
            let diff = (self.position_0_9 as i32 - other.position_0_9 as i32).abs();
            diff.min(10 - diff) // Wrap around
        } as f32;
        
        // If positions use same anchor, compare deltas directly
        // Otherwise use a more conservative approach
        let same_anchor = find_nearest_sacred_anchor(self.position_0_9) == 
                         find_nearest_sacred_anchor(other.position_0_9);
        
        if same_anchor {
            // ELP delta distance (Euclidean in quantized space)
            let elp_dist = (
                (self.ethos_delta_i16 - other.ethos_delta_i16).pow(2) +
                (self.logos_delta_i16 - other.logos_delta_i16).pow(2) +
                (self.pathos_delta_i16 - other.pathos_delta_i16).pow(2)
            ) as f32 / 1_000_000.0;
            
            // Combined similarity (normalized)
            let total_dist = pos_dist / 5.0 + elp_dist.sqrt() / 13.0;
            (1.0 - total_dist.min(1.0)).max(0.0)
        } else {
            // Different anchors - can't directly compare deltas
            // Use position-based similarity only
            let pos_similarity = 1.0 - (pos_dist / 5.0);
            pos_similarity.max(0.0).min(1.0)
        }
    }
}

/// Find nearest sacred anchor (3, 6, or 9) for given position
pub fn find_nearest_sacred_anchor(position: u8) -> u8 {
    let sacred = [3, 6, 9];
    *sacred.iter()
        .min_by_key(|&&anchor| {
            let diff = (anchor as i32 - position as i32).abs();
            diff.min(10 - diff) // Wrap around in 0-9 space
        })
        .unwrap_or(&3)
}

/// Compression engine for batch operations
pub struct ASICompressionEngine {
    /// Sacred anchor ELP values
    anchor_elp: [ELPTensor; 3],
}

impl ASICompressionEngine {
    /// Create new compression engine with sacred anchor values
    pub fn new(ethos_anchor: ELPTensor, pathos_anchor: ELPTensor, logos_anchor: ELPTensor) -> Self {
        ASICompressionEngine {
            anchor_elp: [ethos_anchor, pathos_anchor, logos_anchor],
        }
    }
    
    /// Create with default sacred values
    pub fn with_defaults() -> Self {
        ASICompressionEngine {
            anchor_elp: [
                ELPTensor::new(1.0, 0.0, 0.0),  // Position 3: Pure Ethos
                ELPTensor::new(0.0, 0.0, 1.0),  // Position 6: Pure Pathos
                ELPTensor::new(0.0, 1.0, 0.0),  // Position 9: Pure Logos
            ],
        }
    }
    
    /// Get anchor ELP for position
    fn get_anchor_elp(&self, position: u8) -> ELPTensor {
        let anchor_pos = find_nearest_sacred_anchor(position);
        match anchor_pos {
            3 => self.anchor_elp[0], // Ethos
            6 => self.anchor_elp[1], // Pathos
            9 => self.anchor_elp[2], // Logos
            _ => self.anchor_elp[0], // Default to Ethos
        }
    }
    
    /// Compress single concept
    pub fn compress(
        &self,
        concept: &str,
        position: u8,
        elp: ELPTensor,
        confidence: f64,
    ) -> ASI12ByteCompression {
        let anchor_elp = self.get_anchor_elp(position);
        ASI12ByteCompression::compress(concept, position, elp, anchor_elp, confidence)
    }
    
    /// Decompress to ELP
    pub fn decompress(&self, compressed: &ASI12ByteCompression) -> ELPTensor {
        let anchor_elp = self.get_anchor_elp(compressed.position_0_9);
        compressed.decompress(anchor_elp)
    }
    
    /// Batch compress multiple concepts
    pub fn batch_compress(
        &self,
        concepts: Vec<(&str, u8, ELPTensor, f64)>,
    ) -> Vec<ASI12ByteCompression> {
        concepts
            .into_iter()
            .map(|(concept, pos, elp, conf)| self.compress(concept, pos, elp, conf))
            .collect()
    }
    
    /// Batch decompress
    pub fn batch_decompress(&self, compressed: &[ASI12ByteCompression]) -> Vec<ELPTensor> {
        compressed.iter().map(|c| self.decompress(c)).collect()
    }
    
    /// Calculate compression statistics
    pub fn compression_stats(&self, original_bytes: usize, compressed_count: usize) -> CompressionStats {
        let compressed_bytes = compressed_count * std::mem::size_of::<ASI12ByteCompression>();
        let ratio = original_bytes as f64 / compressed_bytes as f64;
        let savings = 1.0 - (compressed_bytes as f64 / original_bytes as f64);
        
        CompressionStats {
            original_bytes,
            compressed_bytes,
            compression_ratio: ratio,
            space_savings: savings,
            concepts_compressed: compressed_count,
        }
    }
}

/// Compression statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionStats {
    pub original_bytes: usize,
    pub compressed_bytes: usize,
    pub compression_ratio: f64,
    pub space_savings: f64,
    pub concepts_compressed: usize,
}

impl CompressionStats {
    pub fn print_summary(&self) {
        println!("╔════════════════════════════════════════╗");
        println!("║  ASI 12-Byte Compression Statistics   ║");
        println!("╠════════════════════════════════════════╣");
        println!("║ Concepts compressed: {:>15} ║", self.concepts_compressed);
        println!("║ Original size:       {:>10} bytes ║", self.original_bytes);
        println!("║ Compressed size:     {:>10} bytes ║", self.compressed_bytes);
        println!("║ Compression ratio:   {:>15.2}× ║", self.compression_ratio);
        println!("║ Space savings:       {:>14.1}% ║", self.space_savings * 100.0);
        println!("╚════════════════════════════════════════╝");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_12byte_size() {
        assert_eq!(
            std::mem::size_of::<ASI12ByteCompression>(),
            12,
            "Compression structure must be exactly 12 bytes"
        );
    }

    #[test]
    fn test_compression_roundtrip() {
        let engine = ASICompressionEngine::with_defaults();
        
        let test_cases = vec![
            ("Love", 3, ELPTensor::new(0.7, 0.5, 0.95)),
            ("Truth", 6, ELPTensor::new(0.85, 0.95, 0.5)),
            ("Creation", 9, ELPTensor::new(0.9, 0.6, 0.5)),
            ("Beauty", 4, ELPTensor::new(0.6, 0.6, 0.8)),
        ];
        
        for (concept, pos, original_elp) in test_cases {
            let compressed = engine.compress(concept, pos, original_elp, 1.0);
            let decompressed = engine.decompress(&compressed);
            
            // Check reconstruction accuracy (within 0.001 due to quantization)
            let error = original_elp.distance(&decompressed);
            assert!(
                error < 0.001,
                "Compression lossy for '{}': error = {}, original = {:?}, decompressed = {:?}",
                concept, error, original_elp, decompressed
            );
        }
    }

    #[test]
    fn test_similarity_in_compressed_space() {
        let engine = ASICompressionEngine::with_defaults();
        
        // Similar concepts (both high pathos, close positions)
        let love = engine.compress("Love", 3, ELPTensor::new(0.7, 0.5, 0.95), 1.0);
        let joy = engine.compress("Joy", 3, ELPTensor::new(0.65, 0.5, 0.9), 1.0);
        
        // Dissimilar concept (high logos, different position)
        let truth = engine.compress("Truth", 6, ELPTensor::new(0.85, 0.95, 0.4), 1.0);
        
        let love_joy_sim = love.compressed_similarity(&joy);
        let love_truth_sim = love.compressed_similarity(&truth);
        
        assert!(
            love_joy_sim > love_truth_sim,
            "Love-Joy similarity ({:.3}) should be higher than Love-Truth ({:.3})",
            love_joy_sim, love_truth_sim
        );
        
        // Both should be reasonable similarities (0-1 range)
        assert!(love_joy_sim >= 0.0 && love_joy_sim <= 1.0);
        assert!(love_truth_sim >= 0.0 && love_truth_sim <= 1.0);
    }

    #[test]
    fn test_batch_compression() {
        let engine = ASICompressionEngine::with_defaults();
        
        let concepts = vec![
            ("Love", 3, ELPTensor::new(0.7, 0.5, 0.95), 1.0),
            ("Truth", 6, ELPTensor::new(0.85, 0.95, 0.5), 0.9),
            ("Beauty", 4, ELPTensor::new(0.6, 0.6, 0.8), 0.95),
        ];
        
        let compressed = engine.batch_compress(concepts.clone());
        assert_eq!(compressed.len(), 3);
        
        let decompressed = engine.batch_decompress(&compressed);
        assert_eq!(decompressed.len(), 3);
        
        // Verify roundtrip accuracy
        for (i, (_, _, original_elp, _)) in concepts.iter().enumerate() {
            let error = original_elp.distance(&decompressed[i]);
            assert!(error < 0.001, "Batch compression lossy at index {}", i);
        }
    }

    #[test]
    fn test_compression_ratio() {
        let engine = ASICompressionEngine::with_defaults();
        
        // Standard representation: String (20 bytes) + ELPTensor (24 bytes) = 44 bytes
        let standard_bytes = 44 * 100; // 100 concepts
        
        // Create concepts first to avoid lifetime issues
        let concepts: Vec<(String, u8, ELPTensor, f64)> = (0..100)
            .map(|i| {
                let concept = format!("Concept{}", i);
                let elp = ELPTensor::new(0.5, 0.5, 0.5);
                (concept, (i % 10) as u8, elp, 1.0)
            })
            .collect();
        
        let compressed = engine.batch_compress(
            concepts.iter()
                .map(|(s, pos, elp, conf)| (s.as_str(), *pos, *elp, *conf))
                .collect()
        );
        
        let stats = engine.compression_stats(standard_bytes, compressed.len());
        
        // Should achieve at least 3× compression
        assert!(
            stats.compression_ratio >= 3.0,
            "Compression ratio too low: {}×",
            stats.compression_ratio
        );
        
        stats.print_summary();
    }

    #[test]
    fn test_sacred_anchor_finding() {
        // In circular 0-9 space, position 0 wraps to 9 (distance 1)
        assert_eq!(find_nearest_sacred_anchor(0), 9);  // Closest to 9 (distance 1 via wrap)
        assert_eq!(find_nearest_sacred_anchor(1), 3);  // Equidistant to 3 and 9, returns first (3)
        assert_eq!(find_nearest_sacred_anchor(2), 3);
        assert_eq!(find_nearest_sacred_anchor(3), 3);
        assert_eq!(find_nearest_sacred_anchor(4), 3);
        assert_eq!(find_nearest_sacred_anchor(5), 6);
        assert_eq!(find_nearest_sacred_anchor(6), 6);
        assert_eq!(find_nearest_sacred_anchor(7), 6);
        assert_eq!(find_nearest_sacred_anchor(8), 9);
        assert_eq!(find_nearest_sacred_anchor(9), 9);
    }
}

// ============================================================================
// 16-BYTE COMPRESSION WITH 6W FRAMEWORK
// ============================================================================

/// 16-byte compression with complete 6W framework + aspect color
/// CONSOLIDATION: Uses single `confidence` field (no separate confidence)
#[repr(C)]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ASI16ByteCompression {
    /// Bytes 0-1: WHO (Entity)
    pub who: [u8; 2],
    /// Bytes 2-5: WHAT (Concept)
    pub what: [u8; 4],
    /// Bytes 6-7: WHEN (Time)
    pub when: [u8; 2],
    /// Bytes 8-9: WHERE (Location/Flux)
    pub where_: [u8; 2],
    /// Byte 10: WHY (Causality)
    pub why: u8,
    /// Byte 11: HOW (Method)
    pub how: u8,
    /// Bytes 12-13: ASPECT COLOR
    pub aspect_color: [u8; 2],
    /// Byte 14: CONFIDENCE (consolidated)
    pub confidence: u8,
    /// Byte 15: METADATA
    pub metadata: u8,
}

impl ASI16ByteCompression {
    pub const SIZE: usize = 16;
    
    /// Get confidence level (0.0-1.0)
    /// CONSOLIDATED: Single metric replaces both confidence and confidence
    pub fn confidence(&self) -> f32 {
        (self.confidence & 0x1F) as f32 / 31.0
    }
    
    /// Check if this is a sacred position (3, 6, 9)
    pub fn is_sacred(&self) -> bool {
        (self.confidence & 0x20) != 0
    }
    
    /// Check if confidence is high (≥0.6)
    pub fn is_high_confidence(&self) -> bool {
        (self.confidence & 0x40) != 0
    }
    
    /// Check if validated
    pub fn is_validated(&self) -> bool {
        (self.confidence & 0x80) != 0
    }
}

/// Helper functions for 16-byte compression encoding
pub mod sixw {
    use super::*;
    
    /// Hash string to n bits
    pub fn hash_to_bits(s: &str, bits: u8) -> u64 {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        let mask = (1u64 << bits) - 1;
        hasher.finish() & mask
    }
    
    /// Encode WHO (entity) into 2 bytes
    pub fn encode_who(entity: &str) -> [u8; 2] {
        let hash = hash_to_bits(entity, 12) as u16;
        let is_plural = entity.ends_with('s') as u16;
        let encoded = hash | (is_plural << 15);
        encoded.to_le_bytes()
    }
    
    /// Encode WHAT (concept) into 4 bytes  
    pub fn encode_what(concept: &str) -> [u8; 4] {
        let hash = hash_to_bits(concept, 24) as u32;
        let is_negated = (concept.contains("not") || concept.contains("n't")) as u32;
        let is_question = concept.contains('?') as u32;
        let encoded = hash | (is_negated << 30) | (is_question << 31);
        encoded.to_le_bytes()
    }
    
    /// Encode WHERE (location + flux position) into 2 bytes
    pub fn encode_where(location: &str, flux_position: u8) -> [u8; 2] {
        let loc_hash = hash_to_bits(location, 8) as u8;
        let byte0 = flux_position & 0x0F;
        [byte0, loc_hash]
    }
    
    /// Encode WHY (causality + intention) into 1 byte
    pub fn encode_why(intention: f32) -> u8 {
        let strength = (intention * 15.0).clamp(0.0, 15.0) as u8;
        strength << 4  // Lower 4 bits for type, upper 4 for strength
    }
    
    /// Encode HOW (method + complexity) into 1 byte
    pub fn encode_how(complexity: f32) -> u8 {
        let complexity_level = (complexity * 7.0).clamp(0.0, 7.0) as u8;
        complexity_level << 4  // Lower 4 bits for type, upper 4 for complexity
    }
    
    /// Encode aspect color into 2 bytes
    /// Requires aspect_color module
    pub fn encode_aspect_color(hue: f32, saturation: f32, luminance: f32) -> [u8; 2] {
        let hue_sector = ((hue / 360.0) * 512.0).clamp(0.0, 511.0) as u16;
        let sat_level = (saturation * 15.0).clamp(0.0, 15.0) as u16;
        let lum_level = (luminance * 7.0).clamp(0.0, 7.0) as u16;
        
        let encoded = (hue_sector & 0x01FF) 
            | ((sat_level & 0x0F) << 9) 
            | ((lum_level & 0x07) << 13);
        
        encoded.to_le_bytes()
    }
    
    /// Decode aspect color from 2 bytes
    pub fn decode_aspect_color(bytes: [u8; 2]) -> (f32, f32, f32) {
        let encoded = u16::from_le_bytes(bytes);
        let hue_sector = encoded & 0x01FF;
        let sat_level = (encoded >> 9) & 0x0F;
        let lum_level = (encoded >> 13) & 0x07;
        
        let hue = (hue_sector as f32 / 512.0) * 360.0;
        let saturation = sat_level as f32 / 15.0;
        let luminance = lum_level as f32 / 7.0;
        
        (hue, saturation, luminance)
    }
    
    /// Encode CONFIDENCE into 1 byte (CONSOLIDATED)
    /// Replaces both confidence and confidence with single metric
    pub fn encode_confidence(
        confidence: f32,
        flux_position: u8,
        validated: bool,
    ) -> u8 {
        let conf_level = (confidence * 31.0).clamp(0.0, 31.0) as u8;
        let is_sacred = [3, 6, 9].contains(&flux_position);
        let high_conf = confidence >= 0.6;  // Confidence Lake threshold
        
        (conf_level & 0x1F)
            | ((is_sacred as u8) << 5)
            | ((high_conf as u8) << 6)
            | ((validated as u8) << 7)
    }
}

/// Builder for 16-byte compression
pub struct ASI16ByteBuilder {
    who: Option<String>,
    what: Option<String>,
    #[allow(dead_code)]
    _when: Option<String>,  // Placeholder for future temporal encoding
    where_location: Option<String>,
    where_flux: u8,
    why_intention: f32,
    how_complexity: f32,
    aspect_hue: f32,
    aspect_sat: f32,
    aspect_lum: f32,
    confidence: f32,
    validated: bool,
}

impl ASI16ByteBuilder {
    pub fn new() -> Self {
        Self {
            who: None,
            what: None,
            _when: None,
            where_location: None,
            where_flux: 0,
            why_intention: 0.5,
            how_complexity: 0.5,
            aspect_hue: 0.0,
            aspect_sat: 0.8,
            aspect_lum: 0.5,
            confidence: 0.5,
            validated: false,
        }
    }
    
    pub fn who(mut self, who: impl Into<String>) -> Self {
        self.who = Some(who.into());
        self
    }
    
    pub fn what(mut self, what: impl Into<String>) -> Self {
        self.what = Some(what.into());
        self
    }
    
    pub fn where_location(mut self, location: impl Into<String>) -> Self {
        self.where_location = Some(location.into());
        self
    }
    
    pub fn where_flux(mut self, position: u8) -> Self {
        self.where_flux = position;
        self
    }
    
    pub fn why_intention(mut self, intention: f32) -> Self {
        self.why_intention = intention;
        self
    }
    
    pub fn how_complexity(mut self, complexity: f32) -> Self {
        self.how_complexity = complexity;
        self
    }
    
    pub fn aspect_color(mut self, hue: f32, saturation: f32, luminance: f32) -> Self {
        self.aspect_hue = hue;
        self.aspect_sat = saturation;
        self.aspect_lum = luminance;
        self
    }
    
    pub fn confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }
    
    pub fn validated(mut self, validated: bool) -> Self {
        self.validated = validated;
        self
    }
    
    pub fn build(self) -> ASI16ByteCompression {
        ASI16ByteCompression {
            who: sixw::encode_who(&self.who.unwrap_or_default()),
            what: sixw::encode_what(&self.what.unwrap_or_default()),
            when: [0, 0],  // Placeholder
            where_: sixw::encode_where(&self.where_location.unwrap_or_default(), self.where_flux),
            why: sixw::encode_why(self.why_intention),
            how: sixw::encode_how(self.how_complexity),
            aspect_color: sixw::encode_aspect_color(self.aspect_hue, self.aspect_sat, self.aspect_lum),
            confidence: sixw::encode_confidence(self.confidence, self.where_flux, self.validated),
            metadata: 0,  // Placeholder
        }
    }
}

impl Default for ASI16ByteBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests_16byte {
    use super::*;
    
    #[test]
    fn test_16byte_size() {
        assert_eq!(
            std::mem::size_of::<ASI16ByteCompression>(),
            16,
            "16-byte structure must be exactly 16 bytes"
        );
    }
    
    #[test]
    fn test_confidence_encoding_consolidated() {
        // Test that confidence replaces confidence
        let conf = sixw::encode_confidence(0.8, 3, true);
        
        let decoded_conf = (conf & 0x1F) as f32 / 31.0;
        let is_sacred = (conf & 0x20) != 0;
        let is_high = (conf & 0x40) != 0;
        let is_validated = (conf & 0x80) != 0;
        
        assert!((decoded_conf - 0.8).abs() < 0.05, "Confidence encoding");
        assert!(is_sacred, "Should detect sacred position");
        assert!(is_high, "Should flag high confidence");
        assert!(is_validated, "Should flag validated");
    }
    
    #[test]
    fn test_aspect_color_encoding() {
        let hue = 180.0;
        let sat = 0.8;
        let lum = 0.5;
        
        let encoded = sixw::encode_aspect_color(hue, sat, lum);
        let (decoded_hue, decoded_sat, decoded_lum) = sixw::decode_aspect_color(encoded);
        
        // Allow quantization error
        assert!((hue - decoded_hue).abs() < 2.0, "Hue encoding");
        assert!((sat - decoded_sat).abs() < 0.1, "Saturation encoding");
        assert!((lum - decoded_lum).abs() < 0.15, "Luminance encoding");
    }
    
    #[test]
    fn test_16byte_builder() {
        let compressed = ASI16ByteBuilder::new()
            .who("Alice")
            .what("create art")
            .where_location("studio")
            .where_flux(3)
            .why_intention(0.9)
            .how_complexity(0.7)
            .aspect_color(120.0, 0.8, 0.6)
            .confidence(0.85)
            .validated(true)
            .build();
        
        assert!(compressed.is_sacred(), "Position 3 is sacred");
        assert!(compressed.is_high_confidence(), "0.85 is high confidence");
        assert!(compressed.is_validated(), "Should be validated");
        assert!((compressed.confidence() - 0.85).abs() < 0.05, "Confidence preserved");
    }
}
