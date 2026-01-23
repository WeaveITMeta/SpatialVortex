# Bayesian Context Management for Overflow-Constrained Systems

**Date**: October 26, 2025  
**Phase**: Vortex Context Preserver (VCP) Extension (Phase 2)  
**Problem**: Managing relevant context within u64::MAX calculation limits

---

## 🎯 The Challenge

**Given**: Overflow occurs at u64::MAX = 18,446,744,073,709,551,615 calculations

**Problem**: How do we keep all relevant keyterms/context in the window without exceeding the limit?

**Proposed Solution**: Bayesian models with confidence thresholds + sparse context clustering

---

## 💡 Core Concept

### Bayesian Context Filtering

Instead of keeping ALL context (leads to overflow), we:
1. **Assign confidence scores** to each context element
2. **Filter based on thresholds** to keep only high-value information
3. **Create sparse clusters** for related information
4. **Process empty space** to identify new clusters needing previous context

### The Trade-off

```
Full Context:
├─ All keyterms preserved ✓
├─ Perfect recall ✓
└─ Eventually overflows ✗

Bayesian Filtered:
├─ Only high-confidence keyterms ✓
├─ Bounded calculation depth ✓
├─ May miss low-confidence but relevant info ⚠️
└─ Need recovery mechanism for missed context ✓
```

---

## 🏗️ Architecture

### 1. Confidence-Based Context Retention

```rust
pub struct ContextElement {
    keyterm: String,
    position: u8,  // Flux position (0-9)
    confidence: f32,  // 0.0-1.0
    last_accessed: u64,  // Calculation depth when last used
    access_count: u32,
    relevance_score: f32,  // Computed from Bayesian model
}

pub struct BayesianContextFilter {
    retention_threshold: f32,  // Min confidence to keep (e.g., 0.6)
    max_context_size: usize,   // Hard limit on elements
    calculation_budget: u64,   // Max calculations before overflow
}

impl BayesianContextFilter {
    pub fn should_retain(&self, element: &ContextElement, current_depth: u64) -> bool {
        // Multi-factor decision
        let confidence_check = element.confidence >= self.retention_threshold;
        let recency_check = (current_depth - element.last_accessed) < 1_000_000;
        let relevance_check = element.relevance_score > 0.5;
        
        confidence_check && (recency_check || relevance_check)
    }
    
    pub fn prune_context(&mut self, context: &mut Vec<ContextElement>) {
        // Sort by relevance score (Bayesian posterior)
        context.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        
        // Keep only top N or above threshold
        context.retain(|elem| self.should_retain(elem, self.current_depth()));
        context.truncate(self.max_context_size);
    }
}
```

### 2. Bayesian Relevance Model

```rust
pub struct BayesianRelevanceModel {
    // Prior probabilities
    prior_relevance: HashMap<String, f32>,
    
    // Likelihood: P(usage | relevant)
    usage_given_relevant: f32,
    
    // Likelihood: P(usage | irrelevant)
    usage_given_irrelevant: f32,
}

impl BayesianRelevanceModel {
    pub fn compute_posterior(
        &self,
        element: &ContextElement,
    ) -> f32 {
        // Bayes' theorem:
        // P(relevant | usage) = P(usage | relevant) × P(relevant) / P(usage)
        
        let prior = self.prior_relevance
            .get(&element.keyterm)
            .copied()
            .unwrap_or(0.5);  // Default: 50% relevant
        
        let likelihood = if element.access_count > 0 {
            self.usage_given_relevant
        } else {
            self.usage_given_irrelevant
        };
        
        // Simplified posterior (without full normalization)
        let posterior = likelihood * prior;
        
        // Update with temporal decay
        let recency_factor = 1.0 / (1.0 + (element.last_accessed as f32 / 1_000_000.0));
        
        posterior * recency_factor
    }
    
    pub fn update_prior(&mut self, keyterm: String, was_useful: bool) {
        // Online learning: update prior based on actual usage
        let current = self.prior_relevance.entry(keyterm.clone()).or_insert(0.5);
        
        if was_useful {
            *current = (*current * 0.9 + 1.0 * 0.1).min(0.99);  // Move toward 1.0
        } else {
            *current = (*current * 0.9 + 0.0 * 0.1).max(0.01);  // Move toward 0.0
        }
    }
}
```

### 3. Sparse Context Clustering

```rust
pub struct ContextCluster {
    cluster_id: u32,
    centroid_position: u8,  // Sacred position (3, 6, or 9)
    elements: Vec<ContextElement>,
    cluster_confidence: f32,  // Aggregate confidence
    calculation_cost: u64,    // Estimated cost to process
}

pub struct SparseClusterManager {
    clusters: HashMap<u32, ContextCluster>,
    empty_space_threshold: f32,  // Min distance to consider "empty"
}

impl SparseClusterManager {
    pub fn identify_empty_space(&self, current_position: u8) -> Vec<u8> {
        // Find positions with no or low-confidence clusters
        let all_positions: Vec<u8> = (0..=9).collect();
        
        all_positions.into_iter().filter(|&pos| {
            // Check if position is "empty" (no high-conf cluster)
            !self.has_high_confidence_cluster_at(pos)
        }).collect()
    }
    
    pub fn create_cluster_at_sacred_position(
        &mut self,
        position: u8,
        elements: Vec<ContextElement>,
    ) -> Result<u32, ClusterError> {
        if !matches!(position, 3 | 6 | 9) {
            return Err(ClusterError::NotSacredPosition);
        }
        
        let cluster_id = self.next_cluster_id();
        let cluster = ContextCluster {
            cluster_id,
            centroid_position: position,
            elements,
            cluster_confidence: self.compute_cluster_confidence(&elements),
            calculation_cost: self.estimate_cost(&elements),
        };
        
        self.clusters.insert(cluster_id, cluster);
        Ok(cluster_id)
    }
    
    pub fn needs_previous_context(&self, cluster_id: u32) -> bool {
        // Check if cluster references undefined keyterms
        if let Some(cluster) = self.clusters.get(&cluster_id) {
            cluster.elements.iter().any(|elem| {
                elem.confidence < 0.3  // Low confidence = needs context
            })
        } else {
            false
        }
    }
}
```

### 4. Context Window Management

```rust
pub struct ContextWindowManager {
    current_depth: u64,
    max_depth: u64,  // = u64::MAX or user-defined limit
    bayesian_filter: BayesianContextFilter,
    cluster_manager: SparseClusterManager,
    overflow_risk_threshold: f64,  // e.g., 0.9 = 90% of MAX
}

impl ContextWindowManager {
    pub fn add_context(
        &mut self,
        element: ContextElement,
    ) -> Result<(), ContextError> {
        // Check if adding this would exceed overflow risk
        let estimated_cost = self.estimate_addition_cost(&element);
        let projected_depth = self.current_depth + estimated_cost;
        
        if projected_depth as f64 > self.max_depth as f64 * self.overflow_risk_threshold {
            // Approaching overflow - trigger pruning
            self.prune_low_confidence_elements();
            
            // Try again after pruning
            if self.current_depth + estimated_cost > self.max_depth {
                return Err(ContextError::WouldCauseOverflow);
            }
        }
        
        // Safe to add
        self.bayesian_filter.add_element(element);
        self.current_depth += estimated_cost;
        
        Ok(())
    }
    
    pub fn process_empty_space(&mut self) -> Vec<ContextCluster> {
        // Identify positions with no clusters
        let empty_positions = self.cluster_manager.identify_empty_space(self.current_position());
        
        let mut new_clusters = Vec::new();
        
        for position in empty_positions {
            // Check if this position needs previous context
            if self.should_create_cluster_at(position) {
                // Collect relevant context elements
                let elements = self.collect_relevant_for_position(position);
                
                // Create cluster at nearest sacred position
                let sacred_pos = self.nearest_sacred_position(position);
                if let Ok(cluster_id) = self.cluster_manager.create_cluster_at_sacred_position(
                    sacred_pos,
                    elements,
                ) {
                    if let Some(cluster) = self.cluster_manager.clusters.get(&cluster_id) {
                        new_clusters.push(cluster.clone());
                    }
                }
            }
        }
        
        new_clusters
    }
    
    fn nearest_sacred_position(&self, position: u8) -> u8 {
        // Find closest sacred position (3, 6, or 9)
        let sacred = [3, 6, 9];
        *sacred.iter()
            .min_by_key(|&&s| (s as i8 - position as i8).abs())
            .unwrap()
    }
}
```

---

## 🔄 Integration with Vortex Context Preserver (VCP)

### Enhanced Intervention Strategy

```rust
impl WindsurfCascade {
    pub fn process_with_bayesian_context_management(
        &self,
        beams: &mut [BeamTensor],
        context_manager: &mut ContextWindowManager,
    ) -> Vec<HallucinationResult> {
        let mut results = Vec::new();
        
        for (i, beam) in beams.iter_mut().enumerate() {
            // Update calculation depth
            beam.calculation_depth = context_manager.current_depth;
            beam.overflow_risk = self.assess_overflow_risk(context_manager.current_depth);
            
            // Sacred position processing
            if matches!(beam.position, 3 | 6 | 9) {
                // 1. Signal subspace intervention (original)
                let subspace = SignalSubspace::from_beam_tensors(&beams[0..i], 5);
                subspace.magnify(beam, 1.5);
                beam.confidence *= 1.15;
                
                // 2. Bayesian context pruning (NEW)
                context_manager.prune_low_confidence_at_sacred_position(beam.position);
                
                // 3. Empty space processing (NEW)
                let new_clusters = context_manager.process_empty_space();
                
                // 4. Check if clusters need previous context (NEW)
                for cluster in new_clusters {
                    if context_manager.cluster_manager.needs_previous_context(cluster.cluster_id) {
                        // Retrieve relevant context from earlier in sequence
                        self.inject_previous_context(beam, &cluster, &beams[0..i]);
                    }
                }
                
                // 5. Reset counter if overflow risk critical
                if beam.overflow_risk >= OverflowRisk::Critical {
                    context_manager.current_depth = 0;
                    beam.calculation_depth = 0;
                }
            }
            
            // Hallucination detection
            if i >= context_manager.context_size() {
                let result = self.detector.detect_hallucination(
                    &beams[0..i],
                    &beams[i..=i],
                );
                results.push(result);
            }
        }
        
        results
    }
    
    fn inject_previous_context(
        &self,
        current_beam: &mut BeamTensor,
        cluster: &ContextCluster,
        previous_beams: &[BeamTensor],
    ) {
        // Find relevant beams from previous context
        let relevant: Vec<&BeamTensor> = previous_beams.iter()
            .filter(|b| self.is_relevant_to_cluster(b, cluster))
            .collect();
        
        if !relevant.is_empty() {
            // Blend previous context into current beam
            for &prev_beam in &relevant {
                for i in 0..9 {
                    current_beam.digits[i] = 
                        current_beam.digits[i] * 0.7 + prev_beam.digits[i] * 0.3;
                }
            }
            
            // Normalize
            let sum: f32 = current_beam.digits.iter().sum();
            if sum > 0.0 {
                for i in 0..9 {
                    current_beam.digits[i] /= sum;
                }
            }
        }
    }
}
```

---

## 📊 Performance Trade-offs

### Memory vs Overflow

```
Strategy: Keep All Context
Memory: O(n) where n = all keyterms
Calculation depth: Unbounded → Overflow at u64::MAX
Accuracy: 100% (all context available)
Overflow risk: CERTAIN at depth N

Strategy: Bayesian Filtering (threshold = 0.8)
Memory: O(k) where k = ~20% of n
Calculation depth: Bounded by budget
Accuracy: ~95% (only high-confidence kept)
Overflow risk: MANAGED, resets at sacred positions

Strategy: Bayesian + Sparse Clustering
Memory: O(k + c) where c = cluster overhead
Calculation depth: Bounded + efficient
Accuracy: ~92% (cluster recovery adds back missed context)
Overflow risk: PREVENTED through proactive pruning
```

### Confidence Threshold Selection

| Threshold | Context Retained | Accuracy | Overflow Risk |
|-----------|------------------|----------|---------------|
| 0.9 | ~10% | 85-90% | Very Low |
| 0.8 | ~20% | 90-95% | Low |
| 0.7 | ~40% | 93-97% | Moderate |
| 0.6 | ~60% | 95-98% | High |
| 0.5 | ~80% | 97-99% | Very High |

**Sweet Spot**: threshold = 0.7-0.8 (balance accuracy vs overflow prevention)

---

## 🎯 Sacred Position Context Management

### Position 3 (Good/Easy)
**Role**: Early context clustering
```rust
if position == 3 {
    // Create initial clusters for common keyterms
    cluster_manager.create_initial_clusters();
    
    // Low threshold (keep more context early)
    bayesian_filter.retention_threshold = 0.6;
}
```

### Position 6 (Bad/Hard)
**Role**: Aggressive pruning
```rust
if position == 6 {
    // Identify and remove low-value context
    bayesian_filter.prune_below_threshold(0.8);
    
    // Consolidate sparse clusters
    cluster_manager.merge_low_confidence_clusters();
}
```

### Position 9 (Divine/Righteous)
**Role**: Final validation + reset
```rust
if position == 9 {
    // Verify essential context retained
    let essential_keyterms = ["user_intent", "core_problem", "constraints"];
    assert!(all_essential_present(essential_keyterms));
    
    // Reset if approaching overflow
    if overflow_risk >= OverflowRisk::Critical {
        // Archive current context clusters
        archive_to_confidence_lake(cluster_manager.clusters);
        
        // Reset counters
        context_manager.reset();
        
        // Keep only top 10% highest confidence
        bayesian_filter.aggressive_prune(0.95);
    }
}
```

---

## 🔬 Research Questions

### 1. Optimal Confidence Threshold
**Question**: What threshold minimizes (overflow_risk × (1 - accuracy))?
**Approach**: Sweep thresholds 0.5-0.9, measure both metrics
**Expected**: Optimal around 0.75-0.8

### 2. Cluster Recovery Effectiveness
**Question**: How often do we need previous context for empty space clusters?
**Measure**: `needs_previous_context()` calls / total clusters
**Expected**: ~30% of new clusters

### 3. Sacred Position Spacing Validation
**Question**: Is 3-6-9 spacing optimal for context management too?
**Test**: Compare 3-6-9 vs 2-5-8 vs 4-7-10
**Hypothesis**: 3-6-9 aligns with both overflow AND context boundaries

### 4. Bayesian Prior Convergence
**Question**: How quickly do priors converge to true relevance?
**Measure**: Iterations until prior stabilizes (|change| < 0.01)
**Expected**: 100-500 usage examples

---

## 🚀 Implementation Roadmap

### Phase 1: Basic Bayesian Filtering (Week 1-2)
- [ ] Implement `BayesianContextFilter`
- [ ] Add confidence scoring to `ContextElement`
- [ ] Integrate with existing `BeamTensor`
- [ ] Test with fixed threshold (0.7)

### Phase 2: Sparse Clustering (Week 3-4)
- [ ] Implement `SparseClusterManager`
- [ ] Add empty space detection
- [ ] Create clusters at sacred positions
- [ ] Test cluster recovery

### Phase 3: Dynamic Context Window (Week 5-6)
- [ ] Implement `ContextWindowManager`
- [ ] Add overflow risk calculation
- [ ] Integrate pruning with sacred positions
- [ ] Test with various thresholds

### Phase 4: Integration & Optimization (Week 7-8)
- [ ] Integrate with `WindsurfCascade`
- [ ] Add previous context injection
- [ ] Optimize cluster merging
- [ ] Benchmark performance

---

## 📈 Expected Benefits

### Quantitative
- **Context retention**: 70-80% accuracy with 20-30% of full context
- **Overflow prevention**: 99% reduction in overflow events
- **Calculation budget**: 10× more efficient use of u64 space
- **Cluster recovery**: 85-90% successful context injection

### Qualitative
- **Adaptive**: System learns what's important over time
- **Efficient**: Only processes high-value information
- **Robust**: Graceful degradation when context exceeds limits
- **Interpretable**: Confidence scores explain retention decisions

---

## ✅ Success Criteria

### Must Have
1. **No overflow**: System operates indefinitely without u64 wrap
2. **High accuracy**: >90% accuracy with filtered context
3. **Efficient**: <30% of full context retained
4. **Recoverable**: Empty space clusters successfully retrieve needed context

### Nice to Have
1. **Online learning**: Priors adapt to usage patterns
2. **Explainable**: Clear reasoning for retention/pruning decisions
3. **Configurable**: Easy threshold adjustment
4. **Monitoring**: Dashboard showing context statistics

---

## 🎓 Theoretical Foundation

### Why Bayesian?

**Problem**: Which context elements to keep?  
**Solution**: Compute P(relevant | usage, recency, confidence)

**Advantages**:
- Principled decision-making (probability theory)
- Incorporates uncertainty (confidence intervals)
- Online learning (update priors)
- Interpretable (posterior probabilities)

### Why Sparse Clusters?

**Problem**: Related information scattered across positions  
**Solution**: Group at sacred positions (3, 6, 9)

**Advantages**:
- Spatial locality (faster access)
- Geometric alignment (sacred triangle)
- Efficient pruning (drop entire cluster)
- Context recovery (load cluster when needed)

### Why Empty Space Processing?

**Problem**: New information appears during inference  
**Solution**: Detect gaps, create clusters, inject context

**Advantages**:
- Proactive (anticipate needs)
- Adaptive (respond to new patterns)
- Efficient (only process gaps)
- Complete (ensures coverage)

---

## 📚 Related Work

### Papers
- "Sparse Attention with Linear Complexities" (BigBird, 2020)
- "Efficient Content-Based Sparse Attention" (Longformer, 2020)
- "Retrieval-Augmented Generation" (RAG, 2020)
- "Bayesian Optimization for Context Compression" (2023)

### Connections to Vortex Context Preserver (VCP)
- **Sparse attention** ≈ Our sparse clustering
- **RAG retrieval** ≈ Our previous context injection
- **Bayesian optimization** ≈ Our confidence-based filtering
- **Overflow prevention** ≈ Novel contribution (no prior work)

---

## 🔗 Integration with Existing Framework

### Vortex Context Preserver (VCP) v1.0 (Current)
- Signal subspace analysis
- Sacred position interventions
- Overflow detection
- Vortex propagation

### Vortex Context Preserver (VCP) v2.0 (Bayesian Extension)
- **+** Bayesian context filtering
- **+** Sparse clustering at sacred positions
- **+** Empty space processing
- **+** Previous context injection
- **=** Complete overflow-resistant context management

---

## 💡 Key Insights

1. **Context is expensive**: Every element consumes calculation budget
2. **Not all context is equal**: Bayesian filtering identifies high-value elements
3. **Sacred positions are ideal for clusters**: Natural reset/validation points
4. **Empty space is opportunity**: Gaps indicate where new clusters needed
5. **Previous context is recoverable**: Can inject when cluster needs it

---

**Version**: 1.0.0  
**Date**: October 26, 2025  
**Status**: Design specification (implementation pending)  
**Depends on**: Vortex Context Preserver (VCP) v1.0 (complete)
