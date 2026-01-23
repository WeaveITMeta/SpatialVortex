# SpatialVortex Tools

Development and debugging tools for SpatialVortex.

---

## 📂 Directory Structure

### **debug/** - Debug and Diagnostic Tools
Temporary debugging scripts, diagnostic tools, and troubleshooting utilities.

- **FIX_INFERENCE_ENGINE.rs** - Inference engine diagnostic script
- Temporary fix scripts
- Debug utilities
- Diagnostic tools

**Note**: Files in `debug/` are typically temporary and should be moved to proper locations once issues are resolved.

---

## 🔧 Usage

### Running Debug Scripts

```bash
# Compile and run a debug script
rustc tools/debug/FIX_INFERENCE_ENGINE.rs -o tools/debug/fix_inference
./tools/debug/fix_inference
```

### Adding New Tools

1. Choose appropriate subdirectory
2. Create tool with clear naming
3. Add documentation
4. Update this README if permanent

---

## 📝 Tool Guidelines

### Temporary Tools

Tools in `debug/` are temporary by nature:
- ✅ Quick diagnostic scripts
- ✅ One-off fixes
- ✅ Troubleshooting utilities

### Permanent Tools

For permanent tools:
- Move to `scripts/` directory
- Add proper error handling
- Document usage
- Add to CI/CD if needed

---

## 🗑️ Cleanup

Regularly review `debug/` and:
1. Remove obsolete scripts
2. Move useful tools to `scripts/`
3. Document solutions in main codebase
4. Archive historical fixes if needed

---

**Location**: `/tools/`  
**Purpose**: Development utilities and debugging  
**Status**: Active
