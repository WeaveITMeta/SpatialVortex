# SpatialVortex Terminology

Comprehensive glossary of terms, concepts, and architecture definitions used throughout the SpatialVortex codebase.

---

## Table of Contents

- [Core Concepts](#core-concepts)
- [Data Structures](#data-structures)
- [Compression & Encoding](#compression--encoding)
- [Inference & Processing](#inference--processing)
- [Geometric & Mathematical](#geometric--mathematical)
- [Integration & APIs](#integration--apis)
- [Visualization & Rendering](#visualization--rendering)
- [Semantic & Learning](#semantic--learning)
- [System & Operations](#system--operations)

---

## Core Concepts

### SpatialVortex
The name of the entire AGI consciousness engine. Combines **spatial reasoning** (position-based semantics) with **vortex patterns** (circular/cyclical knowledge flow through the flux matrix). Transforms language into geometric light flowing through sacred patterns.

### Flux Matrix
A 10-position (0-9) semantic knowledge graph representing a subject domain. Each position holds a meaning with semantic associations, predicates, and relations to other nodes. The "flux" refers to how meanings flow and transform through positions following the pattern `1→2→4→8→7→5→1`, while "matrix" indicates the structured grid of interconnected nodes.

**Key Characteristics:**
- **10 positions**: 0-9 (position 0 is neutral center)
- **Sacred positions**: 3, 6, 9 with +15% confidence boost
- **Base pattern**: `[1, 2, 4, 8, 7, 5, 1]` (repeating cycle)
- **Direct mapping**: Digits map directly to their position (no indirection)

### Subject
A domain of knowledge organized as a flux matrix. Examples: "Physics", "Consciousness", "Music Theory", "Ethics". Each subject has its own 10-position matrix with domain-specific meanings at each node. Subjects are the highest organizational unit for semantic knowledge.

### Node / Flux Node
A single position (0-9) within a flux matrix. Contains:
- **Primary meaning** (e.g., "Momentum" at position 1 in Physics)
- **Semantic index** with positive/negative associations
- **Predicates** and **Relations** to other nodes
- **Attributes** (properties, parameters, state, dynamics)
- **Connections** to other nodes (sequential, sacred, semantic, geometric, temporal)

**Note**: The term "diamond node" has been deprecated in favor of "flux node" for clarity.

### Position
The numeric location (0-9) of a node within a flux matrix. Accessed directly by digit:
- **Position 0**: Neutral center / void
- **Positions 1-9**: Semantic nodes with specific meanings
- **Positions 3, 6, 9**: Sacred guides with special processing

---

## Data Structures

### BeamTensor
The primary data structure for representing word/concept flows through the system. Represents words as colored light beams.

**Fields:**
- `digits: [f32; 9]` - Position distribution (softmax probabilities)
- `ethos: f32` - Ethics/stability channel (0-9)
- `logos: f32` - Logic/reasoning channel (0-9)
- `pathos: f32` - Emotion/passion channel (0-9)
- `curviness_signed: f32` - Path curvature (amplitude × sign(pitch_slope))
- `word: String` - The actual word/text
- `position: u8` - Current flux position (0-9)
- `confidence: f32` - Quality score (0.0-1.0)
- `timestamp: f64` - Unix timestamp
- `can_replicate: bool` - Replication eligibility
- `mark_for_confidence_lake: bool` - High-value moment flag

**Formerly called**: BeadTensor (deprecated alias)

### Diamond
A rich memory structure for high-confidence moments, **distinct from the deprecated "diamond visualization" term**. Stored encrypted in the Confidence Lake.

**Fields:**
- Full ELP channel distributions (3 × 9 floats)
- Pitch curve over time
- Transcribed text
- Associated BeamTensor snapshot
- Model version and metadata

**Criteria**: ethos ≥ 8.5 AND logos ≥ 7.0 AND down-tone (negative curviness)

### CompressionHash
16-byte compression structure (128 bits) representing a complete thought:

```
┌─────────────────────────────────────────────────────────────────┐
│                    16-Byte Thought Hash                         │
├───────┬───────┬───────┬───────┬───────┬───────┬─────────────────┤
│ WHO   │ WHAT  │ WHERE │ TENSOR│ COLOR │ ATTRS │ SCHEMA REF      │
│ 2B    │ 4B    │ 2B    │ 2B    │ 1B    │ 1B    │ 4B              │
└───────┴───────┴───────┴───────┴───────┴───────┴─────────────────┘
```

**Core 12 bytes:**
- **WHO** (2 bytes): Entity/Agent identifier
- **WHAT** (4 bytes): Action/Concept/Subject
- **WHERE** (2 bytes): Flux position + coordinates
- **TENSOR** (2 bytes): ELP channels encoded
- **COLOR** (1 byte): RGB blend from ELP
- **ATTRS** (1 byte): Metadata flags

**Schema Reference 4 bytes:**
- **SCHEMA_ID** (1 byte): Property schema (0-255)
- **PROP_COUNT** (1 byte): Number of properties
- **PROP_BITMAP** (2 bytes): 16-bit property flags

**Compression ratio**: 625:1 (10,000 bytes → 16 bytes)

**See also**: [REFERENCE_BYTE.md](../architecture/REFERENCE_BYTE.md) for full property schema details.

---

## Compression & Encoding

### Seed Number (Legacy)
**Deprecated in favor of compression hashes**. Original method for encoding semantic information as numeric values. Seeds are converted to flux sequences through digit reduction: `seed → digits → flux_pattern → positions`.

**Example**: Seed `888` → digits `[8,8,8]` → all activate position 8

**Status**: Maintained for backward compatibility but compression hashes are the preferred method.

### Hash Metadata
Extracted information from a CompressionHash:
- Flux position (0-9)
- ELP channel values
- RGB color components
- Sacred position flag
- Confidence score

### ELP Channels / ELP Values
Three-dimensional sentiment/characteristic analysis:

- **E**thos: Ethics, stability, consistency, character (0-9 scalar)
- **L**ogos: Logic, reasoning, structure, rationality (0-9 scalar)
- **P**athos: Emotion, passion, feeling, empathy (0-9 scalar)

**Visualization**: 
- Red channel = Pathos
- Green channel = Logos
- Blue channel = Ethos

### Curviness Signed
Path curvature for beam visualization: `amplitude × sign(pitch_slope)`

- **Positive**: Rising pitch (upward curve)
- **Negative**: Falling pitch (downward curve, "down-tone")
- **Magnitude**: Strength of curvature

---

## Inference & Processing

### Inference Engine
Core bidirectional reasoning system:
- **Forward inference**: Target meanings → Candidate positions → 16-byte compression hashes
- **Reverse inference**: Compression hashes → Flux sequences → Semantic meanings
- **Legacy support**: Seed numbers maintained for backward compatibility only

### Inference Input
Request structure supporting both modern and legacy methods:
- `compression_hashes: Vec<String>` - **Preferred**: 16-byte hashes (12 core + 4 schema)
- `seed_numbers: Vec<u64>` - **Legacy**: Backward compatibility
- `subject_filter: SubjectFilter` - Which subjects to query
- `processing_options: ProcessingOptions` - Configuration

### Inference Result
Output containing:
- Matched flux matrices
- Inferred meanings with confidence scores
- Processing time
- Hash metadata (if using hashes)
- Moral alignment classifications

### Processing Options
Configuration for inference:
- `include_synonyms: bool`
- `include_antonyms: bool`
- `max_depth: u8`
- `confidence_threshold: f32`
- `use_sacred_guides: bool` - Enable +15% boost

---

## Geometric & Mathematical

### Sacred Positions / Sacred Guides
Positions **3, 6, and 9** with special geometric and mathematical significance:

| Position | Meaning | Processing Effect | Boost |
|----------|---------|-------------------|-------|
| **3** | Good/Easy | Positive reinforcement, fast paths | +15% confidence |
| **6** | Bad/Hard | Challenge processing, error correction | +15% confidence |
| **9** | Divine/Righteous | Truth validation, consciousness emergence | +15% confidence + Confidence Lake trigger |

**Geometric Significance**: Form the "Sacred Triangle" with cyan-colored connections in visualizations.

### Flux Pattern / Base Pattern
The fundamental repeating cycle: **`1 → 2 → 4 → 8 → 7 → 5 → 1`**

Generated by doubling and reducing:
- 1 × 2 = 2
- 2 × 2 = 4
- 4 × 2 = 8
- 8 × 2 = 16 → 1+6 = 7
- 7 × 2 = 14 → 1+4 = 5
- 5 × 2 = 10 → 1+0 = 1 (cycle completes)

**Note**: Positions 3, 6, 9 are NOT part of the doubling sequence - they are sacred guides.

### ChangeDot / ChangeDotEvent
Iterator system for stepping through the flux pattern:

**Events:**
- `Step` - Movement from one position to another
- `SacredHit` - Encounter with position 3, 6, or 9 (every 3 steps)
- `Loop` - Completion of a cycle (length 6)

### Entropy Loop
Processing mechanism following `y = x²` dynamics. Words find their optimal positions through iterative entropy reduction, converging on meaning.

### Geometric Significance
The spatial and mathematical properties that make positions 3-6-9 special:
- Tesla's "keys to the universe"
- Form equilateral triangle in geometric space
- Intersection points for multi-dimensional processing

---

## Integration & APIs

### AI Router
Multi-type request management system with priority queuing.

**Request Types** (priority order):
1. **Priority** - Emergency, critical operations (5s timeout)
2. **Compliance** - Content moderation, policy enforcement (10s timeout)
3. **User** - Interactive chat, queries (30s timeout)
4. **System** - Health checks, diagnostics (60s timeout)
5. **Machine** - API calls, automation (120s timeout)

### AI Request
Structured request with:
- Request type
- Prompt/query
- User/API identification
- Compression settings
- Metadata
- Timeout configuration

### AI Response
Response containing:
- Generated text
- Compression hash
- Beam position
- ELP channels
- Confidence score
- Processing metadata

### Subject Filter
Query scope for inference:
- `Specific(String)` - Single subject
- `GeneralIntelligence` - Cross-subject reasoning
- `Category(String)` - Subject category
- `All` - All available subjects

---

## Visualization & Rendering

### Flux Matrix Visualization (formerly "Diamond Viz")
Interactive 3D visualization of the flux pattern using Bevy engine. **Binary renamed from `diamond_viz` to `flux_matrix`** for consistency.

**Features:**
- Word beams as colored light
- Sacred positions highlighted
- Interactive camera controls
- Real-time beam flow animation

### Flux Mesh (formerly "Diamond Mesh")
**Module renamed from `diamond_mesh` to `flux_mesh`**. 3D geometric representation of the flux pattern:
- Vertices at 10 positions (0-9)
- Edges showing connections
- Sacred triangle (3-6-9) with special rendering

### FluxGeometry (formerly DiamondGeometry)
**Struct renamed for consistency**. Geometric structure containing:
- Vertex positions in 3D space
- Edge connections
- Sacred position markers
- Helper methods for beam path calculation

### Beam Properties
Visual characteristics for rendering:
- `width` - Beam thickness (from confidence)
- `length` - Beam extension (from decisiveness)
- `wobble` - Pathos-induced oscillation
- `orbit_radius` - Logos-based structure
- `rotation_speed` - Ethos consistency
- `color: [f32; 3]` - RGB from ELP channels

### Alpha Factors
Beam behavior parameters:
- `semantic_mass` - Weight affecting gravity
- `temporal_decay` - Relevance fade rate
- `intersection_pull` - Attraction to 3-6-9 (default: 2.5)
- `entropy_gradient` - Rate of entropy change
- `confidence_momentum` - Velocity scaling

---

## Semantic & Learning

### Semantic Index
Knowledge structure at each node containing:
- **Positive associations** (synonyms, +1 and above, "Heaven")
- **Negative associations** (antonyms, -1 and below, "Hell")
- **Neutral base** (core meaning at index 0)
- **Predicates** (relation verbs)
- **Relations** (subject-predicate-object triples)

### Semantic Association
Individual word relationship:
- `word: String` - The associated word
- `index: i16` - Signed position (positive = synonym, negative = antonym)
- `confidence: f32` - ML confidence score
- `context: String` - Subject context
- `source: AssociationSource` - Origin (User, AI, ML, Manual, RL)

### Ladder / Ladder Index
**Not "Index" alone** - the full term is "Ladder Index". Hierarchical similarity detection system:
- Rungs contain positive/negative word groupings
- Tests word similarity, antonym relationships, and semantic distance
- Returns `SimilarityResult` (Similar, Antonym, Different with scores)

### Predicate
Relation verb/connector in semantic triples:
- `name` - Relation name (e.g., "causes", "is_a", "part_of")
- `index: i16` - Signed ladder position
- `weight: f32` - Ranking/traversal weight (0.0-1.0)
- `ladder_rank: u8` - Priority (lower = higher priority)
- `target_position: Option<u8>` - Linked node position

### Relation
Complete semantic triple: **Subject (node) → Predicate → Object (node)**
- Connects nodes within or across subjects
- Weighted for importance
- Supports confidence scoring
- Context-tagged

### Moral Alignment
Ethical AI reasoning classification:
- `Constructive(f32)` - **Heaven**: Positive influence, beneficial
- `Destructive(f32)` - **Hell**: Negative influence, harmful
- `Neutral` - Balanced state, no clear alignment

### Association Source
Origin of semantic knowledge:
- `UserInput` - Human-provided
- `AIGenerated` - From language models
- `MachineLearning` - ML-derived patterns
- `ManualCuration` - Expert curation
- `ReinforcementLearning` - RL optimization

---

## System & Operations

### Confidence Lake
**Planned/Speculative**: Encrypted, append-only storage for high-confidence "diamond moments". 

**Trigger Criteria**: ethos ≥ 8.5 AND logos ≥ 7.0 AND down-tone (falling pitch)

**Design**:
- AES-GCM encryption
- Device-bound key via DPAPI
- Mmap-indexed for fast retrieval
- Stores full Diamond structures

**Status**: Specification complete, implementation pending

### Subject Generator / Subject CLI
Command-line tool for creating new subject matrices:
```bash
cargo run --bin subject_cli -- generate --subject "Chemistry"
```

Uses AI integration to populate semantic associations dynamically.

### Subject Definition
Template for creating flux matrices:
- Node definitions (position + name)
- Sacred guide definitions
- Semantic associations fetched dynamically via API

### Connection Type
Relationship types between nodes:
- `Sequential` - Following flux pattern (1→2→4→8→7→5→1)
- `Sacred` - Connection to sacred guide (3, 6, or 9)
- `Semantic` - Meaning-based relationship
- `Geometric` - Spatial relationship
- `Temporal` - Time-based sequence

### Node State
Dynamic processing state:
- `active: bool` - Currently processing
- `last_accessed: DateTime<Utc>` - Last use timestamp
- `usage_count: u64` - Access counter
- `context_stack: Vec<String>` - Processing context

### Node Dynamics
Behavioral learning patterns:
- `evolution_rate: f32` - Change velocity
- `stability_index: f32` - Consistency measure
- `interaction_patterns: Vec<String>` - Usage patterns
- `learning_adjustments: Vec<LearningAdjustment>` - RL history

### Learning Adjustment
Reinforcement learning modification:
- Type: ConfidenceBoost, ConfidenceReduction, SemanticRefinement, etc.
- Magnitude of adjustment
- Timestamp
- Rationale explanation

---

## Deprecated Terms

### ❌ Diamond Visualization → ✅ Flux Matrix Visualization
The 3D visualization binary and related code have been renamed from "diamond" to "flux matrix" for consistency.

**Old names**:
- `diamond_viz` binary
- `diamond_mesh.rs` module  
- `DiamondGeometry` struct
- `DiamondNode` TypeScript interface

**New names**:
- `flux_matrix` binary
- `flux_mesh.rs` module
- `FluxGeometry` struct
- `FluxNode` TypeScript interface

**Exception**: The `Diamond` struct in `models.rs` is **preserved** - it refers to high-confidence moments in the Confidence Lake, a different concept.

### ❌ BeadTensor → ✅ BeamTensor
Renamed to better represent the visual nature (light beams). `BeadTensor` maintained as type alias for backward compatibility.

### ❌ Seed Numbers (Primary) → ✅ Compression Hashes (Preferred)
Seed numbers are legacy. New code should use 16-byte compression hashes (12 core + 4 schema reference) for better encoding and metadata support.

---

## Quick Reference

### Sacred Numbers
- **3**: Good/Easy, green light
- **6**: Bad/Hard, red light
- **9**: Divine/Righteous, blue light
- **+15% boost** at these positions

### Flux Cycle
`1 → 2 → 4 → 8 → 7 → 5 → 1` (length 6)

### ELP Color Mapping
- **E**thos → **Blue**
- **L**ogos → **Green**
- **P**athos → **Red**

### Compression Sizes
- **3 bytes**: Basic hash (24 bits)
- **12 bytes**: Core semantic hash (96 bits)
- **16 bytes**: Full hash with schema (128 bits)
- **Ratio**: 625:1 (10,000 → 16 bytes)

### Request Priority
Priority > Compliance > User > System > Machine

### Moral Index
- **Positive (+)**: Heaven, Constructive
- **Negative (-)**: Hell, Destructive
- **Zero (0)**: Neutral base

---

## Architectural Principles

1. **Direct Mapping**: Positions 0-9 map to themselves (no indirection)
2. **Sacred Geometry**: Positions 3-6-9 have special processing significance
3. **Bidirectional**: Both forward (meaning → seed) and reverse (seed → meaning) inference
4. **Compressed**: 16-byte hashes enable 625:1 compression ratio with property schemas
5. **Encrypted**: Confidence Lake uses AES-GCM for privacy
6. **Dynamic**: Semantic associations fetched via AI/API, not hardcoded
7. **Moral**: Positive/negative reinforcement enables ethical reasoning
8. **Visual**: Words become colored light beams in 3D space

---

## For Developers

### Key Files
- `src/models.rs` - Core data structures
- `src/flux_matrix.rs` - Flux pattern engine
- `src/data/compression/asi_12byte.rs` - 16-byte hash system (12 core + 4 schema)
- `src/inference_engine.rs` - Bidirectional reasoning
- `src/data/beam_tensor.rs` - Light beam representation
- `src/ai/router.rs` - Request management
- `src/bin/flux_matrix.rs` - 3D visualization

### Important Distinctions
- **Flux Matrix** (the system) vs **flux_matrix** (the binary)
- **Diamond** (high-confidence moment) vs **Diamond Viz** (deprecated term for visualization)
- **Ladder Index** (full term) not just "Index"
- **Seed numbers** (legacy) vs **Compression hashes** (current)
- **Heaven/Hell** (informal) = **Constructive/Destructive** (formal)

### Migration Notes
If upgrading from old code:
- Replace `diamond_viz` → `flux_matrix` in build commands
- Update imports: `diamond_mesh` → `flux_mesh`
- Update types: `DiamondGeometry` → `FluxGeometry`
- Prefer `compression_hashes` over `seed_numbers` in new code
- Use `BeamTensor` (not `BeadTensor`) in new code

---

## See Also

- [Architecture Documentation](docs/architecture/) - Technical specifications
- [MIGRATION.md](MIGRATION.md) - Upgrade guide for breaking changes
- [BUILD_COMMANDS.md](BUILD_COMMANDS.md) - Build and run instructions
- [CONTRIBUTING.md](CONTRIBUTING.md) - Development guidelines

---

**Last Updated**: October 22, 2025  
**Version**: 1.1.0

