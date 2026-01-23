# 🌐 Building Flux Matrix 3D for Web (WASM + Svelte)

## Prerequisites

```bash
# Install wasm-pack
cargo install wasm-pack

# Install Node.js/npm (if not already installed)
# https://nodejs.org/

# Install Svelte (via create-vite or SvelteKit)
npm create vite@latest svelte-app -- --template svelte
```

---

## Step 1: Build Rust WASM Module

```bash
cd E:\Libraries\SpatialVortex

# Build for web target with Bevy support
wasm-pack build \
  --target web \
  --out-dir web/svelte-app/src/wasm \
  --features bevy_support \
  wasm/flux_3d_web.rs

# This creates:
# - web/svelte-app/src/wasm/flux_3d_web.js
# - web/svelte-app/src/wasm/flux_3d_web_bg.wasm
# - web/svelte-app/src/wasm/flux_3d_web.d.ts
```

---

## Step 2: Setup Svelte Project

```bash
cd web/svelte-app

# Install dependencies
npm install

# Install any additional packages if needed
npm install -D vite-plugin-wasm vite-plugin-top-level-await
```

### Update `vite.config.js`

```javascript
import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import wasm from 'vite-plugin-wasm'
import topLevelAwait from 'vite-plugin-top-level-await'

export default defineConfig({
  plugins: [
    svelte(),
    wasm(),
    topLevelAwait()
  ],
  server: {
    fs: {
      // Allow serving files from WASM directory
      allow: ['..']
    }
  }
})
```

---

## Step 3: Use Flux Visualization Component

### In `src/App.svelte`:

```svelte
<script>
  import FluxVisualization from './FluxVisualization.svelte';
</script>

<main>
  <h1>🌀 Spatial Vortex - 3D Flux Matrix</h1>
  <FluxVisualization />
</main>

<style>
  main {
    padding: 20px;
    max-width: 1400px;
    margin: 0 auto;
    font-family: system-ui, -apple-system, sans-serif;
  }
  
  h1 {
    text-align: center;
    color: #333;
    margin-bottom: 30px;
  }
</style>
```

---

## Step 4: Run Development Server

```bash
npm run dev
```

Visit: `http://localhost:5173`

---

## Step 5: Build for Production

```bash
# Build Svelte app
npm run build

# Output will be in: dist/
# Deploy to any static host (Netlify, Vercel, etc.)
```

---

## Troubleshooting

### Issue: WASM module not found
**Solution**: Make sure wasm-pack output directory matches import path in `FluxVisualization.svelte`

### Issue: "SharedArrayBuffer is not defined"
**Solution**: Configure server headers for cross-origin isolation:

```javascript
// vite.config.js
export default defineConfig({
  plugins: [/* ... */],
  server: {
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp'
    }
  }
})
```

### Issue: Bevy window not rendering
**Solution**: Ensure canvas has proper ID and dimensions:
```html
<canvas id="bevy-canvas" style="width: 100%; height: 600px;"></canvas>
```

---

## Project Structure

```
SpatialVortex/
├── wasm/
│   └── flux_3d_web.rs          # WASM entry point
├── web/
│   ├── svelte-app/
│   │   ├── src/
│   │   │   ├── wasm/           # Generated WASM files
│   │   │   ├── FluxVisualization.svelte
│   │   │   └── App.svelte
│   │   ├── package.json
│   │   └── vite.config.js
│   └── BUILD_INSTRUCTIONS.md   # This file
└── src/
    └── visualization/
        └── bevy_3d.rs          # 3D rendering logic
```

---

## Features

✅ **Interactive 3D Visualization**
- Auto-rotating camera
- Sacred geometry (3-6-9 triangle)
- ELP color coding (Red/Blue/Green)
- Real-time WebGL rendering

✅ **Sacred Position Halos**
- Positions 3, 6, 9 get glowing halos
- Triangle markers for sacred points

✅ **Flow Lines**
- Connections between adjacent positions
- Sacred flow highlighting

✅ **Svelte Integration**
- Clean component API
- Loading states
- Error handling
- Responsive design

---

## Performance

| Metric | Value |
|--------|-------|
| WASM size | ~2-4 MB (with Bevy) |
| Load time | 2-5s (first load) |
| FPS | 60 (WebGL) |
| Memory | ~50-100 MB |

---

## Next Steps

1. **Custom Camera Controls** - Add mouse drag rotation
2. **Data Updates** - Real-time flux matrix updates
3. **Click Interactions** - Select data points
4. **Animation** - Animate flow between positions
5. **Mobile Support** - Touch controls

---

**Status**: Ready for web deployment with WASM + Svelte!
