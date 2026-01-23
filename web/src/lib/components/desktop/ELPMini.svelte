<script lang="ts">
  import type { ELPValues } from '$lib/types/chat';
  
  export let elp: ELPValues;
  export let position: number | undefined = 0;
  export let confidence: number | undefined = 0;
  
  const sacredPositions = [3, 6, 9];
  const isSacred = position !== undefined && sacredPositions.includes(position);
  
  const max = 13;
  const ethosPercent = ((elp.ethos / max) * 100).toFixed(0);
  const logosPercent = ((elp.logos / max) * 100).toFixed(0);
  const pathosPercent = ((elp.pathos / max) * 100).toFixed(0);
</script>

<div class="elp-mini" class:sacred={isSacred}>
  <div class="elp-bars">
    <div class="bar-group">
      <div class="bar-label">E</div>
      <div class="bar-track">
        <div class="bar-fill ethos" style="width: {ethosPercent}%"></div>
      </div>
      <div class="bar-value">{elp.ethos.toFixed(1)}</div>
    </div>
    <div class="bar-group">
      <div class="bar-label">L</div>
      <div class="bar-track">
        <div class="bar-fill logos" style="width: {logosPercent}%"></div>
      </div>
      <div class="bar-value">{elp.logos.toFixed(1)}</div>
    </div>
    <div class="bar-group">
      <div class="bar-label">P</div>
      <div class="bar-track">
        <div class="bar-fill pathos" style="width: {pathosPercent}%"></div>
      </div>
      <div class="bar-value">{elp.pathos.toFixed(1)}</div>
    </div>
  </div>
  
  <div class="metrics">
    <div class="metric">
      <span class="metric-label">Confidence</span>
      <span class="metric-value">{((confidence || 0) * 100).toFixed(1)}%</span>
    </div>
    <div class="metric">
      <span class="metric-label">Position</span>
      <span class="metric-value" class:sacred={isSacred}>
        {position}{isSacred ? '✨' : ''}
      </span>
    </div>
  </div>
</div>

<style>
  .elp-mini {
    margin-top: 1rem;
    padding: 1rem;
    background: rgba(0, 0, 0, 0.2);
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.05);
  }
  
  .elp-mini.sacred {
    border-color: rgba(236, 72, 153, 0.3);
    background: rgba(236, 72, 153, 0.05);
  }
  
  .elp-bars {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
  }
  
  .bar-group {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.8rem;
  }
  
  .bar-label {
    width: 1.25rem;
    font-weight: 600;
    color: #a1a1aa;
  }
  
  .bar-track {
    flex: 1;
    height: 6px;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 3px;
    overflow: hidden;
  }
  
  .bar-fill {
    height: 100%;
    border-radius: 3px;
    transition: width 0.3s ease;
  }
  
  .bar-fill.ethos {
    background: linear-gradient(90deg, #ef4444 0%, #f87171 100%);
  }
  
  .bar-fill.logos {
    background: linear-gradient(90deg, #3b82f6 0%, #60a5fa 100%);
  }
  
  .bar-fill.pathos {
    background: linear-gradient(90deg, #10b981 0%, #34d399 100%);
  }
  
  .bar-value {
    width: 2.5rem;
    text-align: right;
    color: #e4e4e7;
    font-size: 0.75rem;
    font-weight: 500;
  }
  
  .metrics {
    display: flex;
    justify-content: space-between;
    padding-top: 0.75rem;
    border-top: 1px solid rgba(255, 255, 255, 0.05);
  }
  
  .metric {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.25rem;
  }
  
  .metric-label {
    font-size: 0.65rem;
    color: #71717a;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  
  .metric-value {
    font-size: 0.875rem;
    font-weight: 600;
    color: #a1a1aa;
  }
  
  .metric-value.sacred {
    color: #ec4899;
  }
</style>
