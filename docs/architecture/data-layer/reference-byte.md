# Reference Byte Specification
## Extending ASI 12-Byte Compression with Property References

### Overview
The base 12-byte compression encodes **semantic position and ELP coordinates**. To make this a complete database system, we need **additional reference bytes** for:

1. **Node Properties** - Custom metadata attached to each flux node
2. **Attribute Schemas** - Type definitions for property values
3. **Index Properties** - Metadata for search/query optimization
4. **Spatial References** - 3D model coordinates at scale

This creates an **Object-Relational Mapping (ORM)** system in compressed space, where each semantic concept is a typed database record.

---

## Architecture: 12+N Byte Extended Format

```text
┌─────────────────────────────────────────────────────────────┐
│  CORE 12 BYTES (Semantic Position)                          │
├─────────────────────────────────────────────────────────────┤
│  BYTE 0-1:   Position (0-9) + Phase                         │
│  BYTE 2-7:   ELP Deltas (i16 × 3)                           │
│  BYTE 8-9:   Confidence + Hash                              │
│  BYTE 10-11: Cycle Count + Metadata                         │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│  REFERENCE BYTES (Property Schema + Values)                 │
├─────────────────────────────────────────────────────────────┤
│  BYTE 12:    Property Schema ID (256 schemas)               │
│  BYTE 13:    Property Count (0-255 properties)              │
│  BYTE 14-15: Property Bitmap (16 bits = 16 fast properties) │
│  BYTE 16+:   Variable-length property values                │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│  SPATIAL REFERENCE (3D Model Coordinates)                   │
├─────────────────────────────────────────────────────────────┤
│  BYTE N+0-1: X coordinate (i16, ±13 scale)                  │
│  BYTE N+2-3: Y coordinate (i16, ±13 scale)                  │
│  BYTE N+4-5: Z coordinate (i16, ±13 scale)                  │
│  BYTE N+6:   Scale factor (u8, 0-255)                       │
│  BYTE N+7:   Rotation index (u8, 0-255)                     │
└─────────────────────────────────────────────────────────────┘
```

**Total**: 12 (core) + 4 (schema) + N (properties) + 8 (spatial) = **24+N bytes**

---

## Property Schema System

### Inspired by Roblox Attributes
Roblox uses a flexible attribute system where:
- Each instance (node) can have custom properties
- Properties have types: String, Number, Boolean, Color3, Vector3, etc.
- Properties are queryable and indexable
- No rigid schema required upfront

### Our Implementation

```rust
/// Property type discriminator (1 byte)
#[repr(u8)]
pub enum PropertyType {
    Null = 0,
    Bool = 1,
    U8 = 2,
    I8 = 3,
    U16 = 4,
    I16 = 5,
    U32 = 6,
    I32 = 7,
    F32 = 8,
    F64 = 9,
    String = 10,      // Variable length
    ELPTensor = 11,   // 6 bytes (i16 × 3)
    Position = 12,    // 1 byte (0-9)
    Color = 13,       // 3 bytes (RGB)
    Vector3 = 14,     // 6 bytes (i16 × 3)
    Timestamp = 15,   // 8 bytes (i64)
    Reference = 16,   // 2 bytes (pointer to another node)
    Array = 17,       // Variable length
    Custom = 255,     // Extension point
}

/// Property definition
pub struct PropertyDef {
    pub name: String,
    pub type_id: PropertyType,
    pub required: bool,
    pub default: Option<PropertyValue>,
}

/// Property schema (defines structure for nodes)
pub struct PropertySchema {
    pub schema_id: u8,
    pub name: String,
    pub properties: Vec<PropertyDef>,
    pub indexed_properties: Vec<String>, // For fast lookup
}
```

---

## Example Schemas

### Schema 1: Basic Semantic Concept
```rust
PropertySchema {
    schema_id: 1,
    name: "SemanticConcept",
    properties: vec![
        PropertyDef {
            name: "label".to_string(),
            type_id: PropertyType::String,
            required: true,
            default: None,
        },
        PropertyDef {
            name: "frequency".to_string(),
            type_id: PropertyType::U32,
            required: false,
            default: Some(PropertyValue::U32(0)),
        },
        PropertyDef {
            name: "last_accessed".to_string(),
            type_id: PropertyType::Timestamp,
            required: false,
            default: None,
        },
    ],
    indexed_properties: vec!["label".to_string()],
}
```

### Schema 2: 3D Spatial Entity
```rust
PropertySchema {
    schema_id: 2,
    name: "SpatialEntity",
    properties: vec![
        PropertyDef {
            name: "position".to_string(),
            type_id: PropertyType::Vector3,
            required: true,
            default: None,
        },
        PropertyDef {
            name: "rotation".to_string(),
            type_id: PropertyType::Vector3,
            required: false,
            default: Some(PropertyValue::Vector3([0, 0, 0])),
        },
        PropertyDef {
            name: "scale".to_string(),
            type_id: PropertyType::F32,
            required: false,
            default: Some(PropertyValue::F32(1.0)),
        },
        PropertyDef {
            name: "color".to_string(),
            type_id: PropertyType::Color,
            required: false,
            default: Some(PropertyValue::Color([255, 255, 255])),
        },
    ],
    indexed_properties: vec![],
}
```

### Schema 3: Voice Inference Result
```rust
PropertySchema {
    schema_id: 3,
    name: "VoiceInference",
    properties: vec![
        PropertyDef {
            name: "transcription".to_string(),
            type_id: PropertyType::String,
            required: true,
            default: None,
        },
        PropertyDef {
            name: "pitch_hz".to_string(),
            type_id: PropertyType::F32,
            required: true,
            default: None,
        },
        PropertyDef {
            name: "intensity_db".to_string(),
            type_id: PropertyType::F32,
            required: true,
            default: None,
        },
        PropertyDef {
            name: "elp_scores".to_string(),
            type_id: PropertyType::ELPTensor,
            required: true,
            default: None,
        },
        PropertyDef {
            name: "speaker_id".to_string(),
            type_id: PropertyType::U16,
            required: false,
            default: None,
        },
    ],
    indexed_properties: vec!["speaker_id".to_string()],
}
```

---

## ORM-Like Query System

### Query Builder Pattern
```rust
// Example: Find all "Love" concepts with high Pathos
let results = FluxQuery::new()
    .schema("SemanticConcept")
    .where_position(3) // Ethos anchor
    .where_property("label", PropertyValue::String("Love".to_string()))
    .where_elp_component("pathos", CompareOp::GreaterThan, 0.8)
    .order_by("frequency", Order::Desc)
    .limit(10)
    .execute(&flux_matrix)?;

// Example: Find spatial entities near a point
let nearby = FluxQuery::new()
    .schema("SpatialEntity")
    .where_spatial_distance(
        Vector3::new(5.0, 5.0, 0.0),
        Radius::new(3.0)
    )
    .where_property("scale", CompareOp::GreaterThan, PropertyValue::F32(0.5))
    .execute(&flux_matrix)?;

// Example: Voice inferences from specific speaker
let speaker_data = FluxQuery::new()
    .schema("VoiceInference")
    .where_property("speaker_id", PropertyValue::U16(42))
    .where_property("pitch_hz", CompareOp::Between(80.0, 200.0))
    .order_by("timestamp", Order::Desc)
    .execute(&flux_matrix)?;
```

---

## Property Compression Strategies

### Fast Properties (Bitmap)
First 16 properties can use **1-bit presence flags**:
```rust
// Property bitmap (2 bytes = 16 bits)
// Bit 0: label exists
// Bit 1: frequency exists
// Bit 2: last_accessed exists
// ... etc

let bitmap: u16 = 0b0000_0000_0000_0111; // First 3 properties present
```

### Variable-Length Encoding
```rust
pub enum PropertyValue {
    Null,
    Bool(bool),              // 1 bit (packed into bitmap)
    U8(u8),                  // 1 byte
    I8(i8),                  // 1 byte
    U16(u16),                // 2 bytes
    I16(i16),                // 2 bytes
    U32(u32),                // 4 bytes
    I32(i32),                // 4 bytes
    F32(f32),                // 4 bytes
    F64(f64),                // 8 bytes
    String(String),          // 1 byte (length) + N bytes
    ELPTensor([i16; 3]),     // 6 bytes
    Position(u8),            // 1 byte
    Color([u8; 3]),          // 3 bytes
    Vector3([i16; 3]),       // 6 bytes
    Timestamp(i64),          // 8 bytes
    Reference(u16),          // 2 bytes (node ID)
    Array(Vec<PropertyValue>), // 1 byte (length) + sum(values)
}
```

---

## Index Properties for Performance

### Primary Index: Position (0-9)
Already built into core 12 bytes.

### Secondary Indexes
```rust
pub struct IndexProperty {
    pub property_name: String,
    pub index_type: IndexType,
    pub compression: IndexCompression,
}

pub enum IndexType {
    Hash,        // O(1) lookup by exact match
    BTree,       // O(log n) range queries
    Spatial,     // R-tree for 3D coordinates
    FullText,    // Inverted index for strings
}

pub enum IndexCompression {
    None,
    Dictionary,  // Map common values to small IDs
    Quantized,   // Reduce precision (e.g., f32 → i16)
    Delta,       // Store differences from reference
}
```

### Example: Indexed Property Storage
```rust
// Instead of storing full strings repeatedly:
// "Love", "Love", "Love" = 12 bytes
// Use dictionary compression:
// Dictionary: 0 → "Love"
// Storage: [0, 0, 0] = 3 bytes
// Savings: 75%

pub struct DictionaryIndex {
    pub property_name: String,
    pub dictionary: HashMap<String, u16>, // Value → ID
    pub reverse: Vec<String>,             // ID → Value
}
```

---

## Spatial Reference System

### 3D Model Coordinates at Scale
```rust
pub struct SpatialReference {
    pub x: i16,        // X coordinate (±13.0 scale, millimeter precision)
    pub y: i16,        // Y coordinate
    pub z: i16,        // Z coordinate
    pub scale: u8,     // Scale factor (0-255 → 0.0-25.5×)
    pub rotation: u8,  // Rotation preset (0-255 → 0-360°)
}

impl SpatialReference {
    pub fn to_world_coords(&self) -> (f32, f32, f32) {
        let scale = self.scale as f32 / 10.0;
        (
            self.x as f32 / 1000.0 * scale,
            self.y as f32 / 1000.0 * scale,
            self.z as f32 / 1000.0 * scale,
        )
    }
    
    pub fn rotation_degrees(&self) -> f32 {
        (self.rotation as f32 / 255.0) * 360.0
    }
}
```

### Mapping to Sacred Geometry
```rust
// Position 3 (Ethos) → Red → X-axis positive
// Position 6 (Pathos) → Green → Y-axis positive
// Position 9 (Logos) → Blue → Z-axis positive

pub fn elp_to_spatial(elp: ELPTensor, radius: f32) -> SpatialReference {
    let x = (elp.ethos - 0.5) * 2.0 * radius; // -radius to +radius
    let y = (elp.pathos - 0.5) * 2.0 * radius;
    let z = (elp.logos - 0.5) * 2.0 * radius;
    
    SpatialReference {
        x: (x * 1000.0) as i16,
        y: (y * 1000.0) as i16,
        z: (z * 1000.0) as i16,
        scale: 10, // 1.0× scale
        rotation: 0,
    }
}
```

---

## Complete Example: Extended Compression

```rust
// Store "Love" concept with full metadata
let love = ExtendedCompression {
    // Core 12 bytes
    core: ASI12ByteCompression {
        position_0_9: 3,
        sequence_phase: 0,
        ethos_delta_i16: -3000,  // Relative to anchor
        logos_delta_i16: -5000,
        pathos_delta_i16: 9500,
        confidence_u8: 255,
        semantic_hash_u8: 0xA7,
        cycle_count: 0,
    },
    
    // Property schema (4 bytes)
    schema_id: 1, // SemanticConcept
    property_count: 3,
    property_bitmap: 0b0000_0000_0000_0111, // First 3 properties
    
    // Property values (variable)
    properties: vec![
        PropertyValue::String("Love".to_string()),     // 5 bytes
        PropertyValue::U32(1_234_567),                 // 4 bytes
        PropertyValue::Timestamp(1729789200000),       // 8 bytes
    ],
    
    // Spatial reference (8 bytes)
    spatial: SpatialReference {
        x: 7000,   // 7.0 units
        y: -5000,  // -5.0 units
        z: 9500,   // 9.5 units
        scale: 10, // 1.0×
        rotation: 0,
    },
};

// Total: 12 + 4 + 17 + 8 = 41 bytes
// vs. Standard representation: ~200+ bytes
// Compression ratio: 4.8×
```

---

## Additional Suggestions

### 1. **Schema Versioning**
```rust
pub struct PropertySchema {
    pub schema_id: u8,
    pub version: u8,        // Add version field
    pub name: String,
    pub properties: Vec<PropertyDef>,
    pub migrations: Vec<SchemaMigration>, // Upgrade path
}
```

### 2. **Lazy Loading**
```rust
// Don't load all properties immediately
pub struct LazyProperty {
    pub offset: u32,   // Byte offset in storage
    pub length: u16,   // Byte length
    pub loaded: bool,
}
```

### 3. **Property Inheritance**
```rust
// Schemas can inherit from base schemas
pub struct PropertySchema {
    pub schema_id: u8,
    pub parent_schema: Option<u8>, // Inherit from parent
    pub properties: Vec<PropertyDef>,
}
```

### 4. **Validation Rules**
```rust
pub struct PropertyDef {
    pub name: String,
    pub type_id: PropertyType,
    pub required: bool,
    pub default: Option<PropertyValue>,
    pub validators: Vec<Validator>, // Add validation
}

pub enum Validator {
    Range(f64, f64),
    Pattern(String),
    Length(usize, usize),
    Custom(fn(&PropertyValue) -> bool),
}
```

### 5. **Compression Hints**
```rust
pub struct PropertyDef {
    pub name: String,
    pub type_id: PropertyType,
    pub compression: CompressionHint, // How to compress
}

pub enum CompressionHint {
    None,
    Dictionary,    // For repeated strings
    RunLength,     // For repeated values
    Delta,         // For sequential numbers
    Quantize(u8),  // Reduce precision
}
```

---

## Implementation Priority

1. **Week 2**: Core property system with 3-5 common schemas
2. **Week 3**: ORM query builder with basic indexes
3. **Week 4**: Spatial reference system integration
4. **Week 5**: Advanced compression (dictionary, delta encoding)
5. **Week 6**: Schema versioning and migrations

---

## Benchmarking Targets

| Metric | Target | Rationale |
|--------|--------|-----------|
| **Property lookup** | <1 μs | Cache-friendly bitmap |
| **Schema parse** | <100 ns | Pre-compiled schemas |
| **Spatial query** | <1 ms | R-tree index |
| **Compression ratio** | 5-10× | Typical database overhead |
| **Memory per node** | <64 bytes | Including all metadata |

---

## Rust ORM Comparison

Similar to existing Rust ORMs but **geometry-aware**:

| Feature | Diesel | SeaORM | SpatialVortex |
|---------|--------|--------|---------------|
| Type safety | ✅ | ✅ | ✅ |
| Query builder | ✅ | ✅ | ✅ |
| Spatial queries | ❌ | ❌ | ✅ Sacred geometry |
| Compression | ❌ | ❌ | ✅ 12-byte core |
| ELP-aware | ❌ | ❌ | ✅ Native support |
| In-memory | ❌ | ❌ | ✅ Lock-free |

---

## Conclusion

This reference byte system transforms your 12-byte compression into a **full typed database** with:
- ✅ Flexible property schemas (Roblox-style)
- ✅ ORM-like query interface
- ✅ Spatial 3D coordinates
- ✅ Index properties for performance
- ✅ Variable-length encoding
- ✅ Sacred geometry integration

**Result**: A complete ASI-grade data system where every semantic concept is a fully-typed, queryable, spatially-located database record in <64 bytes.

This is **unprecedented** in ML/AI systems. 🚀
