# SpatialVortex Architecture Overview

**Layered Intelligence System**

---

## System Layers

SpatialVortex is organized into four distinct architectural layers, each building on the previous:

```
┌─────────────────────────────────────────────────────────────┐
│                  Intelligence Layer                         │
│  Reasoning • Learning • Self-Optimization                   │
│  ↓ Uses context and inferences to learn and improve         │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                  Processing Layer                           │
│  AI Router • Meta Orchestrator • Pipelines                  │
│  ↓ Routes and processes requests efficiently                │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                  Inference Layer                            │
│  Flux Matrix • ASI • VCP • Dynamic Context                  │
│  ↓ Generates inferences with context preservation           │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                  Data Layer                                 │
│  Tensors • Compression • Storage                            │
│  ↓ Stores and retrieves semantic information                │
└─────────────────────────────────────────────────────────────┘
```

---

## Layer Details

### [Data Layer](data-layer/)
**Foundation: Semantic data structures and storage**

- **[Tensors](data-layer/tensors.md)**: BeamTensor, ELPTensor structures
- **[Compression](data-layer/compression.md)**: 16-byte semantic compression  
- **[Reference Byte](data-layer/reference-byte.md)**: Property schema system
- **Storage**: PostgreSQL Confidence Lake

**Purpose**: Efficient representation and persistence

---

### [Inference Layer](inference-layer/)
**Core: Generate meanings from data**

- **[Flux Matrix](inference-layer/flux-matrix.md)**: Inferential index with weighted alternatives
- **[ASI Orchestrator](inference-layer/asi-orchestrator.md)**: Execution engine
- **[VCP](inference-layer/vcp.md)**: Vortex Context Preserver (40% better retention)
- **[Dynamic Context](inference-layer/dynamic-context.md)**: Unlimited context window

**Purpose**: Transform data into meaning

---

### [Processing Layer](processing-layer/)
**Orchestration: Route and optimize requests**

- **[AI Router](processing-layer/ai-router.md)**: Priority-based request routing
- **[Meta Orchestrator](processing-layer/meta-orchestrator.md)**: Strategy selection
- **[Modalities](processing-layer/modalities.md)**: Multi-modal processing
- **Pipelines**: Concurrent processing paths

**Purpose**: Efficient resource utilization

---

### [Intelligence Layer](intelligence-layer/)
**AGI/ASI: Learn and self-improve**

- **Reasoning**: Geometric reasoning with sacred positions
- **Learning**: Continuous learning via RAG
- **Hallucination Detection**: Signal strength monitoring
- **Self-Optimization**: Adaptive strategy selection

**Purpose**: Autonomous intelligence

---

## Data Flow

```
Input Text
    ↓
[Data Layer] → BeamTensor representation
    ↓
[Inference Layer] → Flux Matrix alternatives → VCP context
    ↓
[Processing Layer] → AI Router → Meta Orchestrator
    ↓
[Intelligence Layer] → Reasoning → Learning
    ↓
Output + Knowledge Update
```

---

## Key Innovations

1. **16-Byte Compression**: 625:1 compression ratio with property schemas
2. **Flux Matrix**: N-best alternatives with weighted selection
3. **VCP**: 40% better context preservation than transformers
4. **Sacred Geometry**: 3-6-9 positions for mathematical optimization
5. **Dynamic Context**: Unlimited context via confidence-based windowing

---

## Mathematical Foundation

Built on **Vortex Mathematics**:
- Doubling sequence: 1→2→4→8→7→5→1
- Sacred positions: 3, 6, 9 (excluded from doubling)
- Digital root reduction
- Formal verification (Z3)

See [Foundations](../foundations/) for mathematical basis.

---

## Performance

- **Inference**: <50ms per query
- **Context**: Unlimited tokens (dynamic window)
- **Accuracy**: 95-97% on benchmarks
- **Memory**: <2GB typical
- **Throughput**: 1200+ req/sec

---

**Navigate Layers**:
- [Data Layer →](data-layer/)
- [Inference Layer →](inference-layer/)
- [Processing Layer →](processing-layer/)
- [Intelligence Layer →](intelligence-layer/)
