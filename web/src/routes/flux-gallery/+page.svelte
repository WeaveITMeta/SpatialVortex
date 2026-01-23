<script lang="ts">
  let selectedImage = 'sacred_virtues';
  
  const images = [
    {
      id: 'sacred_virtues',
      title: 'Sacred Virtues',
      file: '/flux-images/flux_matrix_sacred_virtues.png',
      description: 'Classic virtues with balanced ELP distribution',
      sacred: {
        3: 'Love (E:0.70 L:0.50 P:0.95) - High Pathos',
        6: 'Truth (E:0.85 L:0.95 P:0.50) - High Logos',
        9: 'Creation (E:0.90 L:0.60 P:0.50) - High Ethos'
      }
    },
    {
      id: 'emotional_spectrum',
      title: 'Emotional Spectrum',
      file: '/flux-images/flux_matrix_emotional_spectrum.png',
      description: 'Emotional states showing Pathos dominance',
      sacred: {
        3: 'Ecstasy (E:0.50 L:0.30 P:0.98) - Extreme Pathos',
        6: 'Despair (E:0.40 L:0.30 P:0.95) - High Pathos',
        9: 'Euphoria (E:0.60 L:0.40 P:0.92) - High Pathos'
      }
    },
    {
      id: 'logical_concepts',
      title: 'Logical Concepts',
      file: '/flux-images/flux_matrix_logical_concepts.png',
      description: 'Mathematical and logical reasoning - Logos dominant',
      sacred: {
        3: 'Axiom (E:0.50 L:0.95 P:0.30) - High Logos',
        6: 'Theorem (E:0.60 L:0.98 P:0.40) - Extreme Logos',
        9: 'Proof (E:0.70 L:0.92 P:0.35) - High Logos'
      }
    },
    {
      id: 'ethical_principles',
      title: 'Ethical Principles',
      file: '/flux-images/flux_matrix_ethical_principles.png',
      description: 'Moral character and values - Ethos dominant',
      sacred: {
        3: 'Integrity (E:0.95 L:0.60 P:0.50) - High Ethos',
        6: 'Honor (E:0.98 L:0.70 P:0.40) - Extreme Ethos',
        9: 'Virtue (E:0.92 L:0.65 P:0.45) - High Ethos'
      }
    },
    {
      id: 'balanced_concepts',
      title: 'Balanced Concepts',
      file: '/flux-images/flux_matrix_balanced_concepts.png',
      description: 'Harmonious concepts with equal ELP distribution',
      sacred: {
        3: 'Harmony (E:0.75 L:0.75 P:0.75) - Perfect Balance',
        6: 'Unity (E:0.80 L:0.80 P:0.80) - Balanced High',
        9: 'Wholeness (E:0.78 L:0.78 P:0.78) - Balanced'
      }
    }
  ];
  
  $: currentImage = images.find(img => img.id === selectedImage) || images[0];
</script>

<svelte:head>
  <title>Flux Matrix Gallery - SpatialVortex</title>
</svelte:head>

<div class="gallery-page">
  <header class="header">
    <h1>🌀 Flux Matrix Gallery</h1>
    <p class="subtitle">Vortex Math Sacred Geometry Visualizations</p>
    <p class="tech">2D Projections • 3-6-9 Pattern • ELP Tensor System</p>
  </header>
  
  <div class="content">
    <!-- Selector Tabs -->
    <div class="tabs">
      {#each images as image}
        <button 
          class="tab"
          class:active={selectedImage === image.id}
          on:click={() => selectedImage = image.id}
        >
          {image.title}
        </button>
      {/each}
    </div>
    
    <!-- Main Display -->
    <div class="main-display">
      <div class="image-container">
        <img 
          src={currentImage.file} 
          alt={currentImage.title}
          class="flux-image"
        />
      </div>
      
      <div class="info-panel">
        <h2>{currentImage.title}</h2>
        <p class="description">{currentImage.description}</p>
        
        <div class="sacred-positions">
          <h3>⭐ Sacred Positions (3-6-9 Triangle)</h3>
          <div class="position-list">
            <div class="position">
              <span class="pos-num">3</span>
              <span class="pos-desc">{currentImage.sacred[3]}</span>
            </div>
            <div class="position">
              <span class="pos-num">6</span>
              <span class="pos-desc">{currentImage.sacred[6]}</span>
            </div>
            <div class="position">
              <span class="pos-num">9</span>
              <span class="pos-desc">{currentImage.sacred[9]}</span>
            </div>
          </div>
        </div>
        
        <div class="legend">
          <h3>🎨 ELP Color Coding</h3>
          <div class="legend-items">
            <div class="legend-item">
              <span class="color-box ethos"></span>
              <span>Ethos (Red) - Character/Credibility</span>
            </div>
            <div class="legend-item">
              <span class="color-box logos"></span>
              <span>Logos (Blue) - Logic/Reasoning</span>
            </div>
            <div class="legend-item">
              <span class="color-box pathos"></span>
              <span>Pathos (Green) - Emotion/Feeling</span>
            </div>
          </div>
        </div>
        
        <div class="actions">
          <a href="/flux-3d" class="btn btn-primary">
            🌐 View in 3D (Bevy WASM)
          </a>
          <a href="https://github.com/WeaveSolutions/SpatialVortex" class="btn btn-secondary" target="_blank">
            📂 Source Code
          </a>
        </div>
      </div>
    </div>
    
    <!-- Visual Elements Guide -->
    <div class="guide">
      <h3>📐 Visual Elements</h3>
      <div class="guide-grid">
        <div class="guide-item">
          <h4>Position Layout</h4>
          <p>Position 9 at top (12 o'clock), clockwise 1→2→3→4→5→6→7→8, Position 0 at center</p>
        </div>
        <div class="guide-item">
          <h4>Sacred Triangle</h4>
          <p>Bold black triangle connecting positions 3, 6, 9 - The key to universal pattern</p>
        </div>
        <div class="guide-item">
          <h4>Internal Star</h4>
          <p>Doubling sequence 1→2→4→8→7→5→1 creates hexagonal flow pattern</p>
        </div>
        <div class="guide-item">
          <h4>Sphere Size</h4>
          <p>Proportional to tensor magnitude |T| = √(E² + L² + P²)</p>
        </div>
      </div>
    </div>
  </div>
  
  <footer class="footer">
    <a href="/">← Home</a>
    <span>|</span>
    <a href="/flux-3d">3D Visualization</a>
    <span>|</span>
    <span class="credit">Generated with Rust + plotters • Based on Tesla's 3-6-9</span>
  </footer>
</div>

<style>
  .gallery-page {
    min-height: 100vh;
    background: linear-gradient(135deg, #0a0a1a 0%, #1a1a2e 100%);
    color: white;
  }
  
  .header {
    padding: 3rem 2rem 2rem;
    text-align: center;
    background: rgba(0, 0, 0, 0.3);
    backdrop-filter: blur(10px);
    border-bottom: 2px solid rgba(255, 255, 255, 0.1);
  }
  
  h1 {
    font-size: 3em;
    font-weight: 300;
    letter-spacing: 3px;
    margin: 0;
    background: linear-gradient(90deg, #ff4444 0%, #4444ff 50%, #44ff44 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }
  
  .subtitle {
    color: #aaa;
    font-size: 1.2em;
    margin: 0.5rem 0 0;
  }
  
  .tech {
    color: #666;
    font-size: 0.9em;
    margin-top: 0.25rem;
  }
  
  .content {
    max-width: 1600px;
    margin: 0 auto;
    padding: 2rem;
  }
  
  .tabs {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 2rem;
    flex-wrap: wrap;
    justify-content: center;
  }
  
  .tab {
    background: rgba(255, 255, 255, 0.05);
    border: 2px solid rgba(255, 255, 255, 0.1);
    color: white;
    padding: 0.75rem 1.5rem;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s;
    font-size: 0.95em;
  }
  
  .tab:hover {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.3);
  }
  
  .tab.active {
    background: linear-gradient(135deg, #ff4444 0%, #4444ff 100%);
    border-color: transparent;
    font-weight: 600;
  }
  
  .main-display {
    display: grid;
    grid-template-columns: 1fr 400px;
    gap: 2rem;
    margin-bottom: 3rem;
  }
  
  .image-container {
    background: rgba(255, 255, 255, 0.02);
    border: 2px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    padding: 1rem;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  
  .flux-image {
    width: 100%;
    height: auto;
    border-radius: 8px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  }
  
  .info-panel {
    background: rgba(0, 0, 0, 0.4);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    padding: 2rem;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }
  
  .info-panel h2 {
    margin: 0;
    color: #ff4444;
    font-size: 1.8em;
  }
  
  .description {
    color: #aaa;
    line-height: 1.6;
    margin: 0;
  }
  
  .sacred-positions h3,
  .legend h3 {
    color: #ffaa44;
    font-size: 1.1em;
    margin: 0 0 1rem 0;
  }
  
  .position-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  
  .position {
    display: flex;
    gap: 1rem;
    align-items: flex-start;
  }
  
  .pos-num {
    background: #ff4444;
    color: white;
    width: 32px;
    height: 32px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: bold;
    flex-shrink: 0;
  }
  
  .pos-desc {
    color: #ccc;
    font-size: 0.9em;
    line-height: 1.5;
    padding-top: 0.4rem;
  }
  
  .legend-items {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  
  .legend-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    font-size: 0.9em;
    color: #ccc;
  }
  
  .color-box {
    width: 24px;
    height: 24px;
    border-radius: 4px;
    box-shadow: 0 0 10px currentColor;
  }
  
  .ethos { background: #ff4444; }
  .logos { background: #4444ff; }
  .pathos { background: #44ff44; }
  
  .actions {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    padding-top: 1rem;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
  }
  
  .btn {
    padding: 1rem;
    border-radius: 8px;
    text-align: center;
    text-decoration: none;
    font-weight: 600;
    transition: all 0.2s;
    display: block;
  }
  
  .btn-primary {
    background: linear-gradient(135deg, #ff4444 0%, #4444ff 100%);
    color: white;
  }
  
  .btn-primary:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 16px rgba(255, 68, 68, 0.4);
  }
  
  .btn-secondary {
    background: rgba(255, 255, 255, 0.05);
    border: 2px solid rgba(255, 255, 255, 0.2);
    color: white;
  }
  
  .btn-secondary:hover {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.4);
  }
  
  .guide {
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    padding: 2rem;
    margin-bottom: 2rem;
  }
  
  .guide h3 {
    color: #ffaa44;
    margin-top: 0;
    font-size: 1.4em;
  }
  
  .guide-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 1.5rem;
    margin-top: 1.5rem;
  }
  
  .guide-item {
    background: rgba(255, 255, 255, 0.03);
    padding: 1.25rem;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.1);
  }
  
  .guide-item h4 {
    color: #4444ff;
    margin: 0 0 0.5rem 0;
    font-size: 1.1em;
  }
  
  .guide-item p {
    color: #aaa;
    font-size: 0.9em;
    line-height: 1.6;
    margin: 0;
  }
  
  .footer {
    padding: 2rem;
    text-align: center;
    background: rgba(0, 0, 0, 0.3);
    border-top: 1px solid rgba(255, 255, 255, 0.1);
  }
  
  .footer a {
    color: #4444ff;
    text-decoration: none;
    transition: color 0.2s;
  }
  
  .footer a:hover {
    color: #6666ff;
  }
  
  .footer span {
    margin: 0 1rem;
    color: #666;
  }
  
  .credit {
    color: #666;
    font-size: 0.9em;
  }
  
  @media (max-width: 1200px) {
    .main-display {
      grid-template-columns: 1fr;
    }
    
    .info-panel {
      order: 2;
    }
  }
  
  @media (max-width: 768px) {
    h1 {
      font-size: 2em;
    }
    
    .tabs {
      gap: 0.25rem;
    }
    
    .tab {
      padding: 0.6rem 1rem;
      font-size: 0.85em;
    }
    
    .guide-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
