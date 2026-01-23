# Data Layer

**Foundation: Semantic Data Structures and Storage**

---

## Overview

The Data Layer provides efficient representation, compression, and persistence of semantic information. All higher layers build on these foundational data structures.

---

## Components

### [Tensors](tensors.md)
Core data structures for semantic representation:
- **BeamTensor**: Words as colored light beams with position, ELP, confidence
- **ELPTensor**: Ethos-Logos-Pathos three-dimensional semantics
- **BeadTensor**: Legacy alias (deprecated)

### [Compression](../../foundations/information-theory/compression.md)
16-byte semantic compression (12 core + 4 schema):
- **WHO** (2 bytes): Entity identifier
- **WHAT** (4 bytes): Concept/action
- **WHERE** (2 bytes): Flux position
- **TENSOR** (2 bytes): ELP channels
- **COLOR** (1 byte): RGB from ELP
- **ATTRS** (1 byte): Metadata
- **SCHEMA REF** (4 bytes): Property schema

**Compression Ratio**: 625:1 (10,000 bytes → 16 bytes)

### [Reference Byte](reference-byte.md)
Property schema system (ORM-like):
- 256 possible schemas
- Variable-length properties
- Type-safe attribute system
- Spatial 3D coordinates
- Query interface

### Storage
**PostgreSQL Confidence Lake**:
- Stores high-confidence moments (signal ≥ 0.7)
- AES-GCM encryption
- Indexed by flux position
- Property-based queries

---

## Key Concepts

### Semantic Links
- **Related**: [ELP Channels](../../concepts/elp-channels.md), [Flux Matrix](../inference-layer/flux-matrix.md)
- **Used By**: All inference and processing layers
- **Foundation**: [Vortex Mathematics](../../foundations/vortex-mathematics/)

### Performance
- **Memory**: 216 bytes per token (with 5 alternatives)
- **Storage**: 16 bytes per compressed concept
- **Query**: <5ms from Confidence Lake
- **Compression**: 625:1 ratio

---

## Data Flow

```
Input Text
    ↓
Tokenization
    ↓
BeamTensor Creation
    ↓
ELP Calculation
    ↓
16-Byte Compression
    ↓
Storage (if signal ≥ 0.7)
```

---

**Navigate**: [← Architecture Overview](../overview.md) | [Inference Layer →](../inference-layer/)
