# Vector Field Consensus System

## Overview

The **Vector Field Consensus** system transforms multi-model LLM responses from flat text into a rich geometric representation in ELP (Ethos-Logos-Pathos) vector space. This enables confidence-weighted aggregation with diversity bonuses, producing higher-quality consensus than simple majority voting or averaging.

## Architecture

### Core Components

```
User Query
    ↓
[4 LLM Models] → Raw Text Responses
    ↓
[Response Vectors] → Map to ELP Space + Confidence Trajectories
    ↓
[Vector Field Aggregation]
    ├─ Filter upward confidence trends
    ├─ Calculate approach diversity
    ├─ Weighted centroid in ELP space
    └─ Aggregate field confidence
    ↓
[Vortex Synthesis] ← Reason from vector field metadata
    ↓
[Confidence Lake Storage] ← Store high-quality consensus
```

### 1. Response Vector (`ResponseVector`)

Each LLM response is mapped to a geometric representation:

```rust
pub struct ResponseVector {
    pub elp: ELPTensor,                    // (Ethos, Logos, Pathos) position
    pub flux_position: u8,                  // 1-9 (sacred positions: 3, 6, 9)
    pub confidence_trajectory: Vec<f32>,    // Confidence over time
    pub approach_type: ProblemSolvingType,  // Analytical/Creative/Ethical/...
    pub text: String,
    pub model_name: String,
    pub latency_ms: u64,
}
```

**Key Methods:**
- `classify_approach()`: Determine problem-solving type from ELP dominance
- `confidence_gradient()`: Calculate trend (positive = rising, negative = falling)
- `trend_weight()`: Weight response by confidence trend (1.0-1.5x for rising)

### 2. Problem-Solving Types

Responses are classified based on ELP channel dominance:

| Type | ELP Pattern | Description | Example Flux Position |
|------|-------------|-------------|----------------------|
| **Analytical** | Logos > 50% | Logical, systematic reasoning | 6 |
| **Creative** | Pathos > 50% | Intuitive, emotional reasoning | 5 |
| **Ethical** | Ethos > 50% | Principled, value-based reasoning | 3 |
| **Procedural** | Balanced | Structured, methodical | 9 |
| **Synthesizing** | Sacred positions | Multi-perspective integration | 3, 6, 9 |

### 3. Consensus Vector Field (`ConsensusVectorField`)

Aggregated geometric representation of all responses:

```rust
pub struct ConsensusVectorField {
    pub vectors: Vec<ResponseVector>,          // Filtered response vectors
    pub consensus_center: ELPTensor,           // Weighted centroid
    pub diversity_score: f32,                  // 0.0-1.0 (unique approaches / total)
    pub field_confidence: f32,                 // 0.0-1.0 (aggregated confidence)
    pub sacred_resonance: f32,                 // 0.0-1.0 (proximity to 3-6-9)
    pub timestamp: DateTime<Utc>,
}
```

**Aggregation Process:**

1. **Classify Approaches**: Determine problem-solving type for each vector
2. **Filter Trends**: Retain responses with gradient > -0.1 (upward/stable)
3. **Calculate Diversity**: `unique_types / total_responses`
4. **Weighted Centroid**:
   ```
   weight = trend_weight × base_confidence × (1 + diversity × 0.5)
   consensus_center = Σ(ELP_i × weight_i) / Σ(weight_i)
   ```
5. **Aggregate Confidence**: Trend-weighted mean of final confidences
6. **Sacred Resonance**: Measure proximity to sacred positions (3, 6, 9)

## Integration with Existing Systems

### 🏞️ Confidence Lake

**Problem Solved**: Previously, text-only LLM responses couldn't be stored in Confidence Lake because they lacked the required `StoredFluxMatrix` structure (ELP distributions, pitch curve, BeadTensor).

**Solution**: Vector fields convert naturally to rich memories:

```rust
impl ConsensusVectorField {
    pub fn to_stored_flux_matrix(&self, text: String) -> Result<StoredFluxMatrix> {
        StoredFluxMatrix {
            ethos_distribution: self.ethos_distribution_by_position(),  // [f32; 9]
            logos_distribution: self.logos_distribution_by_position(),  // [f32; 9]
            pathos_distribution: self.pathos_distribution_by_position(), // [f32; 9]
            pitch_curve: self.aggregate_confidence_curves(),             // Vec<f32>
            text,
            tensor: self.to_bead_tensor()?,                              // BeadTensor
            // ... metadata
        }
    }
}
```

**Storage Policy**:
- **Minimum Confidence**: 0.6 (default)
- **Minimum Diversity**: 0.5 (default)
- **Optional Sacred Resonance**: Can require high sacred resonance (>0.7)
- **Session Limit**: 100 per session (prevent overflow)

**Benefits**:
- ✅ Rich metadata (diversity, approach types, confidence trends)
- ✅ Higher quality memories (consensus-validated)
- ✅ Better retrieval (ELP-based semantic search)

### 📚 RAG System

**Two-Way Integration**:

1. **RAG → Consensus** (Augment Responses):
   ```rust
   // Retrieve context before querying models
   let rag_context = rag.retrieve_relevant(&query, 3).await?;
   let augmented_query = format!("Context:\n{}\n\nQuery: {}", rag_context, query);
   let responses = query_models(&augmented_query).await?;
   ```

2. **Consensus → RAG** (Use for Retrieval):
   ```rust
   // Use consensus ELP as embedding vector
   let embedding = consensus_field.as_embedding_vector();  // [ethos, logos, pathos, conf, div]
   let results = vector_store.query_by_vector(embedding, 5).await?;
   ```

**Feedback Loop**:
```
RAG → Better Consensus → Store in Lake → RAG Retrieves → Even Better Consensus
```

### 🧠 Background Learner

**Learn Optimal Consensus Patterns**:

```rust
impl BackgroundLearner {
    pub async fn learn_from_consensus_history(&mut self) -> Result<()> {
        // Retrieve past consensus fields from Confidence Lake
        let past_fields = self.confidence_lake.query_recent(100).await?;
        
        // Analyze:
        // 1. Which diversity scores → best outcomes?
        // 2. Which ELP regions → highest confidence?
        // 3. Which approach combinations work best?
        
        // Update consensus weights dynamically
        self.update_consensus_weights(optimal_diversity, high_conf_regions);
    }
}
```

**Self-Optimization**: System learns which consensus patterns lead to best results.

### 🔮 Predictive Processing

**Predict Expected Consensus**:

```rust
impl PredictiveProcessor {
    pub fn predict_consensus(&self, query: &str) -> PredictedConsensus {
        PredictedConsensus {
            expected_elp: self.world_model.expected_elp,
            expected_diversity: self.estimate_diversity(query),
            expected_confidence: self.world_model.expected_confidence,
        }
    }
    
    pub fn observe_consensus(&mut self, actual: &ConsensusVectorField) -> SurpriseSignal {
        let surprise = self.compute_surprise(predicted, actual);
        
        if surprise > 0.5 {
            self.update_world_model(actual);  // Learn from unexpected patterns
        }
    }
}
```

**Anomaly Detection**: High surprise = novel information → learning opportunity.

### 🪞 Meta-Cognition

**Quality Control**:

```rust
impl MetaCognitiveMonitor {
    pub fn analyze_consensus(&mut self, field: &ConsensusVectorField) -> ConsensusAnalysis {
        ConsensusAnalysis {
            coherence: self.measure_coherence(field),
            groupthink_risk: self.detect_groupthink(field),          // Low variance = echo chamber
            blind_spots: self.identify_blind_spots(field),           // Under-represented ELP channels
            confidence_calibration: self.check_calibration(field),
        }
    }
}
```

**Bias Detection**: Identifies when consensus is unreliable and triggers re-query.

## API Changes

### Enhanced VortexMessage

```json
{
  "text": "Synthesized response...",
  "confidence": 85.2,
  "flux_position": 6,
  "sources_used": ["llama3.2", "mixtral", "codellama", "mistral-nemo"],
  "consensus_diversity": 0.75,     // NEW
  "sacred_resonance": 0.82         // NEW
}
```

### Vortex Prompt Enhancement

**Before**:
```
Query all models → Vortex synthesizes from raw text
```

**After**:
```
Context from 4 models:
- Consensus ELP: (6.2, 7.8, 5.5)
- Diversity: 0.75 (approach variation)
- Field Confidence: 0.82
- Sacred Resonance: 0.68

Responses:
[llama3.2] ...
[mixtral] ...
[codellama] ...
[mistral-nemo] ...

Synthesize the STRONGEST reasoning path for:
{original_query}
```

**Benefit**: Vortex reasons from **geometric structure**, not just text.

## Example Scenario

**Query**: "How do I secure a REST API?"

### 1. Model Responses

| Model | Confidence Trajectory | ELP | Approach |
|-------|----------------------|-----|----------|
| llama3.2 | 0.6 → 0.8 ↗️ | (5, 9, 4) | Analytical (Logos) |
| mixtral | 0.9 → 0.85 ↘️ | (7, 6, 5) | Ethical (Ethos) |
| codellama | 0.5 → 0.75 ↗️ | (4, 7, 8) | Creative (Pathos) |
| mistral-nemo | 0.7 → 0.9 ↗️ | (8, 5, 6) | Ethical (Ethos) |

### 2. Vector Field Analysis

- **Diversity Score**: 3/4 = 0.75 (high diversity)
- **Filtered Vectors**: mixtral removed (falling confidence)
- **Consensus Center**: (5.7, 7.0, 6.0) ← Balanced with logos tilt
- **Field Confidence**: 0.82 (trend-weighted)
- **Sacred Resonance**: 0.65 (moderate)

### 3. Vortex Synthesis

Vortex receives:
- **Analytical approach** (llama3.2): OAuth2, JWT, rate limiting
- **Creative approach** (codellama): Zero-trust architecture, API gateways
- **Ethical approach** (mistral-nemo): Data privacy, GDPR compliance

**Result**: Synthesizes response covering **technical** (logos), **architectural** (pathos), and **compliance** (ethos) aspects.

### 4. Storage Decision

```
✅ confidence (0.82) ≥ 0.6
✅ diversity (0.75) ≥ 0.5
→ STORE in Confidence Lake
```

**Tags**: `consensus_v1`, `confidence_0.82`, `diversity_0.75`, `sacred_resonance_0.65`, `model_count_3`, `approach_analytical`

## Benefits

### 1. **Robustness**
- Models with falling confidence are downweighted
- Hallucinatory responses naturally filter out

### 2. **Diversity Exploitation**
- Novel approaches get bonuses (up to 1.5x weight)
- Prevents echo chamber effects
- Finds non-obvious solutions

### 3. **Geometric Grounding**
- Vortex reasons from **vector field structure**, not just text
- Sacred positions (3-6-9) act as "attractor basins"
- ELP space provides semantic continuity

### 4. **Confidence Calibration**
- Aggregated confidence more reliable than individual models
- Rising trends signal "model convergence" → trustworthy

### 5. **Rich Memory**
- Stored consensus fields have full ELP distributions
- Queryable by semantic similarity in vector space
- Context-aware retrieval

## Future Enhancements

### Phase 0 Completion (Current)
- ✅ Core vector consensus system
- ✅ Confidence Lake integration
- 🚧 RAG augmentation (partial)
- 🚧 Meta-cognition analysis (partial)

### Phase 1 (Next 2 weeks)
- [ ] Capture confidence trajectories during streaming
- [ ] RAG two-way integration (full)
- [ ] Meta-cognitive quality control triggers
- [ ] Background learner pattern optimization

### Phase 2 (4 weeks)
- [ ] Predictive consensus (predict expected patterns)
- [ ] Flux Transformer integration (multi-layer processing)
- [ ] Causal reasoning from consensus (→ Phase 1 AGI)
- [ ] Goal-oriented consensus synthesis

### Phase 3 (6+ weeks)
- [ ] Transfer learning across consensus domains
- [ ] Curiosity-driven exploration (seek novel consensus patterns)
- [ ] Self-improving consensus weights
- [ ] Meta-learning consensus strategies

## Testing

Run tests:
```bash
cargo test --lib vector_consensus
cargo test --lib consensus_storage
```

Key test cases:
- ✅ Confidence gradient calculation (rising vs falling)
- ✅ Approach classification (logos/ethos/pathos dominance)
- ✅ Diversity calculation (unique types / total)
- ✅ Consensus to StoredFluxMatrix conversion
- ✅ Storage policy thresholds

## Monitoring

**Logs to watch**:
```
🌀 Vector Consensus: 4 vectors, ELP=(6.2,7.8,5.5), conf=0.82, div=0.75, sacred=0.68
📊 ✅ Stored vector consensus in Confidence Lake: conf=0.82, div=0.75, sacred=0.68
📊 Consensus below storage thresholds: Stored 15/100 consensus fields (conf≥0.60, div≥0.50)
```

**Metrics**:
- Consensus storage rate (should be ~20-40% of requests)
- Average diversity score (target: >0.5)
- Sacred resonance (higher = more geometrically coherent)
- Confidence vs. diversity correlation

---

**Status**: ✅ Core system implemented | 🚧 Integration in progress | 🎯 Phase 0 complete by Week 3
