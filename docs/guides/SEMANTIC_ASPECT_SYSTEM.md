# Semantic Aspect Color System

**Core Principle**: Colors represent subject matter **MEANING**Did all of the Color ML get implemented?

What's Needed: ❌ Feature extraction for tensors ❌ Training data generation ❌ Loss functions for color learning ❌ Integration with inference engine ❌ Color-guided generation

Did we act on the integration roadmap for 
Week 1-2: Foundation
 Add to_feature_vector() / from_feature_vector()
 Create AspectTrainingData structure
 Implement color-aware loss functions
 Add tests
Week 3-4: Training
 Integrate with src/ml/training/trainer.rs
 Create color similarity training dataset
 Train embedding model with color supervision
 Validate on semantic similarity tasks
Week 5-6: Inference
 Add color context to InferenceEngine
 Implement color_to_meaning() prediction
 Add color-guided generation
 Benchmark performance
Week 7-8: Visualization
 Create color space visualization tools
 Show ML reasoning trajectories in color
 Interactive color-based search UI
 Production deployment.

---

## Architecture

### Hexagonal Color Wheel Foundation

The system uses the **hexagonal Color Wheel** as its color space:

- **Ascendency**: Colors reach unity in divine light (white at apex)
- **Hexagonal Descent**: Major colors with layered variations
- **Semantic Hash**: Subject meanings map to color wheel positions
- **Intention**: The key that binds similarities/differences between subjects

---

## Core Concepts

### Subjects → Aspects = Flux Matrices with Intention

A **subject** (like "love" or "logic") becomes a **flux matrix** when given intention:

```
Subject("love") + Intention → FluxMatrix + Aspect("love")
```

### Aspects → Subjects = Colors with Intention

An **aspect** is the semantic meaning of a subject, represented by a color:

```
Aspect("love") → Color(hue=X°, saturation=Y, luminance=Z)
```

Similar meanings have nearby colors on the color wheel.

### Nodes Inherit Colors During Processing

**Critical**: Nodes do **NOT** have static colors based on position. Instead:

1. Objects carry aspects (semantic meanings)
2. Aspects carry colors (from color wheel)
3. Nodes inherit object colors during processing
4. Color represents current semantic meaning being processed

---

## Implementation

### AspectColor

RGBA color in hexagonal HSL space:

```rust
pub struct AspectColor {
    pub r: f32,       // Red (0.0-1.0)
    pub g: f32,       // Green (0.0-1.0)
    pub b: f32,       // Blue (0.0-1.0)
    pub a: f32,       // Alpha (0.0-1.0)
    
    // Hexagonal space coordinates
    pub luminance: f32,    // 0.0=pure color, 1.0=white (divine)
    pub hue: f32,          // 0-360 degrees on wheel
    pub saturation: f32,   // 0.0=gray, 1.0=pure
}
```

### AspectOrientation

Semantic meaning with intentional color:

```rust
pub struct AspectOrientation {
    /// Primary semantic meaning
    pub meaning: String,
    
    /// Color from hexagonal wheel
    pub color: AspectColor,
    
    /// Semantic variance (flexibility)
    pub variance: f32,
    
    /// Related aspects (nearby in color space)
    pub related_aspects: Vec<String>,
    
    /// Intention strength
    pub intention: f32,
}
```

---

## Usage

### Creating Aspects from Meaning

```rust
use spatial_vortex::data::{AspectOrientation, AspectColor};

// Create aspect from semantic meaning
let love_aspect = AspectOrientation::from_meaning("love", 0.15);

println!("Meaning: {}", love_aspect.meaning);
println!("Color: {}", love_aspect.color.to_hex());
println!("Hue: {}°", love_aspect.color.hue);
```

### Semantic Similarity

Similar meanings have nearby colors:

```rust
let love = AspectOrientation::from_meaning("love", 0.15);
let affection = AspectOrientation::from_meaning("affection", 0.15);

// Calculate semantic distance via color proximity
let is_similar = love.is_similar_to(&affection);
```

### Color Operations

```rust
let color = AspectColor::from_meaning("hope");

// Ascend toward divine light (increase luminance)
let divine = color.ascend(0.3);

// Descend into pure color (decrease luminance)
let pure = color.descend(0.2);

// Get nearby colors (semantic variance)
let nearby = color.nearby(0.2);

// Blend colors (semantic mixing)
let blend = color1.blend(&color2, 0.5);
```

---

## Semantic Color Space

Global manager for aspects:

```rust
use spatial_vortex::data::SemanticColorSpace;

let mut space = SemanticColorSpace::new();

// Register aspects
let love = AspectOrientation::from_meaning("love", 0.15);
space.register_aspect(love);

// Get or create aspect
let aspect = space.get_or_create_aspect("love", 0.15);

// Find by color proximity (inference engine)
let similar = space.find_by_color(&target_color, 0.3);

// Find related aspects
let related = space.find_related("love");
```

---

## Integration with FluxSubject

### Subject Creation

```rust
// Create subject with semantic meaning
let subject = FluxSubject::from_sacred_position(3, "love");

// Aspect is created from semantic meaning (NOT position)
let aspect = subject.aspect_orientation().unwrap();
println!("Meaning: {}", aspect.meaning);  // "Creative Ethics love"

// Color derives from meaning
let color = subject.aspect_color().unwrap();
println!("Color: {}", color.to_hex());
```

### Same Meaning, Different Positions

```rust
// Position 3 with "love"
let subject1 = FluxSubject::from_sacred_position(3, "love");

// Position 9 with "love"
let subject2 = FluxSubject::from_sacred_position(9, "love");

// Colors will be SIMILAR (same semantic meaning)
let color1 = subject1.aspect_color().unwrap();
let color2 = subject2.aspect_color().unwrap();

let distance = color1.distance(&color2);
// distance will be small (similar meanings = similar colors)
```

### Different Meanings, Same Position

```rust
// Position 3 with "love"
let subject_love = FluxSubject::from_sacred_position(3, "love");

// Position 3 with "hate"
let subject_hate = FluxSubject::from_sacred_position(3, "hate");

// Colors will be DIFFERENT (different meanings)
let color_love = subject_love.aspect_color().unwrap();
let color_hate = subject_hate.aspect_color().unwrap();

assert_ne!(color_love.hue, color_hate.hue);
```

---

## Inference Engine Use Case

### Finding Similar Subjects by Color

```rust
let mut space = SemanticColorSpace::new();

// Register many aspects
for meaning in ["love", "affection", "compassion", "hate", "anger"] {
    let aspect = AspectOrientation::from_meaning(meaning, 0.15);
    space.register_aspect(aspect);
}

// Find aspects similar to "love"
let love_color = AspectColor::from_meaning("love");
let similar = space.find_by_color(&love_color, 0.3);

// Will find: "affection", "compassion" (nearby on color wheel)
// Will NOT find: "hate", "anger" (opposite on color wheel)
```

### Variance and Flexibility

```rust
// Low variance = strict meaning
let strict_love = AspectOrientation::from_meaning("love", 0.05);

// High variance = flexible meaning
let flexible_love = AspectOrientation::from_meaning("love", 0.5);

// Flexible aspects match more similar meanings
let nearby_flexible = flexible_love.color.nearby(0.5);  // Many colors
let nearby_strict = strict_love.color.nearby(0.05);      // Few colors
```

---

## BrickColor Integration

### BrickColor IDs

```rust
// Use BrickColor IDs
let color = AspectColor::from_brick_color_id(21);  // Bright red

// Common IDs:
// 1 = White (divine light)
// 21 = Bright red
// 23 = Bright blue
// 28 = Dark green
// 24 = Bright yellow
// 104 = Bright violet
// 330 = Crimson
```

### Golden Angle Distribution

Colors are distributed using the **golden angle** (137.508°) for even spacing:

```rust
// Semantic hash uses golden angle
let hash = semantic_hash("love");
let hue = (hash * 137.508) % 360.0;
```

This ensures **good color separation** between different meanings.

---

## Divine Ascendency

### White at Apex

Pure white represents **divine unity** at the apex of the hexagonal space:

```rust
let color = AspectColor::from_meaning("hope");

// Ascend toward divine light
let step1 = color.ascend(0.2);  // Lighter
let step2 = step1.ascend(0.2);  // Even lighter
let divine = step2.ascend(0.6); // Nearly white (divine)

// luminance → 1.0 = pure white = divine unity
```

### Hexagonal Descent

Major colors at base with layered variations:

- **Red family**: Passion, anger, energy
- **Orange family**: Creativity, warmth
- **Yellow family**: Joy, intellect
- **Green family**: Growth, nature
- **Blue family**: Truth, calm
- **Violet family**: Spirituality, mystery

---

## Key Principles

1. **Semantic Hashing**: `meaning → hue (0-360°)`
2. **Deterministic**: Same meaning = same color
3. **Distributed**: Golden angle ensures good spacing
4. **Intentional**: Colors carry semantic intent
5. **Dynamic**: Nodes inherit colors during processing
6. **Variance-aware**: Flexible meanings = wider color range
7. **Inference-ready**: Color distance = semantic similarity

---

## Examples

### Example 1: Subject Colors

```rust
let love = FluxSubject::from_sacred_position(3, "love");
let hate = FluxSubject::from_sacred_position(3, "hate");
let compassion = FluxSubject::from_sacred_position(6, "compassion");

// love and compassion will have nearby colors (similar meanings)
// love and hate will have distant colors (opposite meanings)
```

### Example 2: Aspect Similarity

```rust
let aspect1 = AspectOrientation::from_meaning("wisdom", 0.15);
let aspect2 = AspectOrientation::from_meaning("knowledge", 0.15);

if aspect1.is_similar_to(&aspect2) {
    println!("Semantically similar!");
}
```

### Example 3: Inference by Color

```rust
let mut space = SemanticColorSpace::new();

// User provides a color (e.g., from visual input)
let user_color = AspectColor::new(0.8, 0.2, 0.4, 1.0);

// Find aspects matching this color
let matches = space.find_by_color(&user_color, 0.2);

// Returns aspects with similar semantic meanings
```

---

## API Summary

### AspectColor Methods

- `from_meaning(meaning: &str) -> Self` - Create from semantic meaning
- `from_hsl(h, s, l) -> Self` - Create from HSL
- `from_brick_color_id(id) -> Self` - Use BrickColor ID
- `distance(&self, other) -> f32` - Semantic proximity
- `blend(&self, other, ratio) -> Self` - Mix meanings
- `ascend(amount) -> Self` - Toward divine light
- `descend(amount) -> Self` - Toward pure color
- `nearby(variance) -> Vec<Self>` - Similar colors
- `to_hex() -> String` - Hex color code

### AspectOrientation Methods

- `from_meaning(meaning, variance) -> Self` - Create aspect
- `with_color(meaning, color, variance) -> Self` - Explicit color
- `find_similar(candidates, max_dist) -> Vec<String>` - Similar aspects
- `is_similar_to(other) -> bool` - Check similarity

### SemanticColorSpace Methods

- `register_aspect(aspect)` - Add aspect
- `get_or_create_aspect(meaning, variance) -> AspectOrientation` - Get/create
- `find_by_color(target, max_dist) -> Vec<AspectOrientation>` - Inference query
- `find_related(meaning) -> Vec<AspectOrientation>` - Related aspects
- `divine_light() -> AspectColor` - White at apex

---

## Testing

```bash
# Run aspect color tests
cargo test aspect_color --lib

# Run semantic tests
cargo test aspect_semantic --lib
```

---

## Summary

✅ **Colors from MEANING**, not position  
✅ **Hexagonal Color Wheel** space  
✅ **Ascendency to divine light** (white at apex)  
✅ **Golden angle distribution** for spacing  
✅ **Semantic similarity** via color distance  
✅ **Variance-aware** for flexible meanings  
✅ **Inference engine ready** for aspect queries  
✅ **Nodes inherit colors** during processing  
✅ **Intention-driven** semantic system  

**The aspect color system enables semantic visual encoding where similar meanings naturally cluster together in color space, allowing the inference engine to navigate meaning through color proximity.**
