# 🌀 Flux 3D on Your Existing Web App (Port 28082)

## Your Setup

You already have a SvelteKit app running on **http://localhost:28082**

---

## Add Bevy 3D Visualization

### Step 1: Build WASM

```powershell
.\BUILD_BEVY_FOR_WEB.ps1
```

This builds your existing `src/bin/flux_matrix.rs` to WASM.

### Step 2: Start/Restart Web Server

```powershell
cd web
npm run dev
```

### Step 3: Visit the Visualization

**http://localhost:28082/flux-3d**

---

## What's Integrated

✅ Your existing SvelteKit app in `web/`  
✅ New route: `web/src/routes/flux-3d/+page.svelte`  
✅ WASM output: `web/src/lib/wasm/`  
✅ Updated: `web/vite.config.ts` for WASM support  

---

## File Structure

```
web/
├── src/
│   ├── routes/
│   │   ├── (your existing routes)
│   │   └── flux-3d/
│   │       └── +page.svelte    ← NEW! 3D visualization
│   └── lib/
│       └── wasm/               ← WASM files go here
├── vite.config.ts              ← Updated for WASM
└── package.json                ← Already has WASM plugins
```

---

## Removed

❌ `web/svelte-app/` - Conflicting duplicate (removed)

---

**Ready!** Just run `.\BUILD_BEVY_FOR_WEB.ps1` and visit http://localhost:28082/flux-3d
