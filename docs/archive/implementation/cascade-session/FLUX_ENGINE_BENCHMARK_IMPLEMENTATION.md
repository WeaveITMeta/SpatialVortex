# FluxMatrixEngine Benchmark Implementation (Option 1)
## Flow-Centric Architecture

**Key Concepts**:
- **Flow Sequence**: The infinite loop [1,2,4,8,7,5] (doubling pattern)
- **Flow Position**: Where an object is in the sequence (0-5, then wraps)
- **Node Position**: Which digit (0-9) the object is at
- **Objects**: Things flowing through the vortex

---

## 🌊 **Core Data Structures**

```rust
use spatial_vortex::flux_matrix::FluxMatrixEngine;
use spatial_vortex::visualization::{FluxDataPoint, FluxLayout};
use spatial_vortex::models::ELPTensor;
use std::collections::HashMap;

/// Object flowing through the vortex
#[derive(Debug, Clone)]
struct FlowingObject {
    id: String,
    
    /// Position in the flow sequence (0-5, then wraps to 0)
    /// 0→[1], 1→[2], 2→[4], 3→[8], 4→[7], 5→[5], 6→[1]...
    flow_position: usize,
    
    /// Current node (0-9) based on flow position
    current_node: u8,
    
    /// How many complete cycles through the flow
    cycle_count: usize,
    
    /// ELP tensor state
    elp: ELPTensor,
    
    /// Confidence in current position
    confidence: f64,
}

impl FlowingObject {
    fn new(id: String, starting_node: u8, elp: ELPTensor) -> Self {
        let engine = FluxMatrixEngine::new();
        
        // Find initial flow position based on starting node
        let flow_position = engine.base_pattern
            .iter()
            .position(|&n| n == starting_node)
            .unwrap_or(0);
        
        Self {
            id,
            flow_position,
            current_node: starting_node,
            cycle_count: 0,
            elp,
            confidence: 1.0,
        }
    }
    
    /// Advance object to next position in flow sequence
    fn advance_in_flow(&mut self, engine: &FluxMatrixEngine) {
        // Move to next position in flow sequence
        self.flow_position += 1;
        
        // Wrap around when reaching end of sequence
        if self.flow_position >= engine.base_pattern.len() {
            self.flow_position = 0;
            self.cycle_count += 1;
        }
        
        // Update current node based on flow position
        self.current_node = engine.base_pattern[self.flow_position];
    }
    
    /// Check if object is at a sacred node (not in flow sequence)
    fn check_sacred_proximity(&self, engine: &FluxMatrixEngine) -> Option<u8> {
        // Find nearest sacred node (3, 6, 9)
        let distances: Vec<(u8, u8)> = engine.sacred_positions
            .iter()
            .map(|&sacred| {
                let diff = (self.current_node as i16 - sacred as i16).abs() as u8;
                (sacred, diff)
            })
            .collect();
        
        distances.iter()
            .min_by_key(|(_, dist)| dist)
            .map(|(sacred, _)| *sacred)
    }
}

/// Geometric reasoning task with flow awareness
#[derive(Debug)]
struct FlowAwareTask {
    task_id: String,
    
    /// Input geometric parameters
    angle: f64,
    distance: f64,
    complexity: f64,
    
    /// Predicted position from geometric reasoning
    predicted_node: u8,
    
    /// Gold/reference position
    gold_node: Option<u8>,
    
    /// Expected position in flow sequence
    expected_flow_position: Option<usize>,
}

/// Flow-aware evaluation metrics
#[derive(Debug, Default)]
struct FlowAwareEvaluation {
    // Basic metrics
    predicted_node: u8,
    gold_node: u8,
    exact_match: bool,
    
    // Spatial metrics (from gold_position implementation)
    positional_accuracy: f64,
    position_error: f64,
    
    // Flow sequence metrics (NEW)
    predicted_flow_position: usize,
    gold_flow_position: usize,
    flow_distance: usize,  // Steps apart in flow sequence
    
    // Node metrics (using FluxMatrixEngine)
    node_distance: u8,     // Digit reduction distance
    flux_path_similarity: f64,  // Sequence overlap
    
    // Sacred position metrics
    sacred_alignment: bool,
    predicted_sacred_proximity: Option<u8>,
    gold_sacred_proximity: Option<u8>,
    sacred_boost: f64,
    
    // Combined score
    flow_aware_accuracy: f64,
}

impl std::fmt::Display for FlowAwareEvaluation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Flow-Aware Evaluation ===")?;
        writeln!(f, "\n📍 Node Positions:")?;
        writeln!(f, "  Predicted Node: {}", self.predicted_node)?;
        writeln!(f, "  Gold Node: {}", self.gold_node)?;
        writeln!(f, "  Exact Match: {}", self.exact_match)?;
        
        writeln!(f, "\n🌊 Flow Sequence:")?;
        writeln!(f, "  Predicted Flow Position: {} in [1,2,4,8,7,5]", self.predicted_flow_position)?;
        writeln!(f, "  Gold Flow Position: {}", self.gold_flow_position)?;
        writeln!(f, "  Flow Distance: {} steps", self.flow_distance)?;
        
        writeln!(f, "\n📐 Spatial Metrics:")?;
        writeln!(f, "  Positional Accuracy: {:.1}%", self.positional_accuracy * 100.0)?;
        writeln!(f, "  Position Error: {:.2} units", self.position_error)?;
        
        writeln!(f, "\n🔢 Flux Matrix Metrics:")?;
        writeln!(f, "  Node Distance (digit reduction): {}", self.node_distance)?;
        writeln!(f, "  Flux Path Similarity: {:.1}%", self.flux_path_similarity * 100.0)?;
        
        writeln!(f, "\n✨ Sacred Positions:")?;
        writeln!(f, "  Sacred Alignment: {}", self.sacred_alignment)?;
        if let Some(pred_sacred) = self.predicted_sacred_proximity {
            writeln!(f, "  Predicted Near Sacred {}", pred_sacred)?;
        }
        if let Some(gold_sacred) = self.gold_sacred_proximity {
            writeln!(f, "  Gold Near Sacred {}", gold_sacred)?;
        }
        if self.sacred_boost > 0.0 {
            writeln!(f, "  Sacred Boost: +{:.1}%", self.sacred_boost * 100.0)?;
        }
        
        writeln!(f, "\n🎯 Final Score:")?;
        writeln!(f, "  Flow-Aware Accuracy: {:.1}%", self.flow_aware_accuracy * 100.0)?;
        
        Ok(())
    }
}
```

---

## 🔧 **Core Evaluation Function**

```rust
/// Evaluate geometric reasoning task with flow awareness
fn evaluate_with_flow_engine(
    task: &FlowAwareTask,
    layout: &FluxLayout,
) -> FlowAwareEvaluation {
    let engine = FluxMatrixEngine::new();
    
    // Get gold node or return default
    let gold_node = match task.gold_node {
        Some(gold) => gold,
        None => return FlowAwareEvaluation::default(),
    };
    
    // === 1. FLOW SEQUENCE ANALYSIS ===
    
    // Find positions in flow sequence
    let predicted_flow_position = engine.base_pattern
        .iter()
        .position(|&n| n == task.predicted_node)
        .unwrap_or(0);
    
    let gold_flow_position = engine.base_pattern
        .iter()
        .position(|&n| n == gold_node)
        .unwrap_or(0);
    
    // Calculate flow distance (steps apart in sequence)
    let flow_distance = if predicted_flow_position > gold_flow_position {
        predicted_flow_position - gold_flow_position
    } else {
        gold_flow_position - predicted_flow_position
    };
    
    // === 2. NODE DISTANCE (using digit reduction) ===
    
    let node_distance = engine.reduce_digits(
        (task.predicted_node as i16 - gold_node as i16).abs() as u64
    ) as u8;
    
    // === 3. FLUX PATH SIMILARITY ===
    
    let pred_sequence = engine.seed_to_flux_sequence(task.predicted_node as u64);
    let gold_sequence = engine.seed_to_flux_sequence(gold_node as u64);
    
    let sequence_matches = pred_sequence.iter()
        .zip(gold_sequence.iter())
        .filter(|(a, b)| a == b)
        .count();
    
    let flux_path_similarity = sequence_matches as f64 / pred_sequence.len() as f64;
    
    // === 4. SACRED POSITION ANALYSIS ===
    
    let pred_is_sacred = engine.sacred_positions.contains(&task.predicted_node);
    let gold_is_sacred = engine.sacred_positions.contains(&gold_node);
    let sacred_alignment = pred_is_sacred && gold_is_sacred;
    
    // Find nearest sacred positions
    let predicted_sacred_proximity = if !pred_is_sacred {
        engine.sacred_positions
            .iter()
            .min_by_key(|&&sacred| {
                (task.predicted_node as i16 - sacred as i16).abs()
            })
            .copied()
    } else {
        Some(task.predicted_node)
    };
    
    let gold_sacred_proximity = if !gold_is_sacred {
        engine.sacred_positions
            .iter()
            .min_by_key(|&&sacred| {
                (gold_node as i16 - sacred as i16).abs()
            })
            .copied()
    } else {
        Some(gold_node)
    };
    
    // Sacred boost: +15% if both at sacred positions
    let sacred_boost = if sacred_alignment { 0.15 } else { 0.0 };
    
    // === 5. SPATIAL ACCURACY (from gold_position implementation) ===
    
    // Create data point for spatial calculations
    let predicted_coords = layout.positions.get(&task.predicted_node)
        .copied()
        .unwrap_or(layout.center);
    let gold_coords = layout.positions.get(&gold_node)
        .copied()
        .unwrap_or(layout.center);
    
    let position_error = predicted_coords.distance_to(&gold_coords);
    let positional_accuracy = (1.0 - (position_error / 16.0)).max(0.0);
    
    let exact_match = task.predicted_node == gold_node;
    
    // === 6. COMBINED FLOW-AWARE ACCURACY ===
    
    // Weight factors:
    // - 40% spatial accuracy (how close in 2D space)
    // - 30% flow sequence accuracy (how close in flow)
    // - 20% flux path similarity (conceptual closeness)
    // - 10% from sacred alignment
    
    let flow_accuracy = 1.0 - (flow_distance as f64 / engine.base_pattern.len() as f64);
    
    let flow_aware_accuracy = 
        (0.4 * positional_accuracy) +
        (0.3 * flow_accuracy) +
        (0.2 * flux_path_similarity) +
        (0.1 * sacred_boost);
    
    FlowAwareEvaluation {
        predicted_node: task.predicted_node,
        gold_node,
        exact_match,
        positional_accuracy,
        position_error,
        predicted_flow_position,
        gold_flow_position,
        flow_distance,
        node_distance,
        flux_path_similarity,
        sacred_alignment,
        predicted_sacred_proximity,
        gold_sacred_proximity,
        sacred_boost,
        flow_aware_accuracy,
    }
}
```

---

## 🎯 **Flow Tracking Functions**

```rust
/// Track object's progression through flow sequence
fn track_flow_progression(
    object: &mut FlowingObject,
    num_steps: usize,
    engine: &FluxMatrixEngine,
) -> Vec<u8> {
    let mut visited_nodes = Vec::new();
    
    for _ in 0..num_steps {
        visited_nodes.push(object.current_node);
        object.advance_in_flow(engine);
    }
    
    visited_nodes
}

/// Calculate flow metrics for an object
fn calculate_flow_metrics(object: &FlowingObject, engine: &FluxMatrixEngine) -> FlowMetrics {
    FlowMetrics {
        current_node: object.current_node,
        flow_position: object.flow_position,
        cycle_count: object.cycle_count,
        total_steps: object.cycle_count * engine.base_pattern.len() + object.flow_position,
        in_flow_sequence: engine.base_pattern.contains(&object.current_node),
        near_sacred: object.check_sacred_proximity(engine),
    }
}

#[derive(Debug)]
struct FlowMetrics {
    current_node: u8,
    flow_position: usize,
    cycle_count: usize,
    total_steps: usize,
    in_flow_sequence: bool,
    near_sacred: Option<u8>,
}
```

---

## 🧪 **Benchmark Integration**

Add to `benchmarks/custom/geometric_reasoning_benchmark.rs`:

```rust
use spatial_vortex::flux_matrix::FluxMatrixEngine;
use spatial_vortex::visualization::{FluxDataPoint, FluxLayout};

// At the top with other imports

// Replace the code around line 420:
if let Some(gold_pos) = task.gold_position {
    let layout = FluxLayout::sacred_geometry_layout();
    let engine = FluxMatrixEngine::new();
    
    // Create flow-aware task
    let flow_task = FlowAwareTask {
        task_id: task.id.clone(),
        angle: task.angle,
        distance: task.distance,
        complexity: task.complexity,
        predicted_node: predicted_position,
        gold_node: Some(gold_pos),
        expected_flow_position: None, // Can be set if known
    };
    
    // Evaluate with flow awareness
    let evaluation = evaluate_with_flow_engine(&flow_task, &layout);
    
    // Print detailed results
    println!("{}", evaluation);
    
    // Track for aggregate metrics
    all_evaluations.push(evaluation);
}

// At end of benchmark, print aggregate statistics
fn print_aggregate_statistics(evaluations: &[FlowAwareEvaluation]) {
    println!("\n=== Aggregate Flow-Aware Statistics ===");
    
    let total = evaluations.len();
    if total == 0 {
        return;
    }
    
    // Exact matches
    let exact_matches = evaluations.iter().filter(|e| e.exact_match).count();
    println!("Exact Matches: {}/{} ({:.1}%)", 
        exact_matches, total, 
        exact_matches as f64 / total as f64 * 100.0);
    
    // Average accuracies
    let avg_positional = evaluations.iter()
        .map(|e| e.positional_accuracy)
        .sum::<f64>() / total as f64;
    
    let avg_flow_aware = evaluations.iter()
        .map(|e| e.flow_aware_accuracy)
        .sum::<f64>() / total as f64;
    
    println!("Average Positional Accuracy: {:.1}%", avg_positional * 100.0);
    println!("Average Flow-Aware Accuracy: {:.1}%", avg_flow_aware * 100.0);
    
    // Flow distance distribution
    let avg_flow_distance = evaluations.iter()
        .map(|e| e.flow_distance)
        .sum::<usize>() as f64 / total as f64;
    
    println!("Average Flow Distance: {:.2} steps", avg_flow_distance);
    
    // Sacred alignment rate
    let sacred_aligned = evaluations.iter()
        .filter(|e| e.sacred_alignment)
        .count();
    
    println!("Sacred Alignment Rate: {}/{} ({:.1}%)",
        sacred_aligned, total,
        sacred_aligned as f64 / total as f64 * 100.0);
    
    // Flux path similarity
    let avg_flux_similarity = evaluations.iter()
        .map(|e| e.flux_path_similarity)
        .sum::<f64>() / total as f64;
    
    println!("Average Flux Path Similarity: {:.1}%", avg_flux_similarity * 100.0);
}
```

---

## 📊 **Example Output**

```
=== Flow-Aware Evaluation ===

📍 Node Positions:
  Predicted Node: 5
  Gold Node: 7
  Exact Match: false

🌊 Flow Sequence:
  Predicted Flow Position: 5 in [1,2,4,8,7,5]
  Gold Flow Position: 4
  Flow Distance: 1 steps

📐 Spatial Metrics:
  Positional Accuracy: 82.3%
  Position Error: 2.84 units

🔢 Flux Matrix Metrics:
  Node Distance (digit reduction): 2
  Flux Path Similarity: 66.7%

✨ Sacred Positions:
  Sacred Alignment: false
  Predicted Near Sacred 6
  Gold Near Sacred 6
  Sacred Boost: +0.0%

🎯 Final Score:
  Flow-Aware Accuracy: 75.8%

---

=== Aggregate Flow-Aware Statistics ===
Exact Matches: 47/100 (47.0%)
Average Positional Accuracy: 78.5%
Average Flow-Aware Accuracy: 81.2%
Average Flow Distance: 1.8 steps
Sacred Alignment Rate: 12/100 (12.0%)
Average Flux Path Similarity: 72.3%
```

---

## 🎯 **Key Insights**

### **Why Flow Position Matters**:
1. **Objects flow through sequence** [1,2,4,8,7,5,1,2,4...]
2. **Flow position** tells you where in the cycle an object is
3. **Nodes are static** - objects visit them in flow order
4. **Sacred nodes (3,6,9)** are outside the flow but influence it

### **Flow Distance vs Node Distance**:
- **Flow Distance**: Steps apart in flow sequence (0-5)
- **Node Distance**: Digit reduction between nodes (0-9)
- Both are important for understanding prediction quality

### **Combined Accuracy**:
- 40% spatial (2D coordinates)
- 30% flow sequence (progression through pattern)
- 20% flux path (conceptual similarity)
- 10% sacred alignment (special positions)

---

## ✅ **Complete Implementation Checklist**

- [✅] Define `FlowingObject` with flow_position
- [✅] Define `FlowAwareTask` structure
- [✅] Implement `FlowAwareEvaluation` with all metrics
- [✅] Create `evaluate_with_flow_engine()` function
- [✅] Add flow tracking functions
- [✅] Integrate into benchmark
- [✅] Print detailed per-task results
- [✅] Calculate aggregate statistics
- [ ] **Copy to benchmark file** and test

---

**This implementation properly treats objects as flowing through the sequence while nodes remain static!** 🌊✨
