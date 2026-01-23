# Contributing to SpatialVortex

Thank you for your interest in contributing to SpatialVortex! This guide will help you get started.

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Code Standards](#code-standards)
- [Testing Requirements](#testing-requirements)
- [Documentation](#documentation)
- [Pull Request Process](#pull-request-process)

---

## Code of Conduct

- Be respectful and constructive
- Focus on technical merit and project goals
- Help others learn and grow
- Report issues professionally

---

## Getting Started

### Prerequisites

```bash
# Rust toolchain (1.70+)
rustup update stable

# Node.js & package manager (for web UI)
node --version  # 18+
bun --version   # preferred, or pnpm/npm

# WASM target (for 3D visualization)
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
```

### Initial Setup

```bash
# Clone and build
git clone https://github.com/WeaveSolutions/SpatialVortex.git
cd SpatialVortex
cargo build --release

# Run tests
cargo test --lib

# Build web UI
cd web
bun install
bun run dev
```

See [docs/guides/QUICK_START.md](docs/guides/QUICK_START.md) for detailed setup.

---

## Development Workflow

### Branch Strategy

- `main` - Production-ready code
- `develop` - Integration branch for features
- `feature/*` - New features
- `fix/*` - Bug fixes
- `docs/*` - Documentation updates

### Typical Workflow

```bash
# Create feature branch
git checkout -b feature/your-feature-name

# Make changes and commit
git add .
git commit -m "feat: add new inference algorithm"

# Push and create PR
git push origin feature/your-feature-name
```

### Commit Message Format

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**
- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation changes
- `style:` Code formatting (no logic change)
- `refactor:` Code restructuring
- `test:` Adding/updating tests
- `chore:` Maintenance tasks

**Examples:**
```bash
feat(inference): add sacred geometry boost to position 3
fix(compression): handle edge case in 12-byte hash
docs(architecture): update tensor documentation
refactor(flux_matrix): simplify node traversal logic
```

---

## Code Standards

### Rust Code Style

Follow the [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/):

```rust
// Use descriptive names
pub struct FluxMatrix {
    pub nodes: HashMap<u8, FluxNode>,
    pub sacred_guides: HashMap<u8, SacredGuide>,
}

// Document public APIs
/// Calculate flux position from semantic seed
/// 
/// # Arguments
/// * `seed` - 16-byte semantic seed
/// * `subject` - Target subject matrix
/// 
/// # Returns
/// Flux position (0-9) with confidence score
pub fn calculate_position(seed: &[u8; 16], subject: &str) -> Result<(u8, f32)> {
    // Implementation
}

// Use Result for error handling
pub type Result<T> = std::result::Result<T, SpatialVortexError>;

// Prefer explicit types
let position: u8 = 3;
let confidence: f32 = 0.85;
```

### TypeScript Code Style

```typescript
// Use explicit types
interface BeamTensor {
  ethos: number;
  logos: number;
  pathos: number;
  confidence: number;
}

// Document functions
/**
 * Compress message to 12-byte hash
 * @param message - Input text
 * @param options - Compression options
 * @returns 12-byte hash as hex string
 */
export async function compressMessage(
  message: string,
  options?: CompressionOptions
): Promise<string> {
  // Implementation
}

// Use const/let, never var
const SACRED_POSITIONS = [3, 6, 9] as const;
let currentPosition = 0;
```

### Code Organization

```
src/
├── core/              # Core algorithms (compression, inference)
├── models/            # Data structures
├── api/               # External APIs
├── bin/               # Binary executables
└── lib.rs             # Public API exports

web/src/
├── lib/
│   ├── components/    # Svelte components
│   ├── api/           # API clients
│   ├── types/         # TypeScript definitions
│   └── utils/         # Helper functions
└── routes/            # SvelteKit routes
```

---

## Testing Requirements

### Rust Tests

All new features must include tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flux_position_calculation() {
        let seed = [0u8; 16];
        let (position, confidence) = calculate_position(&seed, "Consciousness")
            .expect("calculation failed");
        
        assert!(position < 10, "position must be 0-9");
        assert!(confidence >= 0.0 && confidence <= 1.0);
    }
}
```

**Run tests:**
```bash
# All tests
cargo test

# Specific module
cargo test flux_matrix

# Integration tests
cargo test --test integration_tests

# With output
cargo test -- --nocapture
```

### TypeScript Tests

Use Vitest for unit tests:

```typescript
import { describe, it, expect } from 'vitest';
import { compressMessage } from '$lib/api/compression';

describe('Compression API', () => {
  it('should compress to 12 bytes', async () => {
    const hash = await compressMessage('test message');
    expect(hash).toHaveLength(24); // 12 bytes = 24 hex chars
  });
});
```

### Test Coverage

Aim for:
- **Core algorithms**: 90%+ coverage
- **API endpoints**: 80%+ coverage
- **UI components**: 70%+ coverage

---

## Documentation

### Code Documentation

**Rust:**
```rust
/// Brief one-line summary
///
/// Longer description with details about the function,
/// algorithm, or data structure.
///
/// # Arguments
/// * `param1` - Description
///
/// # Returns
/// Description of return value
///
/// # Errors
/// When this function might return an error
///
/// # Examples
/// ```
/// let result = my_function(42);
/// assert_eq!(result, 84);
/// ```
pub fn my_function(param1: i32) -> Result<i32> {
    Ok(param1 * 2)
}
```

**TypeScript:**
```typescript
/**
 * Brief one-line summary
 * 
 * Longer description with details.
 * 
 * @param param1 - Description
 * @returns Description of return value
 * @throws {Error} When this might throw
 * 
 * @example
 * ```typescript
 * const result = myFunction(42);
 * console.log(result); // 84
 * ```
 */
export function myFunction(param1: number): number {
  return param1 * 2;
}
```

### Markdown Documentation

- Use clear headings (`##`, `###`)
- Include code examples
- Add diagrams where helpful
- Link to related docs
- Keep updated with code changes

Place documentation in appropriate subdirectories:
- `docs/architecture/` - System design
- `docs/guides/` - How-to guides
- `docs/integration/` - External integrations
- `docs/design/` - UI/UX specifications

---

## Pull Request Process

### Before Submitting

1. **Update from main:**
   ```bash
   git checkout main
   git pull origin main
   git checkout your-branch
   git rebase main
   ```

2. **Run checks:**
   ```bash
   # Format code
   cargo fmt
   
   # Run linter
   cargo clippy
   
   # Run tests
   cargo test
   
   # Build release
   cargo build --release
   ```

3. **Update documentation:**
   - Update relevant docs in `docs/`
   - Add examples if needed
   - Update CHANGELOG.md

### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Tests pass locally
- [ ] New tests added for new features
- [ ] Integration tests updated

## Documentation
- [ ] Code comments added/updated
- [ ] README updated (if needed)
- [ ] Docs updated in `docs/`

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-reviewed the code
- [ ] No warnings from compiler/linter
- [ ] Dependent changes merged
```

### Review Process

1. **Automated checks** must pass (tests, linting)
2. **Code review** by maintainer
3. **Testing** on dev environment
4. **Approval** and merge to `develop`
5. **Release** merged to `main` periodically

---

## Project Structure

```
SpatialVortex/
├── src/                    # Rust library source
│   ├── bin/               # Binary executables
│   └── *.rs              # Core modules
├── tests/                 # Integration tests
├── web/                   # Svelte UI
│   ├── src/lib/          # Components & utilities
│   └── src/routes/       # SvelteKit routes
├── backend-rs/           # Actix-Web API server
├── docs/                 # Documentation
│   ├── architecture/     # System design
│   ├── guides/           # How-to guides
│   ├── integration/      # External integrations
│   ├── design/           # UI/UX specs
│   └── reports/          # Progress reports
├── examples/             # Example code
├── .archive/            # Historical files
├── Cargo.toml           # Rust dependencies
├── FEATURES.md          # Feature list
└── README.md            # Project overview
```

---

## Getting Help

- **Documentation**: Check `docs/` directory
- **Issues**: [GitHub Issues](https://github.com/WeaveSolutions/SpatialVortex/issues)
- **Discussions**: [GitHub Discussions](https://github.com/WeaveSolutions/SpatialVortex/discussions)

---

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
