# SpatialVortex Root Directory Guide

**Last Updated**: October 30, 2025

Complete guide to the SpatialVortex project structure and root directory organization.

---

## 📂 Root Directory Structure

```
SpatialVortex/
├── 📄 Configuration Files
│   ├── Cargo.toml              # Rust project manifest
│   ├── Cargo.lock              # Dependency lock file
│   ├── config.toml             # Application configuration
│   ├── config.toml.example     # Configuration template
│   ├── .env.example            # Environment variables template
│   ├── .gitignore              # Git ignore rules
│   ├── .gitattributes          # Git attributes
│   └── .dockerignore           # Docker ignore rules
│
├── 🐳 Deployment
│   ├── Dockerfile              # Docker container definition
│   └── docker-compose.yml      # Multi-container setup
│
├── 📚 Documentation
│   ├── README.md               # Main project README
│   ├── LICENSE                 # Project license
│   └── docs/                   # Complete documentation (200+ files)
│
├── 💻 Source Code
│   ├── src/                    # Rust source code (90+ files)
│   ├── examples/               # Example programs (18 files)
│   ├── tests/                  # Test suite (20+ files)
│   └── benches/                # Benchmark suite
│
├── 🗄️ Data & Resources
│   ├── database/               # Database migrations
│   ├── migrations/             # SQL migrations
│   ├── models/                 # ML models (gitignored)
│   └── assets/                 # Static assets (images, etc.)
│
├── 🌐 Web & Frontend
│   ├── web/                    # Web frontend
│   ├── viewer/                 # 3D viewer application
│   ├── wasm/                   # WebAssembly builds
│   └── api/                    # API server
│
├── 🔧 Development Tools
│   ├── scripts/                # Build and utility scripts
│   ├── tools/                  # Development tools
│   └── .logs/                  # Build logs (gitignored)
│
├── 🚀 Build Artifacts
│   ├── target/                 # Cargo build output (gitignored)
│   └── benchmarks/             # Benchmark data (gitignored)
│
└── 🔌 Backend Services
    └── backend-rs/             # Additional backend services
```

---

## 📋 Directory Purposes

### Configuration Files

**Cargo.toml** - Rust project manifest
- Package metadata
- Dependencies
- Feature flags
- Build configuration

**config.toml** - Application configuration
- Runtime settings
- API endpoints
- Database connections
- Feature toggles

**.env.example** - Environment variables template
- Copy to `.env` for local development
- Contains required environment variables
- Never commit actual `.env` file

**Docker files** - Container deployment
- `Dockerfile` - Container image definition
- `docker-compose.yml` - Multi-service orchestration

---

### Documentation (`docs/`)

**Organization**: 19 categories, 200+ files

Key entry points:
- **README.md** - Documentation hub
- **INDEX.md** - Complete navigation
- **getting-started/** - New user onboarding
- **architecture/** - System design
- **guides/** - How-to tutorials

See `docs/README.md` for complete guide.

---

### Source Code (`src/`)

**Organization**: 8 top-level modules, 90+ files

Key modules:
- **core/** - Mathematical foundation
- **ml/** - Machine Learning & AI
- **data/** - Data structures
- **storage/** - Persistence layer
- **processing/** - Runtime processing
- **ai/** - AI integration & API
- **visualization/** - 3D rendering
- **voice_pipeline/** - Voice processing

See `src/lib.rs` for module exports.

---

### Examples (`examples/`)

**Organization**: 4 categories, 18 examples

Categories:
- **core/** - Core functionality (2 files)
- **ml_ai/** - ML & AI examples (7 files)
- **pipelines/** - Full pipelines (3 files)
- **visualization/** - Graphics (6 files)

See `examples/README.md` for usage guide.

---

### Tests (`tests/`)

**Organization**: 4 categories, 20+ tests

Categories:
- **unit/** - Unit tests (8 files)
- **integration/** - Integration tests (8 files)
- **api/** - API tests (2 files)
- **performance/** - Performance tests (1 file)

See `tests/README.md` for testing guide.

---

### Web & Frontend

**web/** - Main web application
- TypeScript/JavaScript
- React-based frontend
- 300+ files

**viewer/** - 3D visualization viewer
- Dedicated 3D viewer
- WebGL/WebGPU
- 150+ files

**wasm/** - WebAssembly builds
- Rust → WASM compilation
- Browser-compatible modules

**api/** - REST API server
- Actix-web based
- RESTful endpoints
- JSON responses

---

### Data & Resources

**database/** - Database files
- SQLite databases
- Schema definitions

**migrations/** - Database migrations
- SQL migration scripts
- Version tracking

**models/** - ML models (gitignored)
- Large model files (~90MB)
- Download separately
- Not in version control

**assets/** - Static assets
- Images, icons
- Visualizations
- Media files

---

### Development Tools

**scripts/** - Build and utility scripts
- 13 utility scripts
- Build automation
- Deployment helpers

**tools/** - Development tools
- Debug utilities
- Diagnostic scripts
- Temporary fixes

**.logs/** - Build logs (gitignored)
- Compiler output
- Error logs
- Debug traces

---

### Build Artifacts (Gitignored)

**target/** - Cargo build output
- Compiled binaries
- Intermediate files
- Release builds

**benchmarks/** - Benchmark data
- Large benchmark datasets
- Performance results
- >100MB files

---

## 🗂️ File Organization Rules

### What Goes in Root?

✅ **Configuration files** - Project-wide config
✅ **Documentation entry** - README.md
✅ **License** - LICENSE file
✅ **Docker files** - Deployment config
✅ **Build manifest** - Cargo.toml

❌ **Source code** - Goes in `src/`
❌ **Tests** - Goes in `tests/`
❌ **Documentation** - Goes in `docs/`
❌ **Scripts** - Goes in `scripts/` or `tools/`
❌ **Assets** - Goes in `assets/`
❌ **Logs** - Goes in `.logs/`

---

## 🧹 Keeping Root Clean

### Regular Maintenance

**Weekly**:
- Review `.logs/` and clean old logs
- Check `tools/debug/` for obsolete scripts
- Remove temporary files

**Monthly**:
- Review `assets/` for unused files
- Clean up old branches
- Update documentation

**Before Commits**:
- Verify no loose files in root
- Check `.gitignore` is up to date
- Ensure proper file organization

---

## 📝 Adding New Files

### Decision Tree

```
New file to add?
├─ Is it configuration? → Root directory
├─ Is it source code? → src/
├─ Is it a test? → tests/
├─ Is it an example? → examples/
├─ Is it documentation? → docs/
├─ Is it a script? → scripts/ or tools/
├─ Is it an asset? → assets/
└─ Is it temporary? → tools/debug/ or .logs/
```

### File Naming

- **Configuration**: `kebab-case.toml`
- **Source code**: `snake_case.rs`
- **Documentation**: `SCREAMING_SNAKE_CASE.md` or `kebab-case.md`
- **Scripts**: `snake_case.sh` or `kebab-case.ps1`

---

## 🔍 Finding Files

### Common Lookups

**"Where is the main README?"**
→ `/README.md`

**"Where are the docs?"**
→ `/docs/` (see `/docs/INDEX.md`)

**"Where is the source code?"**
→ `/src/` (see `/src/lib.rs`)

**"How do I run examples?"**
→ `/examples/` (see `/examples/README.md`)

**"Where are the tests?"**
→ `/tests/` (see `/tests/README.md`)

**"Where are build scripts?"**
→ `/scripts/`

**"Where do I put images?"**
→ `/assets/images/`

**"Where do logs go?"**
→ `/.logs/` (gitignored)

---

## 🎯 Quick Reference

| Need | Location | README |
|------|----------|--------|
| **Getting Started** | `/docs/getting-started/` | ✅ |
| **API Reference** | `/docs/api/` | ✅ |
| **Examples** | `/examples/` | ✅ |
| **Tests** | `/tests/` | ✅ |
| **Source Code** | `/src/` | `/src/README.md` (if exists) |
| **Scripts** | `/scripts/` | `/scripts/README.md` (if exists) |
| **Tools** | `/tools/` | ✅ |
| **Assets** | `/assets/` | ✅ |

---

## 🚀 Development Workflow

### New Developer Setup

1. Clone repository
2. Read `/README.md`
3. Follow `/docs/getting-started/SETUP.md`
4. Review this guide
5. Explore examples in `/examples/`

### Daily Development

1. Pull latest changes
2. Check `/docs/planning/NEXT_STEPS_FOR_YOU.md`
3. Work in appropriate directories
4. Follow organization rules
5. Commit with clean root

### Before Commits

```bash
# Verify clean structure
ls -la | grep -v "^d"  # Check for loose files

# Review changes
git status

# Ensure proper organization
# All files in correct directories
```

---

## 📊 Directory Statistics

| Directory | Files | Purpose | Status |
|-----------|-------|---------|--------|
| **docs/** | 200+ | Documentation | ✅ Organized |
| **src/** | 90+ | Source code | ✅ Complete |
| **examples/** | 18 | Examples | ✅ Organized |
| **tests/** | 20+ | Test suite | ✅ Organized |
| **scripts/** | 13 | Utilities | ✅ Active |
| **tools/** | Variable | Dev tools | ✅ New |
| **assets/** | Growing | Media | ✅ New |
| **web/** | 300+ | Frontend | ✅ Active |
| **viewer/** | 150+ | 3D Viewer | ✅ Active |

---

## 🎓 Best Practices

### Do's ✅

✅ Keep root directory minimal and clean
✅ Use appropriate subdirectories
✅ Follow naming conventions
✅ Update READMEs when adding files
✅ Use `.gitignore` for build artifacts
✅ Document new directory structures

### Don'ts ❌

❌ Leave loose files in root
❌ Commit build artifacts
❌ Mix concerns (code + docs in same dir)
❌ Create deep nesting (max 3-4 levels)
❌ Use unclear directory names
❌ Forget to update documentation

---

## 🆘 Troubleshooting

### "Where should this file go?"

1. Check decision tree above
2. Ask: "Is this temporary or permanent?"
3. Ask: "What is its primary purpose?"
4. Choose most specific directory
5. When in doubt, ask in team chat

### "Root directory is cluttered"

1. Review this guide
2. Run cleanup checklist
3. Move files to proper locations
4. Update `.gitignore` if needed
5. Commit cleanup separately

### "Can't find a file"

1. Check common lookups above
2. Use `find` or search tool
3. Check `.gitignore` (might be excluded)
4. Review git history (`git log --all --full-history -- **/filename`)

---

## 📚 Additional Resources

- **Main README**: `/README.md`
- **Documentation Index**: `/docs/INDEX.md`
- **Getting Started**: `/docs/getting-started/START_HERE.md`
- **Project Status**: `/docs/status/PROJECT_STATUS.md`

---

**Last Updated**: October 27, 2025  
**Organization Status**: ✅ Complete  
**Maintainability**: High  
**Cleanliness**: Excellent

**Keep it clean!** 🧹
