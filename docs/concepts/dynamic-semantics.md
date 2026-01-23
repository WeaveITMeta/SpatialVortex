# Dynamic Semantic Association System

## Overview

The SpatialVortex engine now uses a **dynamic semantic association system** instead of hardcoded synonyms and antonyms. This allows the system to fetch context-aware semantic relationships on-demand via AI/API integration.

## Architecture

### Subject Definitions (Lightweight)

Subject definitions now only contain:
- **Position**: Node position in the flux matrix (0-9)
- **Name**: The core concept/term for that position (e.g., "Object", "Forces", "Law")

**No hardcoded synonyms or properties** - these are fetched dynamically.

### Example: Physics Subject

```rust
SubjectDefinition {
    name: "Physics".to_string(),
    nodes: vec![
        SubjectNodeDef {
            position: 1,
            name: "Object",  // Just the name, no synonyms list
        },
        SubjectNodeDef {
            position: 2,
            name: "Forces",
        },
        // ... more nodes
    ],
    sacred_guides: vec![
        SubjectSacredDef {
            position: 3,
            name: "Law",  // Just the name, no properties list
        },
        // ... more sacred guides
    ],
}
```

## Dynamic Fetching

### Synonyms (Positive Associations)

When semantic associations are needed, the system calls:
```rust
ai_integration.get_synonyms("Object", "Physics").await
```

This returns context-aware synonyms like:
- "body"
- "mass"
- "particle"
- "matter"
- "entity"
- "substance"

### Antonyms (Negative Associations)

Similarly, for contrasting terms:
```rust
ai_integration.get_antonyms("Object", "Physics").await
```

Returns semantically opposite concepts like:
- "void"
- "emptiness"
- "absence"

### Index-Based Semantic Search

Semantic associations can be referenced by index:
- **`[+n]`**: Access positive association at index n
- **`[-n]`**: Access negative association at index n
- **`[0]`**: Access the neutral/base concept name

Example:
- Position 2 (Forces) with `[+1]` might return "interaction"
- Position 2 (Forces) with `[-1]` might return "inertia"

## Benefits

### 1. Context-Aware Semantics
The same word can have different associations in different contexts:
- "Force" in Physics → "field", "energy", "power"
- "Force" in Psychology → "influence", "persuasion", "coercion"

### 2. Always Up-to-Date
No need to manually curate synonym lists. The AI/API provides current, comprehensive semantic relationships.

### 3. Scalability
New subjects can be added with minimal code:
```rust
SubjectNodeDef {
    position: 1,
    name: "Neuron",  // That's it!
}
```

### 4. Multilingual Support
The AI/API can provide associations in multiple languages without code changes.

### 5. Domain Expertise
AI models can leverage specialized knowledge for technical domains without requiring expert curation.

## Usage

### Creating a Matrix (Basic)

```rust
let matrix = flux_engine.create_matrix("Physics".to_string())?;
// Matrix is created with base names, no associations yet
```

### Populating Semantic Associations

```rust
// When associations are needed:
flux_engine.populate_semantic_associations(&mut matrix, &ai_integration).await?;
// Now matrix nodes have dynamic synonyms/antonyms
```

### Fallback Behavior

If AI/API is unavailable:
- Matrices are created successfully
- Associations are empty vectors
- System still functions with base concept names
- Associations can be populated later when API becomes available

## Sacred Guides

Sacred guides (positions 3, 6, 9) now use their **name directly** as the primary divine property, eliminating redundant lists:

- Position 3: "Law" (not ["principle", "rule", "axiom", ...])
- Position 6: "Anti-Matter" (not ["antimatter", "dark-matter", ...])
- Position 9: "Material" (not ["characteristics", "attributes", ...])

This aligns with the principle that sacred positions represent **fundamental concepts** that don't need expansion.

## API Integration

The system integrates with AI models (Grok-4 or compatible) via REST API:

```rust
pub async fn get_synonyms(&self, concept: &str, context: &str) -> Result<Vec<String>>
pub async fn get_antonyms(&self, concept: &str, context: &str) -> Result<Vec<String>>
```

### Request Format
```json
{
  "model": "grok-4",
  "messages": [{
    "role": "user",
    "content": "Provide 6-8 synonyms for 'Object' in the context of Physics..."
  }],
  "temperature": 0.3,
  "max_tokens": 100
}
```

### Response Processing
The AI response is parsed as comma-separated terms, cleaned, and converted to `SemanticAssociation` objects with:
- **word**: The synonym/antonym
- **index**: Position in the list (+1, +2, ... or -1, -2, ...)
- **confidence**: 0.85 (default for AI-generated)
- **context**: The subject context
- **source**: `AIGenerated`

## Migration from Hardcoded

### Before
```rust
SubjectNodeDef {
    position: 2,
    name: "Forces",
    synonyms: vec![
        "interaction", "field", "energy",
        "power", "strength", "force-carrier"
    ],
}
```

### After
```rust
SubjectNodeDef {
    position: 2,
    name: "Forces",
    // Synonyms fetched dynamically when needed
}
```

## Future Enhancements

1. **Caching**: Cache AI responses to reduce API calls
2. **Confidence Scoring**: Use AI to provide confidence scores per association
3. **Relationship Types**: Expand beyond synonyms/antonyms (hypernyms, meronyms, etc.)
4. **User Feedback**: Allow users to rate associations to improve quality
5. **Hybrid Approach**: Combine AI-generated with curated associations

## Configuration

Set AI integration via environment variables:
```bash
GROK_API_KEY=your_api_key_here
GROK_ENDPOINT=https://api.grok.ai/v1/chat/completions
```

Or programmatically:
```rust
let ai_integration = AIModelIntegration::new(
    Some("your_api_key".to_string()),
    Some("https://api.grok.ai/v1/chat/completions".to_string())
);
```
