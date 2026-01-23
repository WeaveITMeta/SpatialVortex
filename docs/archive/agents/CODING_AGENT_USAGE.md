# SpatialVortex Coding Agent - Complete Usage Guide

**Version**: 0.7.0  
**Status**: Production Ready (Phase 4 Complete)

---

## 📚 Table of Contents

- [Overview](#overview)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [CLI Usage](#cli-usage)
- [Programmatic API](#programmatic-api)
- [Features](#features)
- [Sacred Geometry Integration](#sacred-geometry-integration)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)

---

## Overview

The SpatialVortex Coding Agent is a multi-language AI coding assistant that:
- ✅ Generates code in **24+ programming languages**
- ✅ Executes code in **sandboxed Docker containers**
- ✅ Solves math problems **symbolically**
- ✅ Self-corrects errors (up to 3 attempts)
- ✅ Routes tasks via **sacred geometry** (3-6-9 positions)
- ✅ Learns from successful code (**Confidence Lake**)
- ✅ Retrieves examples via **RAG**

---

## Installation

### Prerequisites

1. **Rust** (1.75+):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Docker** (for code execution):
   ```bash
   # Windows: Download Docker Desktop
   # Linux: sudo apt install docker.io
   # Mac: brew install docker
   ```

3. **Ollama** (for local LLM):
   ```bash
   # Windows/Linux/Mac
   curl -fsSL https://ollama.com/install.sh | sh
   
   # Pull models
   ollama pull codellama:13b
   ollama pull mistral:7b
   
   # Start server
   ollama serve
   ```

### Build

```bash
# Clone repository
git clone https://github.com/yourusername/SpatialVortex
cd SpatialVortex

# Build with agents feature
cargo build --release --features agents

# Build CLI tool
cargo build --release --bin coding_agent_cli --features agents
```

---

## Quick Start

### 1. Command Line

```bash
# Simple code generation
coding_agent_cli "Write a Python function to sort a list"

# With execution
coding_agent_cli "Create Rust factorial function" --execute

# Symbolic math
coding_agent_cli "Solve x^2 + 3x + 2 = 0"

# Different language
coding_agent_cli "Create Go HTTP server" --language go
```

### 2. Programmatic

```rust
use spatial_vortex::agents::{CodingAgent, AgentConfig, LLMConfig, LLMBackend};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Configure LLM
    let llm_config = LLMConfig {
        backend: LLMBackend::Ollama {
            url: "http://localhost:11434".to_string(),
            model: "codellama:13b".to_string(),
        },
        temperature: 0.2,
        ..Default::default()
    };
    
    // Create agent
    let agent = CodingAgent::with_llm(AgentConfig::default(), llm_config)?;
    
    // Generate code
    let result = agent.execute_task("Write Python factorial function").await?;
    
    println!("Code:\n{}", result.code);
    Ok(())
}
```

---

## CLI Usage

### Basic Syntax

```bash
coding_agent_cli [OPTIONS] <TASK>
```

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `-l, --language <LANG>` | Force specific language | Auto-detect |
| `-e, --execute` | Execute code in Docker | false |
| `-c, --correct` | Enable self-correction | true |
| `--no-correct` | Disable self-correction | false |
| `-p, --flux-position <POS>` | Force flux position (3/6/9) | Auto-route |
| `-b, --backend <BACKEND>` | LLM backend (ollama/openai/anthropic) | ollama |
| `-m, --model <MODEL>` | Model name | codellama:13b |
| `-t, --temperature <TEMP>` | Temperature (0.0-1.0) | 0.2 |
| `-v, --verbose` | Verbose output | false |
| `-f, --output-format <FMT>` | Output format (text/json) | text |

### Examples

#### 1. Basic Code Generation

```bash
coding_agent_cli "Write a Python function to reverse a string"
```

Output:
```
📊 Results:
   Language: Python
   Flux Position: None
   Attempts: 1

📝 Generated Code:
```python
def reverse_string(s):
    return s[::-1]
```\u200B
```

#### 2. With Execution

```bash
coding_agent_cli "Write Python hello world" --execute
```

Output includes:
```
🔧 Execution:
   Status: ✅ Success
   
   Output:
   Hello, World!
   
   Exit Code: 0
```

#### 3. Symbolic Math

```bash
coding_agent_cli "Solve x^2 + 3x + 2 = 0" --language rust
```

Output:
```
📊 Results:
   Language: Rust
   Flux Position: Some(9)  ← Logos (Math)
   Attempts: 1
   
📝 Generated Code:
```rust
fn calculate(x: f64) -> f64 {
    // Solution for: x^2 + 3x + 2 = 0
    x.powf(2.0) + 3.0 * x + 2.0
}
```\u200B
```

#### 4. Force Sacred Position

```bash
# Position 3 (Ethos) - Architecture
coding_agent_cli "Design a plugin system" --flux-position 3

# Position 6 (Pathos) - UI/UX
coding_agent_cli "Create beautiful button" --flux-position 6

# Position 9 (Logos) - Math/Logic
coding_agent_cli "Implement quicksort" --flux-position 9
```

#### 5. Different LLM Backends

```bash
# OpenAI
coding_agent_cli "Write Rust code" \
  --backend openai \
  --model gpt-4 \
  --openai-api-key sk-...

# Anthropic
coding_agent_cli "Write Python code" \
  --backend anthropic \
  --model claude-3-sonnet-20240229 \
  --anthropic-api-key sk-ant-...

# Local Ollama (default)
coding_agent_cli "Write Go code" \
  --backend ollama \
  --model codellama:13b
```

#### 6. JSON Output

```bash
coding_agent_cli "Sort array" --output-format json
```

Output:
```json
{
  "success": true,
  "language": "Python",
  "flux_position": null,
  "attempts": 1,
  "elapsed_ms": 2341,
  "code": "def sort_array(arr):\n    return sorted(arr)",
  "stdout": null,
  "stderr": null,
  "exit_code": null
}
```

---

## Programmatic API

### Creating an Agent

#### With Ollama (Local)

```rust
use spatial_vortex::agents::{CodingAgent, AgentConfig, LLMConfig, LLMBackend};

let llm_config = LLMConfig {
    backend: LLMBackend::Ollama {
        url: "http://localhost:11434".to_string(),
        model: "codellama:13b".to_string(),
    },
    temperature: 0.2,
    max_tokens: 2048,
    ..Default::default()
};

let agent = CodingAgent::with_llm(AgentConfig::default(), llm_config)?;
```

#### With OpenAI

```rust
let llm_config = LLMConfig {
    backend: LLMBackend::OpenAI {
        api_key: std::env::var("OPENAI_API_KEY")?,
        model: "gpt-4".to_string(),
    },
    temperature: 0.2,
    ..Default::default()
};

let agent = CodingAgent::with_llm(AgentConfig::default(), llm_config)?;
```

#### With Anthropic

```rust
let llm_config = LLMConfig {
    backend: LLMBackend::Anthropic {
        api_key: std::env::var("ANTHROPIC_API_KEY")?,
        model: "claude-3-sonnet-20240229".to_string(),
    },
    temperature: 0.2,
    ..Default::default()
};

let agent = CodingAgent::with_llm(AgentConfig::default(), llm_config)?;
```

### Executing Tasks

```rust
let result = agent.execute_task("Write a Python function to calculate fibonacci").await?;

println!("Language: {}", result.language.name());
println!("Code:\n{}", result.code);
println!("Flux Position: {:?}", result.flux_position);
println!("Attempts: {}", result.attempts);

if let Some(exec) = result.execution {
    println!("Success: {}", exec.success);
    println!("Output: {}", exec.stdout);
}
```

### Configuration

```rust
let config = AgentConfig {
    max_correction_attempts: 3,      // Self-correction attempts
    enable_self_correction: true,    // Enable error correction
    enable_flux_routing: true,       // Sacred geometry routing
    memory_limit: "256m".to_string(), // Docker memory limit
    cpu_limit: "0.5".to_string(),    // Docker CPU limit
};

let agent = CodingAgent::with_llm(config, llm_config)?;
```

---

## Features

### 1. Multi-Language Support (24+)

| Category | Languages |
|----------|-----------|
| **Systems** | Rust, C, C++, Zig |
| **Scripting** | Python, Ruby, Elixir |
| **Web** | JavaScript, TypeScript |
| **Functional** | Haskell, OCaml, F# |
| **JVM** | Java, Kotlin, Scala |
| **.NET** | C#, F# |
| **Multi-target** | Nim, Haxe |
| **Modern** | Go, Swift |
| **Domain** | SQL, GLSL, WGSL, WASM |

### 2. Docker Sandboxing

All code executions run in isolated Docker containers with:
- ❌ No network access (`--network=none`)
- 💾 Memory limit (256MB default)
- ⚡ CPU limit (0.5 cores default)
- 🔒 Read-only filesystem
- ⏱️ 30-second timeout

### 3. Self-Correction Loop

If code fails to execute:
1. Extract error message
2. Generate correction prompt
3. Re-generate code
4. Execute again
5. Repeat up to 3 attempts

### 4. Symbolic Mathematics

Math tasks are solved symbolically before code generation:
- **Simplify**: `x + 0` → `x`
- **Differentiate**: `x^2` → `2*x`
- **Factor**: `x^2 - 1` → `(x-1)(x+1)`
- **Expand**: `(x+1)^2` → `x^2 + 2x + 1`
- **Solve**: Equation solving
- **Integrate**: Integration (future)

### 5. Knowledge Management

Stores successful code in memory (future: Confidence Lake + RAG):
- Signal strength ≥ 0.6 required
- Similar code retrieval
- Learning from past successes

---

## Sacred Geometry Integration

Tasks are automatically routed to sacred positions (3-6-9) based on content:

| Position | Channel | Focus | Keywords |
|----------|---------|-------|----------|
| **3** | Ethos | Architecture, Design | design, architecture, pattern, structure |
| **6** | Pathos | UI/UX, Readability | ui, ux, interface, visual |
| **9** | Logos | Math, Logic | algorithm, math, solve, equation |

Example routing:
```rust
"Solve equation" → Position 9 (Logos)
"Design clean architecture" → Position 3 (Ethos)
"Create beautiful UI" → Position 6 (Pathos)
```

This influences LLM prompts to emphasize the appropriate dimension.

---

## Examples

See `examples/` directory:
- `coding_agent_demo.rs` - Multi-language demos
- `symbolica_math_demo.rs` - Symbolic math demos

Run with:
```bash
cargo run --example coding_agent_demo --features agents
cargo run --example symbolica_math_demo --features agents
```

---

## Troubleshooting

### Ollama Not Running

```bash
Error: Ollama request failed: connection refused
```

**Fix**:
```bash
ollama serve  # Start in separate terminal
```

### Docker Not Available

```bash
Error: Docker error: Cannot connect to the Docker daemon
```

**Fix**:
- Windows: Start Docker Desktop
- Linux: `sudo systemctl start docker`
- Mac: Start Docker.app

### Model Not Found

```bash
Error: model 'codellama:13b' not found
```

**Fix**:
```bash
ollama pull codellama:13b
ollama list  # Verify
```

### Out of Memory

```bash
Error: Docker container OOMKilled
```

**Fix**: Increase memory limit
```bash
coding_agent_cli "task" --memory-limit 512m
```

---

## Advanced Usage

### Custom Prompts

```rust
let mut agent = CodingAgent::with_llm(config, llm_config)?;

// Add custom few-shot examples
agent.prompt_builder.add_example(
    "Sort numbers".to_string(),
    "def sort(arr): return sorted(arr)".to_string(),
);
```

### Knowledge Base

```rust
use spatial_vortex::agents::CodeKnowledge;

let mut knowledge = CodeKnowledge::new();

// Store successful code
knowledge.store(CodeExample::new(
    "Sort list".to_string(),
    "code".to_string(),
    Language::Python,
    Some(9),
    true,
    1,
)).await?;

// Retrieve similar
let examples = knowledge.retrieve_similar(
    "Sort array",
    Some(Language::Python),
    5
).await?;
```

---

**Status**: Production Ready (Phase 4 Complete) ✅  
**Progress**: 100% (All 4 phases complete)  
**Version**: 0.7.0
