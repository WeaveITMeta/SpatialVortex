# Scripts Quick Reference

Quick command reference for common script operations.

---

## 🏗️ Build Commands

```powershell
# Build web frontend
.\scripts\build\build_web_powershell.ps1

# Build Bevy for web
.\scripts\build\build_bevy_for_web.ps1

# Build Epic Flux 3D
.\scripts\build\build_epic_flux_3d.ps1

# Build documentation
.\scripts\build\build_docs.ps1
```

---

## 🧪 Testing Commands

```powershell
# API Testing
.\scripts\testing\test_health.ps1              # Health check
.\scripts\testing\test_available_endpoints.ps1 # List endpoints
.\scripts\testing\test_all_routes.ps1          # Test all routes
.\scripts\testing\test_chat.ps1                # Test chat API
.\scripts\testing\test_asi.ps1                 # Test ASI endpoints
.\scripts\testing\test_asi_post.ps1            # Test ASI POST

# Agent Testing
.\scripts\testing\run_coding_agent_tests.ps1   # Coding agent tests
.\scripts\testing\test_coding_challenges.ps1   # Challenge tests

# Coverage
.\scripts\testing\measure_coverage.ps1         # Generate coverage
start coverage/index.html                      # View report

# Quick test
.\scripts\testing\quick_test.ps1               # Fast smoke test
```

---

## 🔧 Maintenance Commands

```powershell
# Clean git history
.\scripts\maintenance\clean_history.ps1

# Remove large files
.\scripts\maintenance\remove_large_files.ps1

# Rename VCP references (historical)
.\scripts\maintenance\rename_vcp.ps1
```

---

## 🛠️ Utility Commands

```powershell
# Development
.\scripts\utilities\start_server_dev.ps1       # Start dev server

# Asset Management
.\scripts\utilities\copy_images.ps1            # Copy images to assets

# Quick fixes
.\scripts\utilities\quick_fixes.ps1            # Quick fixes (PowerShell)
./scripts/utilities/quick_fixes.sh             # Quick fixes (Bash)
```

---

## 📦 Common Workflows

### Full Build Pipeline

```powershell
# 1. Build documentation
.\scripts\build\build_docs.ps1

# 2. Build web components
.\scripts\build\build_web_powershell.ps1

# 3. Build 3D visualization
.\scripts\build\build_epic_flux_3d.ps1

# 4. Measure coverage
.\scripts\testing\measure_coverage.ps1
```

### Pre-Release Checklist

```powershell
# 1. Run health checks
.\scripts\testing\test_health.ps1
.\scripts\testing\test_available_endpoints.ps1

# 2. Run full test suite
cargo test
.\scripts\testing\test_all_routes.ps1
.\scripts\testing\test_chat.ps1
.\scripts\testing\test_asi.ps1

# 3. Run agent tests
.\scripts\testing\run_coding_agent_tests.ps1
.\scripts\testing\test_coding_challenges.ps1

# 4. Measure coverage
.\scripts\testing\measure_coverage.ps1

# 5. Build all targets
.\scripts\build\build_web_powershell.ps1
.\scripts\build\build_epic_flux_3d.ps1

# 6. Build documentation
.\scripts\build\build_docs.ps1

# 7. Verify builds
cargo build --release --all-features
```

### Repository Cleanup

```powershell
# 1. Clean build artifacts
cargo clean

# 2. Remove large files
.\scripts\maintenance\remove_large_files.ps1

# 3. Clean history (if needed)
.\scripts\maintenance\clean_history.ps1
```

---

## 🎯 One-Liners

### Build All
```powershell
@("build_web_powershell", "build_epic_flux_3d", "build_docs") | % { .\scripts\build\$_.ps1 }
```

### Clean Everything
```powershell
cargo clean; Remove-Item -Recurse -Force target, .logs -ErrorAction SilentlyContinue
```

### Test All APIs
```powershell
@("test_health", "test_all_routes", "test_chat", "test_asi") | % { .\scripts\testing\$_.ps1 }
```

### Full Test Suite
```powershell
cargo test --all-features && .\scripts\testing\measure_coverage.ps1
```

### Development Server
```powershell
.\scripts\utilities\start_server_dev.ps1
```

---

## 🔗 See Also

- **Full Documentation**: [scripts/README.md](README.md)
- **Build Guide**: `/docs/guides/BUILD_COMMANDS.md`
- **Testing Guide**: `/tests/README.md`

---

**Last Updated**: October 29, 2025  
**Quick Access**: Keep this file handy for common commands

---

## 📝 Recent Updates

**October 29, 2025**:
- ✅ Organized all test scripts into `scripts/testing/`
- ✅ Added 8 API testing scripts (health, routes, chat, ASI, etc.)
- ✅ Added coding agent test scripts
- ✅ Moved `start_server_dev.ps1` to `scripts/utilities/`
- ✅ Updated one-liners and workflows
- ✅ Root directory cleaned of loose scripts
