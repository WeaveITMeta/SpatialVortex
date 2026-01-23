<script lang="ts">
  import type { ELPValues } from '../types/chat';
  
  export let elp: ELPValues;
  export let position: number | undefined = undefined;
  export let confidence: number | undefined = undefined;
  
  // Confidence is the only metric (no separate signal strength)
  $: conf = confidence || 0;
  
  const positionNames: Record<number, string> = {
    0: 'Divine Source',
    1: 'New Beginning',
    2: 'Duality',
    3: 'Sacred Trinity',
    4: 'Foundation',
    5: 'Transformation',
    6: 'Sacred Balance',
    7: 'Wisdom',
    8: 'Potential',
    9: 'Sacred Completion',
  };
  
  function getConfidenceColor(confidence: number): string {
    if (confidence >= 0.7) return '#44ff44';
    if (confidence >= 0.5) return '#ffaa44';
    return '#ff4444';
  }
  
  function getConfidenceLabel(confidence: number): string {
    if (confidence >= 0.7) return 'High';
    if (confidence >= 0.5) return 'Moderate';
    if (confidence >= 0.3) return 'Low';
    return 'Very Low';
  }
  
  $: confPercent = (conf * 100).toFixed(0);
  $: confColor = getConfidenceColor(conf);
  $: confLabel = getConfidenceLabel(conf);
</script>

<div class="elp-viz">
  <div class="elp-channels">
    <div class="channel ethos">
      <div class="channel-label">
        <span class="icon">❤️</span>
        <span>Ethos</span>
      </div>
      <div class="channel-bar">
        <div
          class="channel-fill ethos-fill"
          style="width: {(elp.ethos / 13) * 100}%"
        ></div>
      </div>
      <div class="channel-value">{elp.ethos.toFixed(1)}</div>
    </div>
    
    <div class="channel logos">
      <div class="channel-label">
        <span class="icon">🧠</span>
        <span>Logos</span>
      </div>
      <div class="channel-bar">
        <div
          class="channel-fill logos-fill"
          style="width: {(elp.logos / 13) * 100}%"
        ></div>
      </div>
      <div class="channel-value">{elp.logos.toFixed(1)}</div>
    </div>
    
    <div class="channel pathos">
      <div class="channel-label">
        <span class="icon">💚</span>
        <span>Pathos</span>
      </div>
      <div class="channel-bar">
        <div
          class="channel-fill pathos-fill"
          style="width: {(elp.pathos / 13) * 100}%"
        ></div>
      </div>
      <div class="channel-value">{elp.pathos.toFixed(1)}</div>
    </div>
  </div>
  
  <div class="metrics">
    <div class="metric confidence-metric">
      <div class="metric-label">Confidence</div>
      <div class="metric-value" style="color: {confColor}">
        {confPercent}% <span class="metric-badge">{confLabel}</span>
      </div>
      <div class="confidence-bar">
        <div
          class="confidence-fill"
          style="width: {confPercent}%; background: {confColor}"
        ></div>
      </div>
    </div>
    
    {#if position !== undefined}
      <div class="metric position-metric">
        <div class="metric-label">Flux Position</div>
        <div class="metric-value position-value">
          <span class="position-number">{position}</span>
          <span class="position-name">{positionNames[position]}</span>
        </div>
        {#if [3, 6, 9].includes(position)}
          <div class="sacred-badge">✨ Sacred Position</div>
        {/if}
      </div>
    {/if}
    
    {#if confidence !== undefined}
      <div class="metric confidence-metric">
        <div class="metric-label">Confidence</div>
        <div class="metric-value">{(confidence * 100).toFixed(0)}%</div>
      </div>
    {/if}
  </div>
</div>

<style>
  .elp-viz {
    margin-top: 1rem;
    padding: 1rem;
    background: rgba(0, 0, 0, 0.3);
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.1);
  }
  
  .elp-channels {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    margin-bottom: 1rem;
  }
  
  .channel {
    display: grid;
    grid-template-columns: 100px 1fr 50px;
    gap: 0.5rem;
    align-items: center;
  }
  
  .channel-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.9em;
    color: #aaa;
  }
  
  .icon {
    font-size: 1.2em;
  }
  
  .channel-bar {
    height: 8px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 4px;
    overflow: hidden;
  }
  
  .channel-fill {
    height: 100%;
    transition: width 0.5s ease-out;
    border-radius: 4px;
  }
  
  .ethos-fill {
    background: linear-gradient(90deg, #ff4444 0%, #ff6666 100%);
  }
  
  .logos-fill {
    background: linear-gradient(90deg, #4444ff 0%, #6666ff 100%);
  }
  
  .pathos-fill {
    background: linear-gradient(90deg, #44ff44 0%, #66ff66 100%);
  }
  
  .channel-value {
    text-align: right;
    font-weight: 600;
    color: white;
    font-size: 0.9em;
  }
  
  .metrics {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 1rem;
    padding-top: 1rem;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
  }
  
  .metric {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  
  .metric-label {
    font-size: 0.75em;
    color: #888;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  
  .metric-value {
    font-size: 1.1em;
    font-weight: 600;
    color: white;
  }
  
  .metric-badge {
    display: inline-block;
    font-size: 0.7em;
    padding: 0.2em 0.5em;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 4px;
    margin-left: 0.5em;
  }
  
  .confidence-bar {
    height: 6px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 3px;
    overflow: hidden;
  }
  
  .confidence-fill {
    height: 100%;
    transition: width 0.5s ease-out;
    border-radius: 3px;
  }
  
  .position-value {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
  }
  
  .position-number {
    font-size: 1.5em;
    background: linear-gradient(135deg, #9944ff 0%, #bb66ff 100%);
    background-clip: text;
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }
  
  .position-name {
    font-size: 0.8em;
    color: #999;
  }
  
  .sacred-badge {
    font-size: 0.75em;
    color: #9944ff;
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }
</style>
