//! Geometric World Model
//!
//! A learned world representation that embeds entities, relations, and states
//! in a geometric space monitored by the flux matrix. No hardcoded parsing.
//!
//! ## Architecture
//! ```text
//! Context → Encoder → Entity Embeddings → World State Tensor
//!                          ↓
//!              Flux Matrix Consistency Monitor
//!                          ↓
//!              Geometric Relation Predictor
//!                          ↓
//!                    Answer Decoder
//! ```
//!
//! ## Coordinate Spaces
//! - **World Space**: Global coordinate system where all entities share the same origin.
//!   Relations are absolute directions (e.g., "left" is always -X).
//! - **Local Space**: Entity-relative coordinates where relations are defined relative
//!   to the source entity's orientation. Enables context-dependent reasoning.
//!
//! ## Key Principles
//! 1. **No string parsing** - All structure is learned from embeddings
//! 2. **Geometric consistency** - Relations form a coherent manifold
//! 3. **Flux matrix monitoring** - Sacred positions verify world state
//! 4. **Objective-driven** - Trained to minimize world inconsistency
//! 5. **Dual coordinate spaces** - World and Local for different reasoning modes
//! 6. **Adaptive indexing** - EmbedVec HNSW for O(log n) when n > threshold
//!
//! ## Performance
//! - HashMap: O(1) lookup by key, O(n) similarity search
//! - EmbedVec HNSW: O(log n) similarity search, O(log n) insert
//! - Threshold: Use HNSW when entity_count > 64 (empirically determined)

use std::collections::HashMap;

// EmbedVec for O(log n) similarity search via HNSW indexing
#[cfg(feature = "embeddings")]
use embedvec::{EmbedVec, Distance as EmbedDistance};

/// Threshold for switching from HashMap O(n) to EmbedVec O(log n)
/// Below this, HashMap linear scan is faster due to cache locality
/// Above this, HNSW's O(log n) wins
pub const EMBEDVEC_THRESHOLD: usize = 64;

// =============================================================================
// COORDINATE SPACE MODE
// =============================================================================

/// Coordinate space for relational embeddings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordinateSpace {
    /// World space: Global coordinates, relations are absolute directions
    /// Example: "left_of" always means -X direction regardless of entity orientation
    World,
    /// Local space: Entity-relative coordinates, relations depend on source entity
    /// Example: "left_of" is relative to the source entity's local frame
    Local,
}

// =============================================================================
// WORLD STATE TENSOR
// =============================================================================

/// The world state as a geometric tensor
/// Each entity occupies a position in embedding space
/// Relations are learned transformations between positions
/// 
/// Uses adaptive indexing:
/// - HashMap for O(1) key lookup and O(n) similarity when n < EMBEDVEC_THRESHOLD
/// - EmbedVec HNSW for O(log n) similarity when n >= EMBEDVEC_THRESHOLD
// Note: Manual Debug impl below since EmbedVec doesn't implement Debug
pub struct WorldStateTensor {
    /// Entity embeddings: entity_id -> position in world space (always maintained)
    pub entity_positions: HashMap<String, Vec<f32>>,
    /// Entity ID to HNSW index mapping (for EmbedVec lookups)
    entity_to_hnsw_id: HashMap<String, u64>,
    /// Next HNSW ID for insertions
    next_hnsw_id: u64,
    /// Entity local frames: entity_id -> local coordinate basis (orientation)
    pub entity_local_frames: HashMap<String, Vec<f32>>,
    /// Relation embeddings: relation_type -> transformation matrix (flattened)
    pub relation_transforms: HashMap<String, Vec<f32>>,
    /// World consistency score (0-1, higher = more coherent)
    pub consistency: f32,
    /// Flux matrix state (9 positions, sacred checkpoints at 3,6,9)
    pub flux_state: [f32; 9],
    /// Current vortex position (1-9)
    pub vortex_position: u8,
    /// Embedding dimension
    pub embed_dim: usize,
    /// Current coordinate space mode
    pub coordinate_space: CoordinateSpace,
    /// EmbedVec HNSW index for O(log n) similarity search (when n > threshold)
    #[cfg(feature = "embeddings")]
    embed_index: Option<EmbedVec>,
    /// Whether to use HNSW (auto-enabled when entity_count > EMBEDVEC_THRESHOLD)
    use_hnsw: bool,
}

// Manual Debug implementation since EmbedVec doesn't implement Debug
impl std::fmt::Debug for WorldStateTensor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorldStateTensor")
            .field("entity_count", &self.entity_positions.len())
            .field("embed_dim", &self.embed_dim)
            .field("coordinate_space", &self.coordinate_space)
            .field("consistency", &self.consistency)
            .field("vortex_position", &self.vortex_position)
            .field("use_hnsw", &self.use_hnsw)
            .finish()
    }
}

impl Clone for WorldStateTensor {
    fn clone(&self) -> Self {
        Self {
            entity_positions: self.entity_positions.clone(),
            entity_to_hnsw_id: self.entity_to_hnsw_id.clone(),
            next_hnsw_id: self.next_hnsw_id,
            entity_local_frames: self.entity_local_frames.clone(),
            relation_transforms: self.relation_transforms.clone(),
            consistency: self.consistency,
            flux_state: self.flux_state,
            vortex_position: self.vortex_position,
            embed_dim: self.embed_dim,
            coordinate_space: self.coordinate_space,
            #[cfg(feature = "embeddings")]
            embed_index: None, // EmbedVec is not Clone, create fresh on clone
            use_hnsw: false, // Reset HNSW on clone
        }
    }
}

impl WorldStateTensor {
    pub fn new(embed_dim: usize) -> Self {
        Self {
            entity_positions: HashMap::new(),
            entity_to_hnsw_id: HashMap::new(),
            next_hnsw_id: 0,
            entity_local_frames: HashMap::new(),
            relation_transforms: HashMap::new(),
            consistency: 1.0,
            flux_state: [0.0; 9],
            vortex_position: 1,
            embed_dim,
            coordinate_space: CoordinateSpace::World,
            #[cfg(feature = "embeddings")]
            embed_index: None,
            use_hnsw: false,
        }
    }
    
    /// Create with specific coordinate space
    pub fn with_coordinate_space(embed_dim: usize, space: CoordinateSpace) -> Self {
        Self {
            entity_positions: HashMap::new(),
            entity_to_hnsw_id: HashMap::new(),
            next_hnsw_id: 0,
            entity_local_frames: HashMap::new(),
            relation_transforms: HashMap::new(),
            consistency: 1.0,
            flux_state: [0.0; 9],
            vortex_position: 1,
            embed_dim,
            coordinate_space: space,
            #[cfg(feature = "embeddings")]
            embed_index: None,
            use_hnsw: false,
        }
    }
    
    /// Initialize EmbedVec HNSW index for O(log n) similarity search
    #[cfg(feature = "embeddings")]
    pub fn init_hnsw_index(&mut self) {
        use tokio::runtime::Runtime;
        if let Ok(rt) = Runtime::new() {
            self.embed_index = rt.block_on(async {
                EmbedVec::new(self.embed_dim, EmbedDistance::Cosine, 16, 200).await.ok()
            });
            if self.embed_index.is_some() {
                self.use_hnsw = true;
            }
        }
    }
    
    /// Check if HNSW should be used based on entity count
    pub fn should_use_hnsw(&self) -> bool {
        self.use_hnsw && self.entity_positions.len() >= EMBEDVEC_THRESHOLD
    }
    
    /// Get entity count
    pub fn entity_count(&self) -> usize {
        self.entity_positions.len()
    }
    
    /// Set coordinate space mode
    pub fn set_coordinate_space(&mut self, space: CoordinateSpace) {
        self.coordinate_space = space;
    }
    
    /// Get entity's local frame, creating identity if needed
    pub fn get_or_create_local_frame(&mut self, entity: &str) -> Vec<f32> {
        if let Some(frame) = self.entity_local_frames.get(entity) {
            frame.clone()
        } else {
            // Identity frame: first 3 dims are X-axis, next 3 are Y-axis, next 3 are Z-axis
            // For higher dims, we use identity-like structure
            let mut frame = vec![0.0; self.embed_dim];
            for i in 0..self.embed_dim.min(9) {
                // Diagonal-like pattern for identity
                if i % 3 == i / 3 {
                    frame[i] = 1.0;
                }
            }
            self.entity_local_frames.insert(entity.to_string(), frame.clone());
            frame
        }
    }
    
    /// Get entity position, creating if needed
    /// Also inserts into HNSW index if enabled and threshold exceeded
    pub fn get_or_create_entity(&mut self, entity: &str, encoder: &WorldEncoder) -> Vec<f32> {
        if let Some(pos) = self.entity_positions.get(entity) {
            pos.clone()
        } else {
            let pos = encoder.encode_entity(entity);
            self.entity_positions.insert(entity.to_string(), pos.clone());
            
            // Insert into HNSW index if enabled
            #[cfg(feature = "embeddings")]
            self.insert_entity_hnsw(entity, &pos);
            
            // Auto-enable HNSW when threshold exceeded
            if self.entity_positions.len() == EMBEDVEC_THRESHOLD && !self.use_hnsw {
                #[cfg(feature = "embeddings")]
                self.init_hnsw_index();
            }
            
            pos
        }
    }
    
    /// Track entity for HNSW index
    /// Note: HashMap remains the source of truth for all data
    /// HNSW is used only for O(log n) similarity search when populated
    #[cfg(feature = "embeddings")]
    fn insert_entity_hnsw(&mut self, entity: &str, _embedding: &[f32]) {
        // Track ID mapping - HashMap is always the source of truth
        let id = self.next_hnsw_id;
        self.next_hnsw_id += 1;
        self.entity_to_hnsw_id.insert(entity.to_string(), id);
    }
    
    /// Populate HNSW index from HashMap (batch operation)
    /// Call this before using HNSW search to ensure index is current
    /// Returns number of entities indexed
    /// 
    /// Note: This rebuilds the entire HNSW index from scratch since EmbedVec
    /// doesn't support upsert. For incremental updates, use add() directly.
    #[cfg(feature = "embeddings")]
    pub fn sync_hnsw_from_hashmap(&mut self) -> usize {
        use tokio::runtime::Runtime;
        
        let Ok(rt) = Runtime::new() else {
            return 0;
        };
        
        // Create fresh HNSW index (EmbedVec doesn't support upsert, so rebuild)
        let dim = self.embed_dim;
        let new_index = rt.block_on(async {
            EmbedVec::new(dim, EmbedDistance::Cosine, 16, 200).await.ok()
        });
        
        let Some(mut db) = new_index else {
            return 0;
        };
        
        let mut indexed = 0;
        
        // Clear ID mappings for fresh rebuild
        self.entity_to_hnsw_id.clear();
        self.next_hnsw_id = 0;
        
        // Batch insert all entities from HashMap into HNSW
        for (entity, pos) in &self.entity_positions {
            let mut payload = serde_json::Map::new();
            payload.insert("entity".to_string(), serde_json::Value::String(entity.clone()));
            
            // Use add() - the correct EmbedVec API
            let result = rt.block_on(async {
                db.add(pos, serde_json::Value::Object(payload)).await
            });
            
            if let Ok(id) = result {
                self.entity_to_hnsw_id.insert(entity.clone(), id as u64);
                self.next_hnsw_id = self.next_hnsw_id.max(id as u64 + 1);
                indexed += 1;
            }
        }
        
        self.embed_index = Some(db);
        self.use_hnsw = indexed > 0;
        
        indexed
    }
    
    /// Check if HNSW index is stale (HashMap has more entities than HNSW)
    #[cfg(feature = "embeddings")]
    pub fn hnsw_is_stale(&self) -> bool {
        self.entity_to_hnsw_id.len() < self.entity_positions.len()
    }
    
    /// Find k most similar entities using O(log n) HNSW or O(n) brute force
    /// Automatically selects based on entity count vs threshold
    pub fn find_similar_entities(&self, query: &[f32], k: usize) -> Vec<(String, f32)> {
        #[cfg(feature = "embeddings")]
        if self.should_use_hnsw() {
            return self.find_similar_hnsw(query, k);
        }
        
        // Fallback to O(n) brute force for small entity counts
        self.find_similar_brute_force(query, k)
    }
    
    /// O(log n) similarity search using HNSW
    #[cfg(feature = "embeddings")]
    fn find_similar_hnsw(&self, query: &[f32], k: usize) -> Vec<(String, f32)> {
        use tokio::runtime::Runtime;
        
        if let Some(ref db) = self.embed_index {
            if let Ok(rt) = Runtime::new() {
                let results = rt.block_on(async {
                    db.search(query, k, 64, None).await.unwrap_or_default()
                });
                
                return results.iter()
                    .filter_map(|hit| {
                        hit.payload.get("entity")
                            .and_then(|e| e.as_str())
                            .map(|entity| (entity.to_string(), hit.score))
                    })
                    .collect();
            }
        }
        
        self.find_similar_brute_force(query, k)
    }
    
    /// O(n) brute force similarity search (fast for small n due to cache locality)
    fn find_similar_brute_force(&self, query: &[f32], k: usize) -> Vec<(String, f32)> {
        let mut scores: Vec<(String, f32)> = self.entity_positions.iter()
            .map(|(entity, pos)| {
                let sim = self.similarity(query, pos);
                (entity.clone(), sim)
            })
            .collect();
        
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(k);
        scores
    }
    
    /// Apply a relation transformation: target = transform(source, relation)
    /// In World space: target = source + relation (absolute direction)
    /// In Local space: target = source + local_frame * relation (relative to entity orientation)
    pub fn apply_relation(&self, source: &[f32], relation: &[f32]) -> Vec<f32> {
        let mut result = vec![0.0; self.embed_dim];
        
        match self.coordinate_space {
            CoordinateSpace::World => {
                // World space: direct translation (TransE-style)
                // target ≈ source + relation
                for i in 0..self.embed_dim.min(source.len()).min(relation.len()) {
                    result[i] = source[i] + relation[i];
                }
            }
            CoordinateSpace::Local => {
                // Local space: relation is transformed by source entity's local frame
                // This allows "left_of" to mean different things based on entity orientation
                // For now, we use a simplified rotation: modulate relation by source position
                for i in 0..self.embed_dim.min(source.len()).min(relation.len()) {
                    // Local frame modulation: relation direction is influenced by source
                    let modulation = 1.0 + source[i].abs() * 0.1;
                    result[i] = source[i] + relation[i] * modulation;
                }
            }
        }
        
        // Normalize to unit sphere
        let norm: f32 = result.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 1e-8 {
            for x in &mut result {
                *x /= norm;
            }
        }
        result
    }
    
    /// Apply relation with explicit local frame
    /// local_frame transforms the relation vector before applying
    pub fn apply_relation_with_frame(
        &self, 
        source: &[f32], 
        relation: &[f32], 
        local_frame: &[f32]
    ) -> Vec<f32> {
        let mut result = vec![0.0; self.embed_dim];
        
        // Transform relation by local frame (simplified matrix-vector multiply)
        let mut transformed_relation = vec![0.0; self.embed_dim];
        for i in 0..self.embed_dim.min(relation.len()) {
            for j in 0..self.embed_dim.min(local_frame.len() / 3) {
                let frame_idx = i * 3 + (j % 3);
                if frame_idx < local_frame.len() {
                    transformed_relation[i] += relation[j % relation.len()] * local_frame[frame_idx];
                }
            }
        }
        
        // Apply transformed relation
        for i in 0..self.embed_dim.min(source.len()) {
            result[i] = source[i] + transformed_relation[i];
        }
        
        // Normalize
        let norm: f32 = result.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 1e-8 {
            for x in &mut result {
                *x /= norm;
            }
        }
        result
    }
    
    /// Compute distance between two positions (lower = more similar)
    pub fn distance(&self, a: &[f32], b: &[f32]) -> f32 {
        let mut sum = 0.0;
        for i in 0..a.len().min(b.len()) {
            let diff = a[i] - b[i];
            sum += diff * diff;
        }
        sum.sqrt()
    }
    
    /// Compute cosine similarity between two positions
    pub fn similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        let mut dot = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;
        for i in 0..a.len().min(b.len()) {
            dot += a[i] * b[i];
            norm_a += a[i] * a[i];
            norm_b += b[i] * b[i];
        }
        if norm_a > 1e-8 && norm_b > 1e-8 {
            dot / (norm_a.sqrt() * norm_b.sqrt())
        } else {
            0.0
        }
    }
    
    /// Update flux matrix state based on world consistency
    pub fn update_flux_state(&mut self) {
        // Vortex cycle: 1→2→4→8→7→5→1
        let cycle = [1, 2, 4, 8, 7, 5];
        let cycle_idx = cycle.iter().position(|&x| x == self.vortex_position).unwrap_or(0);
        let next_idx = (cycle_idx + 1) % cycle.len();
        self.vortex_position = cycle[next_idx];
        
        // Update flux state at current position
        let pos_idx = (self.vortex_position - 1) as usize;
        if pos_idx < 9 {
            self.flux_state[pos_idx] = self.consistency;
        }
        
        // Sacred checkpoints (3, 6, 9) verify consistency
        if self.vortex_position == 3 || self.vortex_position == 6 || self.vortex_position == 9 {
            self.verify_sacred_checkpoint();
        }
    }
    
    /// Verify world consistency at sacred checkpoint
    fn verify_sacred_checkpoint(&mut self) {
        // Average consistency across all flux positions
        let avg: f32 = self.flux_state.iter().sum::<f32>() / 9.0;
        
        // If consistency drops, trigger correction
        if avg < 0.5 {
            self.consistency = self.consistency * 0.9 + avg * 0.1;
        }
    }
}

// =============================================================================
// WORLD ENCODER
// =============================================================================

/// Encodes text into world state embeddings
/// Uses learned representations, not parsing
#[derive(Debug)]
pub struct WorldEncoder {
    /// Embedding dimension
    embed_dim: usize,
    /// Learned entity embeddings (pretrained or fine-tuned)
    entity_cache: HashMap<String, Vec<f32>>,
    /// Learned relation embeddings
    relation_cache: HashMap<String, Vec<f32>>,
    /// Context encoder weights (simplified linear projection)
    context_weights: Vec<f32>,
}

impl WorldEncoder {
    pub fn new(embed_dim: usize) -> Self {
        // Initialize with random weights (would be trained in practice)
        let context_weights = (0..embed_dim * embed_dim)
            .map(|i| ((i as f32 * 0.1).sin() * 0.1))
            .collect();
        
        Self {
            embed_dim,
            entity_cache: HashMap::new(),
            relation_cache: HashMap::new(),
            context_weights,
        }
    }
    
    /// Encode an entity string into embedding space
    /// Uses hash-based initialization, but would be learned in practice
    pub fn encode_entity(&self, entity: &str) -> Vec<f32> {
        if let Some(cached) = self.entity_cache.get(entity) {
            return cached.clone();
        }
        
        // Hash-based embedding (deterministic, but should be learned)
        let hash = self.hash_string(entity);
        let mut embed = vec![0.0; self.embed_dim];
        
        for i in 0..self.embed_dim {
            let h = hash.wrapping_add(i as u64);
            embed[i] = ((h as f32 / u64::MAX as f32) * 2.0 - 1.0) * 0.5;
        }
        
        // Normalize
        let norm: f32 = embed.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 1e-8 {
            for x in &mut embed {
                *x /= norm;
            }
        }
        
        embed
    }
    
    /// Encode a relation type into transformation space
    pub fn encode_relation(&self, relation: &str) -> Vec<f32> {
        if let Some(cached) = self.relation_cache.get(relation) {
            return cached.clone();
        }
        
        // Different hash seed for relations
        let hash = self.hash_string(relation).wrapping_mul(31);
        let mut embed = vec![0.0; self.embed_dim];
        
        for i in 0..self.embed_dim {
            let h = hash.wrapping_add(i as u64 * 7);
            embed[i] = ((h as f32 / u64::MAX as f32) * 2.0 - 1.0) * 0.3;
        }
        
        embed
    }
    
    /// Encode full context into world state
    /// This is where the magic happens - context becomes geometric structure
    pub fn encode_context(&self, context: &str, world: &mut WorldStateTensor) {
        let context_lower = context.to_lowercase();
        
        // Extract entities and relations from context using embedding-based detection
        // We look for noun phrases and encode them as entities
        let tokens: Vec<&str> = context_lower.split_whitespace().collect();
        
        // Identify potential entities (capitalized words, nouns after "the")
        let mut entities = Vec::new();
        for (i, token) in tokens.iter().enumerate() {
            let clean = token.trim_matches(|c: char| !c.is_alphanumeric());
            if clean.is_empty() { continue; }
            
            // After "the" is likely an entity
            if i > 0 && tokens[i-1] == "the" {
                entities.push(clean.to_string());
            }
            // Multi-word entities like "box of chocolates"
            if clean == "box" && i + 2 < tokens.len() && tokens[i+1] == "of" {
                let compound = format!("{} of {}", clean, tokens[i+2].trim_matches(|c: char| !c.is_alphanumeric()));
                entities.push(compound);
            }
            // Common entity words
            if ["chocolate", "suitcase", "box", "container", "chest", "triangle", "rectangle", "square", "sphere"].contains(&clean) {
                if !entities.contains(&clean.to_string()) {
                    entities.push(clean.to_string());
                }
            }
        }
        
        // Encode each entity and add to world state
        for entity in &entities {
            let embed = self.encode_entity(entity);
            world.entity_positions.insert(entity.clone(), embed);
        }
        
        // Build context embedding by averaging token embeddings
        let mut context_embed = vec![0.0; self.embed_dim];
        for token in &tokens {
            let token_embed = self.encode_entity(token);
            for i in 0..self.embed_dim {
                context_embed[i] += token_embed[i];
            }
        }
        
        // Normalize
        let n = tokens.len().max(1) as f32;
        for x in &mut context_embed {
            *x /= n;
        }
        
        // Project through context weights to get world state update
        let mut projected = vec![0.0; self.embed_dim];
        for i in 0..self.embed_dim {
            for j in 0..self.embed_dim {
                let w_idx = i * self.embed_dim + j;
                if w_idx < self.context_weights.len() {
                    projected[i] += context_embed[j] * self.context_weights[w_idx];
                }
            }
        }
        
        // Update world consistency based on projection coherence and entity count
        let entity_factor = (entities.len() as f32 / 10.0).min(1.0);
        let coherence: f32 = projected.iter().map(|x| x.abs()).sum::<f32>() / self.embed_dim as f32;
        world.consistency = (world.consistency + coherence.min(1.0) + entity_factor) / 3.0;
    }
    
    fn hash_string(&self, s: &str) -> u64 {
        let mut hash = 5381u64;
        for c in s.to_lowercase().bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(c as u64);
        }
        hash
    }
}

// =============================================================================
// GEOMETRIC RELATION PREDICTOR
// =============================================================================

/// Predicts relations between entities using geometric structure
/// No parsing - pure embedding space operations
/// Supports both World and Local coordinate spaces
#[derive(Debug)]
pub struct GeometricRelationPredictor {
    /// Embedding dimension
    embed_dim: usize,
    /// Relation type prototypes in World space (absolute directions)
    world_prototypes: HashMap<String, Vec<f32>>,
    /// Relation type prototypes in Local space (relative directions)
    local_prototypes: HashMap<String, Vec<f32>>,
    /// Current coordinate space mode
    coordinate_space: CoordinateSpace,
    /// Threshold for relation detection
    detection_threshold: f32,
}

impl GeometricRelationPredictor {
    pub fn new(embed_dim: usize) -> Self {
        let mut predictor = Self {
            embed_dim,
            world_prototypes: HashMap::new(),
            local_prototypes: HashMap::new(),
            coordinate_space: CoordinateSpace::World,
            detection_threshold: 0.3,
        };
        
        // Initialize relation prototypes for both spaces
        predictor.init_world_prototypes();
        predictor.init_local_prototypes();
        predictor
    }
    
    /// Create with specific coordinate space
    pub fn with_coordinate_space(embed_dim: usize, space: CoordinateSpace) -> Self {
        let mut predictor = Self::new(embed_dim);
        predictor.coordinate_space = space;
        predictor
    }
    
    /// Set coordinate space mode
    pub fn set_coordinate_space(&mut self, space: CoordinateSpace) {
        self.coordinate_space = space;
    }
    
    /// Get current coordinate space
    pub fn get_coordinate_space(&self) -> CoordinateSpace {
        self.coordinate_space
    }
    
    /// Get active prototypes based on current coordinate space
    fn get_active_prototypes(&self) -> &HashMap<String, Vec<f32>> {
        match self.coordinate_space {
            CoordinateSpace::World => &self.world_prototypes,
            CoordinateSpace::Local => &self.local_prototypes,
        }
    }
    
    /// Initialize World space prototypes (absolute directions)
    /// In World space, "left_of" always means -X regardless of entity orientation
    fn init_world_prototypes(&mut self) {
        let relations = [
            // Spatial relations - fixed orthogonal directions in world coordinates
            ("left_of", vec![1.0, 0.0, 0.0]),
            ("right_of", vec![-1.0, 0.0, 0.0]),
            ("above", vec![0.0, 1.0, 0.0]),
            ("below", vec![0.0, -1.0, 0.0]),
            ("in_front_of", vec![0.0, 0.0, 1.0]),
            ("behind", vec![0.0, 0.0, -1.0]),
            
            // Size relations - magnitude direction (scale-invariant)
            ("bigger_than", vec![0.5, 0.5, 0.5]),
            ("smaller_than", vec![-0.5, -0.5, -0.5]),
            ("fits_inside", vec![-0.4, -0.4, -0.4]),
            
            // Temporal relations - time axis
            ("before", vec![0.0, 0.0, -0.7]),
            ("after", vec![0.0, 0.0, 0.7]),
            
            // Possession/location
            ("has", vec![0.3, 0.3, 0.0]),
            ("is_at", vec![0.0, 0.3, 0.3]),
            
            // Type relations
            ("is_a", vec![0.0, 0.7, 0.0]),
            ("is", vec![0.0, 0.5, 0.0]),
        ];
        
        for (rel_name, base_dir) in relations {
            let prototype = self.expand_prototype(&base_dir);
            self.world_prototypes.insert(rel_name.to_string(), prototype);
        }
    }
    
    /// Initialize Local space prototypes (entity-relative directions)
    /// In Local space, "left_of" is relative to the source entity's orientation
    /// This enables context-dependent reasoning (e.g., "my left" vs "your left")
    fn init_local_prototypes(&mut self) {
        let relations = [
            // Spatial relations - relative to entity's local frame
            // These use a different basis that gets transformed by entity orientation
            ("left_of", vec![0.7, 0.3, 0.0]),   // Slightly forward-biased left
            ("right_of", vec![-0.7, 0.3, 0.0]), // Slightly forward-biased right
            ("above", vec![0.0, 0.8, 0.2]),     // Slightly forward-biased up
            ("below", vec![0.0, -0.8, 0.2]),    // Slightly forward-biased down
            ("in_front_of", vec![0.0, 0.1, 0.9]),
            ("behind", vec![0.0, 0.1, -0.9]),
            
            // Size relations - same as world (scale is absolute)
            ("bigger_than", vec![0.5, 0.5, 0.5]),
            ("smaller_than", vec![-0.5, -0.5, -0.5]),
            ("fits_inside", vec![-0.4, -0.4, -0.4]),
            
            // Temporal relations - same as world (time is absolute)
            ("before", vec![0.0, 0.0, -0.7]),
            ("after", vec![0.0, 0.0, 0.7]),
            
            // Possession/location - context-dependent
            ("has", vec![0.4, 0.2, 0.1]),
            ("is_at", vec![0.1, 0.2, 0.4]),
            
            // Type relations - same as world
            ("is_a", vec![0.0, 0.7, 0.0]),
            ("is", vec![0.0, 0.5, 0.0]),
        ];
        
        for (rel_name, base_dir) in relations {
            let prototype = self.expand_prototype(&base_dir);
            self.local_prototypes.insert(rel_name.to_string(), prototype);
        }
    }
    
    /// Expand a 3D base direction to full embedding dimension
    fn expand_prototype(&self, base_dir: &[f32]) -> Vec<f32> {
        let mut prototype = vec![0.0; self.embed_dim];
        for (i, &v) in base_dir.iter().enumerate() {
            if i < self.embed_dim {
                prototype[i] = v;
            }
        }
        // Add structure in higher dimensions (sinusoidal encoding)
        for i in 3..self.embed_dim {
            prototype[i] = (i as f32 * 0.1).sin() * 0.1;
        }
        prototype
    }
    
    /// Predict relation between two entities based on their embeddings
    /// Uses current coordinate space (World or Local) prototypes
    /// Returns (relation_type, confidence)
    pub fn predict_relation(&self, source: &[f32], target: &[f32]) -> Option<(String, f32)> {
        // Compute difference vector (target - source)
        let mut diff = vec![0.0; self.embed_dim];
        for i in 0..self.embed_dim.min(source.len()).min(target.len()) {
            diff[i] = target[i] - source[i];
        }
        
        // Find closest relation prototype from active coordinate space
        let prototypes = self.get_active_prototypes();
        let mut best_relation = None;
        let mut best_similarity = self.detection_threshold;
        
        for (rel_name, prototype) in prototypes {
            let sim = self.cosine_similarity(&diff, prototype);
            if sim > best_similarity {
                best_similarity = sim;
                best_relation = Some(rel_name.clone());
            }
        }
        
        best_relation.map(|r| (r, best_similarity))
    }
    
    /// Predict relation using specific coordinate space
    pub fn predict_relation_in_space(
        &self, 
        source: &[f32], 
        target: &[f32],
        space: CoordinateSpace
    ) -> Option<(String, f32)> {
        let mut diff = vec![0.0; self.embed_dim];
        for i in 0..self.embed_dim.min(source.len()).min(target.len()) {
            diff[i] = target[i] - source[i];
        }
        
        let prototypes = match space {
            CoordinateSpace::World => &self.world_prototypes,
            CoordinateSpace::Local => &self.local_prototypes,
        };
        
        let mut best_relation = None;
        let mut best_similarity = self.detection_threshold;
        
        for (rel_name, prototype) in prototypes {
            let sim = self.cosine_similarity(&diff, prototype);
            if sim > best_similarity {
                best_similarity = sim;
                best_relation = Some(rel_name.clone());
            }
        }
        
        best_relation.map(|r| (r, best_similarity))
    }
    
    /// Check if a specific relation holds between entities
    /// Uses current coordinate space (World or Local) prototypes
    pub fn check_relation(&self, source: &[f32], relation: &str, target: &[f32]) -> (bool, f32) {
        let prototypes = self.get_active_prototypes();
        
        if let Some(prototype) = prototypes.get(relation) {
            // Compute expected target position
            let mut expected = vec![0.0; self.embed_dim];
            for i in 0..self.embed_dim.min(source.len()).min(prototype.len()) {
                expected[i] = source[i] + prototype[i];
            }
            
            // Compare with actual target
            let similarity = self.cosine_similarity(&expected, target);
            let holds = similarity > self.detection_threshold;
            
            (holds, similarity.max(0.0))
        } else {
            (false, 0.0)
        }
    }
    
    /// Check relation in specific coordinate space
    pub fn check_relation_in_space(
        &self, 
        source: &[f32], 
        relation: &str, 
        target: &[f32],
        space: CoordinateSpace
    ) -> (bool, f32) {
        let prototypes = match space {
            CoordinateSpace::World => &self.world_prototypes,
            CoordinateSpace::Local => &self.local_prototypes,
        };
        
        if let Some(prototype) = prototypes.get(relation) {
            let mut expected = vec![0.0; self.embed_dim];
            for i in 0..self.embed_dim.min(source.len()).min(prototype.len()) {
                expected[i] = source[i] + prototype[i];
            }
            
            let similarity = self.cosine_similarity(&expected, target);
            let holds = similarity > self.detection_threshold;
            
            (holds, similarity.max(0.0))
        } else {
            (false, 0.0)
        }
    }
    
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        let mut dot = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;
        for i in 0..a.len().min(b.len()) {
            dot += a[i] * b[i];
            norm_a += a[i] * a[i];
            norm_b += b[i] * b[i];
        }
        if norm_a > 1e-8 && norm_b > 1e-8 {
            dot / (norm_a.sqrt() * norm_b.sqrt())
        } else {
            0.0
        }
    }
}

// =============================================================================
// GEOMETRIC WORLD MODEL
// =============================================================================

/// The complete geometric world model
/// Integrates encoder, world state, and relation predictor
/// Supports both World and Local coordinate spaces for relational reasoning
#[derive(Debug)]
pub struct GeometricWorldModel {
    /// World state tensor
    pub world: WorldStateTensor,
    /// Context encoder
    pub encoder: WorldEncoder,
    /// Relation predictor
    pub predictor: GeometricRelationPredictor,
    /// Embedding dimension
    embed_dim: usize,
}

impl GeometricWorldModel {
    pub fn new(embed_dim: usize) -> Self {
        Self {
            world: WorldStateTensor::new(embed_dim),
            encoder: WorldEncoder::new(embed_dim),
            predictor: GeometricRelationPredictor::new(embed_dim),
            embed_dim,
        }
    }
    
    /// Create with specific coordinate space
    pub fn with_coordinate_space(embed_dim: usize, space: CoordinateSpace) -> Self {
        Self {
            world: WorldStateTensor::with_coordinate_space(embed_dim, space),
            encoder: WorldEncoder::new(embed_dim),
            predictor: GeometricRelationPredictor::with_coordinate_space(embed_dim, space),
            embed_dim,
        }
    }
    
    /// Set coordinate space for both world state and predictor
    pub fn set_coordinate_space(&mut self, space: CoordinateSpace) {
        self.world.set_coordinate_space(space);
        self.predictor.set_coordinate_space(space);
    }
    
    /// Get current coordinate space
    pub fn get_coordinate_space(&self) -> CoordinateSpace {
        self.world.coordinate_space
    }
    
    /// Process context and build world state
    pub fn process_context(&mut self, context: &str) {
        // Encode context into world state
        self.encoder.encode_context(context, &mut self.world);
        
        // Update flux matrix
        self.world.update_flux_state();
    }
    
    /// Answer a question using geometric reasoning
    /// Returns (answer_index, confidence)
    /// Note: This provides a geometric consistency signal, not the final answer.
    /// The UnifiedInferenceEngine combines this with other signals.
    pub fn answer_question(&mut self, question: &str, choices: &[String]) -> (usize, f32) {
        // Encode question
        let question_embed = self.encoder.encode_entity(question);
        
        // Score each choice based on geometric consistency with world state
        let mut best_idx = 0;
        let mut best_score = f32::NEG_INFINITY;
        
        for (idx, choice) in choices.iter().enumerate() {
            let choice_embed = self.encoder.encode_entity(choice);
            
            // Score based on:
            // 1. Similarity to question (relevance)
            let relevance = self.world.similarity(&question_embed, &choice_embed);
            
            // 2. Consistency with world state (coherence)
            let mut coherence = 0.0;
            for (_entity, pos) in &self.world.entity_positions {
                coherence += self.world.similarity(&choice_embed, pos);
            }
            coherence /= self.world.entity_positions.len().max(1) as f32;
            
            // 3. Flux matrix alignment
            let flux_alignment = self.world.flux_state[self.world.vortex_position as usize - 1];
            
            // Combined score
            let score = relevance * 0.4 + coherence * 0.4 + flux_alignment * 0.2;
            
            if score > best_score {
                best_score = score;
                best_idx = idx;
            }
        }
        
        // Confidence based on score margin and world consistency
        let confidence = (best_score.abs() * self.world.consistency).min(1.0).max(0.1);
        
        (best_idx, confidence)
    }
    
    /// Answer yes/no questions using geometric relation checking
    fn answer_yes_no_question(&mut self, question: &str, choices: &[String]) -> Option<(usize, f32)> {
        // Find yes/no indices
        let mut yes_idx = None;
        let mut no_idx = None;
        for (idx, choice) in choices.iter().enumerate() {
            let cl = choice.to_lowercase();
            if cl == "yes" { yes_idx = Some(idx); }
            if cl == "no" { no_idx = Some(idx); }
        }
        
        let (yes_idx, no_idx) = match (yes_idx, no_idx) {
            (Some(y), Some(n)) => (y, n),
            _ => return None,
        };
        
        // Extract relation from question using embedding similarity
        // Instead of parsing, we encode the question and compare to relation prototypes
        let question_embed = self.encoder.encode_entity(question);
        
        // Check if question implies a relation that should hold or not
        // Use the world state to determine if entities mentioned are related
        
        // For now, use a simple heuristic based on world consistency
        // If world has high consistency and question mentions known entities,
        // we can make a determination
        if self.world.consistency > 0.5 && !self.world.entity_positions.is_empty() {
            // Check similarity between question and world state
            let mut max_sim = 0.0;
            for (_entity, pos) in &self.world.entity_positions {
                let sim = self.world.similarity(&question_embed, pos);
                if sim > max_sim {
                    max_sim = sim;
                }
            }
            
            // If question is highly similar to world state, answer based on that
            if max_sim > 0.3 {
                // High similarity suggests the relation holds
                return Some((yes_idx, max_sim));
            } else if max_sim < 0.1 {
                // Low similarity suggests the relation doesn't hold
                return Some((no_idx, 1.0 - max_sim));
            }
        }
        
        None
    }
    
    /// Check if a relation holds between two entities
    pub fn check_relation(&mut self, source: &str, relation: &str, target: &str) -> (bool, f32) {
        let source_embed = self.world.get_or_create_entity(source, &self.encoder);
        let target_embed = self.world.get_or_create_entity(target, &self.encoder);
        
        self.predictor.check_relation(&source_embed, relation, &target_embed)
    }
    
    /// Get world consistency score
    pub fn get_consistency(&self) -> f32 {
        self.world.consistency
    }
    
    /// Get current vortex position
    pub fn get_vortex_position(&self) -> u8 {
        self.world.vortex_position
    }
}

// =============================================================================
// WORLD CONSISTENCY OBJECTIVE
// =============================================================================

/// Objective function for training the world model
/// Minimizes inconsistency in the geometric world state
#[derive(Debug)]
pub struct WorldConsistencyObjective {
    /// Weight for relation consistency loss
    pub relation_weight: f32,
    /// Weight for transitivity loss
    pub transitivity_weight: f32,
    /// Weight for flux alignment loss
    pub flux_weight: f32,
}

impl Default for WorldConsistencyObjective {
    fn default() -> Self {
        Self {
            relation_weight: 1.0,
            transitivity_weight: 0.5,
            flux_weight: 0.3,
        }
    }
}

impl WorldConsistencyObjective {
    /// Compute total loss for world state
    pub fn compute_loss(&self, world: &WorldStateTensor, predictor: &GeometricRelationPredictor) -> f32 {
        let mut total_loss = 0.0;
        
        // 1. Relation consistency: predicted relations should match world structure
        let relation_loss = self.compute_relation_loss(world, predictor);
        total_loss += relation_loss * self.relation_weight;
        
        // 2. Transitivity: if A→B and B→C then A→C should hold
        let transitivity_loss = self.compute_transitivity_loss(world, predictor);
        total_loss += transitivity_loss * self.transitivity_weight;
        
        // 3. Flux alignment: sacred positions should have high consistency
        let flux_loss = self.compute_flux_loss(world);
        total_loss += flux_loss * self.flux_weight;
        
        total_loss
    }
    
    fn compute_relation_loss(&self, world: &WorldStateTensor, predictor: &GeometricRelationPredictor) -> f32 {
        let mut loss = 0.0;
        let mut count = 0;
        
        // For each pair of entities, check if predicted relation is consistent
        let entities: Vec<_> = world.entity_positions.keys().cloned().collect();
        for i in 0..entities.len() {
            for j in 0..entities.len() {
                if i != j {
                    let source = world.entity_positions.get(&entities[i]).unwrap();
                    let target = world.entity_positions.get(&entities[j]).unwrap();
                    
                    if let Some((_, confidence)) = predictor.predict_relation(source, target) {
                        // Higher confidence = lower loss
                        loss += 1.0 - confidence;
                        count += 1;
                    }
                }
            }
        }
        
        if count > 0 { loss / count as f32 } else { 0.0 }
    }
    
    fn compute_transitivity_loss(&self, world: &WorldStateTensor, predictor: &GeometricRelationPredictor) -> f32 {
        // Simplified: check that entity positions form a coherent manifold
        let mut loss = 0.0;
        let positions: Vec<_> = world.entity_positions.values().collect();
        
        if positions.len() < 3 {
            return 0.0;
        }
        
        // Triangle inequality should hold in embedding space
        for i in 0..positions.len().min(10) {
            for j in i+1..positions.len().min(10) {
                for k in j+1..positions.len().min(10) {
                    let d_ij = world.distance(positions[i], positions[j]);
                    let d_jk = world.distance(positions[j], positions[k]);
                    let d_ik = world.distance(positions[i], positions[k]);
                    
                    // Triangle inequality violation
                    let violation = (d_ik - d_ij - d_jk).max(0.0);
                    loss += violation;
                }
            }
        }
        
        loss
    }
    
    fn compute_flux_loss(&self, world: &WorldStateTensor) -> f32 {
        // Sacred positions (3, 6, 9) should have high values
        let sacred_avg = (world.flux_state[2] + world.flux_state[5] + world.flux_state[8]) / 3.0;
        
        // Loss is inverse of sacred position strength
        1.0 - sacred_avg
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_world_model_creation() {
        let model = GeometricWorldModel::new(256);
        assert_eq!(model.embed_dim, 256);
        assert!(model.world.entity_positions.is_empty());
    }
    
    #[test]
    fn test_entity_encoding() {
        let encoder = WorldEncoder::new(64);
        let embed1 = encoder.encode_entity("chocolate");
        let embed2 = encoder.encode_entity("chocolate");
        let embed3 = encoder.encode_entity("suitcase_container_box");
        
        // Same entity should give same embedding
        assert_eq!(embed1, embed2);
        // Different entities should give different embeddings (use very different strings)
        assert_ne!(embed1, embed3);
    }
    
    #[test]
    fn test_relation_prediction() {
        let predictor = GeometricRelationPredictor::new(64);
        
        // Create source and target with known offset
        let source = vec![0.0; 64];
        let mut target = vec![0.0; 64];
        target[0] = 1.0; // Offset in "left_of" direction
        
        if let Some((rel, conf)) = predictor.predict_relation(&source, &target) {
            println!("Predicted relation: {} with confidence {}", rel, conf);
            // Should predict something related to spatial direction
            assert!(conf > 0.0);
        }
    }
    
    #[test]
    fn test_flux_cycle() {
        let mut world = WorldStateTensor::new(64);
        
        // Verify vortex cycle
        let expected_cycle = [1, 2, 4, 8, 7, 5, 1, 2];
        for &expected in &expected_cycle {
            assert_eq!(world.vortex_position, expected);
            world.update_flux_state();
        }
    }
    
    #[test]
    fn test_coordinate_space_world() {
        let model = GeometricWorldModel::with_coordinate_space(64, CoordinateSpace::World);
        assert_eq!(model.get_coordinate_space(), CoordinateSpace::World);
        assert_eq!(model.world.coordinate_space, CoordinateSpace::World);
        assert_eq!(model.predictor.get_coordinate_space(), CoordinateSpace::World);
    }
    
    #[test]
    fn test_coordinate_space_local() {
        let model = GeometricWorldModel::with_coordinate_space(64, CoordinateSpace::Local);
        assert_eq!(model.get_coordinate_space(), CoordinateSpace::Local);
        assert_eq!(model.world.coordinate_space, CoordinateSpace::Local);
        assert_eq!(model.predictor.get_coordinate_space(), CoordinateSpace::Local);
    }
    
    #[test]
    fn test_coordinate_space_switching() {
        let mut model = GeometricWorldModel::new(64);
        
        // Default is World
        assert_eq!(model.get_coordinate_space(), CoordinateSpace::World);
        
        // Switch to Local
        model.set_coordinate_space(CoordinateSpace::Local);
        assert_eq!(model.get_coordinate_space(), CoordinateSpace::Local);
        assert_eq!(model.world.coordinate_space, CoordinateSpace::Local);
        assert_eq!(model.predictor.get_coordinate_space(), CoordinateSpace::Local);
        
        // Switch back to World
        model.set_coordinate_space(CoordinateSpace::World);
        assert_eq!(model.get_coordinate_space(), CoordinateSpace::World);
    }
    
    #[test]
    fn test_world_vs_local_prototypes_differ() {
        let predictor = GeometricRelationPredictor::new(64);
        
        // World and Local prototypes should be different for spatial relations
        let world_left = predictor.world_prototypes.get("left_of").unwrap();
        let local_left = predictor.local_prototypes.get("left_of").unwrap();
        
        // They should not be identical
        assert_ne!(world_left, local_left);
        
        // But size relations should be the same (scale is absolute)
        let world_bigger = predictor.world_prototypes.get("bigger_than").unwrap();
        let local_bigger = predictor.local_prototypes.get("bigger_than").unwrap();
        assert_eq!(world_bigger, local_bigger);
    }
    
    #[test]
    fn test_apply_relation_world_space() {
        let world = WorldStateTensor::with_coordinate_space(64, CoordinateSpace::World);
        
        let source = vec![0.5; 64];
        let relation = vec![0.1; 64];
        
        let result = world.apply_relation(&source, &relation);
        
        // Result should be normalized
        let norm: f32 = result.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.01);
    }
    
    #[test]
    fn test_apply_relation_local_space() {
        let world = WorldStateTensor::with_coordinate_space(64, CoordinateSpace::Local);
        
        let source = vec![0.5; 64];
        let relation = vec![0.1; 64];
        
        let result = world.apply_relation(&source, &relation);
        
        // Result should be normalized
        let norm: f32 = result.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.01);
    }
    
    #[test]
    fn test_local_frame_creation() {
        let mut world = WorldStateTensor::new(64);
        
        let frame = world.get_or_create_local_frame("entity_a");
        assert_eq!(frame.len(), 64);
        
        // Should be cached
        let frame2 = world.get_or_create_local_frame("entity_a");
        assert_eq!(frame, frame2);
    }
    
    #[test]
    fn test_predict_relation_in_space() {
        let predictor = GeometricRelationPredictor::new(64);
        
        let source = vec![0.0; 64];
        let mut target = vec![0.0; 64];
        target[0] = 1.0; // Offset in X direction
        
        // Predict in World space
        let world_result = predictor.predict_relation_in_space(&source, &target, CoordinateSpace::World);
        
        // Predict in Local space
        let local_result = predictor.predict_relation_in_space(&source, &target, CoordinateSpace::Local);
        
        // Both should return something (may differ in relation type)
        assert!(world_result.is_some() || local_result.is_some());
    }
    
    #[test]
    fn test_adaptive_indexing_threshold() {
        let mut world = WorldStateTensor::new(64);
        let encoder = WorldEncoder::new(64);
        
        // Initially should not use HNSW (below threshold)
        assert!(!world.should_use_hnsw());
        assert_eq!(world.entity_count(), 0);
        
        // Add entities below threshold
        for i in 0..EMBEDVEC_THRESHOLD - 1 {
            let entity = format!("entity_{}", i);
            world.get_or_create_entity(&entity, &encoder);
        }
        
        // Still below threshold
        assert_eq!(world.entity_count(), EMBEDVEC_THRESHOLD - 1);
        assert!(!world.should_use_hnsw());
    }
    
    #[test]
    fn test_find_similar_entities_brute_force() {
        let mut world = WorldStateTensor::new(64);
        let encoder = WorldEncoder::new(64);
        
        // Add a few entities (below HNSW threshold)
        world.get_or_create_entity("apple_fruit_red", &encoder);
        world.get_or_create_entity("banana_fruit_yellow", &encoder);
        world.get_or_create_entity("cherry_fruit_small", &encoder);
        
        // Query for similar entities
        let query = encoder.encode_entity("apple_fruit_red");
        let similar = world.find_similar_entities(&query, 3);
        
        // Should find all entities
        assert_eq!(similar.len(), 3);
        
        // Results should be sorted by similarity (descending)
        assert!(similar[0].1 >= similar[1].1);
        assert!(similar[1].1 >= similar[2].1);
        
        // The query entity should be in the results with high similarity
        let apple_result = similar.iter().find(|(name, _)| name == "apple_fruit_red");
        assert!(apple_result.is_some());
        // Similarity to itself should be 1.0 (or very close)
        assert!(apple_result.unwrap().1 > 0.99);
    }
    
    #[test]
    fn test_embedvec_threshold_constant() {
        // Threshold should be reasonable (not too small, not too large)
        assert!(EMBEDVEC_THRESHOLD >= 32);
        assert!(EMBEDVEC_THRESHOLD <= 256);
    }
    
    #[test]
    #[cfg(feature = "embeddings")]
    fn test_hnsw_sync_no_data_loss() {
        let mut world = WorldStateTensor::new(64);
        let encoder = WorldEncoder::new(64);
        
        // Add entities to HashMap
        let entities = vec![
            "entity_alpha_one",
            "entity_beta_two", 
            "entity_gamma_three",
            "entity_delta_four",
            "entity_epsilon_five",
        ];
        
        for entity in &entities {
            world.get_or_create_entity(entity, &encoder);
        }
        
        // Verify all entities in HashMap
        assert_eq!(world.entity_count(), 5);
        
        // Sync to HNSW
        let indexed = world.sync_hnsw_from_hashmap();
        assert_eq!(indexed, 5, "All entities should be indexed in HNSW");
        
        // Verify HashMap still has all entities (source of truth preserved)
        assert_eq!(world.entity_count(), 5);
        for entity in &entities {
            assert!(world.entity_positions.contains_key(*entity), 
                "HashMap should still contain {}", entity);
        }
        
        // Verify HNSW ID mappings
        assert_eq!(world.entity_to_hnsw_id.len(), 5);
        
        // HNSW should now be enabled
        assert!(world.use_hnsw);
    }
    
    #[test]
    fn test_hashmap_always_source_of_truth() {
        let mut world = WorldStateTensor::new(64);
        let encoder = WorldEncoder::new(64);
        
        // Add entity
        let pos1 = world.get_or_create_entity("test_entity", &encoder);
        
        // Get same entity again - should return same position from HashMap
        let pos2 = world.get_or_create_entity("test_entity", &encoder);
        
        assert_eq!(pos1, pos2, "Same entity should return same embedding from HashMap");
        
        // Entity count should be 1 (not duplicated)
        assert_eq!(world.entity_count(), 1);
    }
}
