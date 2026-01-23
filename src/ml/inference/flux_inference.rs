use crate::compression::CompressionHash;
use crate::data::attributes::Attributes;
use crate::error::Result;
use crate::flux_matrix::FluxMatrixEngine;
use crate::models::*;
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

/// Core inference engine for bidirectional reasoning:
/// - Forward inference: meanings → seeds
/// - Reverse inference: seeds → meanings
pub struct InferenceEngine {
    flux_engine: FluxMatrixEngine,
    subject_matrices: HashMap<String, FluxMatrix>,
    cached_inferences: HashMap<String, InferenceResult>,
}

impl Default for InferenceEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl InferenceEngine {
    pub fn new() -> Self {
        Self {
            flux_engine: FluxMatrixEngine::new(),
            subject_matrices: HashMap::new(),
            cached_inferences: HashMap::new(),
        }
    }

    /// Load subject matrices from storage
    pub fn load_subject_matrices(&mut self, matrices: Vec<FluxMatrix>) -> Result<()> {
        for matrix in matrices {
            self.subject_matrices.insert(matrix.subject.clone(), matrix);
        }
        Ok(())
    }

    /// Process inference with compression hashes (preferred method)
    /// Hashes → Flux positions → ELP channels → Matrix positions → Semantic meanings
    pub async fn process_inference(&mut self, input: InferenceInput) -> Result<InferenceResult> {
        let start_time = std::time::Instant::now();
        let result_id = Uuid::new_v4();

        // Process compression hashes if provided
        let mut hash_metadata = Vec::new();
        let mut all_sequences = Vec::new();

        if !input.compression_hashes.is_empty() {
            // NEW: Process compression hashes
            for hash_str in &input.compression_hashes {
                match CompressionHash::from_hex(hash_str) {
                    Ok(hash) => {
                        // Extract metadata from hash
                        let elp = hash.elp_channels();
                        let mut attributes = Attributes::with_elp(elp.ethos, elp.logos, elp.pathos);
                        attributes.set_confidence(hash.confidence());
                        
                        let metadata = HashMetadata {
                            hash_hex: hash_str.clone(),
                            flux_position: hash.flux_position(),
                            attributes,
                            rgb_color: hash.rgb_color(),
                            is_sacred: hash.is_sacred(),
                            confidence: hash.confidence(),
                        };
                        hash_metadata.push(metadata);

                        // Use hash position to generate flux sequence
                        let position = hash.flux_position() as u64;
                        let seed = position * 100 + (hash.confidence() * 100.0) as u64;
                        let sequence = self.flux_engine.seed_to_flux_sequence(seed);
                        all_sequences.push((seed, sequence, Some(hash)));
                    }
                    Err(e) => {
                        eprintln!("Warning: Invalid compression hash {}: {}", hash_str, e);
                    }
                }
            }
        } else if !input.seed_numbers.is_empty() {
            // LEGACY: Process seed numbers
            for seed in &input.seed_numbers {
                let sequence = self.flux_engine.seed_to_flux_sequence(*seed);
                all_sequences.push((*seed, sequence, None));
            }
        } else {
            return Err(crate::error::SpatialVortexError::InvalidInput(
                "No compression hashes or seed numbers provided".to_string(),
            ));
        }

        // Find matching matrices based on subject filter
        let matched_matrices = self.find_matching_matrices(&input.subject_filter)?;

        // Generate inferences for each sequence and matrix combination
        let mut all_inferences = Vec::new();
        for (_seed, sequence, hash_opt) in &all_sequences {
            for matrix in &matched_matrices {
                let mut inferences = self
                    .generate_inferences_for_sequence(
                        sequence,
                        matrix,
                        &input.processing_options,
                    )
                    .await?;

                // Enhance inferences with hash metadata if available
                if let Some(hash) = hash_opt {
                    let elp = hash.elp_channels();
                    for inference in &mut inferences {
                        // Adjust confidence based on ELP channel alignment
                        let channel_boost = elp.intensity() * 0.1;
                        inference.contextual_relevance = 
                            (inference.contextual_relevance + channel_boost).min(1.0);
                        
                        // Mark sacred positions
                        if hash.is_sacred() {
                            inference.contextual_relevance = 
                                (inference.contextual_relevance * 1.15).min(1.0);
                        }
                    }
                }

                all_inferences.extend(inferences);
            }
        }

        // Calculate overall confidence score
        let confidence_score = self.calculate_overall_confidence(&all_inferences);

        // Use microseconds for more precise timing, then convert to millis (minimum 1ms)
        let processing_time = ((start_time.elapsed().as_micros() as u64) / 1000).max(1);

        let result = InferenceResult {
            id: result_id,
            input,
            matched_matrices: matched_matrices.clone(),
            inferred_meanings: all_inferences,
            confidence_score,
            processing_time_ms: processing_time,
            created_at: Utc::now(),
            hash_metadata: if hash_metadata.is_empty() {
                None
            } else {
                Some(hash_metadata)
            },
        };

        // Cache result for future use
        let cache_key = format!("inference_{}", result_id);
        self.cached_inferences.insert(cache_key, result.clone());

        Ok(result)
    }

    /// Legacy: Process seed numbers (deprecated - use process_inference with InferenceInput)
    #[deprecated(since = "0.2.0", note = "Use process_inference with InferenceInput instead")]
    #[allow(deprecated)]  // This method itself uses deprecated types for backward compatibility
    pub async fn process_seed_input(&mut self, seed_input: SeedInput) -> Result<InferenceResult> {
        self.process_inference(seed_input.into()).await
    }

    /// REMOVED OLD IMPLEMENTATION - Now delegates to process_inference
    #[allow(dead_code)]  // Kept for reference but not actively used
    #[allow(deprecated)] // Uses deprecated SeedInput type
    async fn _legacy_process_seed_input(&mut self, seed_input: SeedInput) -> Result<InferenceResult> {
        let start_time = std::time::Instant::now();
        let result_id = Uuid::new_v4();

        // Convert seed numbers to flux sequences
        let mut all_sequences = Vec::new();
        for seed in &seed_input.seed_numbers {
            let sequence = self.flux_engine.seed_to_flux_sequence(*seed);
            all_sequences.push((*seed, sequence));
        }

        // Find matching matrices based on subject filter
        let matched_matrices = self.find_matching_matrices(&seed_input.subject_filter)?;

        // Generate inferences for each sequence and matrix combination
        let mut all_inferences = Vec::new();
        for (_seed, sequence) in &all_sequences {
            for matrix in &matched_matrices {
                let inferences = self
                    .generate_inferences_for_sequence(
                        sequence,
                        matrix,
                        &seed_input.processing_options,
                    )
                    .await?;
                all_inferences.extend(inferences);
            }
        }

        // Calculate overall confidence score
        let confidence_score = self.calculate_overall_confidence(&all_inferences);

        // Use microseconds for more precise timing, then convert to millis (minimum 1ms)
        let processing_time = ((start_time.elapsed().as_micros() as u64) / 1000).max(1);

        let result = InferenceResult {
            id: result_id,
            input: seed_input.into(),
            matched_matrices: matched_matrices.clone(),
            inferred_meanings: all_inferences,
            confidence_score,
            processing_time_ms: processing_time,
            created_at: Utc::now(),
            hash_metadata: None,
        };

        // Cache result for future use
        let cache_key = format!("inference_{}", result_id);
        self.cached_inferences.insert(cache_key, result.clone());

        Ok(result)
    }

    /// Find matrices matching the subject filter
    fn find_matching_matrices(&self, filter: &SubjectFilter) -> Result<Vec<FluxMatrix>> {
        let mut matches = Vec::new();

        match filter {
            SubjectFilter::Specific(subject) => {
                if let Some(matrix) = self.subject_matrices.get(subject) {
                    matches.push(matrix.clone());
                }
            }
            SubjectFilter::GeneralIntelligence => {
                // Filter for matrices related to general intelligence
                for (subject, matrix) in &self.subject_matrices {
                    if subject.to_lowercase().contains("intelligence")
                        || subject.to_lowercase().contains("general")
                        || subject.to_lowercase().contains("cognitive")
                    {
                        matches.push(matrix.clone());
                    }
                }
            }
            SubjectFilter::Category(category) => {
                // Filter by category keywords
                for (subject, matrix) in &self.subject_matrices {
                    if subject.to_lowercase().contains(&category.to_lowercase()) {
                        matches.push(matrix.clone());
                    }
                }
            }
            SubjectFilter::All => {
                matches.extend(self.subject_matrices.values().cloned());
            }
        }

        Ok(matches)
    }

    /// Generate inferences for a specific sequence and matrix
    async fn generate_inferences_for_sequence(
        &self,
        sequence: &[u8],
        matrix: &FluxMatrix,
        options: &ProcessingOptions,
    ) -> Result<Vec<InferredMeaning>> {
        let mut inferences = Vec::new();

        for (i, &flux_value) in sequence.iter().enumerate() {
            // Find corresponding position in matrix
            if let Some(position) = self.flux_engine.flux_value_to_position(flux_value) {
                let inference = self
                    .create_inference_for_position(
                        position,
                        matrix,
                        options,
                        i as f32 / sequence.len() as f32, // Position weight
                    )
                    .await?;

                if let Some(inf) = inference {
                    inferences.push(inf);
                }
            }
        }

        Ok(inferences)
    }

    /// Create inference for specific matrix position
    async fn create_inference_for_position(
        &self,
        position: u8,
        matrix: &FluxMatrix,
        options: &ProcessingOptions,
        position_weight: f32,
    ) -> Result<Option<InferredMeaning>> {
        let mut semantic_associations = Vec::new();
        let mut moral_alignment = MoralAlignment::Neutral;
        let primary_meaning: String;

        // Check if position is a regular node
        if let Some(node) = matrix.nodes.get(&position) {
            primary_meaning = node.semantic_index.neutral_base.clone();

            // Include positive associations if requested
            if options.include_synonyms {
                for assoc in &node.semantic_index.positive_associations {
                    if assoc.confidence >= options.confidence_threshold as f64 {
                        semantic_associations.push(assoc.clone());
                    }
                }

                // Determine moral alignment based on positive associations
                let positive_weight: f32 = node
                    .semantic_index
                    .positive_associations
                    .iter()
                    .map(|a| (a.confidence as f32) * (a.index as f32).abs())
                    .sum();

                if positive_weight > 0.0 {
                    moral_alignment = MoralAlignment::Constructive(positive_weight);
                }
            }

            // Include negative associations if requested
            if options.include_antonyms {
                for assoc in &node.semantic_index.negative_associations {
                    if assoc.confidence >= options.confidence_threshold as f64 {
                        semantic_associations.push(assoc.clone());
                    }
                }

                // Update moral alignment based on negative associations
                let negative_weight: f32 = node
                    .semantic_index
                    .negative_associations
                    .iter()
                    .map(|a| (a.confidence as f32) * (a.index as f32).abs())
                    .sum();

                if negative_weight > 0.0 {
                    moral_alignment = match moral_alignment {
                        MoralAlignment::Constructive(pos) => {
                            if negative_weight > pos {
                                MoralAlignment::Destructive(negative_weight - pos)
                            } else {
                                MoralAlignment::Constructive(pos - negative_weight)
                            }
                        }
                        _ => MoralAlignment::Destructive(negative_weight),
                    };
                }
            }
        }
        // Check if position is a sacred guide
        else if let Some(guide) = matrix.sacred_guides.get(&position) {
            primary_meaning = format!(
                "Sacred Guide {}: {}",
                position,
                guide.divine_properties.join(", ")
            );

            // Sacred guides have inherently positive moral alignment
            moral_alignment = MoralAlignment::Constructive(1.5);

            // Add sacred properties as semantic associations
            for property in &guide.divine_properties {
                let mut assoc = SemanticAssociation {
                    word: property.clone(),
                    index: position as i16, // Use position as positive index
                    confidence: 0.95,       // High confidence for sacred properties
                    attributes: HashMap::new(),
                };
                // Add context as attribute
                assoc.set_attribute("context".to_string(), 1.0); // Sacred Guide
                semantic_associations.push(assoc);
            }
        } else {
            return Ok(None); // Position not found in matrix
        }

        let contextual_relevance =
            self.calculate_contextual_relevance(&semantic_associations, position_weight, options);

        Ok(Some(InferredMeaning {
            subject: matrix.subject.clone(),
            node_position: position,
            primary_meaning,
            semantic_associations,
            contextual_relevance,
            moral_alignment,
        }))
    }

    /// Calculate contextual relevance score
    fn calculate_contextual_relevance(
        &self,
        associations: &[SemanticAssociation],
        position_weight: f32,
        options: &ProcessingOptions,
    ) -> f32 {
        if associations.is_empty() {
            return 0.1; // Minimal relevance for empty associations
        }

        let avg_confidence: f32 =
            associations.iter().map(|a| a.confidence as f32).sum::<f32>() / associations.len() as f32;

        let association_count_factor = (associations.len() as f32).min(10.0) / 10.0;
        let position_factor = 1.0 - position_weight * 0.3; // Earlier positions slightly more relevant

        let base_relevance = avg_confidence * association_count_factor * position_factor;

        // Apply options modifiers
        let mut relevance_modifier = 1.0;
        if options.use_sacred_guides {
            relevance_modifier *= 1.1;
        }
        if options.include_synonyms && options.include_antonyms {
            relevance_modifier *= 1.2; // Bonus for comprehensive analysis
        }

        (base_relevance * relevance_modifier).min(1.0)
    }

    /// Calculate overall confidence score for inference result
    fn calculate_overall_confidence(&self, inferences: &[InferredMeaning]) -> f32 {
        if inferences.is_empty() {
            return 0.0;
        }

        let total_relevance: f32 = inferences.iter().map(|inf| inf.contextual_relevance).sum();

        let avg_relevance = total_relevance / inferences.len() as f32;

        // Factor in moral alignment consistency
        let moral_consistency = self.calculate_moral_consistency(inferences);

        (avg_relevance * 0.7 + moral_consistency * 0.3).min(1.0)
    }

    /// Calculate moral consistency across inferences
    fn calculate_moral_consistency(&self, inferences: &[InferredMeaning]) -> f32 {
        let mut constructive_count = 0;
        let mut destructive_count = 0;
        let mut neutral_count = 0;

        for inference in inferences {
            match &inference.moral_alignment {
                MoralAlignment::Constructive(_) => constructive_count += 1,
                MoralAlignment::Destructive(_) => destructive_count += 1,
                MoralAlignment::Neutral => neutral_count += 1,
            }
        }

        let total = inferences.len() as f32;
        let max_alignment = constructive_count.max(destructive_count).max(neutral_count) as f32;

        max_alignment / total // Higher consistency = higher score
    }

    /// Forward inference: find seed numbers that would produce given words/meanings
    /// Meanings → Matrix positions → Candidate seed numbers
    pub async fn forward_inference(
        &self,
        target_meanings: Vec<String>,
        subject_filter: &SubjectFilter,
    ) -> Result<Vec<u64>> {
        let mut candidate_seeds = Vec::new();
        let matched_matrices = self.find_matching_matrices(subject_filter)?;

        // Search through possible seed ranges
        for seed in 1u64..=9999999999u64 {
            // Search up to 10-digit seeds
            let sequence = self.flux_engine.seed_to_flux_sequence(seed);

            if self
                .sequence_matches_meanings(&sequence, &target_meanings, &matched_matrices)
                .await?
            {
                candidate_seeds.push(seed);
            }

            if candidate_seeds.len() >= 10 {
                // Limit results
                break;
            }
        }

        Ok(candidate_seeds)
    }

    /// Check if a sequence produces the target meanings
    async fn sequence_matches_meanings(
        &self,
        sequence: &[u8],
        target_meanings: &[String],
        matrices: &[FluxMatrix],
    ) -> Result<bool> {
        for matrix in matrices {
            for &flux_value in sequence {
                if let Some(position) = self.flux_engine.flux_value_to_position(flux_value) {
                    if let Some(node) = matrix.nodes.get(&position) {
                        let node_meaning = &node.semantic_index.neutral_base;

                        for target in target_meanings {
                            if node_meaning.to_lowercase().contains(&target.to_lowercase())
                                || target.to_lowercase().contains(&node_meaning.to_lowercase())
                            {
                                return Ok(true);
                            }
                        }

                        // Check semantic associations
                        for assoc in &node.semantic_index.positive_associations {
                            for target in target_meanings {
                                if assoc.word.to_lowercase().contains(&target.to_lowercase()) {
                                    return Ok(true);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(false)
    }

    /// Get cached inference result
    pub fn get_cached_inference(&self, inference_id: &Uuid) -> Option<&InferenceResult> {
        let cache_key = format!("inference_{}", inference_id);
        self.cached_inferences.get(&cache_key)
    }

    /// Clear inference cache
    pub fn clear_cache(&mut self) {
        self.cached_inferences.clear();
    }

    /// Get matrix for subject
    pub fn get_subject_matrix(&self, subject: &str) -> Option<&FluxMatrix> {
        self.subject_matrices.get(subject)
    }

    /// Add or update subject matrix
    pub fn update_subject_matrix(&mut self, matrix: FluxMatrix) {
        self.subject_matrices.insert(matrix.subject.clone(), matrix);
    }

    /// Get inference statistics
    pub fn get_statistics(&self) -> InferenceStatistics {
        InferenceStatistics {
            total_matrices: self.subject_matrices.len(),
            cached_inferences: self.cached_inferences.len(),
            subjects: self.subject_matrices.keys().cloned().collect(),
        }
    }
}

/// Statistics about the inference engine state
#[derive(Debug, Clone)]
pub struct InferenceStatistics {
    pub total_matrices: usize,
    pub cached_inferences: usize,
    pub subjects: Vec<String>,
}
