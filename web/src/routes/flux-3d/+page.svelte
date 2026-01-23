<script lang="ts">
  import { onMount } from 'svelte';
  
  let canvasElement: HTMLCanvasElement;
  let isLoading = true;
  let error: string | null = null;
  let wasmNotBuilt = false;
  
  onMount(async () => {
    try {
      // Try to load the WASM module for Bevy visualization
      const module = await import('$lib/wasm/spatial_vortex.js');
      await module.default();
      
      isLoading = false;
      console.log('✅ Flux Matrix 3D WASM loaded successfully');
    } catch (err) {
      // WASM not built yet
      wasmNotBuilt = true;
      isLoading = false;
      console.log('ℹ️ WASM module not built yet. Run build script first.');
    }
  });
</script>

<svelte:head>
  <title>Flux Matrix 3D - SpatialVortex</title>
</svelte:head>

<div class="flux-page">
  <header class="flux-header">
    <h1>🌀 Flux Matrix 3D Visualization</h1>
    <p class="subtitle">Interactive Sacred Geometry with Bevy + WASM</p>
  </header>
  
  <div class="flux-container">
    {#if isLoading}
      <div class="loading">
        <div class="spinner"></div>
        <p>Loading Flux Matrix 3D...</p>
      </div>
    {:else if wasmNotBuilt}
      <div class="build-instructions">
        <h2>🔨 Build Required</h2>
        <p>The WASM module hasn't been built yet. Follow these steps:</p>
        
        <div class="steps">
          <h3>PowerShell (Easiest)</h3>
          <pre><code>.\BUILD_BEVY_FOR_WEB.ps1</code></pre>
          
          <h3>Or Manual Build</h3>
          <pre><code>wasm-pack build --target web --out-dir web/src/lib/wasm --features bevy_support</code></pre>
        </div>
        
        <div class="info-box">
          <h4>📋 What This Does</h4>
          <ul>
            <li>Compiles your existing <code>src/bin/flux_matrix.rs</code> to WASM</li>
            <li>Creates <code>web/src/lib/wasm/spatial_vortex.js</code></li>
            <li>Creates <code>web/src/lib/wasm/spatial_vortex_bg.wasm</code></li>
            <li>Ready for WebGL 3D rendering in browser</li>
          </ul>
        </div>
        
        <div class="note">
          <p><strong>Note:</strong> First build takes 10-20 minutes (Bevy is large). Subsequent builds are much faster.</p>
        </div>
        
        <button on:click={() => window.location.reload()} class="refresh-btn">
          🔄 Refresh After Building
        </button>
      </div>
    {:else if error}
      <div class="error">
        <h3>⚠️ Error Loading Visualization</h3>
        <p>{error}</p>
        <details>
          <summary>Troubleshooting</summary>
          <ol>
            <li>Rebuild WASM: <code>.\BUILD_BEVY_FOR_WEB.ps1</code></li>
            <li>Check browser console for details</li>
            <li>Ensure Bevy compiled with WASM target</li>
          </ol>
        </details>
      </div>
    {:else}
      <div class="canvas-wrapper">
        <canvas id="bevy-canvas" bind:this={canvasElement}></canvas>
        
        <div class="info-overlay">
          <h3>🌀 Sacred Positions</h3>
          <div class="legend">
            <div class="legend-item">
              <span class="dot ethos"></span>
              <span>Ethos (Red)</span>
            </div>
            <div class="legend-item">
              <span class="dot logos"></span>
              <span>Logos (Blue)</span>
            </div>
            <div class="legend-item">
              <span class="dot pathos"></span>
              <span>Pathos (Green)</span>
            </div>
          </div>
          <div class="sacred-info">
            <h4>⭐ 3-6-9 Pattern</h4>
            <p>Position 3: Love (Green)</p>
            <p>Position 6: Truth (Red)</p>
            <p>Position 9: Creation (Blue)</p>
          </div>
          <div class="controls">
            <h3>🎮 Controls</h3>
            <p>Auto-rotating camera</p>
            <p>Space: Spawn word beam</p>
          </div>
          <canvas id="bevy-canvas"></canvas>
          {#if wasmNotBuilt}
            <div class="warning">
              <h2>⚠️ WASM Module Not Built</h2>
              <p>Run the build script first:</p>
              <pre>.\BUILD_BEVY_FOR_WEB.ps1</pre>
            </div>
          {/if}
        </div>
      </div>
    {/if}
  </div>
  
  <footer class="flux-footer">
    <a href="/">← Back to Home</a>
    <span>|</span>
    <a href="/docs">Documentation</a>
  </footer>
</div>

<style>
  .flux-page {
    width: 100%;
    min-height: 100vh;
    background: linear-gradient(135deg, #0a0a1a 0%, #1a1a2e 100%);
    color: white;
    display: flex;
    flex-direction: column;
  }
  
  .flux-header {
    padding: 2rem;
    text-align: center;
    background: rgba(0, 0, 0, 0.3);
    backdrop-filter: blur(10px);
    border-bottom: 2px solid rgba(255, 255, 255, 0.1);
  }
  
  h1 {
    font-size: 2.5em;
    font-weight: 300;
    letter-spacing: 2px;
    margin: 0;
    background: linear-gradient(90deg, #ff4444 0%, #4444ff 50%, #44ff44 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }
  
  .subtitle {
    color: #aaa;
    margin-top: 0.5rem;
  }
  
  .flux-container {
    flex: 1;
    position: relative;
    padding: 2rem;
    display: flex;
    justify-content: center;
    align-items: center;
  }
  
  .loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1.5rem;
  }
  
  .spinner {
    width: 60px;
    height: 60px;
    border: 4px solid rgba(255, 255, 255, 0.1);
    border-top: 4px solid #ff4444;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }
  
  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }
  
  .error {
    background: rgba(255, 68, 68, 0.1);
    border: 2px solid #ff4444;
    border-radius: 12px;
    padding: 2rem;
    max-width: 600px;
    text-align: center;
  }
  
  .error h3 {
    color: #ff4444;
    margin-top: 0;
  }
  
  .error details {
    margin-top: 1rem;
    text-align: left;
  }
  
  .error code {
    background: rgba(0, 0, 0, 0.5);
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    font-size: 0.9em;
  }
  
  .canvas-wrapper {
    width: 100%;
    max-width: 1400px;
    height: 700px;
    position: relative;
    background: rgba(0, 0, 0, 0.3);
    border-radius: 12px;
    overflow: hidden;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  }
  
  #bevy-canvas {
    width: 100%;
    height: 100%;
    display: block;
  }
  
  .info-overlay {
    position: absolute;
    top: 20px;
    right: 20px;
    background: rgba(0, 0, 0, 0.8);
    backdrop-filter: blur(10px);
    padding: 1.5rem;
    border-radius: 10px;
    min-width: 220px;
    border: 1px solid rgba(255, 255, 255, 0.2);
    max-height: calc(100% - 40px);
    overflow-y: auto;
  }
  
  .info-overlay h3,
  .info-overlay h4 {
    margin: 0 0 1rem 0;
    color: #ff4444;
    font-size: 1.1em;
  }
  
  .info-overlay h4 {
    margin-top: 1.5rem;
    font-size: 0.95em;
    color: #ffaa44;
  }
  
  .legend {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  
  .legend-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    font-size: 0.9em;
  }
  
  .dot {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    box-shadow: 0 0 10px currentColor;
  }
  
  .ethos { background: #ff4444; }
  .logos { background: #4444ff; }
  .pathos { background: #44ff44; }
  
  .sacred-info,
  .controls-info {
    padding-top: 1rem;
    border-top: 1px solid rgba(255, 255, 255, 0.2);
    font-size: 0.85em;
  }
  
  .sacred-info p,
  .controls-info p {
    margin: 0.25rem 0;
    color: #ccc;
  }
  
  .flux-footer {
    padding: 1.5rem;
    text-align: center;
    background: rgba(0, 0, 0, 0.3);
    border-top: 1px solid rgba(255, 255, 255, 0.1);
  }
  
  .flux-footer a {
    color: #4444ff;
    text-decoration: none;
    transition: color 0.2s;
  }
  
  .flux-footer a:hover {
    color: #6666ff;
  }
  
  .flux-footer span {
    margin: 0 1rem;
    color: #666;
  }
  
  .build-instructions {
    max-width: 800px;
    background: rgba(0, 0, 0, 0.3);
    border: 2px solid rgba(68, 68, 255, 0.5);
    border-radius: 12px;
    padding: 2rem;
    text-align: left;
  }
  
  .build-instructions h2 {
    color: #4444ff;
    margin-top: 0;
  }
  
  .steps {
    margin: 1.5rem 0;
  }
  
  .steps h3 {
    color: #ffaa44;
    margin-top: 1.5rem;
    margin-bottom: 0.5rem;
  }
  
  .steps pre {
    background: rgba(0, 0, 0, 0.6);
    padding: 1rem;
    border-radius: 6px;
    overflow-x: auto;
  }
  
  .steps code {
    color: #44ff44;
    font-family: 'Consolas', monospace;
  }
  
  .info-box {
    background: rgba(68, 68, 255, 0.1);
    border-left: 4px solid #4444ff;
    padding: 1rem;
    margin: 1.5rem 0;
    border-radius: 4px;
  }
  
  .info-box h4 {
    color: #4444ff;
    margin-top: 0;
  }
  
  .info-box ul {
    margin: 0.5rem 0;
    padding-left: 1.5rem;
  }
  
  .info-box li {
    margin: 0.5rem 0;
  }
  
  .info-box code {
    background: rgba(0, 0, 0, 0.4);
    padding: 0.2rem 0.4rem;
    border-radius: 3px;
    font-size: 0.9em;
  }
  
  .note {
    background: rgba(255, 170, 68, 0.1);
    border-left: 4px solid #ffaa44;
    padding: 1rem;
    margin: 1rem 0;
    border-radius: 4px;
  }
  
  .note strong {
    color: #ffaa44;
  }
  
  .refresh-btn {
    background: #4444ff;
    color: white;
    border: none;
    padding: 1rem 2rem;
    border-radius: 8px;
    font-size: 1em;
    cursor: pointer;
    transition: background 0.2s;
    margin-top: 1.5rem;
  }
  
  .refresh-btn:hover {
    background: #6666ff;
  }
  
  @media (max-width: 768px) {
    .canvas-wrapper {
      height: 500px;
    }
    
    .info-overlay {
      position: static;
      margin-top: 1rem;
      width: 100%;
    }
    
    .build-instructions {
      padding: 1.5rem;
    }
  }
</style>
