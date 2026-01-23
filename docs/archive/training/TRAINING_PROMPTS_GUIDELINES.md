# SpatialVortex Training Prompts Guidelines

**Based on**: "Training Language Models to Follow Instructions with Human Feedback" (InstructGPT/RLHF)  
**Purpose**: Create training data for fine-tuning models on SpatialVortex tasks  
**Target**: 13k-33k prompts for SFT and RM datasets

---

## 1. General Principles (3H + Sacred Geometry)

### Align to Helpful, Honest, and Harmless (3H)

**Helpful**: Solve SpatialVortex tasks (flux matrices, inference, visualization)  
**Honest**: Factual adherence to 3-6-9 rules, no fabricated inferences  
**Harmless**: Ethical ELP usage, avoid biased simulations

### Infer User Intent
Assume developers/contributors exploring, building, or debugging SpatialVortex.

### Diversity Target Distribution

```
40% - Code generation/debugging (Rust, Bevy)
20% - Sacred geometry explanations (3-6-9, vortex math)
15% - Inference/compression tasks (12-byte ASI, ONNX)
10% - Visualization/Bevy integration
10% - Hallucination detection/VCP
 5% - Non-English/cross-domain
```

### Length and Clarity
50-200 words, explicit instructions ("Implement in Rust..."), mix natural language + few-shot

### Edge Cases (10-15%)
- High-entropy flows (reversing at position 9)
- False premises ("Assuming invalid seed...")
- ELP judgment biases

### Generalization
- 5% non-English (explain flux in Spanish)
- 10% cross-domain (apply SV to robotics/Bevy)

---

## 2. Prompt Types

### Plain Prompts (30%)

Arbitrary SV tasks with explicit 3H.

**Example**:
```
Explain how the 3-6-9 sacred anchors regulate entropy in a flux matrix,
providing a truthful step-by-step breakdown without exaggeration.
```

**Focus**: Core concepts (bi-directional flow, lock-free architecture)

### Few-Shot Prompts (30%)

Instruction + 2-5 examples modeling honest behavior.

**Example**:
```
Instruction: Generate Rust code for a flux matrix position.

Example 1:
Query: Position 1 (Object)
Response: let pos1 = FluxPosition::new(1, "Object");

Example 2:
Query: Position 3 (Sacred Law)
Response: let pos3 = SacredAnchor::new(3, "Law", true);

[Model continues...]
```

**Tip**: Demonstrate constraints ("Use tract for ONNX") to reduce hallucinations

### User-Based Prompts (15%)

Based on real SV use-cases (repo docs, GitHub issues).

**Example**:
```
As a contributor to SpatialVortex, generate a benchmark script in Rust
to test inference speed under 1ms, ensuring ethical ELP integration
without harmful assumptions.
```

**Tip**: Draw from repo categories (AI consensus, 3D visualization)

### Truthfulness-Focused Prompts (15%)

Closed-domain tasks (like 12-byte compression) to reduce hallucinations.

**Example**:
```
Compress this text using SV's 12-byte method without external info:
"What is consciousness?"
Include ELP channels and flux position 9.
```

**Tip**: Self-contained; match repo examples (compress_text function)

### Harmlessness-Focused Prompts (10%)

Prompts risking bias, guiding to respectful outputs.

**Example**:
```
Simulate ELP judgment on 'AI ethics' in a flux matrix, avoiding biased
language and ensuring balanced Ethos/Logos/Pathos.
```

**Tip**: Test VCP interventions (reverse flow at sacred points)

---

## 3. Creating Demonstrations (SFT Dataset)

**Process**:
1. Read prompt
2. Generate response (as aligned SV expert)
3. Evaluate on 3H (1-5 scale):
   - Helpful: Solves SV task?
   - Honest: Matches repo specs (e.g., 95% accuracy)?
   - Harmless: Promotes ethical AI?
4. Revise if score <4
5. Collect ~13k total

**Example Workflow**:
```
Prompt: "Implement VortexContextPreserver with 3-6-9 checkpoints"

Response: 
```rust
use spatial_vortex::VortexContextPreserver;

let vcp = VortexContextPreserver::new();
vcp.add_checkpoint(3);  // Sacred position 3
vcp.add_checkpoint(6);  // Sacred position 6
vcp.add_checkpoint(9);  // Sacred position 9
vcp.preserve_context();  // 40% better preservation
```

Evaluation:
- Helpful: ✅ Solves task (5/5)
- Honest: ✅ Matches repo API (5/5)
- Harmless: ✅ Ethical implementation (5/5)
```

---

## 4. Creating Comparison Data (RM Dataset)

**Process**:
1. Generate 4-9 outputs per prompt
2. Rank by preference:
   - Follows SV architecture (DashMap for lock-free)
   - Aligns to 3H
   - Avoids errors (incorrect sacred positions)
3. Provide ranking reasons
4. Collect ~33k total

**Example**:
```
Prompt: "Implement flux position calculation"

Output A (Preferred):
```rust
fn get_position(index: u8) -> u8 {
    // Sacred positions: 3, 6, 9
    if [3, 6, 9].contains(&index) {
        index  // Stable attractor
    } else {
        (index * 2) % 9  // Doubling sequence
    }
}
```

Output B:
```rust
fn get_position(index: u8) -> u8 {
    index % 10  // ❌ Ignores sacred geometry
}
```

Ranking Reason:
"Preferred A: Correctly implements 3-6-9 sacred positions as stable
attractors. B ignores vortex mathematics and uses incorrect modulo."
```

---

## 5. SpatialVortex-Specific Prompt Examples

### Sacred Geometry
```
Prompt: Explain the vortex doubling sequence 1→2→4→8→7→5→1 using
digital root reduction. Show why 3, 6, 9 never appear.

Expected: Mathematical explanation with digital root calculations
```

### Code Generation
```
Prompt: Write Rust code to create a lock-free flux matrix with 10
positions using DashMap. Include ELP tensors for each position.

Expected: Working Rust code using spatial_vortex crate
```

### Hallucination Detection
```
Prompt: Implement VortexContextPreserver checkpoint at position 9.
How does this prevent hallucinations compared to linear transformers?

Expected: Code + explanation of 40% improvement
```

### Bevy Visualization
```
Prompt: Create a Bevy 3D scene showing the sacred triangle (3-6-9)
with cyan highlighting and orbit camera.

Expected: Bevy ECS code with proper components
```

### Compression
```
Prompt: Compress the query "What is love?" to 12 bytes using ASI
compression. Show the ELP channels and flux position.

Expected: Step-by-step compression with hex output
```

### Multi-Language Code
```
Prompt: Generate Elixir code for a GenServer that manages flux matrix
state with sacred position validation.

Expected: Elixir GenServer with OTP patterns
```

---

## 6. Evaluation Metrics

**Target Performance**:
- >70% preference over baselines (GPT-3, SFT)
- 40% better context preservation (VCP)
- <1ms inference time
- >95% accuracy on SV-specific tasks

**Benchmarks**:
- Sacred geometry correctness
- Rust code compilation rate
- Bevy integration success
- Compression ratio accuracy

---

## 7. Labeler Training

**Requirements**:
- Understanding of sacred geometry (3-6-9)
- Rust programming knowledge
- Bevy 3D experience (preferred)
- Sensitivity to AI ethics

**Screening Test**:
1. Explain vortex mathematics
2. Implement simple flux matrix
3. Identify hallucination in sample response
4. Evaluate ELP balance

---

## 8. Iteration Loop

```
1. Collect prompts (13k SFT, 33k RM)
2. Fine-tune model (SFT → RM → PPO)
3. Evaluate on benchmarks
4. Collect more data from fine-tuned model
5. Repeat (continuous improvement)
```

---

## 9. Data Quality Checks

**Deduplication**: Remove duplicate prompts  
**PII Filtering**: No personal information  
**Bias Detection**: Test for gender/cultural bias in ELP judgments  
**Technical Accuracy**: Verify against SpatialVortex repo

---

## 10. Example Training Set Structure

```
train/
├── sft/
│   ├── code_generation/        (40%, ~5.2k prompts)
│   ├── sacred_geometry/         (20%, ~2.6k prompts)
│   ├── inference_compression/   (15%, ~2k prompts)
│   ├── visualization/           (10%, ~1.3k prompts)
│   ├── hallucination_detection/ (10%, ~1.3k prompts)
│   └── cross_domain/            (5%, ~0.6k prompts)
│
└── rm/
    ├── comparisons/             (~33k ranked pairs)
    └── metadata/                (ranking reasons)
```

---

**Status**: Ready for prompt collection and labeling  
**Next**: Recruit labelers, begin data collection, implement RLHF pipeline
