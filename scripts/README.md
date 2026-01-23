# SpatialVortex Scripts

Automation and utility scripts for building, testing, and maintaining SpatialVortex.

---

## 📂 Script Organization

### **build/** - Build Scripts
Compilation and build automation scripts.

**Scripts**:
- **build_bevy_for_web.ps1** - Build Bevy 3D for WebAssembly
- **build_epic_flux_3d.ps1** - Build Epic Flux 3D visualization
- **build_web_powershell.ps1** - Build web frontend
- **build_docs.ps1** - Generate documentation

---

### **testing/** - Testing Scripts
Test execution, coverage measurement, and quality assurance.

**Scripts**:
- **measure_coverage.ps1** - Measure test coverage with tarpaulin

---

### **maintenance/** - Maintenance Scripts
Repository maintenance, cleanup, and refactoring utilities.

**Scripts**:
- **clean_history.ps1** - Clean git history
- **remove_large_files.ps1** - Remove large files from repository
- **rename_vcp.ps1** - Rename Windsurf Cascade → VCP
- **rename_vcp_simple.ps1** - Simple VCP renaming

---

### **utilities/** - General Utilities
Helper scripts and general-purpose utilities.

**Scripts**:
- **copy_images.ps1** - Copy image assets
- **quick_fixes.ps1** - Quick fixes and patches (PowerShell)
- **quick_fixes.sh** - Quick fixes and patches (Bash)

---

## 🚀 Quick Reference

### Build Scripts

**Build Web Frontend**:
```powershell
.\scripts\build\build_web_powershell.ps1
```

**Build Bevy for Web**:
```powershell
.\scripts\build\build_bevy_for_web.ps1
```

**Build Epic Flux 3D**:
```powershell
.\scripts\build\build_epic_flux_3d.ps1
```

**Build Documentation**:
```powershell
.\scripts\build\build_docs.ps1
```

---

### Testing Scripts

**Measure Test Coverage**:
```powershell
.\scripts\testing\measure_coverage.ps1
```

Requirements:
- cargo-tarpaulin installed
- Rust toolchain

---

### Maintenance Scripts

**Clean Git History**:
```powershell
.\scripts\maintenance\clean_history.ps1
```

**Remove Large Files**:
```powershell
.\scripts\maintenance\remove_large_files.ps1
```

**Rename VCP** (Historical):
```powershell
.\scripts\maintenance\rename_vcp.ps1
```

---

### Utilities

**Copy Images**:
```powershell
.\scripts\utilities\copy_images.ps1
```

**Quick Fixes**:
```powershell
# PowerShell
.\scripts\utilities\quick_fixes.ps1

# Bash (Linux/Mac)
./scripts/utilities/quick_fixes.sh
```

---

## 📋 Script Naming Convention

All scripts follow consistent naming:
- **snake_case.ps1** - PowerShell scripts
- **snake_case.sh** - Bash scripts
- **Descriptive names** - Clear purpose
- **Category prefix** - build_, test_, etc. (optional)

---

## 🔧 Prerequisites

### PowerShell Scripts
- Windows PowerShell 5.1+ or PowerShell Core 7+
- Execution policy: `Set-ExecutionPolicy RemoteSigned -Scope CurrentUser`

### Bash Scripts
- Bash 4.0+
- Unix-like environment (Linux, macOS, WSL)

### Build Scripts
- Rust toolchain (rustc, cargo)
- wasm-pack (for WebAssembly builds)
- Node.js (for web builds)

### Testing Scripts
- cargo-tarpaulin: `cargo install cargo-tarpaulin`

---

## 📝 Adding New Scripts

### 1. Choose Category

- **build/** - Compilation, building
- **testing/** - Testing, coverage, QA
- **maintenance/** - Cleanup, refactoring
- **utilities/** - General helpers

### 2. Create Script

```powershell
# Example: scripts/build/build_new_component.ps1

# Description: Build new component
# Usage: .\scripts\build\build_new_component.ps1

Write-Host "Building new component..." -ForegroundColor Cyan

# Your build logic here
cargo build --package new_component --release

Write-Host "✅ Build complete!" -ForegroundColor Green
```

### 3. Document

- Add entry to this README
- Include usage examples
- List prerequisites
- Add comments in script

---

## 🎯 Script Templates

### PowerShell Build Script Template

```powershell
#!/usr/bin/env pwsh
# Script: build_component.ps1
# Purpose: Build specific component
# Usage: .\scripts\build\build_component.ps1 [options]

param(
    [switch]$Release,
    [switch]$Verbose
)

$ErrorActionPreference = "Stop"

Write-Host "🔨 Building Component" -ForegroundColor Cyan
Write-Host "=====================" -ForegroundColor Cyan
Write-Host ""

try {
    # Build logic
    if ($Release) {
        cargo build --release
    } else {
        cargo build
    }
    
    Write-Host ""
    Write-Host "✅ Build successful!" -ForegroundColor Green
    exit 0
}
catch {
    Write-Host ""
    Write-Host "❌ Build failed: $_" -ForegroundColor Red
    exit 1
}
```

### Bash Script Template

```bash
#!/bin/bash
# Script: build_component.sh
# Purpose: Build specific component
# Usage: ./scripts/build/build_component.sh [options]

set -e

echo "🔨 Building Component"
echo "====================="
echo ""

# Build logic
if [ "$1" == "--release" ]; then
    cargo build --release
else
    cargo build
fi

echo ""
echo "✅ Build successful!"
```

---

## 🔍 Script Guidelines

### Do's ✅

✅ Use clear, descriptive names  
✅ Add error handling  
✅ Include usage comments  
✅ Test before committing  
✅ Follow naming convention  
✅ Add to appropriate category  
✅ Document in README  

### Don'ts ❌

❌ Hardcode sensitive data  
❌ Use absolute paths  
❌ Ignore errors  
❌ Create deeply nested scripts  
❌ Mix multiple concerns  
❌ Leave undocumented scripts  

---

## 🐛 Debugging Scripts

### PowerShell

```powershell
# Enable verbose output
$VerbosePreference = "Continue"
.\script.ps1

# Debug mode
Set-PSDebug -Trace 1
.\script.ps1
Set-PSDebug -Off

# See all errors
$ErrorActionPreference = "Continue"
```

### Bash

```bash
# Enable debug mode
bash -x ./script.sh

# Verbose output
bash -v ./script.sh

# Both
bash -xv ./script.sh
```

---

## 📊 Script Inventory

### By Category

| Category | Scripts | Purpose |
|----------|---------|---------|
| **build/** | 4 | Compilation and building |
| **testing/** | 1 | Test coverage and QA |
| **maintenance/** | 4 | Repository maintenance |
| **utilities/** | 3 | General utilities |

**Total**: 12 scripts

### By Platform

| Platform | Count |
|----------|-------|
| **PowerShell** | 11 |
| **Bash** | 1 |

---

## 🔒 Security Notes

### Execution Policy

PowerShell scripts require appropriate execution policy:

```powershell
# Check current policy
Get-ExecutionPolicy

# Set for current user
Set-ExecutionPolicy RemoteSigned -Scope CurrentUser

# Or bypass for single script
powershell -ExecutionPolicy Bypass -File .\script.ps1
```

### Script Signing

For production, consider signing scripts:
```powershell
# Get certificate
$cert = Get-ChildItem Cert:\CurrentUser\My -CodeSigningCert

# Sign script
Set-AuthenticodeSignature -FilePath .\script.ps1 -Certificate $cert
```

---

## 📈 Performance Tips

### Build Scripts

- Use `--release` for production builds
- Use `--target` to specify target platform
- Use `-j` for parallel builds: `cargo build -j 4`
- Cache dependencies between builds

### Test Scripts

- Run tests in parallel: `cargo test -- --test-threads=4`
- Use `--release` for performance tests
- Profile with `cargo flamegraph`

---

## 🆘 Common Issues

### PowerShell: Cannot Run Scripts

**Problem**: "cannot be loaded because running scripts is disabled"

**Solution**:
```powershell
Set-ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### Bash: Permission Denied

**Problem**: "Permission denied"

**Solution**:
```bash
chmod +x ./script.sh
./script.sh
```

### Build Fails

**Problem**: Build script fails

**Solutions**:
1. Check Rust toolchain: `rustc --version`
2. Update dependencies: `cargo update`
3. Clean build: `cargo clean && cargo build`
4. Check error messages in output

---

## 🔗 Related Documentation

- **Build Guide**: `/docs/guides/BUILD_COMMANDS.md`
- **Testing Guide**: `/tests/README.md`
- **Contributing**: `/docs/getting-started/CONTRIBUTING.md`
- **Project Status**: `/docs/status/PROJECT_STATUS.md`

---

## 📚 Additional Resources

### PowerShell
- [PowerShell Documentation](https://docs.microsoft.com/powershell/)
- [About Execution Policies](https://docs.microsoft.com/powershell/module/microsoft.powershell.core/about/about_execution_policies)

### Bash
- [Bash Reference Manual](https://www.gnu.org/software/bash/manual/)
- [ShellCheck](https://www.shellcheck.net/) - Script analyzer

### Cargo/Rust
- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [Rust Build Scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html)

---

## 🎓 Best Practices

### Script Structure

1. **Header comment** - Purpose, usage, author
2. **Parameter handling** - Clear options
3. **Error handling** - Try/catch, set -e
4. **Progress output** - User feedback
5. **Exit codes** - 0 for success, non-zero for failure

### Error Handling

```powershell
# PowerShell
$ErrorActionPreference = "Stop"
try {
    # Script logic
}
catch {
    Write-Host "Error: $_" -ForegroundColor Red
    exit 1
}
```

```bash
# Bash
set -e  # Exit on error
set -u  # Error on undefined variable
set -o pipefail  # Error in pipeline

trap 'echo "Error on line $LINENO"' ERR
```

---

## 📝 Maintenance

### Regular Reviews

- Monthly: Review for obsolete scripts
- Quarterly: Update documentation
- Yearly: Refactor and consolidate

### Deprecation

When removing scripts:
1. Mark as deprecated in README
2. Add deprecation notice in script
3. Wait one version cycle
4. Remove script
5. Update documentation

---

**Location**: `/scripts/`  
**Organization**: ✅ Complete  
**Maintained**: Actively  
**Documentation**: Comprehensive

**Ready to automate!** 🚀
