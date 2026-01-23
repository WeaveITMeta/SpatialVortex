<script lang="ts">
  import { onMount } from 'svelte';
  
  let isLoading = true;
  let error: string | null = null;
  let mounted = false;
  
  onMount(async () => {
    mounted = true;
    console.log('Page mounted, attempting to load WASM...');
    
    try {
      console.log('Importing epic_flux_3d module...');
      const module = await import('$lib/wasm/epic_flux_3d.js');
      console.log('Module imported successfully!');
      console.log('Available functions:', Object.keys(module));
      
      // First initialize the WASM module
      console.log('Initializing WASM module...');
      await module.default();
      console.log('✅ WASM module initialized!');
      
      // Wait for next frame to ensure canvas is in DOM
      await new Promise(resolve => requestAnimationFrame(resolve));
      
      // Verify canvas exists
      const canvas = document.getElementById('bevy-canvas');
      if (!canvas) {
        throw new Error('Canvas element #bevy-canvas not found in DOM');
      }
      console.log('✅ Canvas element found:', canvas);
      
      // Call init function (has guard to prevent multiple calls)
      console.log('Calling epic_flux_3d_init()...');
      module.epic_flux_3d_init();
      
      isLoading = false;
      console.log('✅ Epic Flux 3D started!');
    } catch (err) {
      console.error('❌ Failed to load WASM:', err);
      error = `WASM module error.\n\nError: ${err}`;
      isLoading = false;
    }
  });
</script>

<svelte:head>
  <title>🌀 Epic Flux 3D - SpatialVortex</title>
</svelte:head>

<div class="epic-page">
  <header>
    <h1>🌀 Epic Flux Matrix 3D</h1>
    <p>Consolidated Sacred Geometry Visualization</p>
  </header>
  
  <div class="canvas-container">
    <!-- Canvas is always present for Bevy to find -->
    <canvas id="bevy-canvas" style="display: {isLoading || error ? 'none' : 'block'}; width: 100%; height: 100%;"></canvas>
    
    {#if !mounted}
      <div class="loading">
        <div class="spinner"></div>
        <p>Initializing...</p>
      </div>
    {:else if isLoading}
      <div class="loading">
        <div class="spinner"></div>
        <p>Loading Epic Visualization...</p>
        <p class="note">Initializing WASM and Bevy...</p>
        <p class="debug">Check browser console (F12) for status</p>
      </div>
    {:else if error}
      <div class="error">
        <h3>⚠️ Error Loading Visualization</h3>
        <pre class="details">{error}</pre>
        <div class="instructions">
          <h4>Troubleshooting:</h4>
          <ul>
            <li>Check browser console (F12) for details</li>
            <li>Ensure WebGL/WebGPU is enabled</li>
            <li>Try refreshing the page</li>
          </ul>
          <button on:click={() => window.location.reload()}>🔄 Refresh Page</button>
        </div>
      </div>
    {/if}
  </div>
  
  <footer>
    <div class="legend">
      <h3>Epic Features</h3>
      <ul>
        <li>✨ Sacred Triangle (3-6-9) in Cyan</li>
        <li>🔄 Flow Pattern (1→2→4→8→7→5→1)</li>
        <li>🎨 ELP Color Channels (Red/Green/Blue)</li>
        <li>📦 Shape Architecture (Box/Cylinder/Sphere)</li>
        <li>💫 Sacred Intersection Effects</li>
        <li>🎥 Auto-rotating Camera</li>
      </ul>
    </div>
  </footer>
</div>

<style>
  .epic-page {
    width: 100vw;
    height: 100vh;
    background: linear-gradient(135deg, #0a0a1a 0%, #1a1a2e 100%);
    color: white;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  
  header {
    padding: 0.5rem;
    text-align: center;
    background: rgba(0, 0, 0, 0.5);
    border-bottom: 1px solid rgba(0, 191, 255, 0.3);
  }
  
  h1 {
    margin: 0;
    font-size: 1.2em;
    background: linear-gradient(90deg, #00bfff, #00ffff);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }
  
  header p {
    display: none;  /* Hide subtitle to save space */
  }
  
  .canvas-container {
    flex: 1;
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  
  #bevy-canvas {
    width: 100%;
    height: 100%;
  }
  
  .loading, .error {
    padding: 3rem;
    border-radius: 12px;
    background: rgba(20, 20, 40, 0.95);
    backdrop-filter: blur(10px);
    text-align: center;
    max-width: 700px;
    border: 2px solid rgba(0, 191, 255, 0.5);
    box-shadow: 0 8px 32px rgba(0, 191, 255, 0.2);
  }
  
  .loading p {
    color: #ddd;
    line-height: 1.6;
  }
  
  .debug {
    color: #888;
    font-size: 0.85em;
    margin-top: 1rem;
  }
  
  .spinner {
    width: 60px;
    height: 60px;
    margin: 0 auto 1rem;
    border: 4px solid rgba(0, 191, 255, 0.2);
    border-top: 4px solid #00bfff;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }
  
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  
  .loading .note {
    color: #00bfff;
    font-size: 0.9em;
    margin-top: 1rem;
  }
  
  .error {
    border: 2px solid #ffaa44;
  }
  
  .error h3 {
    color: #ffaa44;
    margin-top: 0;
  }
  
  .error .details {
    font-size: 0.8em;
    color: #aaa;
    margin: 1.5rem 0;
    padding: 1rem;
    background: rgba(0, 0, 0, 0.5);
    border-radius: 6px;
    font-family: 'Consolas', monospace;
    text-align: left;
    max-height: 200px;
    overflow-y: auto;
    white-space: pre-wrap;
    word-break: break-word;
  }
  
  .instructions ul {
    text-align: left;
    margin: 1rem 0;
    padding-left: 2rem;
  }
  
  .instructions li {
    margin: 0.5rem 0;
    color: #ddd;
  }
  
  .instructions {
    margin-top: 2rem;
    padding: 1.5rem;
    background: rgba(0, 191, 255, 0.1);
    border-radius: 8px;
    border: 1px solid rgba(0, 191, 255, 0.3);
  }
  
  .instructions h4 {
    color: #00bfff;
    margin: 0 0 1rem 0;
  }
  
  button {
    margin-top: 1rem;
    padding: 0.75rem 2rem;
    background: #00bfff;
    color: white;
    border: none;
    border-radius: 6px;
    font-size: 1em;
    cursor: pointer;
    transition: background 0.2s;
  }
  
  button:hover {
    background: #0099cc;
  }
  
  footer {
    padding: 1rem;
    background: rgba(0, 0, 0, 0.3);
    border-top: 1px solid rgba(255, 255, 255, 0.1);
  }
  
  .legend {
    max-width: 900px;
    margin: 0 auto;
  }
  
  .legend h3 {
    color: #00bfff;
    margin: 0 0 0.75rem 0;
    font-size: 1.1em;
  }
  
  .legend ul {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 0.5rem;
    list-style: none;
    padding: 0;
    margin: 0;
  }
  
  .legend li {
    font-size: 0.9em;
    padding: 0.25rem 0;
  }
  
  @media (max-width: 768px) {
    h1 {
      font-size: 1.8em;
    }
    
    .loading, .error {
      padding: 2rem 1rem;
    }
    
    .legend ul {
      grid-template-columns: 1fr;
    }
  }
</style>
