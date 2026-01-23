/// Beam Tensor: Light-based word representation for AGI consciousness
/// Words become beams of colored light flowing through the flux pattern

use crate::models::BeamTensor;
use crate::flux_matrix::FluxMatrixEngine;
use crate::error::Result;
use crate::metrics::VCP_OVERFLOW_RISK_TOTAL;
use crate::data::attributes::AttributeValue;

/// Alpha factors determining beam curvature and behavior
#[derive(Debug, Clone)]
pub struct AlphaFactors {
    pub semantic_mass: f32,      // Weight of meaning (affects gravity)
    pub temporal_decay: f32,     // Relevance fade rate
    pub intersection_pull: f32,  // Attraction strength to 3-6-9
    pub entropy_gradient: f32,   // Rate of entropy change
    pub confidence_momentum: f32,// How confidence affects velocity
    pub subspace_pull: f32,      // Attraction to signal subspaces (TSFM hallucination mitigation)
}

impl Default for AlphaFactors {
    fn default() -> Self {
        Self {
            semantic_mass: 1.0,
            temporal_decay: 0.95,
            intersection_pull: 2.5,  // Strong pull to sacred positions
            entropy_gradient: 0.1,
            confidence_momentum: 1.5,
            subspace_pull: 1.5,      // Moderate pull to signal subspaces
        }
    }
}

impl AlphaFactors {
    /// Calculate gravitational pull based on semantic mass
    pub fn calculate_gravity(&self, distance: f32) -> f32 {
        // Inverse square law with semantic mass
        self.semantic_mass / (distance * distance + 1.0)
    }
    
    /// Apply temporal decay to confidence over time
    pub fn apply_temporal_decay(&self, confidence: f32, time_delta: f32) -> f32 {
        confidence * self.temporal_decay.powf(time_delta)
    }
    
    /// Calculate pull force toward sacred positions (3, 6, 9)
    pub fn calculate_sacred_pull(&self, beam_position: u8, sacred_position: u8) -> f32 {
        let distance = (beam_position as f32 - sacred_position as f32).abs();
        self.intersection_pull / (distance + 1.0)
    }
    
    /// Calculate entropy change rate
    pub fn calculate_entropy_change(&self, current_entropy: f32, target_entropy: f32) -> f32 {
        (target_entropy - current_entropy) * self.entropy_gradient
    }
    
    /// Calculate velocity from confidence using momentum factor
    pub fn calculate_velocity(&self, confidence: f32) -> f32 {
        confidence * self.confidence_momentum
    }
}

/// Beam visualization properties
#[derive(Debug, Clone)]
pub struct BeamProperties {
    pub width: f32,          // Beam thickness (confidence)
    pub length: f32,         // Beam extension (decisiveness)
    pub wobble: f32,         // Attribute-induced oscillation
    pub orbit_radius: f32,   // Attribute-based structure
    pub rotation_speed: f32, // Attribute consistency
    pub color: [f32; 3],     // RGB from attribute channels
}

/// Ladder Index for similarity/antonym detection
#[derive(Debug, Clone)]
pub struct LadderIndex {
    rungs: Vec<SemanticRung>,
    /// Threshold for considering words similar (0.0-1.0)
    similarity_threshold: f32,
}

#[derive(Debug, Clone)]
pub struct SemanticRung {
    pub positive_words: Vec<String>,  // Synonyms (+index)
    pub negative_words: Vec<String>,  // Antonyms (-index)
    pub neutral_center: String,       // Base concept (0 index)
    pub confidence: f32,
}

#[derive(Debug)]
pub enum SimilarityResult {
    Similar(f32),    // Same rung, confidence score
    Antonym(f32),    // Opposite rung
    Different(f32),  // Distance between rungs
}

impl LadderIndex {
    pub fn new() -> Self {
        Self {
            rungs: Vec::new(),
            similarity_threshold: 0.7,
        }
    }
    
    /// Test if two words are similar, antonyms, or different
    pub fn test_similarity(&self, word1: &str, word2: &str) -> SimilarityResult {
        // First check if they're antonyms (regardless of rung)
        if self.are_antonyms(word1, word2, 0, 0) {
            return SimilarityResult::Antonym(0.8);
        }
        
        // Find rungs for each word
        let rung1_idx = self.find_rung_index(word1);
        let rung2_idx = self.find_rung_index(word2);
        
        match (rung1_idx, rung2_idx) {
            (Some(idx1), Some(idx2)) if idx1 == idx2 => {
                // Same rung and not antonyms = similar
                let rung = &self.rungs[idx1];
                // USE similarity_threshold to determine if confidence is high enough
                if rung.confidence >= self.similarity_threshold {
                    SimilarityResult::Similar(rung.confidence)
                } else {
                    SimilarityResult::Different(1.0 - rung.confidence)
                }
            }
            (Some(idx1), Some(idx2)) => {
                // Different rungs = calculate distance
                let distance = ((idx1 as f32) - (idx2 as f32)).abs() / self.rungs.len() as f32;
                SimilarityResult::Different(distance)
            }
            _ => SimilarityResult::Different(1.0), // Not found
        }
    }
    
    fn find_rung_index(&self, word: &str) -> Option<usize> {
        self.rungs.iter().position(|rung| {
            rung.neutral_center == word ||
            rung.positive_words.contains(&word.to_string()) ||
            rung.negative_words.contains(&word.to_string())
        })
    }
    
    fn are_antonyms(&self, word1: &str, word2: &str, _idx1: usize, _idx2: usize) -> bool {
        // Check if words are on opposite sides of ANY rung
        for rung in &self.rungs {
            let word1_positive = rung.positive_words.contains(&word1.to_string());
            let word1_negative = rung.negative_words.contains(&word1.to_string());
            let word2_positive = rung.positive_words.contains(&word2.to_string());
            let word2_negative = rung.negative_words.contains(&word2.to_string());
            
            if (word1_positive && word2_negative) || (word1_negative && word2_positive) {
                return true;
            }
        }
        false
    }
}

/// Beam Tensor Engine for processing words through flux pattern
pub struct BeamTensorEngine {
    flux_engine: FluxMatrixEngine,
    /// Alpha factors for ELP channel weighting
    alpha_factors: AlphaFactors,
    /// Ladder index for semantic similarity
    ladder_index: LadderIndex,
}

impl BeamTensorEngine {
    pub fn new() -> Self {
        Self {
            flux_engine: FluxMatrixEngine::new(),
            alpha_factors: AlphaFactors::default(),
            ladder_index: LadderIndex::new(),
        }
    }
    
    /// Initialize a word into the entropy loop
    pub fn initialize_word(&mut self, word: &str, context: &str) -> Result<BeamTensor> {
        // Create initial beam tensor
        let mut beam = BeamTensor::default();
        beam.word = word.to_string();
        
        // Determine initial position based on inference
        let initial_position = self.infer_initial_position(word, context)?;
        beam.position = initial_position;
        
        // Begin entropy loop to find optimal position
        self.run_entropy_loop(&mut beam)?;
        
        Ok(beam)
    }
    
    /// Infer where a word should start based on context
    fn infer_initial_position(&self, word: &str, _context: &str) -> Result<u8> {
        // TODO: Use actual inference engine with context
        // For now, hash word to position
        let hash = word.bytes().fold(0u64, |acc, b| acc.wrapping_add(b as u64));
        Ok((hash % 10) as u8)
    }
    
    /// Run the entropy loop: y = x² reduction
    fn run_entropy_loop(&mut self, beam: &mut BeamTensor) -> Result<()> {
        let mut entropy = 1.0;
        let mut iterations = 0;
        const MAX_ITERATIONS: u32 = 100;
        const STABILITY_THRESHOLD: f32 = 0.1;
        let start_time = std::time::Instant::now();
        
        while entropy > STABILITY_THRESHOLD && iterations < MAX_ITERATIONS {
            // Calculate y = x²
            let x = beam.position as u64;
            let y = x * x;
            
            // Reduce to single digit
            let next_position = self.flux_engine.reduce_digits(y) as u8;
            
            // Calculate variances from sacred anchors
            let variance_3 = self.calculate_variance_from(beam, 3); // Good/Easy
            let variance_6 = self.calculate_variance_from(beam, 6); // Bad/Hard  
            let variance_9 = self.calculate_variance_from(beam, 9); // Divine/Righteous
            
            // USE alpha_factors to calculate sacred pull forces
            let pull_3 = self.alpha_factors.calculate_sacred_pull(beam.position, 3);
            let pull_6 = self.alpha_factors.calculate_sacred_pull(beam.position, 6);
            let pull_9 = self.alpha_factors.calculate_sacred_pull(beam.position, 9);
            
            // Apply gravitational effects from semantic mass
            let total_pull = pull_3 + pull_6 + pull_9;
            let gravity_factor = self.alpha_factors.calculate_gravity(total_pull);
            
            // Update beam weights with gravity and alpha factors
            self.update_beam_weights(beam, variance_3 * gravity_factor, variance_6 * gravity_factor, variance_9 * gravity_factor);
            
            // Check if we're at a sacred intersection
            let is_sacred_reset = if next_position == 3 || next_position == 6 || next_position == 9 {
                self.process_at_sacred_intersection(beam, next_position)?;
                match beam.overflow_risk {
                    crate::models::OverflowRisk::Warning | crate::models::OverflowRisk::Critical | crate::models::OverflowRisk::Imminent => {
                        beam.calculation_depth = 0;
                        beam.overflow_risk = crate::models::OverflowRisk::Safe;
                        true  // Mark that we performed a reset
                    }
                    _ => false
                }
            } else {
                false
            };
            
            // Move to next position
            beam.position = next_position;
            
            // Calculate entropy
            let current_entropy = self.calculate_entropy(beam);
            
            // USE entropy_gradient to smooth entropy changes
            let entropy_change = self.alpha_factors.calculate_entropy_change(entropy, current_entropy);
            entropy = current_entropy + entropy_change;
            
            // USE temporal_decay to reduce confidence over time
            let time_delta = start_time.elapsed().as_secs_f32();
            beam.confidence = self.alpha_factors.apply_temporal_decay(beam.confidence, time_delta);
            
            // USE confidence_momentum to determine velocity
            let velocity = self.alpha_factors.calculate_velocity(beam.confidence);
            
            // Check for replication conditions (high confidence + low entropy + high velocity)
            if beam.confidence > 0.8 && entropy < 0.3 && velocity > 1.0 {
                // Word is confident, stable, and fast enough to replicate
                beam.can_replicate = true;
            }
            // Increment calculation depth with checked add (skip if we just did a sacred reset)
            if !is_sacred_reset {
                if let Some(new_depth) = beam.calculation_depth.checked_add(1) {
                    beam.calculation_depth = new_depth;
                } else {
                    beam.calculation_depth = u64::MAX;
                }
            }
            // Update overflow risk thresholds and metrics
            let risk_label = if beam.calculation_depth >= 100_000 {
                beam.overflow_risk = crate::models::OverflowRisk::Imminent;
                Some("imminent")
            } else if beam.calculation_depth >= 50_000 {
                beam.overflow_risk = crate::models::OverflowRisk::Critical;
                Some("critical")
            } else if beam.calculation_depth >= 10_000 {
                beam.overflow_risk = crate::models::OverflowRisk::Warning;
                Some("warning")
            } else {
                None
            };
            if let Some(label) = risk_label {
                VCP_OVERFLOW_RISK_TOTAL.with_label_values(&[label]).inc();
            }
            
            iterations += 1;
        }
        
        Ok(())
    }
    
    #[cfg(test)]
    pub(crate) fn run_entropy_loop_for_test(&mut self, beam: &mut BeamTensor) -> Result<()> {
        self.run_entropy_loop(beam)
    }
    
    /// Calculate variance from a sacred anchor position
    fn calculate_variance_from(&self, beam: &BeamTensor, anchor: u8) -> f32 {
        let position_diff = (beam.position as f32 - anchor as f32).abs();
        
        // Get attribute values dynamically
        let ethos = beam.ethos();
        let logos = beam.logos();
        let pathos = beam.pathos();
        
        let semantic_diff = match anchor {
            3 => pathos - logos,      // Good/Easy vs complexity
            6 => logos - ethos,        // Bad/Hard vs stability
            9 => ethos - pathos,       // Divine vs emotion
            _ => 0.0,
        };
        
        position_diff * 0.1 + semantic_diff.abs()
    }
    
    /// Update beam weights based on sacred variances
    fn update_beam_weights(
        &mut self,
        beam: &mut BeamTensor,
        var_3: f32,
        var_6: f32,
        var_9: f32,
    ) {
        // Normalize variances
        let total = var_3 + var_6 + var_9;
        if total < 0.001 {
            return;
        }
        
        let weight_3 = var_3 / total;
        let weight_6 = var_6 / total;
        let weight_9 = var_9 / total;
        
        // Get current attribute values
        let ethos = beam.ethos();
        let logos = beam.logos();
        let pathos = beam.pathos();
        
        // USE alpha_factors to weight the attribute channels appropriately
        // Sacred positions influence channels based on intersection_pull strength
        let new_ethos = ethos * 0.9 + weight_9 * 9.0 * 0.1 * self.alpha_factors.intersection_pull / 2.5;
        let new_logos = logos * 0.9 + weight_6 * 6.0 * 0.1 * self.alpha_factors.confidence_momentum / 1.5;
        let new_pathos = pathos * 0.9 + weight_3 * 3.0 * 0.1 * self.alpha_factors.temporal_decay;
        
        // Set updated values
        beam.set_ethos(new_ethos);
        beam.set_logos(new_logos);
        beam.set_pathos(new_pathos);
        
        // Update confidence based on balance
        let balance = 1.0 - (weight_3 - weight_6).abs() - (weight_6 - weight_9).abs();
        beam.confidence = beam.confidence * 0.95 + balance * 0.05;
    }
    
    /// Process beam at sacred intersection
    fn process_at_sacred_intersection(
        &mut self,
        beam: &mut BeamTensor,
        intersection: u8,
    ) -> Result<()> {
        match intersection {
            3 => {
                // Good/Easy - Accelerate processing
                beam.confidence *= 1.2;
                beam.set_pathos(beam.pathos() * 1.1); // Increase positive emotion
            }
            6 => {
                // Bad/Hard - Deep analysis
                beam.set_logos(beam.logos() * 1.3);  // Increase logical processing
                beam.confidence *= 0.9; // Question assumptions
            }
            9 => {
                // Divine/Righteous - Truth validation
                beam.set_ethos(beam.ethos() * 1.5);  // Strengthen ethical alignment
                if beam.is_diamond_moment() {
                    beam.mark_for_confidence_lake = true;
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Calculate current entropy of beam
    fn calculate_entropy(&self, beam: &BeamTensor) -> f32 {
        // Shannon entropy of digit distribution
        beam.digits.iter()
            .filter(|&&p| p > 1e-6)
            .map(|&p| -p * p.ln())
            .sum::<f32>() / 9.0_f32.ln()
    }
    
    /// Convert beam to visual properties for rendering
    pub fn calculate_beam_properties(&self, beam: &BeamTensor) -> BeamProperties {
        // Alpha factors affect visual properties based on confidence and semantic mass
        let visual_scale = self.alpha_factors.semantic_mass * beam.confidence;
        
        let ethos = beam.ethos();
        let logos = beam.logos();
        let pathos = beam.pathos();
        
        BeamProperties {
            width: beam.confidence * 10.0 * visual_scale,
            length: (1.0 - self.calculate_entropy(beam)) * 50.0 * visual_scale,
            wobble: pathos * 0.5 * self.alpha_factors.temporal_decay,
            orbit_radius: logos * 10.0 * self.alpha_factors.intersection_pull,
            rotation_speed: ethos * 0.1 * self.alpha_factors.confidence_momentum,
            color: [
                pathos / 9.0,  // Red: Emotion
                logos / 9.0,   // Green: Logic
                ethos / 9.0,   // Blue: Ethics
            ],
        }
    }
    
    /// Check semantic similarity between two words using ladder index
    pub fn check_word_similarity(&self, word1: &str, word2: &str) -> SimilarityResult {
        self.ladder_index.test_similarity(word1, word2)
    }
    
    /// Add a word to the ladder index at specific rung
    pub fn add_to_ladder(&mut self, word: &str, rung_center: &str, is_positive: bool) {
        if let Some(rung) = self.ladder_index.rungs.iter_mut()
            .find(|r| r.neutral_center == rung_center) {
            if is_positive {
                rung.positive_words.push(word.to_string());
            } else {
                rung.negative_words.push(word.to_string());
            }
        }
    }
}

/// Extensions to BeamTensor
impl BeamTensor {
    /// Generate color from attribute channels
    pub fn calculate_color(&self) -> [f32; 3] {
        [
            self.pathos() / 9.0,  // Red channel: Emotion
            self.logos() / 9.0,   // Green channel: Logic  
            self.ethos() / 9.0,   // Blue channel: Ethics
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_entropy_loop() {
        let mut engine = BeamTensorEngine::new();
        let beam = engine.initialize_word("test", "context").unwrap();
        
        // Should have found a stable position
        assert!(beam.position <= 9);
        assert!(beam.confidence > 0.0);
    }
    
    #[test]
    fn test_ladder_similarity() {
        let mut ladder = LadderIndex::new();
        ladder.rungs.push(SemanticRung {
            positive_words: vec!["good".to_string(), "great".to_string()],
            negative_words: vec!["bad".to_string(), "terrible".to_string()],
            neutral_center: "quality".to_string(),
            confidence: 0.9,
        });
        
        match ladder.test_similarity("good", "great") {
            SimilarityResult::Similar(conf) => assert!(conf > 0.8),
            _ => panic!("Should be similar"),
        }
        
        match ladder.test_similarity("good", "bad") {
            SimilarityResult::Antonym(_) => {},
            _ => panic!("Should be antonyms"),
        }
    }
    
    #[test]
    fn test_sacred_intersection_processing() {
        let mut engine = BeamTensorEngine::new();
        let mut beam = BeamTensor::default();
        beam.position = 3;
        beam.confidence = 0.9; // Start with some confidence
        
        engine.process_at_sacred_intersection(&mut beam, 3).unwrap();
        assert!(beam.confidence > 1.0); // Should be boosted by 1.2x
        
        beam.position = 9;
        beam.set_ethos(9.0);
        beam.set_logos(8.0);
        beam.curviness_signed = -0.5;
        
        engine.process_at_sacred_intersection(&mut beam, 9).unwrap();
        assert!(beam.mark_for_confidence_lake); // High-confidence Flux Matrix moment
    }

    #[test]
    fn test_overflow_metric_increments_warning() {
        let mut engine = BeamTensorEngine::new();
        // Prepare a beam right below warning threshold so first iteration crosses it
        let mut beam = BeamTensor::default();
        beam.position = 2; // 2^2=4, non-sacred next step
        beam.calculation_depth = 9_999;

        let before = VCP_OVERFLOW_RISK_TOTAL
            .get_metric_with_label_values(&["warning"]).unwrap()
            .get();

        // Run loop (private method accessible within module tests)
        engine.run_entropy_loop_for_test(&mut beam).unwrap();

        let after = VCP_OVERFLOW_RISK_TOTAL
            .get_metric_with_label_values(&["warning"]).unwrap()
            .get();

        assert!(after > before, "expected warning counter to increase (before={}, after={})", before, after);
    }

    #[test]
    fn test_sacred_positions_reset_overflow_risk() {
        let mut engine = BeamTensorEngine::new();
        let mut beam = BeamTensor::default();
        // Set up as if we are already in overflow warning state
        beam.calculation_depth = 12_000;
        beam.overflow_risk = crate::models::OverflowRisk::Warning;
        // Choose position so next_position is sacred (3^2 -> 9)
        beam.position = 3;

        engine.run_entropy_loop(&mut beam).unwrap();

        assert!(matches!(beam.overflow_risk, crate::models::OverflowRisk::Safe));
        assert_eq!(beam.calculation_depth, 0, "calculation_depth should reset at sacred checkpoints");
    }
}
