# Subject Generation Quick Start

## 🚀 Generate Your First Subject in 3 Steps

### Step 1: Set Your API Key
```bash
export GROK_API_KEY=your_api_key_here
```

### Step 2: Generate a Subject
```bash
cargo run --bin subject_cli -- "Chemistry"
```

### Step 3: Build and Use
```bash
cargo fmt
cargo check
cargo build
```

## ✨ That's It!

Your new subject is now available:

```rust
let matrix = flux_engine.create_matrix("Chemistry".to_string())?;
```

## 📋 What Just Happened?

1. **AI analyzed "Chemistry"** and determined optimal node names
2. **Generated `src/subjects/chemistry.rs`** with complete definition
3. **Updated `src/subjects/mod.rs`** to register the new module
4. **Made it available** in the subject getter function

## 🎯 Example Output

```
[Subject Generator] === Creating Subject: Chemistry ===

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
```

## 🔧 Via API

```bash
curl -X POST http://localhost:8080/api/v1/subjects/generate \
  -H "Content-Type: application/json" \
  -d '{"subject_name": "Chemistry"}'
```

## 📚 More Examples

```bash
# Generate multiple subjects
cargo run --bin subject_cli -- "Biology"
cargo run --bin subject_cli -- "Economics"
cargo run --bin subject_cli -- "Psychology"
cargo run --bin subject_cli -- "Linguistics"
cargo run --bin subject_cli -- "Music Theory"
```

## 🎨 What Gets Generated

**File: `src/subjects/chemistry.rs`**
```rust
pub fn get_chemistry_definition() -> SubjectDefinition {
    SubjectDefinition {
        name: "Chemistry".to_string(),
        nodes: vec![
            SubjectNodeDef { position: 1, name: "Atom".to_string() },
            SubjectNodeDef { position: 2, name: "Bonds".to_string() },
            // ... more nodes
        ],
        sacred_guides: vec![
            SubjectSacredDef { position: 3, name: "Equilibrium".to_string() },
            // ... more guides
        ],
    }
}
```

## ⚡ Key Features

- ✅ **No Manual Coding** - AI handles everything
- ✅ **Type-Safe** - Full Rust compilation
- ✅ **Modular** - Each subject is its own file
- ✅ **Context-Aware** - Semantic associations fetched dynamically
- ✅ **Instant Integration** - Works with existing code

## 🛠️ Troubleshooting

**API Key Error?**
```bash
# Make sure it's exported
echo $GROK_API_KEY
```

**Subject Exists?**
Delete it first:
```bash
rm src/subjects/chemistry.rs
```

**Compilation Error?**
```bash
cargo fmt      # Format first
cargo check    # Then check
```

## 📖 Full Documentation

See `docs/SUBJECT_GENERATION.md` for complete details.
