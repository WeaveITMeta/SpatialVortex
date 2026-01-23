# SpatialVortex

[![Crates.io](https://img.shields.io/crates/v/spatial-vortex)](https://crates.io/crates/spatial-vortex)
[![Documentation](https://docs.rs/spatial-vortex/badge.svg)](https://docs.rs/spatial-vortex)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

Revolutionary Flux Matrix engine for AI inference, semantic reasoning, and dynamic subject generation with sacred geometry integration (positions 3, 6, 9).

## Features

- **🌀 Flux Matrix Engine**: Direct digit-to-position mapping for deterministic inference
- **🧠 Dynamic Semantic Associations**: AI-powered synonym/antonym fetching via external APIs
- **📊 Subject Generation**: Automatically create modular subject definitions using AI
- **🔺 Sacred Geometry**: Special positions (3, 6, 9) with geometric significance
- **⚡ Index-Based Semantic Search**: Access positive `[+n]` and negative `[-n]` associations
- **🎯 Type-Safe**: Full Rust compilation with zero runtime overhead
- **🔌 REST API**: Production-ready Actix-web server with comprehensive endpoints

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
spatial-vortex = "0.1"
```

### Basic Usage

```rust
use spatial_vortex::{
    flux_matrix::FluxMatrixEngine,
    inference_engine::InferenceEngine,
    models::*,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize engine
    let flux_engine = FluxMatrixEngine::new();
    let mut inference_engine = InferenceEngine::new();
    
    // Create a subject matrix
    let matrix = flux_engine.create_matrix("Physics".to_string())?;
    inference_engine.update_subject_matrix(matrix);
    
    // Process seed number (direct digit mapping)
    let seed_input = SeedInput {
        seed_numbers: vec![36901248751],
        subject_filter: SubjectFilter::Specific("Physics".to_string()),
        processing_options: ProcessingOptions {
            include_synonyms: true,
            include_antonyms: true,
            max_depth: 5,
            confidence_threshold: 0.3,
            use_sacred_guides: true,
        },
    };
    
    let result = inference_engine.process_seed_input(seed_input).await?;
    
    println!("Inferred meanings: {}", result.inferred_meanings.len());
    println!("Confidence: {:.2}%", result.confidence_score * 100.0);
    
    Ok(())
}
```

## Core Concepts

### Seed Numbers → Positions

Each digit in a seed number maps directly to a node position (0-9):
- Seed: `369` → Positions: `[3, 6, 9]` (all sacred guides!)
- Seed: `1248` → Positions: `[1, 2, 4, 8]` (regular nodes)

### Sacred Geometry

Positions **3, 6, 9** are special:
- Form triangular patterns
- Provide geometric anchoring
- Enable divine computational pathways
- Connect to other sacred positions

### Dynamic Semantics

Semantic associations are fetched dynamically via AI/API:
```rust
// Synonyms for "Forces" in Physics context
ai_integration.get_synonyms("Forces", "Physics").await?
// Returns: ["interaction", "field", "energy", "power", ...]

// Antonyms
ai_integration.get_antonyms("Forces", "Physics").await?
// Returns: ["inertia", "resistance", "stillness", ...]
```

### Subject Generation

Generate new subjects with one command:
```bash
cargo run --bin subject_cli -- "Chemistry"
```

This creates `src/subjects/chemistry.rs` with AI-determined node names, automatically registered in the module system.

## REST API

Start the server:
```bash
cargo run --release
```

### Key Endpoints

**Generate Subject**
```bash
curl -X POST http://localhost:8080/api/v1/subjects/generate \
  -H "Content-Type: application/json" \
  -d '{"subject_name": "Chemistry"}'
```

**Reverse Inference (Seeds → Meanings)**
```bash
curl -X POST http://localhost:8080/api/v1/inference/reverse \
  -H "Content-Type: application/json" \
  -d '{
    "seed_numbers": [12345],
    "subject_filter": "physics",
    "include_synonyms": true
  }'
```

**Generate Flux Matrix**
```bash
curl -X POST http://localhost:8080/api/v1/flux/matrix/generate \
  -H "Content-Type: application/json" \
  -d '{
    "subject": "Physics",
    "seed_number": 369
  }'
```

## Architecture

### Modular Subjects

Each subject is a separate Rust module:
```
src/subjects/
├── mod.rs        # Central registry
├── physics.rs    # Physics subject
├── chemistry.rs  # Generated subject
└── biology.rs    # Generated subject
```

### Position Mapping

```
Position 0: Void/Center
Position 1: Regular node (e.g., "Object")
Position 2: Regular node (e.g., "Forces")
Position 3: Sacred guide (e.g., "Law")
Position 4: Regular node (e.g., "Value")
Position 5: Regular node (e.g., "Unit")
Position 6: Sacred guide (e.g., "Anti-Matter")
Position 7: Regular node (e.g., "Assembly")
Position 8: Regular node (e.g., "Constraints")
Position 9: Sacred guide (e.g., "Material")
```

## Configuration

Set environment variables:
```bash
export GROK_API_KEY=your_api_key_here
export GROK_ENDPOINT=https://api.x.ai/v1/chat/completions
```

## Documentation

- **Full Documentation**: [docs.rs/spatial-vortex](https://docs.rs/spatial-vortex)
- **Repository**: [github.com/yourusername/SpatialVortex](https://github.com/yourusername/SpatialVortex)
- **Examples**: See `examples/` directory
- **API Specification**: `api/swagger.yml`

## Examples

### Flux Matrix Creation
```rust
let flux_engine = FluxMatrixEngine::new();
let matrix = flux_engine.create_matrix("Physics".to_string())?;

println!("Nodes: {}", matrix.nodes.len());
println!("Sacred Guides: {}", matrix.sacred_guides.len());
```

### Semantic Association
```rust
// Populate dynamic associations
flux_engine.populate_semantic_associations(&mut matrix, &ai_integration).await?;

// Access node associations
if let Some(node) = matrix.nodes.get(&2) {
    println!("Position 2: {}", node.semantic_index.neutral_base);
    for assoc in &node.semantic_index.positive_associations {
        println!("  +{}: {}", assoc.index, assoc.word);
    }
}
```

### Sacred Guide Access
```rust
if let Some(guide) = matrix.sacred_guides.get(&3) {
    println!("Sacred position 3: {}", guide.geometric_significance);
    for prop in &guide.divine_properties {
        println!("  Property: {}", prop);
    }
}
```

## Features

- `full` - All features enabled (default)
- `server` - REST API server only
- `cli` - Command-line tools only
- `minimal` - Core flux matrix engine only

## License

Apache License 2.0 - see [LICENSE](LICENSE) file for details.

## Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for release history.
