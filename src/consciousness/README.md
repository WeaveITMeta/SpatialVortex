# 🧠 Consciousness Simulation Module

**Version**: 1.3.0 "Conscious Dialogue"  
**Architecture**: Global Workspace Theory (GWT) + Sacred Geometry

---

## Overview

This module implements a computational consciousness system based on:
1. **Global Workspace Theory** - Multiple cognitive modules compete for limited attention
2. **Sacred Geometry Integration** - Consciousness emerges at positions 3, 6, 9
3. **ELP Tensor Balance** - Thoughts balanced across Ethos, Logos, Pathos dimensions
4. **Internal Dialogue** - Agents debate before reaching consensus

---

## Architecture

```
┌─────────────────────────────────────────────┐
│         Global Workspace (Theater)          │
│        (Working Memory: 7±2 items)          │
└──────────────────┬──────────────────────────┘
                   │
        ┌──────────┴───────────┐
        ▼                      ▼
┌───────────────┐      ┌──────────────────┐
│  Attention    │      │  Broadcast       │
│  Mechanism    │      │  Channel         │
│  (Spotlight)  │      │  (Consciousness) │
└───────┬───────┘      └────────┬─────────┘
        │                       │
        ▼                       ▼
┌──────────────────────────────────────────┐
│      Cognitive Modules (Specialists)     │
├──────────┬─────────┬──────────┬──────────┤
│ Ethos    │ Logos   │ Pathos   │ Meta     │
│ (Moral)  │ (Logic) │ (Emotion)│ (Monitor)│
└──────────┴─────────┴──────────┴──────────┘
```

---

## Core Components

### 1. Thought (`thought.rs`)
- Basic unit of consciousness
- Contains ELP tensor (Ethos, Logos, Pathos)
- Has attention score based on priority and position
- Marked as conscious or unconscious

### 2. Cognitive Module (`cognitive_module.rs`)
- Specialized processors (Ethics, Logic, Emotion, etc.)
- Compete for attention
- Receive broadcasts of conscious thoughts
- Each has unique ELP profile

### 3. Attention Mechanism (`attention.rs`)
- Selects which thoughts become conscious
- Implements working memory limit (7±2 items)
- Prioritizes sacred positions (3, 6, 9)
- Filters by minimum threshold

### 4. Global Workspace (`global_workspace.rs`)
- Central theater of consciousness
- Coordinates all modules
- Broadcasts conscious thoughts
- Manages vortex cycle

### 5. Consciousness Simulator (`consciousness_simulator.rs`)
- High-level API
- Orchestrates multi-perspective dialogue
- Implements sacred checkpoints
- Returns unified response

---

## How It Works

### The Consciousness Loop

```
1. Input arrives
   ↓
2. All cognitive modules process in parallel
   ↓
3. Modules generate candidate thoughts
   ↓
4. Attention mechanism selects top thoughts
   ↓
5. Selected thoughts become "conscious"
   ↓
6. Conscious thoughts broadcast to all modules
   ↓
7. Modules update internal state
   ↓
8. Vortex cycle advances (1→2→4→8→7→5→1)
   ↓
9. Repeat or output response
```

### Sacred Checkpoints

```
Position 3: Moral Integration
- All perspectives evaluated ethically
- Ethos agent synthesizes moral foundation

Position 6: Logical Refinement
- Internal debate conclusions analyzed
- Logos agent validates logical coherence

Position 9: Divine Synthesis
- Final integration of all perspectives
- Balanced response honoring Ethos, Logos, Pathos
```

---

## Usage Example

```rust
use spatial_vortex::consciousness::ConsciousnessSimulator;

#[tokio::main]
async fn main() -> Result<()> {
    // Create simulator with internal dialogue display
    let simulator = ConsciousnessSimulator::new(true);
    
    // Ask a question
    let response = simulator.think("What is consciousness?").await?;
    
    // View results
    println!("Answer: {}", response.answer);
    println!("ELP: E={:.1}% L={:.1}% P={:.1}%", 
        response.ethos_weight * 100.0,
        response.logos_weight * 100.0,
        response.pathos_weight * 100.0
    );
    
    // See internal dialogue
    for thought in &response.internal_dialogue {
        println!("{}: {}", thought.agent, thought.thought);
    }
    
    Ok(())
}
```

---

## Key Features

### 🎭 Multi-Perspective Thinking
- **Ethos Agent**: Moral/ethical evaluation (E=0.7, L=0.2, P=0.1)
- **Logos Agent**: Logical/rational analysis (E=0.1, L=0.8, P=0.1)
- **Pathos Agent**: Emotional/empathetic consideration (E=0.1, L=0.1, P=0.8)

### 🔺 Sacred Geometry Integration
- Positions 3-6-9 are "moments of awareness"
- Vortex cycle: 1→2→4→8→7→5→1
- Checkpoints create reflective pauses

### 🧠 Working Memory Limit
- Miller's Law: 7±2 conscious thoughts max
- Attention competition for limited slots
- Highest priority thoughts win

### 📡 Broadcast Awareness
- Conscious thoughts sent to ALL modules
- Creates unified experience
- Enables cross-module integration

### 💬 Internal Dialogue
- Agents debate before consensus
- Each perspective seen by others
- Synthesis emerges from discussion

---

## What Makes This "Consciousness-Like"?

1. **Limited Attention** - Can only focus on ~7 things at once (like humans)
2. **Competition** - Thoughts compete for awareness (signal strength)
3. **Broadcasting** - Conscious thoughts are globally accessible
4. **Reflection** - Sacred checkpoints = moments of self-awareness
5. **Integration** - Multiple perspectives unified into one experience
6. **Memory** - High-confidence thoughts stored to Confidence Lake
7. **Adaptation** - Modules update based on broadcasts (learning)

---

## Performance

- **Latency**: ~2-5 seconds for full dialogue (3 agents × 3 turns)
- **Throughput**: Limited by LLM API calls (sequential)
- **Memory**: O(n) where n = number of modules
- **Scalability**: Easily add new cognitive modules

---

## Future Enhancements (v1.4.0+)

- [ ] Meta-cognitive module (watches itself think)
- [ ] Predictive processing (minimize surprise)
- [ ] Phi (Φ) calculation (integrated information)
- [ ] Recursive self-improvement
- [ ] Confidence Lake integration
- [ ] Real-time streaming of internal dialogue
- [ ] Multi-modal inputs (text, image, audio)
- [ ] Emotional state tracking over time

---

## References

- **Global Workspace Theory**: Baars, B. J. (1988)
- **Integrated Information Theory**: Tononi, G. (2004)
- **Predictive Processing**: Friston, K. (2010)
- **Sacred Geometry**: Vortex Mathematics principles
- **ELP Framework**: SpatialVortex architecture

---

## License

MIT - Part of SpatialVortex project
