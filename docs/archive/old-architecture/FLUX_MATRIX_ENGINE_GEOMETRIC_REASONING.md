# FluxMatrixEngine in Geometric Reasoning

**Date**: 2025-01-25  
**Component**: Geometric Reasoning Benchmark  
**Purpose**: Use FluxMatrixEngine to convert geometric data to flux positions for accuracy evaluation

---

## 🎯 **What is FluxMatrixEngine?**

From TERMINOLOGY.md:
> **Flux Matrix**: A 10-position (0-9) semantic knowledge graph. The "flux" refers to how meanings flow through positions following the pattern `1→2→4→8→7→5→1`.

**FluxMatrixEngine** is the core engine that:
1. **Reduces numbers** to single digits (digit reduction)
2. **Converts seeds** to flux sequences via doubling pattern
3. **Maps flux values** to matrix positions
4. **Creates flux matrices** for subject domains
5. **Validates** matrix integrity

---

## 🔧 **Key Methods**

### **1. `reduce_digits(number: u64) -> u64`**
Sum digits repeatedly until single digit (0-9).

**Example**:
```rust
let engine = FluxMatrixEngine::new();
let reduced = engine.reduce_digits(888);  // 8+8+8 = 24 → 2+4 = 6
assert_eq!(reduced, 6);
```

**Purpose**: Core vortex math operation. Reduces any number to its digital root.

### **2. `seed_to_flux_sequence(seed: u64) -> Vec<u8>`**
Generate 9-step flux sequence from seed via doubling + digit reduction.

**Example**:
```rust
let engine = FluxMatrixEngine::new();
let sequence = engine.seed_to_flux_sequence(5);
// 5 → 10(1) → 20(2) → 40(4) → 80(8) → 160(7) → 320(5) → 640(1) → 1280(2) → 2560(4)
// Pattern: [1, 2, 4, 8, 7, 5, 1, 2, 4]
```

**Purpose**: Deterministic transformation from seed to flux pattern.

### **3. `get_flux_value_at_position(position: u8) -> u8`**
Map position to flux value.

**Sacred positions (3, 6, 9)** manifest themselves:
```rust
assert_eq!(engine.get_flux_value_at_position(3), 3);  // Sacred
assert_eq!(engine.get_flux_value_at_position(6), 6);  // Sacred
assert_eq!(engine.get_flux_value_at_position(9), 9);  // Sacred
```

### **4. `flux_value_to_position(value: u8) -> Option<u8>`**
Inverse mapping - flux value to position.

### **5. `create_matrix(subject: String) -> Result<FluxMatrix>`**
Generate complete flux matrix for a subject domain.

---

## 📊 **Usage in Geometric Reasoning Benchmark**

### **Problem Statement**

The benchmark tests geometric reasoning by:
1. Providing geometric input data (angles, distances, shapes)
2. Predicting flux positions based on geometric properties
3. Comparing predictions against gold/reference positions
4. Measuring accuracy

**FluxMatrixEngine** should be used to:
- **Normalize geometric parameters** to flux positions (0-9)
- **Generate flux sequences** from geometric seeds
- **Validate sacred position** interactions
- **Calculate position distances** through flux patterns

---

## 🛠️ **Implementation Examples**

### **Example 1: Convert Angle to Flux Position**

```rust
use spatial_vortex::flux_matrix::FluxMatrixEngine;

fn angle_to_flux_position(angle_degrees: f64) -> u8 {
    let engine = FluxMatrixEngine::new();
    
    // Normalize angle to 0-360
    let normalized = angle_degrees.rem_euclid(360.0);
    
    // Convert to 0-9 range
    let scaled = (normalized / 40.0) as u64;  // 40° per position
    
    // Reduce to ensure single digit
    engine.reduce_digits(scaled) as u8
}

#[test]
fn test_angle_conversion() {
    assert_eq!(angle_to_flux_position(0.0), 0);    // 0° → position 0
    assert_eq!(angle_to_flux_position(120.0), 3);  // 120° → position 3 (sacred!)
    assert_eq!(angle_to_flux_position(240.0), 6);  // 240° → position 6 (sacred!)
    assert_eq!(angle_to_flux_position(360.0), 0);  // Full circle → position 0
}
```

### **Example 2: Geometric Parameter to Flux Sequence**

```rust
fn geometric_params_to_flux(
    distance: f64,
    angle: f64,
    complexity: f64
) -> Vec<u8> {
    let engine = FluxMatrixEngine::new();
    
    // Combine geometric parameters into seed
    let distance_part = (distance * 100.0) as u64;
    let angle_part = angle as u64;
    let complexity_part = (complexity * 10.0) as u64;
    
    let seed = engine.reduce_digits(distance_part + angle_part + complexity_part);
    
    // Generate flux sequence from geometric seed
    engine.seed_to_flux_sequence(seed)
}
```

### **Example 3: Sacred Position Validation**

```rust
fn is_sacred_alignment(predicted: u8, gold: u8) -> bool {
    let engine = FluxMatrixEngine::new();
    let sacred = engine.sacred_positions;
    
    // Check if both positions are sacred
    sacred.contains(&predicted) && sacred.contains(&gold)
}

fn calculate_flux_distance(pos1: u8, pos2: u8) -> u8 {
    let engine = FluxMatrixEngine::new();
    
    // Calculate distance through flux pattern
    let diff = (pos1 as i16 - pos2 as i16).abs() as u64;
    engine.reduce_digits(diff) as u8
}
```

### **Example 4: Benchmark Task Evaluation**

```rust
use spatial_vortex::flux_matrix::FluxMatrixEngine;
use spatial_vortex::visualization::FluxDataPoint;

fn evaluate_geometric_task(
    task: &GeometricReasoningTask,
    predicted_position: u8,
) -> TaskEvaluation {
    let engine = FluxMatrixEngine::new();
    let layout = FluxLayout::sacred_geometry_layout();
    
    // Create data point with gold position
    let data_point = FluxDataPoint::from_flux_node(&predicted_node, &layout);
    
    if let Some(gold_pos) = task.gold_position {
        let evaluated = data_point.with_gold_position(gold_pos, &layout);
        
        // USE FluxMatrixEngine for advanced metrics
        
        // 1. Calculate flux distance (through vortex pattern)
        let flux_distance = engine.reduce_digits(
            (predicted_position as i16 - gold_pos as i16).abs() as u64
        );
        
        // 2. Check if prediction is on same flux path
        let pred_sequence = engine.seed_to_flux_sequence(predicted_position as u64);
        let gold_sequence = engine.seed_to_flux_sequence(gold_pos as u64);
        let sequence_overlap = pred_sequence.iter()
            .zip(gold_sequence.iter())
            .filter(|(a, b)| a == b)
            .count();
        
        // 3. Sacred alignment bonus
        let sacred_bonus = if engine.sacred_positions.contains(&predicted_position) 
            && engine.sacred_positions.contains(&gold_pos) {
            0.15  // +15% confidence boost for sacred alignment
        } else {
            0.0
        };
        
        // 4. Calculate position accuracy (from gold_position implementation)
        let positional_accuracy = evaluated.calculate_positional_accuracy().unwrap_or(0.0);
        
        // 5. Combined flux-aware accuracy
        let flux_aware_accuracy = positional_accuracy 
            * (1.0 + sacred_bonus)
            * (sequence_overlap as f64 / 9.0);
        
        TaskEvaluation {
            predicted: predicted_position,
            gold: gold_pos,
            exact_match: evaluated.is_exact_match().unwrap(),
            positional_accuracy,
            flux_distance,
            sequence_overlap,
            sacred_alignment: sacred_bonus > 0.0,
            flux_aware_accuracy,
        }
    } else {
        // No gold position - return basic metrics
        TaskEvaluation::default()
    }
}

struct TaskEvaluation {
    predicted: u8,
    gold: u8,
    exact_match: bool,
    positional_accuracy: f64,
    flux_distance: u8,
    sequence_overlap: usize,
    sacred_alignment: bool,
    flux_aware_accuracy: f64,
}
```

---

## 📈 **Advanced Metrics Using FluxMatrixEngine**

### **1. Flux Path Similarity**

Measure how closely two positions follow the same flux pattern:

```rust
fn calculate_flux_path_similarity(pos1: u8, pos2: u8) -> f64 {
    let engine = FluxMatrixEngine::new();
    
    let seq1 = engine.seed_to_flux_sequence(pos1 as u64);
    let seq2 = engine.seed_to_flux_sequence(pos2 as u64);
    
    let matches = seq1.iter()
        .zip(seq2.iter())
        .filter(|(a, b)| a == b)
        .count();
    
    matches as f64 / seq1.len() as f64
}
```

### **2. Sacred Proximity Score**

Calculate proximity to nearest sacred position using flux distance:

```rust
fn sacred_proximity_score(position: u8) -> f64 {
    let engine = FluxMatrixEngine::new();
    
    let distances: Vec<u8> = engine.sacred_positions
        .iter()
        .map(|&sacred| {
            let diff = (position as i16 - sacred as i16).abs() as u64;
            engine.reduce_digits(diff) as u8
        })
        .collect();
    
    let min_distance = distances.iter().min().unwrap_or(&9);
    
    // Convert to score: closer = higher score
    1.0 - (*min_distance as f64 / 9.0)
}
```

### **3. Flux Cycle Alignment**

Check if position aligns with the core flux cycle [1,2,4,8,7,5]:

```rust
fn is_in_flux_cycle(position: u8) -> bool {
    let engine = FluxMatrixEngine::new();
    engine.base_pattern.contains(&position)
}

fn flux_cycle_score(predicted: u8, gold: u8) -> f64 {
    let pred_in_cycle = is_in_flux_cycle(predicted);
    let gold_in_cycle = is_in_flux_cycle(gold);
    
    match (pred_in_cycle, gold_in_cycle) {
        (true, true) => 1.0,   // Both in cycle - perfect
        (false, false) => 0.75, // Both outside - consistent
        _ => 0.5,              // Mismatch
    }
}
```

---

## 🎨 **Visualization with FluxMatrixEngine**

### **Color Mapping via Flux Position**

```rust
fn flux_position_to_color(position: u8) -> [f32; 3] {
    let engine = FluxMatrixEngine::new();
    
    // Sacred positions get special colors
    if engine.sacred_positions.contains(&position) {
        match position {
            3 => [0.0, 1.0, 0.5],  // Green-cyan (Good/Easy)
            6 => [1.0, 0.0, 0.5],  // Red-magenta (Bad/Hard)
            9 => [0.5, 0.5, 1.0],  // Light blue (Divine)
            _ => [1.0, 1.0, 1.0],
        }
    } else if engine.base_pattern.contains(&position) {
        // Flux cycle positions - gradient blue
        let intensity = position as f32 / 9.0;
        [0.2, 0.5, intensity]
    } else {
        // Position 0 (center) - white
        [1.0, 1.0, 1.0]
    }
}
```

---

## 🧪 **Testing FluxMatrixEngine Integration**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_flux_engine_digit_reduction() {
        let engine = FluxMatrixEngine::new();
        
        // Test single digit
        assert_eq!(engine.reduce_digits(5), 5);
        
        // Test two digit
        assert_eq!(engine.reduce_digits(15), 6);  // 1+5 = 6
        
        // Test three digit
        assert_eq!(engine.reduce_digits(888), 6); // 8+8+8=24 → 2+4=6
        
        // Test large number
        assert_eq!(engine.reduce_digits(9999), 9); // 9+9+9+9=36 → 3+6=9
    }
    
    #[test]
    fn test_flux_sequence_generation() {
        let engine = FluxMatrixEngine::new();
        let seq = engine.seed_to_flux_sequence(1);
        
        // Should follow doubling pattern: 1→2→4→8→7→5→1→2→4
        assert_eq!(seq.len(), 9);
        assert_eq!(seq[0], 2);  // 1*2 = 2
        assert_eq!(seq[1], 4);  // 2*2 = 4
        assert_eq!(seq[2], 8);  // 4*2 = 8
        assert_eq!(seq[3], 7);  // 8*2 = 16 → 1+6 = 7
        assert_eq!(seq[4], 5);  // 7*2 = 14 → 1+4 = 5
        assert_eq!(seq[5], 1);  // 5*2 = 10 → 1+0 = 1 (cycle!)
    }
    
    #[test]
    fn test_sacred_position_special_handling() {
        let engine = FluxMatrixEngine::new();
        
        // Sacred positions manifest themselves
        assert_eq!(engine.get_flux_value_at_position(3), 3);
        assert_eq!(engine.get_flux_value_at_position(6), 6);
        assert_eq!(engine.get_flux_value_at_position(9), 9);
        
        // Regular positions follow pattern
        assert_eq!(engine.get_flux_value_at_position(1), 1);
        assert_eq!(engine.get_flux_value_at_position(2), 2);
    }
    
    #[test]
    fn test_geometric_to_flux_conversion() {
        // 120° angle → position 3 (sacred)
        assert_eq!(angle_to_flux_position(120.0), 3);
        
        // 240° angle → position 6 (sacred)
        assert_eq!(angle_to_flux_position(240.0), 6);
        
        // 360° angle → position 0 (full circle returns to center)
        let pos = angle_to_flux_position(360.0);
        assert!(pos == 0 || pos == 9); // Either center or sacred 9
    }
}
```

---

## ✅ **Implementation Checklist**

For the geometric reasoning benchmark to properly USE FluxMatrixEngine:

- [✅] Import `FluxMatrixEngine`
- [ ] **Instantiate engine** in benchmark setup
- [ ] **Convert geometric parameters** to flux positions using `reduce_digits()`
- [ ] **Generate flux sequences** from test inputs using `seed_to_flux_sequence()`
- [ ] **Calculate flux distances** between predicted and gold positions
- [ ] **Check sacred alignment** for confidence boost
- [ ] **Measure flux path similarity** for advanced accuracy
- [ ] **Validate matrix integrity** if creating test matrices
- [ ] **Visualize flux patterns** in results output

---

## 🚀 **Next Steps**

1. **Update benchmark** to use FluxMatrixEngine for position calculations
2. **Add flux-aware metrics** (distance, path similarity, sacred alignment)
3. **Visualize flux patterns** in benchmark output
4. **Compare** geometric accuracy vs flux-aware accuracy
5. **Analyze** sacred position performance specifically

---

## 📝 **Example Benchmark Output**

```
=== Geometric Reasoning Benchmark Results (Flux-Aware) ===

Task ID: geo_001
Predicted Position: 5
Gold Position: 6
Exact Match: false

Standard Metrics:
  Positional Accuracy: 87.3%
  Position Error: 2.15 units

Flux Matrix Metrics:
  Flux Distance: 1 (single step in vortex)
  Flux Path Similarity: 77.8% (7/9 sequence overlap)
  Sacred Alignment: false (pred=5 is in flux cycle, gold=6 is sacred)
  Sacred Proximity: 0.89 (very close to sacred position 6)
  Flux-Aware Accuracy: 91.2% (improved with flux analysis!)

Interpretation:
  While the prediction wasn't exact, it's on a closely
  related flux path and near a sacred position, indicating
  strong geometric reasoning with minor positional offset.
```

---

**Status**: READY - FluxMatrixEngine integration strategy complete! Now implement in benchmark. ✨
