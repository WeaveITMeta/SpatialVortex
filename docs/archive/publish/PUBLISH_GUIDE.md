# Publishing to Crates.io

## Pre-Publication Checklist

### 1. Update Cargo.toml Metadata
- [x] Add description
- [x] Add license
- [x] Add authors
- [x] Add keywords (max 5)
- [x] Add categories
- [x] Add repository URL
- [x] **Update repository URL to your actual GitHub URL**
- [x] **Update homepage URL**

### 2. Required Files
- [x] `LICENSE` file created (Apache-2.0)
- [x] `README.md` exists
- [x] `CRATES_IO_README.md` created (concise version)
- [ ] Create `CONTRIBUTING.md` (optional but recommended)
- [ ] Create `CHANGELOG.md` (recommended)

### 3. Code Quality
```bash
# Format code
cargo fmt

# Check for issues
cargo clippy -- -D warnings

# Run tests
cargo test

# Check documentation
cargo doc --no-deps --open

# Verify package builds
cargo build --release
```

### 4. Documentation
```bash
# Generate docs locally
cargo doc --no-deps

# Test doc examples
cargo test --doc

# Check doc coverage
cargo +nightly rustdoc -- -Z unstable-options --show-coverage
```

### 5. Security Audit
```bash
# Install cargo-audit
cargo install cargo-audit

# Run security audit
cargo audit

# Check for outdated dependencies
cargo outdated
```

## Publishing Steps

### 1. Create Crates.io Account
1. Go to https://crates.io/
2. Sign in with GitHub
3. Go to Account Settings → API Tokens
4. Generate new token

### 2. Login to Crates.io
```bash
cargo login <your-api-token>
```

### 3. Dry Run
```bash
# Test package creation without publishing
cargo package --allow-dirty

# Review package contents
cargo package --list

# Check package size
du -sh target/package/spatial-vortex-0.1.0.crate
```

### 4. Publish
```bash
# First release
cargo publish

# If you need to yank a version
cargo yank --vers 0.1.0

# Un-yank if needed
cargo yank --vers 0.1.0 --undo
```

## Post-Publication

### 1. Verify Publication
- Check https://crates.io/crates/spatial-vortex
- Check https://docs.rs/spatial-vortex
- Test installation: `cargo install spatial-vortex`

### 2. Update Documentation
```bash
# Docs.rs builds automatically, but you can trigger rebuild
# Go to https://docs.rs/crate/spatial-vortex/0.1.0
```

### 3. Create Git Tag
```bash
git tag -a v0.1.0 -m "Release version 0.1.0"
git push origin v0.1.0
```

### 4. Create GitHub Release
1. Go to GitHub repository
2. Releases → Create new release
3. Tag: v0.1.0
4. Title: "SpatialVortex v0.1.0 - Initial Release"
5. Add release notes from CHANGELOG.md

## Version Updates

### Semantic Versioning
- **MAJOR** (1.0.0): Breaking API changes
- **MINOR** (0.1.0): New features, backwards compatible
- **PATCH** (0.1.1): Bug fixes, backwards compatible

### Publishing Updates
```bash
# Update version in Cargo.toml
# Update CHANGELOG.md

cargo publish
git tag -a v0.1.1 -m "Release version 0.1.1"
git push origin v0.1.1
```

## Common Issues

### Package Too Large
Max size: 10 MB

**Solution:**
```toml
# Add to Cargo.toml
exclude = [
    "docs/*",
    "tests/*",
    "examples/large_files/*",
    ".github/*",
    "*.mp4",
    "*.png",
]
```

### Missing Dependencies
```bash
# List all dependencies
cargo tree

# Check for missing features
cargo check --all-features
```

### Documentation Errors
```bash
# Check for broken doc links
cargo rustdoc -- -D warnings

# Test all doc examples
cargo test --doc
```

### Failed Build on Docs.rs
- Check docs.rs build log
- Ensure all dependencies are available
- Add `.cargo/config.toml` if needed for custom build configuration

## Repository Setup

### 1. Create GitHub Repository
```bash
# Initialize if not already done
git init
git add .
git commit -m "Initial commit"

# Add remote
git remote add origin https://github.com/yourusername/SpatialVortex.git
git push -u origin main
```

### 2. Add GitHub Workflows
Create `.github/workflows/ci.yml`:
```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test
      - run: cargo clippy -- -D warnings
      - run: cargo fmt -- --check
```

### 3. Add Badges to README
```markdown
[![Build Status](https://github.com/yourusername/SpatialVortex/workflows/CI/badge.svg)](https://github.com/yourusername/SpatialVortex/actions)
[![Crates.io](https://img.shields.io/crates/v/spatial-vortex)](https://crates.io/crates/spatial-vortex)
[![Documentation](https://docs.rs/spatial-vortex/badge.svg)](https://docs.rs/spatial-vortex)
```

## External Testing Setup

### Test Installation
```bash
# In a new project
cargo new test-spatial-vortex
cd test-spatial-vortex

# Add dependency
cargo add spatial-vortex

# Test basic usage
```

### Integration Testing
Create example projects that depend on your crate:
```bash
# Clone test repository
git clone https://github.com/yourusername/spatial-vortex-examples

# Run tests
cd spatial-vortex-examples
cargo test
```

## Maintenance

### Monthly Tasks
- [ ] Update dependencies: `cargo update`
- [ ] Security audit: `cargo audit`
- [ ] Check for deprecations
- [ ] Review issues and PRs

### Before Each Release
- [ ] Update CHANGELOG.md
- [ ] Update version in Cargo.toml
- [ ] Run full test suite
- [ ] Update documentation
- [ ] Review security audit
- [ ] Test examples

## Resources

- **Crates.io Guide**: https://doc.rust-lang.org/cargo/reference/publishing.html
- **Docs.rs FAQ**: https://docs.rs/about
- **Rust API Guidelines**: https://rust-lang.github.io/api-guidelines/
- **Semantic Versioning**: https://semver.org/
