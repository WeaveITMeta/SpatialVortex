# Dynamic Subject Generation

## Overview

SpatialVortex now supports **dynamic subject generation**, where new subject modules are automatically created as separate Rust files using AI. Each subject becomes its own module in `src/subjects/`.

## Architecture

### Modular Subject System

```
src/subjects/
├── mod.rs              # Central registry
├── physics.rs          # Physics subject module
├── chemistry.rs        # Chemistry subject module (generated)
├── biology.rs          # Biology subject module (generated)
└── psychology.rs       # Psychology subject module (generated)
```

### Benefits

1. **Modularity**: Each subject is self-contained
2. **Scalability**: Add unlimited subjects without bloating a single file
3. **AI-Powered**: AI determines optimal node names for each subject
4. **Type-Safe**: Full Rust compilation checks
5. **Version Control**: Each subject can be tracked separately in git

## Generation Methods

### 1. CLI Tool (Recommended)

```bash
# Set your API key
export GROK_API_KEY=your_api_key_here

# Generate a new subject
cargo run --bin subject_cli -- "Chemistry"
cargo run --bin subject_cli -- "Biology"
cargo run --bin subject_cli -- "Economics"
```

**Output:**
```
=== SpatialVortex Subject Generator CLI ===

Configuration:
  API Key: Set
  Endpoint: https://api.x.ai/v1/chat/completions
  Subject: Chemistry

[Subject Generator] === Creating Subject: Chemistry ===

[Subject Generator] Generating subject: Chemistry
[Subject Generator] Generated structure:
  Nodes: 6 regular nodes
    Position 1: Atom
    Position 2: Bonds
    Position 4: Molecule
    Position 5: Reaction
    Position 7: Compound
    Position 8: Properties
  Sacred Guides: 3 guides
    Position 3: Equilibrium
    Position 6: Catalyst
    Position 9: State

[Subject Generator] Created file: src/subjects/chemistry.rs
[Subject Generator] Updated mod.rs with module 'chemistry'
[Subject Generator] Added 'chemistry' to subject getter

[Subject Generator] === Subject 'Chemistry' Created Successfully ===
[Subject Generator] File: chemistry.rs
[Subject Generator] Next steps:
  1. Run 'cargo fmt' to format the new code
  2. Run 'cargo check' to verify compilation
  3. Rebuild your application to use the new subject
```

### 2. REST API Endpoint

**Endpoint:** `POST /api/v1/subjects/generate`

**Request:**
```json
{
  "subject_name": "Chemistry"
}
```

**Response:**
```json
{
  "success": true,
  "subject_name": "Chemistry",
  "module_name": "chemistry",
  "filename": "chemistry.rs",
  "message": "Subject 'Chemistry' generated successfully. Rebuild application to use it."
}
```

**cURL Example:**
```bash
curl -X POST http://localhost:8080/api/v1/subjects/generate \
  -H "Content-Type: application/json" \
  -d '{"subject_name": "Chemistry"}'
```

## Generated File Structure

Example generated `chemistry.rs`:

```rust
/// Chemistry subject matter definition
/// 
/// Defines the semantic structure for Chemistry concepts mapped to
/// the 9-position flux matrix (1-9). Sacred guides at positions 3, 6, 9
/// provide geometric anchoring, while regular nodes occupy 1, 2, 4, 5, 7, 8.
/// 
/// Semantic associations (synonyms/antonyms) are fetched dynamically via AI/API.

use super::{SubjectDefinition, SubjectNodeDef, SubjectSacredDef};

/// Get the complete Chemistry subject definition
pub fn get_chemistry_definition() -> SubjectDefinition {
    SubjectDefinition {
        name: "Chemistry".to_string(),
        nodes: vec![
            SubjectNodeDef {
                position: 1,
                name: "Atom".to_string(),
            },
            SubjectNodeDef {
                position: 2,
                name: "Bonds".to_string(),
            },
            SubjectNodeDef {
                position: 4,
                name: "Molecule".to_string(),
            },
            SubjectNodeDef {
                position: 5,
                name: "Reaction".to_string(),
            },
            SubjectNodeDef {
                position: 7,
                name: "Compound".to_string(),
            },
            SubjectNodeDef {
                position: 8,
                name: "Properties".to_string(),
            },
        ],
        sacred_guides: vec![
            SubjectSacredDef {
                position: 3,
                name: "Equilibrium".to_string(),
            },
            SubjectSacredDef {
                position: 6,
                name: "Catalyst".to_string(),
            },
            SubjectSacredDef {
                position: 9,
                name: "State".to_string(),
            },
        ],
    }
}
```

## Automatic Updates

When a subject is generated, three files are automatically updated:

### 1. Subject Module File
`src/subjects/chemistry.rs` - The new subject definition

### 2. Module Registry
`src/subjects/mod.rs` gets:
```rust
pub mod chemistry;  // Added automatically
```

### 3. Subject Getter
`src/subjects/mod.rs` match statement updated:
```rust
pub fn get_subject_definition(subject_name: &str) -> Option<SubjectDefinition> {
    match subject_name.to_lowercase().as_str() {
        "physics" => Some(physics::get_physics_definition()),
        "chemistry" => Some(chemistry::get_chemistry_definition()),  // Added
        _ => None,
    }
}
```

## AI Prompt Structure

The AI receives this prompt format:

```
For the subject 'Chemistry', define a flux matrix with 9 positions (1-9).
Positions 3, 6, and 9 are sacred guides representing fundamental principles.
Regular nodes occupy positions 1, 2, 4, 5, 7, 8.

Provide:
1. For each regular node position (1,2,4,5,7,8): A single-word or short phrase concept name
2. For each sacred guide position (3,6,9): A single-word fundamental principle name

Format your response as JSON:
{
  "nodes": [
    {"position": 1, "name": "ConceptName"},
    ...
  ],
  "sacred_guides": [
    {"position": 3, "name": "PrincipleName"},
    ...
  ]
}

Make the names specific to Chemistry and avoid generic terms.
```

## Workflow

```
User Request
    ↓
AI Generates Structure
    ↓
Parse JSON Response
    ↓
Generate Rust Code
    ↓
Write File (chemistry.rs)
    ↓
Update mod.rs
    ↓
Update Subject Getter
    ↓
Success!
    ↓
Run: cargo fmt
    ↓
Run: cargo check
    ↓
Rebuild Application
    ↓
Use New Subject
```

## Best Practices

### 1. Subject Names
- Use proper capitalization: "Chemistry", not "chemistry"
- Single words or short phrases work best
- Avoid special characters

### 2. Review Generated Code
Always review the generated subject definition:
- Check node names make sense for the domain
- Verify sacred guide names are appropriate
- Edit if needed (it's just Rust code!)

### 3. Version Control
```bash
git add src/subjects/chemistry.rs
git commit -m "Add Chemistry subject definition"
```

### 4. Testing
Create tests for each new subject:
```rust
#[tokio::test]
async fn test_chemistry_seed_inference() {
    let flux_engine = FluxMatrixEngine::new();
    let matrix = flux_engine.create_matrix("Chemistry".to_string()).unwrap();
    
    assert_eq!(matrix.subject, "Chemistry");
    assert_eq!(matrix.nodes.len(), 6);
    assert_eq!(matrix.sacred_guides.len(), 3);
}
```

## Troubleshooting

### Subject Already Exists
```
Error: Subject file already exists: src/subjects/chemistry.rs
```
**Solution:** Delete the existing file or use a different subject name

### API Key Not Set
```
Error: GROK_API_KEY not set
```
**Solution:** 
```bash
export GROK_API_KEY=your_key_here
```

### Compilation Errors
After generating, run:
```bash
cargo fmt           # Format the code
cargo check        # Check for errors
```

### Invalid JSON from AI
The AI might occasionally produce invalid JSON. If this happens:
1. Check your prompt
2. Try again (AI responses vary)
3. Manually create the subject file if needed

## Examples

### Generate Multiple Subjects
```bash
# Generate a batch of subjects
for subject in "Chemistry" "Biology" "Economics" "Psychology"; do
    cargo run --bin subject_cli -- "$subject"
    sleep 2  # Rate limiting
done

cargo fmt
cargo check
```

### Check Generated Subjects
```bash
ls -la src/subjects/
# physics.rs
# chemistry.rs
# biology.rs
# economics.rs
# psychology.rs
```

### Use Generated Subjects
```rust
// In your application
let matrix = flux_engine.create_matrix("Chemistry".to_string())?;
println!("Chemistry matrix: {} nodes", matrix.nodes.len());
```

## Future Enhancements

1. **Interactive Mode**: CLI wizard for subject generation
2. **Batch Generation**: Generate multiple subjects at once
3. **Template System**: Custom templates for different subject types
4. **Subject Validation**: Verify node/guide positions are valid
5. **Hot Reload**: Load new subjects without rebuilding (development only)

## Security Considerations

1. **API Key Protection**: Never commit API keys to git
2. **Input Validation**: Subject names are sanitized before file creation
3. **File Permissions**: Generated files have standard Rust source permissions
4. **Rate Limiting**: Consider rate limiting the generation endpoint

## Configuration

### Environment Variables
```bash
GROK_API_KEY=your_api_key          # Required
GROK_ENDPOINT=https://...          # Optional (defaults to X.AI endpoint)
SUBJECTS_DIR=src/subjects          # Optional (default shown)
```

### Cargo.toml
The CLI binary is defined in your Cargo.toml:
```toml
[[bin]]
name = "subject_cli"
path = "src/bin/subject_cli.rs"
```
