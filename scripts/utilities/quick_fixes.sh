#!/bin/bash
# Quick Fixes for SpatialVortex Warnings
# Run this script to automatically fix code warnings

echo "🔧 Applying automatic fixes..."

# Fix 1: Automatically fix unused imports and variables
echo "Fixing unused code..."
cargo fix --lib --allow-dirty --allow-staged
cargo fix --bin geometric_reasoning_benchmark --allow-dirty --allow-staged

# Fix 2: Kill locked processes (Windows - uncomment if needed)
# taskkill /F /IM spatial-vortex.exe 2>/dev/null || true

# Fix 3: Run tests after fixes
echo "Running tests..."
cargo test --lib

# Fix 4: Re-check for warnings
echo "Checking for remaining warnings..."
cargo check --benches

echo "✅ Fixes applied! Check output above for results."
