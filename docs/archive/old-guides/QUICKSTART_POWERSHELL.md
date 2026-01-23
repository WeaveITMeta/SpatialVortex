# 🚀 Quickstart for PowerShell Users

## PowerShell Commands (Not Bash!)

PowerShell doesn't support `&&` - use these instead:

---

## 2D Visualization (Works Now!)

```powershell
cargo run --example flux_2d_visualization
```

**Output**: `flux_matrix_2d.png`

---

## 3D Desktop (Your Existing Code!)

```powershell
cargo run --bin flux_matrix --features bevy_support
```

**Interactive 3D with Bevy**

---

## 3D Web (WASM + Svelte)

### Option 1: Automated Script

```powershell
.\BUILD_WEB_POWERSHELL.ps1
```

### Option 2: Manual Steps

```powershell
# 1. Install wasm-pack (one-time)
cargo install wasm-pack

# 2. Build WASM
wasm-pack build --target web --out-dir web/wasm --features bevy_support

# 3. Create Svelte app (one-time)
cd web
npm create vite@latest svelte-app -- --template svelte

# 4. Install dependencies
cd svelte-app
npm install
npm install -D vite-plugin-wasm vite-plugin-top-level-await

# 5. Run dev server
npm run dev

# 6. Return to root
cd ..\..
```

---

## Common PowerShell Tips

### ❌ Don't Use (Bash syntax)
```bash
cd web && npm install && npm run dev
```

### ✅ Use Instead (PowerShell)
```powershell
cd web
npm install
npm run dev
```

### Or Use Semicolons
```powershell
cd web; npm install; npm run dev
```

---

## Quick Test

```powershell
# Test if tools are installed
Get-Command cargo
Get-Command npm
Get-Command wasm-pack

# Check Rust features
cargo build --features bevy_support
```

---

## Troubleshooting

### "The token '&&' is not a valid statement separator"
**Solution**: Use `;` or separate lines in PowerShell

### "wasm-pack: command not found"
```powershell
cargo install wasm-pack
# Add to PATH: $env:PATH += ";$HOME\.cargo\bin"
```

### "npm: command not found"
Download Node.js from: https://nodejs.org/

---

## File Locations

After build:
- WASM: `web/wasm/spatial_vortex_bg.wasm`
- JS: `web/wasm/spatial_vortex.js`
- Svelte: `web/svelte-app/src/`

---

**Ready to visualize!** 🌀
