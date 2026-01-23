# Implementation Guides
**Purpose**: How-to guides and step-by-step tutorials  
**Audience**: Developers implementing features

---

## 📂 Contents

### Getting Started
- **QUICK_START.md** - Get up and running in 30 minutes
  - Installation
  - Basic usage
  - First examples
  - Common patterns

### Implementation Roadmaps
- **ACTION_PLAN_ZERO_TO_95.md** - Complete implementation plan
  - Task breakdown
  - Priority ordering
  - Time estimates
  - Success criteria

- **BUILDING_THE_STACK.md** - Tech stack guide
  - Dependencies
  - Architecture decisions
  - Tool selection
  - Best practices

### Setup Guides
- **Database Setup**: `../../database/README.md`
  - PostgreSQL installation
  - Schema creation
  - Connection configuration

- **Voice Pipeline**: `../../src/voice_pipeline/README.md`
  - Audio capture setup
  - FFT configuration
  - Real-time processing

---

## 🎯 Quick Reference

**Just starting?** Follow this path:
1. QUICK_START.md (30 min)
2. Pick a task from ACTION_PLAN_ZERO_TO_95.md
3. Reference BUILDING_THE_STACK.md as needed

**Setting up components?** Check:
- Database: `../../database/README.md`
- Voice: `../../src/voice_pipeline/README.md`
- Redis: `../REDIS_SETUP_WINDOWS.md`

**Building from scratch?** Read:
1. BUILDING_THE_STACK.md - Understand the stack
2. ACTION_PLAN_ZERO_TO_95.md - Task list
3. Architecture docs for design patterns

---

## 📊 Guide Status

| Guide | Status | Last Updated |
|-------|--------|--------------|
| **QUICK_START.md** | ✅ Current | 2025-10-26 |
| **ACTION_PLAN_ZERO_TO_95.md** | ✅ Current | 2025-10-23 |
| **BUILDING_THE_STACK.md** | ✅ Current | 2025-10-23 |
| **Database Guide** | ✅ Current | 2025-10-26 |
| **Voice Pipeline Guide** | ✅ Current | 2025-10-26 |

---

## 🚀 Common Tasks

### Task: Add a new feature
1. Check `../IMPLEMENTATION_STATUS.md` for dependencies
2. Read relevant architecture docs
3. Follow patterns in BUILDING_THE_STACK.md
4. Write tests first (TDD)
5. Implement feature
6. Update documentation

### Task: Set up development environment
1. Follow QUICK_START.md
2. Install dependencies from BUILDING_THE_STACK.md
3. Run `cargo build`
4. Run `cargo test`
5. Try examples in `../../examples/`

### Task: Optimize performance
1. Check `../../benchmarks/README.md`
2. Profile with `cargo flamegraph`
3. Reference lock-free patterns in `../architecture/`
4. Benchmark improvements
5. Document results

---

## 🔗 Related Documentation

- **Architecture**: `../architecture/` - Design patterns
- **Specs**: `../specs/` - Technical specifications
- **Examples**: `../../examples/` - Working code samples
- **Milestones**: `../milestones/` - What's been built

---

**Last Updated**: 2025-10-26  
**Total Guides**: 5+ (including external references)  
**Status**: Comprehensive coverage for common tasks
