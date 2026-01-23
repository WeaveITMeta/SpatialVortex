# ELP to Attributes Migration Guide

This document describes the migration from the legacy ELP (Ethos, Logos, Pathos) system to the new universal Attributes system compatible with EustressEngine.

## Overview

The new `Attributes` system provides:
- **EustressEngine Compatibility**: Same API as `eustress_common::attributes`
- **Backward Compatibility**: Legacy ELP accessors still work
- **Extensibility**: Add any custom attributes beyond ELP
- **Tags Support**: CollectionService-style tagging

## Quick Migration

### Before (Legacy ELP)
```rust
// Old way - hardcoded ELP tensor
let elp: [f32; 3] = [0.5, 0.3, 0.2];  // [ethos, logos, pathos]
envelope.elp = Some(elp);

// Access
let ethos = envelope.elp.unwrap()[0];
```

### After (New Attributes)
```rust
use spatial_vortex::data::{Attributes, AttributeValue};

// New way - flexible attributes
let mut attrs = Attributes::with_elp(0.5, 0.3, 0.2);
attrs.set("custom_field", AttributeValue::String("value".into()));

let envelope = DataEnvelope::with_attributes("id", data, attrs);

// Access ELP (backward compatible)
let ethos = envelope.attributes.ethos();  // 0.5
let elp = envelope.get_elp();  // [0.5, 0.3, 0.2]

// Or access custom attributes
let custom = envelope.get_attribute("custom_field");
```

## API Reference

### Creating Attributes

```rust
// Empty
let attrs = Attributes::new();

// With single attribute
let attrs = Attributes::with_attribute("key", AttributeValue::Number(42.0));

// With ELP (backward compatibility)
let attrs = Attributes::with_elp(0.5, 0.3, 0.2);

// From ELP tensor
let attrs = Attributes::from_elp_tensor([0.5, 0.3, 0.2]);
```

### Setting Values

```rust
attrs.set("name", AttributeValue::String("Player".into()));
attrs.set("health", AttributeValue::Number(100.0));
attrs.set("active", AttributeValue::Bool(true));
attrs.set("position", AttributeValue::Vector3([1.0, 2.0, 3.0]));
attrs.set("color", AttributeValue::Color([1.0, 0.0, 0.0, 1.0]));
```

### Getting Values

```rust
// Type-safe getters
let name: Option<&str> = attrs.get_string("name");
let health: Option<f64> = attrs.get_number("health");
let active: Option<bool> = attrs.get_bool("active");
let pos: Option<[f32; 3]> = attrs.get_vector3("position");
let color: Option<[f32; 4]> = attrs.get_color("color");

// Generic getter
let value: Option<&AttributeValue> = attrs.get("name");
```

### ELP Compatibility

```rust
// Get individual components
let ethos = attrs.ethos();   // f32
let logos = attrs.logos();   // f32
let pathos = attrs.pathos(); // f32

// Get as tensor
let elp = attrs.elp_tensor();  // [f32; 3]

// Get normalized (sums to 1.0)
let normalized = attrs.elp_normalized();  // [f32; 3]

// Get dominant dimension
let dominant = attrs.elp_dominant();  // "ethos" | "logos" | "pathos"

// Set ELP
attrs.set_ethos(0.6);
attrs.set_logos(0.3);
attrs.set_pathos(0.1);
attrs.set_elp_tensor([0.6, 0.3, 0.1]);
```

### Sacred Geometry Integration

```rust
// Signal strength
attrs.set_confidence(0.8);
let signal = attrs.confidence();  // 0.8

// Digital root flux (1-9)
attrs.set_digital_root_flux(3);
let flux = attrs.digital_root_flux();  // 3

// Check sacred position
let is_sacred = attrs.is_sacred_position();  // true for 3, 6, 9

// Confidence
attrs.set_confidence(0.95);
let conf = attrs.confidence();  // 0.95
```

### Tags

```rust
use spatial_vortex::data::Tags;

// Create tags
let mut tags = Tags::new();
tags.add("enemy");
tags.add("boss");

// Check tags
let has_enemy = tags.has("enemy");  // true

// Remove tags
tags.remove("enemy");

// Iterate
for tag in tags.iter() {
    println!("{}", tag);
}

// Sorted list
let sorted = tags.sorted();  // Vec<&String>
```

## DataEnvelope Changes

### New Fields

```rust
pub struct DataEnvelope {
    // ... existing fields ...
    
    /// Universal attributes (EustressEngine compatible)
    pub attributes: Attributes,
    
    /// Tags for categorization (CollectionService-style)
    pub tags: Tags,
    
    /// ELP tensor - backward compatibility (prefer attributes.elp_tensor())
    pub elp: Option<[f32; 3]>,
}
```

### New Methods

```rust
// Create with attributes
let envelope = DataEnvelope::with_attributes(id, data, attrs);

// Get/set ELP (syncs with attributes)
let elp = envelope.get_elp();
envelope.set_elp([0.5, 0.3, 0.2]);

// Get/set signal strength (syncs with attributes)
let signal = envelope.get_confidence();
envelope.set_confidence(0.8);

// Attribute access
envelope.set_attribute("key", AttributeValue::Number(42.0));
let value = envelope.get_attribute("key");

// Tag access
envelope.add_tag("important");
let has_tag = envelope.has_tag("important");
```

## AttributeValue Types

| Type | Rust Type | Example |
|------|-----------|---------|
| `String` | `String` | `AttributeValue::String("hello".into())` |
| `Number` | `f64` | `AttributeValue::Number(42.0)` |
| `Bool` | `bool` | `AttributeValue::Bool(true)` |
| `Vector3` | `[f32; 3]` | `AttributeValue::Vector3([1.0, 2.0, 3.0])` |
| `Vector2` | `[f32; 2]` | `AttributeValue::Vector2([1.0, 2.0])` |
| `Color` | `[f32; 4]` | `AttributeValue::Color([1.0, 0.0, 0.0, 1.0])` |
| `Int` | `i64` | `AttributeValue::Int(42)` |
| `EntityRef` | `u32` | `AttributeValue::EntityRef(123)` |
| `Transform` | struct | Position, rotation, scale |
| `NumberRange` | struct | Min/max range |
| `NumberSequence` | `Vec<Keypoint>` | Animation keyframes |
| `ColorSequence` | `Vec<Keypoint>` | Color gradient |
| `Tensor` | `Vec<f32>` | ML tensor data |

## JSON Serialization

```rust
// Convert to JSON-compatible format
let json_map = attrs.to_json_map();  // HashMap<String, AttributeValueJson>

// Create from JSON
let attrs = Attributes::from_json_map(json_map);
```

## EustressEngine Integration

The Attributes system is designed to be compatible with EustressEngine's `eustress_common::attributes` module:

```rust
// In EustressEngine, convert to SpatialVortex format
use spatial_vortex::data::{Attributes, AttributeValue};

fn convert_attributes(eustress_attrs: &eustress_common::Attributes) -> Attributes {
    let mut attrs = Attributes::new();
    for (key, value) in eustress_attrs.iter() {
        attrs.set(key.clone(), convert_value(value));
    }
    attrs
}

fn convert_value(value: &eustress_common::AttributeValue) -> AttributeValue {
    match value {
        eustress_common::AttributeValue::String(s) => AttributeValue::String(s.clone()),
        eustress_common::AttributeValue::Number(n) => AttributeValue::Number(*n),
        eustress_common::AttributeValue::Bool(b) => AttributeValue::Bool(*b),
        eustress_common::AttributeValue::Vector3(v) => AttributeValue::Vector3([v.x, v.y, v.z]),
        // ... etc
    }
}
```

## AttributeAccessor Trait (Generic Property-Based Access)

The `AttributeAccessor` trait provides a **generic** property-based interface for accessing attributes, similar to EustressEngine's pattern. The trait is designed for **any** attribute - health, mana, position, team, or domain-specific data like ethos/logos/pathos.

### Core Design Philosophy

**Generic Only**: The trait provides only generic methods: `get_attribute()`, `set_attribute()`, `get_f32()`, `get_string()`, `get_bool()`, etc.

**No Domain-Specific Methods**: There are no special methods for ELP or any other domain. All attributes are accessed through the generic interface.

### Implementing AttributeAccessor

```rust
use spatial_vortex::data::{Attributes, AttributeAccessor};

struct MyEntity {
    attributes: Attributes,
    // ... other fields
}

impl AttributeAccessor for MyEntity {
    fn attributes(&self) -> &Attributes {
        &self.attributes
    }
    
    fn attributes_mut(&mut self) -> &mut Attributes {
        &mut self.attributes
    }
}
```

### Using AttributeAccessor

Once implemented, you get automatic access to generic attribute methods:

```rust
// Generic attribute access
my_entity.set_attribute("health", AttributeValue::Number(100.0));
my_entity.set_attribute("name", AttributeValue::String("Player".into()));
my_entity.set_attribute("active", AttributeValue::Bool(true));
my_entity.set_attribute("position", AttributeValue::Vector3([1.0, 2.0, 3.0]));

// Type-safe getters
let health = my_entity.get_f32("health");      // Some(100.0)
let name = my_entity.get_string("name");       // Some("Player")
let active = my_entity.get_bool("active");     // Some(true)
let pos = my_entity.get_vector3("position");   // Some([1.0, 2.0, 3.0])

// Check and remove
let has_health = my_entity.has_attribute("health");
my_entity.remove_attribute("health");

// Domain-specific attributes (e.g., ELP for semantic analysis)
my_entity.set_attribute("ethos", AttributeValue::Number(0.6));
my_entity.set_attribute("logos", AttributeValue::Number(0.3));
my_entity.set_attribute("pathos", AttributeValue::Number(0.1));

let ethos = my_entity.get_f32("ethos").unwrap_or(0.33);
let logos = my_entity.get_f32("logos").unwrap_or(0.34);
let pathos = my_entity.get_f32("pathos").unwrap_or(0.33);

// Sacred geometry attributes
my_entity.set_attribute("confidence", AttributeValue::Number(0.85));
my_entity.set_attribute("digital_root_flux", AttributeValue::Int(3));
my_entity.set_attribute("flux_position", AttributeValue::Number(3.0));

let confidence = my_entity.get_f32("confidence").unwrap_or(1.0);
let flux = my_entity.get_number("digital_root_flux").map(|n| n as u8).unwrap_or(1);
let is_sacred = flux == 3 || flux == 6 || flux == 9;
```

### Built-in Implementations

The following types already implement `AttributeAccessor`:

- **`Attributes`** - Identity implementation (self-reference)
- **`BeamTensor`** - Access via `.attributes` field
- **`BeadTensor`** - Alias for BeamTensor

### Migration from Direct Field Access

**Before (deprecated):**
```rust
// Direct field access (generates warnings)
let ethos = beam.ethos.unwrap_or(0.33);
let logos = beam.logos.unwrap_or(0.34);
let pathos = beam.pathos.unwrap_or(0.33);
```

**After (using AttributeAccessor trait):**
```rust
use spatial_vortex::data::AttributeAccessor;

// Trait-based property access (no warnings)
let ethos = beam.ethos();
let logos = beam.logos();
let pathos = beam.pathos();

// Or use the tensor method
let elp = beam.elp_tensor();  // [ethos, logos, pathos]
```

### Benefits of AttributeAccessor

1. **Universal**: Works with any attribute - health, mana, position, team, ethos, logos, etc.
2. **Type Safety**: Compile-time guarantees for attribute access
3. **Consistency**: Same API across all types with attributes
4. **Extensibility**: Easy to add new attribute-based types
5. **EustressEngine Compatible**: Matches the property-based pattern
6. **No Deprecation Warnings**: Modern API replaces deprecated fields

### Why Generic Methods Only?

The `AttributeAccessor` trait uses **only generic** methods because:

- **Attributes are universal**: The system handles any attribute, not domain-specific ones
- **No special cases**: All attributes use the same access pattern
- **Flexibility**: Store any data type (numbers, strings, vectors, colors, etc.)
- **Consistency**: One pattern for all attribute types
- **Simplicity**: No need to learn domain-specific methods

**Example:**
```rust
// All attributes use the same pattern
entity.set_attribute("health", AttributeValue::Number(100.0));
entity.set_attribute("team", AttributeValue::String("red".into()));
entity.set_attribute("position", AttributeValue::Vector3([1.0, 2.0, 3.0]));
entity.set_attribute("ethos", AttributeValue::Number(0.6));

// All retrievals use type-safe helpers
let health = entity.get_f32("health");
let team = entity.get_string("team");
let pos = entity.get_vector3("position");
let ethos = entity.get_f32("ethos");
```

## Reserved Attribute Keys

These keys have special meaning in the Attributes system:

| Key | Type | Purpose |
|-----|------|---------|
| `ethos` | Number | ELP ethos component |
| `logos` | Number | ELP logos component |
| `pathos` | Number | ELP pathos component |
| `confidence` | Number | VCP signal strength (0.0-1.0) |
| `digital_root_flux` | Int | Vortex position (1-9) |
| `flux_position` | Number | Flux position (0-9) |

## Migration Checklist

### Phase 1: Basic Attributes Migration
- [ ] Replace `elp: [f32; 3]` with `Attributes::with_elp(e, l, p)`
- [ ] Replace `envelope.elp.unwrap()[0]` with `envelope.attributes.ethos()`
- [ ] Replace hardcoded ELP access with `get_elp()` / `set_elp()`
- [ ] Add custom attributes for domain-specific data
- [ ] Use Tags for categorization instead of string arrays
- [ ] Update JSON serialization to use `to_json_map()` / `from_json_map()`

### Phase 2: AttributeAccessor Trait Migration
- [ ] Import `AttributeAccessor` trait: `use spatial_vortex::data::AttributeAccessor;`
- [ ] Replace `beam.ethos.unwrap_or(0.33)` with `beam.ethos()`
- [ ] Replace `beam.logos.unwrap_or(0.34)` with `beam.logos()`
- [ ] Replace `beam.pathos.unwrap_or(0.33)` with `beam.pathos()`
- [ ] Replace `elp_tensor.ethos` with `elp_tensor.to_attributes().ethos()`
- [ ] Replace direct field mutations with trait setters: `beam.set_ethos(value)`
- [ ] Implement `AttributeAccessor` for custom types with `attributes` field
- [ ] Use `elp_tensor()` method instead of manual array construction
- [ ] Use `elp_normalized()` for normalized ELP values
- [ ] Use `elp_dominant()` to get dominant channel name

## File Locations

- **Attributes Module**: `src/data/attributes.rs`
- **Data Types**: `src/pipeline/data_types.rs`
- **EustressEngine Reference**: `E:\Workspace\EustressEngine\eustress\crates\common\src\attributes.rs`
- **spatial-llm Types**: `E:\Workspace\EustressEngine\eustress\crates\spatial-llm\src\types.rs`
