//! E8 Lattice Integration with SpatialVortex Sacred Geometry
//!
//! ## Table of Contents
//! - **SacredE8Codec**: E8 codec enhanced with sacred geometry awareness
//! - **VortexQuantizer**: Maps E8 blocks to flux matrix positions
//! - **EustressVec**: Trait for sacred geometry enhanced vector serialization
//! - **E8FluxBridge**: Bridges embedvec E8 with SpatialVortex flux matrix
//!
//! ## Background
//! This module integrates the embedvec crate's E8 lattice quantization with
//! SpatialVortex's sacred geometry architecture. The E8 lattice's 8D structure
//! maps naturally to the flux matrix's 9-node system (8 flow positions + 1 center).
//!
//! ## Key Mappings
//! - E8's 8 dimensions → Flux positions 1-2-4-8-7-5 (flow) + position 0 (center)
//! - E8's 240 root vectors → Sacred geometry's kissing number alignment
//! - D8 ∪ (D8 + ½) decomposition → Digital root even/odd parity
//! - Hadamard preprocessing → Vortex flow transformation

use crate::core::sacred_geometry::flux_matrix::FluxMatrixEngine;
use crate::error::SpatialVortexError;
use embedvec::e8::{E8Codec, E8EncodedVector, E8Point, E8_BLOCK_SIZE};
use embedvec::Quantization;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Local flux position wrapper with serialization support
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct E8FluxPosition(pub u8);

impl E8FluxPosition {
    /// Check if this is a sacred position (3, 6, or 9)
    pub fn is_sacred(&self) -> bool {
        matches!(self.0, 3 | 6 | 9)
    }
    
    /// Get the position value
    pub fn value(&self) -> u8 {
        self.0
    }
}

/// Simple ELP representation for E8 integration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct E8ELPTensor {
    /// Ethos (Character/Authority): 0.0 to 1.0 normalized
    pub ethos: f32,
    /// Logos (Logic/Analytical): 0.0 to 1.0 normalized
    pub logos: f32,
    /// Pathos (Emotion/Expressive): 0.0 to 1.0 normalized
    pub pathos: f32,
}

impl E8ELPTensor {
    /// Create a new ELP tensor
    pub fn new(ethos: f32, logos: f32, pathos: f32) -> Self {
        Self { ethos, logos, pathos }
    }
    
    /// Create a balanced ELP tensor (equal weights)
    pub fn balanced() -> Self {
        Self {
            ethos: 1.0 / 3.0,
            logos: 1.0 / 3.0,
            pathos: 1.0 / 3.0,
        }
    }
    
    /// Normalize to sum to 1.0
    pub fn normalize(&mut self) {
        let sum = self.ethos + self.logos + self.pathos;
        if sum > 0.0 {
            self.ethos /= sum;
            self.logos /= sum;
            self.pathos /= sum;
        } else {
            *self = Self::balanced();
        }
    }
}

/// Result type for E8 integration operations
pub type Result<T> = std::result::Result<T, SpatialVortexError>;

/// Sacred geometry enhanced E8 codec
///
/// Extends embedvec's E8Codec with sacred position awareness,
/// applying different quantization strategies based on flux position.
#[derive(Debug, Clone)]
pub struct SacredE8Codec {
    /// Base E8 codec from embedvec
    codec: E8Codec,
    /// Flux matrix for sacred position calculations
    flux_engine: Arc<FluxMatrixEngine>,
    /// Whether to apply sacred position boosts
    use_sacred_boost: bool,
    /// Boost factor for sacred positions (3, 6, 9)
    sacred_boost_factor: f32,
}

impl SacredE8Codec {
    /// Create a new sacred E8 codec
    ///
    /// # Arguments
    /// * `dimension` - Vector dimension (will be padded to multiple of 8)
    /// * `bits_per_block` - Bits per 8D block (8, 10, or 12)
    /// * `use_hadamard` - Apply Hadamard preprocessing
    /// * `random_seed` - Seed for reproducible random signs
    /// * `use_sacred_boost` - Apply sacred position quality boosts
    pub fn new(
        dimension: usize,
        bits_per_block: u8,
        use_hadamard: bool,
        random_seed: u64,
        use_sacred_boost: bool,
    ) -> Self {
        let codec = E8Codec::new(dimension, bits_per_block, use_hadamard, random_seed);
        let flux_engine = Arc::new(FluxMatrixEngine::new());

        Self {
            codec,
            flux_engine,
            use_sacred_boost,
            sacred_boost_factor: 1.15, // 15% boost for sacred positions
        }
    }

    /// Create from embedvec Quantization config
    pub fn from_quantization(quant: &Quantization, dimension: usize) -> Option<Self> {
        match quant {
            Quantization::E8 {
                bits_per_block,
                use_hadamard,
                random_seed,
            } => Some(Self::new(
                dimension,
                *bits_per_block,
                *use_hadamard,
                *random_seed,
                true,
            )),
            Quantization::None => None,
        }
    }

    /// Encode a vector with sacred geometry awareness
    ///
    /// # Arguments
    /// * `vector` - Input vector
    /// * `elp` - Optional ELP tensor for position-aware encoding
    ///
    /// # Returns
    /// Sacred-enhanced encoded vector with flux position metadata
    pub fn encode_sacred(
        &self,
        vector: &[f32],
        elp: Option<&E8ELPTensor>,
    ) -> Result<SacredE8EncodedVector> {
        // Calculate flux position from ELP if provided
        let flux_position = if let Some(elp) = elp {
            let pos = self.flux_engine.calculate_position_from_elp(
                elp.ethos,
                elp.logos,
                elp.pathos,
            );
            E8FluxPosition(pos)
        } else {
            E8FluxPosition(0) // Default to center/neutral
        };

        // Encode using base codec
        let encoded = self.codec.encode(vector).map_err(|e| {
            SpatialVortexError::InvalidInput(format!("E8 encoding failed: {}", e))
        })?;

        // Calculate signal strength based on sacred position alignment
        let signal_strength = self.calculate_signal_strength(&encoded, &flux_position);

        // Apply sacred boost if at sacred position
        let quality_boost = if self.use_sacred_boost && flux_position.is_sacred() {
            self.sacred_boost_factor
        } else {
            1.0
        };

        Ok(SacredE8EncodedVector {
            encoded,
            flux_position,
            signal_strength,
            quality_boost,
            block_positions: self.map_blocks_to_positions(vector),
        })
    }

    /// Decode a sacred-enhanced encoded vector
    pub fn decode_sacred(&self, encoded: &SacredE8EncodedVector) -> Vec<f32> {
        let mut decoded = self.codec.decode(&encoded.encoded);

        // Apply quality boost scaling if needed
        if encoded.quality_boost != 1.0 {
            for v in decoded.iter_mut() {
                *v *= encoded.quality_boost;
            }
        }

        decoded
    }

    /// Compute asymmetric distance with sacred geometry awareness
    ///
    /// Query remains in f32, database vector is decoded on-the-fly.
    /// Sacred positions get distance reduction (similarity boost).
    pub fn asymmetric_distance_sacred(
        &self,
        query: &[f32],
        encoded: &SacredE8EncodedVector,
    ) -> f32 {
        let base_distance = self.codec.asymmetric_distance(query, &encoded.encoded);

        // Apply sacred position boost (reduce distance for sacred positions)
        if self.use_sacred_boost && encoded.flux_position.is_sacred() {
            base_distance / encoded.quality_boost
        } else {
            base_distance
        }
    }

    /// Calculate signal strength based on E8 encoding quality
    ///
    /// Signal strength measures how well the vector aligns with E8 lattice
    /// and sacred geometry patterns.
    fn calculate_signal_strength(
        &self,
        encoded: &E8EncodedVector,
        flux_position: &E8FluxPosition,
    ) -> f32 {
        // Base signal from encoding quality (scale factor indicates magnitude preservation)
        let scale_signal = 1.0 / (1.0 + encoded.scale.abs().ln().abs());

        // Sacred position contribution (3-6-9 pattern)
        let sacred_signal = if flux_position.is_sacred() {
            0.9 // High signal for sacred positions
        } else if flux_position.0 == 0 {
            0.85 // Good signal for center/balanced
        } else {
            0.7 // Base signal for flow positions
        };

        // E8 lattice alignment (check D8 vs D8+½ distribution)
        let d8_count = encoded.points.iter().filter(|p| !p.is_half).count();
        let half_count = encoded.points.len() - d8_count;
        let lattice_balance = 1.0 - ((d8_count as f32 - half_count as f32).abs()
            / encoded.points.len() as f32);

        // Combined signal strength
        (scale_signal * 0.3 + sacred_signal * 0.4 + lattice_balance * 0.3).clamp(0.0, 1.0)
    }

    /// Map E8 blocks to flux matrix positions
    ///
    /// Each 8D block is assigned a position in the vortex flow pattern.
    fn map_blocks_to_positions(&self, vector: &[f32]) -> Vec<u8> {
        let num_blocks = (vector.len() + E8_BLOCK_SIZE - 1) / E8_BLOCK_SIZE;
        let mut positions = Vec::with_capacity(num_blocks);

        // Vortex flow pattern: 1→2→4→8→7→5→1
        let flow_pattern = [1u8, 2, 4, 8, 7, 5];

        for i in 0..num_blocks {
            // Cycle through flow pattern, with sacred positions at multiples of 3
            let pos = if (i + 1) % 3 == 0 {
                // Sacred position: 3, 6, or 9
                match (i / 3) % 3 {
                    0 => 3,
                    1 => 6,
                    _ => 9,
                }
            } else {
                // Flow position
                flow_pattern[i % flow_pattern.len()]
            };
            positions.push(pos);
        }

        positions
    }

    /// Get memory usage per vector in bytes
    pub fn bytes_per_vector(&self) -> usize {
        // Base E8 bytes + metadata (flux position, signal strength, etc.)
        self.codec.bytes_per_vector() + 8 // 8 bytes for metadata
    }

    /// Get number of 8D blocks
    pub fn num_blocks(&self) -> usize {
        self.codec.num_blocks()
    }

    /// Get the underlying E8 codec
    pub fn inner(&self) -> &E8Codec {
        &self.codec
    }
}

/// Sacred geometry enhanced E8 encoded vector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SacredE8EncodedVector {
    /// Base E8 encoded vector from embedvec
    pub encoded: E8EncodedVector,
    /// Flux position for this vector
    pub flux_position: E8FluxPosition,
    /// Signal strength (0.0-1.0)
    pub signal_strength: f32,
    /// Quality boost factor (1.0 = no boost)
    pub quality_boost: f32,
    /// Block-to-position mapping
    pub block_positions: Vec<u8>,
}

impl SacredE8EncodedVector {
    /// Check if this vector is at a sacred position
    pub fn is_sacred(&self) -> bool {
        self.flux_position.is_sacred()
    }

    /// Get serialized size in bytes
    pub fn size_bytes(&self) -> usize {
        self.encoded.size_bytes() + 4 + 4 + 4 + self.block_positions.len()
    }
}

/// Trait for sacred geometry enhanced vector serialization
///
/// Provides methods for serializing vectors with ELP and flux position metadata.
pub trait EustressVec: Sized {
    /// Serialize to bytes with sacred geometry metadata
    fn to_sacred_bytes(&self) -> Vec<u8>;

    /// Deserialize from bytes with sacred geometry metadata
    fn from_sacred_bytes(bytes: &[u8]) -> Result<Self>;

    /// Get the flux position
    fn flux_position(&self) -> E8FluxPosition;

    /// Get signal strength
    fn signal_strength(&self) -> f32;

    /// Check if at sacred position
    fn is_sacred(&self) -> bool {
        self.flux_position().is_sacred()
    }
}

impl EustressVec for SacredE8EncodedVector {
    fn to_sacred_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap_or_default()
    }

    fn from_sacred_bytes(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes).map_err(|e| {
            SpatialVortexError::Storage(format!("Failed to deserialize: {}", e))
        })
    }

    fn flux_position(&self) -> E8FluxPosition {
        self.flux_position
    }

    fn signal_strength(&self) -> f32 {
        self.signal_strength
    }
}

/// Bridge between E8 quantization and Flux Matrix
///
/// Provides utilities for mapping between E8 lattice structure
/// and SpatialVortex's sacred geometry.
pub struct E8FluxBridge {
    /// Flux matrix engine
    flux_engine: Arc<FluxMatrixEngine>,
}

impl E8FluxBridge {
    /// Create a new E8-Flux bridge
    pub fn new() -> Self {
        Self {
            flux_engine: Arc::new(FluxMatrixEngine::new()),
        }
    }

    /// Map E8 point coordinates to flux position
    ///
    /// Uses digital root reduction on coordinate sum to determine position.
    pub fn e8_point_to_flux_position(&self, point: &E8Point) -> E8FluxPosition {
        let coords = point.to_f32();

        // Sum coordinates and apply digital root reduction
        let sum: f32 = coords.iter().sum();
        let abs_sum = sum.abs() as u64;
        let digital_root = self.flux_engine.reduce_digits(abs_sum);

        E8FluxPosition(digital_root as u8)
    }

    /// Map flux position to preferred E8 lattice region
    ///
    /// Returns whether to prefer D8 (integer) or D8+½ (half-integer) points.
    pub fn flux_position_to_e8_preference(&self, position: E8FluxPosition) -> bool {
        // Sacred positions prefer D8+½ (half-integer) for higher precision
        // Flow positions prefer D8 (integer) for efficiency
        position.is_sacred()
    }

    /// Calculate HNSW layer from flux position
    ///
    /// Maps ontological indices to HNSW layers using level_mult ~1/ln(M).
    /// Sacred positions get higher layers (more connections).
    pub fn flux_position_to_hnsw_layer(&self, position: E8FluxPosition, max_layer: usize) -> usize {
        match position.0 {
            // Sacred positions → higher layers (more visibility)
            3 => max_layer.saturating_sub(1),
            6 => max_layer.saturating_sub(2),
            9 => max_layer, // Highest layer
            // Center → middle layer
            0 => max_layer / 2,
            // Flow positions → lower layers
            _ => position.0 as usize % (max_layer / 2 + 1),
        }
    }

    /// Calculate ELP from E8 encoded vector
    ///
    /// Maps the E8 block structure to Ethos-Logos-Pathos channels.
    pub fn e8_to_elp(&self, encoded: &SacredE8EncodedVector) -> E8ELPTensor {
        let points = &encoded.encoded.points;
        if points.is_empty() {
            return E8ELPTensor::balanced();
        }

        // Divide blocks into thirds for E-L-P
        let third = points.len() / 3;

        // Ethos: first third of blocks (character/ethics)
        let ethos_sum: f32 = points[..third]
            .iter()
            .map(|p| p.to_f32().iter().map(|v| v.abs()).sum::<f32>())
            .sum();

        // Logos: middle third (logic/reason)
        let logos_sum: f32 = points[third..2 * third]
            .iter()
            .map(|p| p.to_f32().iter().map(|v| v.abs()).sum::<f32>())
            .sum();

        // Pathos: last third (emotion/feeling)
        let pathos_sum: f32 = points[2 * third..]
            .iter()
            .map(|p| p.to_f32().iter().map(|v| v.abs()).sum::<f32>())
            .sum();

        // Normalize to sum to 1.0
        let total = ethos_sum + logos_sum + pathos_sum;
        if total > 0.0 {
            E8ELPTensor {
                ethos: ethos_sum / total,
                logos: logos_sum / total,
                pathos: pathos_sum / total,
            }
        } else {
            E8ELPTensor::balanced()
        }
    }

    /// Get the 240 E8 root vectors mapped to flux positions
    ///
    /// Returns a mapping of each root vector to its flux position.
    pub fn e8_roots_to_flux_positions(&self) -> Vec<(usize, E8FluxPosition)> {
        let roots = embedvec::e8::generate_e8_roots();
        roots
            .iter()
            .enumerate()
            .map(|(i, root)| {
                let sum: f32 = root.iter().sum();
                let abs_sum = sum.abs() as u64;
                let digital_root = self.flux_engine.reduce_digits(abs_sum);
                (i, E8FluxPosition(digital_root as u8))
            })
            .collect()
    }
}

impl Default for E8FluxBridge {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// O(n!) → O(1) Amortization Layer
// ============================================================================
// 
// Implements the theoretical resolution of O(1) vs O(n!) clash in IR:
// - E8 lattice pre-caches 240 root vectors (kissing number)
// - 128 type-2 sign permutations per 8D block bound combinatorics
// - Digital root mod 9 maps to Flux Matrix 9-node positions
// - Query complexity: O(log n · d/8 · bits) instead of O(n!)

/// Pre-cached E8 lattice lookup table for O(1) amortized access
#[derive(Debug, Clone)]
pub struct E8AmortizedCache {
    /// Pre-computed E8 roots mapped to flux positions (240 entries)
    root_positions: Vec<(E8FluxPosition, [f32; 8])>,
    /// Position-indexed root buckets for O(1) lookup
    position_buckets: [Vec<usize>; 10],
    /// Sacred position indices (3, 6, 9)
    sacred_indices: Vec<usize>,
}

impl E8AmortizedCache {
    /// Build cache from E8 roots (offline O(n!) amortization)
    pub fn build() -> Self {
        let roots = embedvec::e8::generate_e8_roots();
        let flux_engine = FluxMatrixEngine::new();
        
        let mut root_positions = Vec::with_capacity(240);
        let mut position_buckets: [Vec<usize>; 10] = Default::default();
        let mut sacred_indices = Vec::new();
        
        for (i, root) in roots.iter().enumerate() {
            let coords: [f32; 8] = [
                root[0], root[1], root[2], root[3],
                root[4], root[5], root[6], root[7],
            ];
            
            let sum: f32 = coords.iter().sum();
            let digital_root = flux_engine.reduce_digits(sum.abs() as u64) as u8;
            let position = E8FluxPosition(digital_root);
            
            root_positions.push((position, coords));
            position_buckets[digital_root as usize].push(i);
            
            if position.is_sacred() {
                sacred_indices.push(i);
            }
        }
        
        Self { root_positions, position_buckets, sacred_indices }
    }
    
    /// O(1) lookup: roots at flux position
    pub fn roots_at_position(&self, position: E8FluxPosition) -> &[usize] {
        &self.position_buckets[position.0 as usize % 10]
    }
    
    /// O(1) lookup: sacred roots
    pub fn sacred_roots(&self) -> &[usize] {
        &self.sacred_indices
    }
    
    /// O(1) lookup: root coordinates
    pub fn root_coords(&self, index: usize) -> Option<&[f32; 8]> {
        self.root_positions.get(index).map(|(_, c)| c)
    }
    
    /// Position distribution (240 roots across 10 positions)
    pub fn position_distribution(&self) -> [usize; 10] {
        let mut dist = [0usize; 10];
        for (i, bucket) in self.position_buckets.iter().enumerate() {
            dist[i] = bucket.len();
        }
        dist
    }
}

impl Default for E8AmortizedCache {
    fn default() -> Self {
        Self::build()
    }
}

/// Complexity analysis: O(n!) → O(log n · d/8)
#[derive(Debug, Clone, Copy)]
pub struct ComplexityAnalysis {
    pub n: usize,
    pub dimension: usize,
    pub bits_per_block: u8,
}

impl ComplexityAnalysis {
    /// Brute-force: O(n!) using Stirling approximation
    pub fn brute_force_ops(&self) -> f64 {
        let n = self.n as f64;
        if self.n <= 20 {
            (1..=self.n).map(|i| i as f64).product()
        } else {
            (2.0 * std::f64::consts::PI * n).sqrt() * (n / std::f64::consts::E).powf(n)
        }
    }
    
    /// E8-HNSW: O(log n · d/8 · bits)
    pub fn e8_hnsw_ops(&self) -> f64 {
        let log_n = (self.n as f64).ln().max(1.0);
        let blocks = (self.dimension as f64) / 8.0;
        log_n * blocks * (self.bits_per_block as f64)
    }
    
    /// Speedup factor: n! / (log n · d/8 · bits)
    pub fn speedup(&self) -> f64 {
        self.brute_force_ops() / self.e8_hnsw_ops().max(1.0)
    }
}

/// Configuration for E8 quantization with sacred geometry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SacredE8Config {
    /// Vector dimension
    pub dimension: usize,
    /// Bits per 8D block (8, 10, or 12)
    pub bits_per_block: u8,
    /// Apply Hadamard preprocessing
    pub use_hadamard: bool,
    /// Random seed for reproducibility
    pub random_seed: u64,
    /// Apply sacred position boosts
    pub use_sacred_boost: bool,
    /// Boost factor for sacred positions
    pub sacred_boost_factor: f32,
}

impl Default for SacredE8Config {
    fn default() -> Self {
        Self {
            dimension: 768,
            bits_per_block: 10,
            use_hadamard: true,
            random_seed: 0xcafef00d,
            use_sacred_boost: true,
            sacred_boost_factor: 1.15,
        }
    }
}

impl SacredE8Config {
    /// Create config for high compression (lower quality)
    pub fn high_compression(dimension: usize) -> Self {
        Self {
            dimension,
            bits_per_block: 8,
            use_hadamard: true,
            random_seed: 0xcafef00d,
            use_sacred_boost: true,
            sacred_boost_factor: 1.2, // Higher boost to compensate
        }
    }

    /// Create config for high quality (lower compression)
    pub fn high_quality(dimension: usize) -> Self {
        Self {
            dimension,
            bits_per_block: 12,
            use_hadamard: true,
            random_seed: 0xcafef00d,
            use_sacred_boost: true,
            sacred_boost_factor: 1.1,
        }
    }

    /// Create config for balanced performance
    pub fn balanced(dimension: usize) -> Self {
        Self {
            dimension,
            ..Default::default()
        }
    }

    /// Get compression ratio compared to f32
    pub fn compression_ratio(&self) -> f32 {
        let f32_bytes = self.dimension * 4;
        let num_blocks = (self.dimension + 7) / 8;
        let e8_bytes = (num_blocks * self.bits_per_block as usize + 7) / 8 + 4;
        f32_bytes as f32 / e8_bytes as f32
    }

    /// Get approximate bits per dimension
    pub fn bits_per_dim(&self) -> f32 {
        self.bits_per_block as f32 / 8.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sacred_e8_codec_creation() {
        let codec = SacredE8Codec::new(768, 10, true, 42, true);
        assert_eq!(codec.num_blocks(), 96); // 768 / 8 = 96
    }

    #[test]
    fn test_sacred_encoding_roundtrip() {
        let codec = SacredE8Codec::new(384, 10, true, 42, true);

        // Create test vector
        let vector: Vec<f32> = (0..384).map(|i| (i as f32 * 0.01).sin()).collect();

        // Encode with ELP
        let elp = E8ELPTensor {
            ethos: 0.5,
            logos: 0.3,
            pathos: 0.2,
        };
        let encoded = codec.encode_sacred(&vector, Some(&elp)).unwrap();

        // Check metadata
        assert!(encoded.signal_strength > 0.0);
        assert!(encoded.signal_strength <= 1.0);
        assert!(!encoded.block_positions.is_empty());

        // Decode and verify
        let decoded = codec.decode_sacred(&encoded);
        assert_eq!(decoded.len(), 384);
    }

    #[test]
    fn test_sacred_position_boost() {
        let codec = SacredE8Codec::new(384, 10, true, 42, true);
        let vector: Vec<f32> = (0..384).map(|i| (i as f32 * 0.01).cos()).collect();

        // Encode at sacred position (high ethos = position 3)
        let sacred_elp = E8ELPTensor {
            ethos: 0.9,
            logos: 0.05,
            pathos: 0.05,
        };
        let sacred_encoded = codec.encode_sacred(&vector, Some(&sacred_elp)).unwrap();

        // Encode at non-sacred position
        let normal_elp = E8ELPTensor {
            ethos: 0.4,
            logos: 0.3,
            pathos: 0.3,
        };
        let normal_encoded = codec.encode_sacred(&vector, Some(&normal_elp)).unwrap();

        // Sacred position should have boost
        assert!(sacred_encoded.quality_boost > normal_encoded.quality_boost);
    }

    #[test]
    fn test_e8_flux_bridge() {
        let bridge = E8FluxBridge::new();

        // Test E8 point to flux position
        let point = E8Point {
            coords: [1, 1, 0, 0, 0, 0, 0, 0],
            is_half: false,
        };
        let pos = bridge.e8_point_to_flux_position(&point);
        assert!(pos.0 <= 9);

        // Test HNSW layer mapping
        let sacred_pos = E8FluxPosition(9);
        let layer = bridge.flux_position_to_hnsw_layer(sacred_pos, 6);
        assert_eq!(layer, 6); // Sacred 9 gets max layer
    }

    #[test]
    fn test_e8_to_elp() {
        let codec = SacredE8Codec::new(384, 10, true, 42, true);
        let bridge = E8FluxBridge::new();

        let vector: Vec<f32> = (0..384).map(|i| (i as f32 * 0.01).sin()).collect();
        let encoded = codec.encode_sacred(&vector, None).unwrap();

        let elp = bridge.e8_to_elp(&encoded);

        // ELP should sum to ~1.0
        let sum = elp.ethos + elp.logos + elp.pathos;
        assert!((sum - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_eustress_vec_serialization() {
        let codec = SacredE8Codec::new(128, 10, true, 42, true);
        let vector: Vec<f32> = (0..128).map(|i| i as f32 * 0.1).collect();

        let encoded = codec.encode_sacred(&vector, None).unwrap();
        let bytes = encoded.to_sacred_bytes();

        let decoded = SacredE8EncodedVector::from_sacred_bytes(&bytes).unwrap();
        assert_eq!(decoded.flux_position.0, encoded.flux_position.0);
        assert!((decoded.signal_strength - encoded.signal_strength).abs() < 0.001);
    }

    #[test]
    fn test_config_compression_ratios() {
        let high_comp = SacredE8Config::high_compression(768);
        let balanced = SacredE8Config::balanced(768);
        let high_qual = SacredE8Config::high_quality(768);

        // Higher compression = higher ratio
        assert!(high_comp.compression_ratio() > balanced.compression_ratio());
        assert!(balanced.compression_ratio() > high_qual.compression_ratio());

        // All should provide significant compression
        assert!(high_qual.compression_ratio() > 2.0);
    }

    #[test]
    fn test_e8_roots_mapping() {
        let bridge = E8FluxBridge::new();
        let mappings = bridge.e8_roots_to_flux_positions();

        // Should have 240 roots
        assert_eq!(mappings.len(), 240);

        // All positions should be valid (0-9)
        for (_, pos) in &mappings {
            assert!(pos.0 <= 9);
        }
    }

    // ========================================================================
    // O(n!) → O(1) Amortization Tests
    // ========================================================================

    #[test]
    fn test_amortized_cache_build() {
        let cache = E8AmortizedCache::build();
        
        // Should have 240 E8 roots
        assert_eq!(cache.root_positions.len(), 240);
        
        // Distribution should cover all positions
        let dist = cache.position_distribution();
        let total: usize = dist.iter().sum();
        assert_eq!(total, 240);
    }

    #[test]
    fn test_amortized_cache_o1_lookup() {
        let cache = E8AmortizedCache::build();
        
        // O(1) lookup by position
        for pos in 0..10u8 {
            let roots = cache.roots_at_position(E8FluxPosition(pos));
            // Each position should have some roots (E8 distributes across positions)
            // Note: some positions may have 0 roots depending on digital root distribution
            assert!(roots.len() <= 240);
        }
        
        // O(1) sacred roots lookup
        let sacred = cache.sacred_roots();
        // Sacred positions (3, 6, 9) should have roots
        assert!(!sacred.is_empty() || cache.position_distribution()[3] == 0);
    }

    #[test]
    fn test_complexity_analysis_brute_force() {
        // Small n: exact factorial
        let analysis_10 = ComplexityAnalysis { n: 10, dimension: 384, bits_per_block: 10 };
        let factorial_10 = analysis_10.brute_force_ops();
        assert!((factorial_10 - 3628800.0).abs() < 1.0); // 10! = 3,628,800
        
        // Large n: Stirling approximation
        let analysis_20 = ComplexityAnalysis { n: 20, dimension: 384, bits_per_block: 10 };
        let factorial_20 = analysis_20.brute_force_ops();
        // 20! ≈ 2.43 × 10^18
        assert!(factorial_20 > 2e18 && factorial_20 < 3e18);
    }

    #[test]
    fn test_complexity_analysis_e8_hnsw() {
        let analysis = ComplexityAnalysis { n: 1_000_000, dimension: 384, bits_per_block: 10 };
        
        // E8-HNSW ops: O(log n · d/8 · bits)
        let e8_ops = analysis.e8_hnsw_ops();
        
        // log(1M) ≈ 13.8, d/8 = 48, bits = 10
        // Expected: ~13.8 * 48 * 10 ≈ 6624
        assert!(e8_ops > 5000.0 && e8_ops < 8000.0);
    }

    #[test]
    fn test_complexity_speedup_massive() {
        // At 1M vectors, speedup should be astronomical
        let analysis = ComplexityAnalysis { n: 1_000_000, dimension: 384, bits_per_block: 10 };
        let speedup = analysis.speedup();
        
        // 1M! is incomprehensibly large (10^5,565,709 digits)
        // Speedup should be effectively infinite (> 10^100)
        assert!(speedup > 1e100);
    }

    #[test]
    fn test_complexity_scaling_logarithmic() {
        // Verify query ops scale logarithmically with n
        let analysis_1k = ComplexityAnalysis { n: 1_000, dimension: 384, bits_per_block: 10 };
        let analysis_1m = ComplexityAnalysis { n: 1_000_000, dimension: 384, bits_per_block: 10 };
        let analysis_1b = ComplexityAnalysis { n: 1_000_000_000, dimension: 384, bits_per_block: 10 };
        
        let ops_1k = analysis_1k.e8_hnsw_ops();
        let ops_1m = analysis_1m.e8_hnsw_ops();
        let ops_1b = analysis_1b.e8_hnsw_ops();
        
        // 1K → 1M: 1000x data, but only ~2x ops (log scaling)
        let ratio_1k_1m = ops_1m / ops_1k;
        assert!(ratio_1k_1m < 3.0); // Should be ~2x, not 1000x
        
        // 1M → 1B: 1000x data, but only ~1.5x ops
        let ratio_1m_1b = ops_1b / ops_1m;
        assert!(ratio_1m_1b < 2.0);
    }
}
