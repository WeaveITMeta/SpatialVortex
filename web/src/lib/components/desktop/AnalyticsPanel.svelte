<script lang="ts">
  import type { ChatMessage } from '$lib/types/chat';
  
  export let messages: ChatMessage[] = [];
  
  $: aiMessages = messages.filter(m => m.role === 'assistant' && m.elp);
  $: avgEthos = aiMessages.length > 0
    ? aiMessages.reduce((sum, m) => sum + (m.elp?.ethos || 0), 0) / aiMessages.length
    : 0;
  $: avgLogos = aiMessages.length > 0
    ? aiMessages.reduce((sum, m) => sum + (m.elp?.logos || 0), 0) / aiMessages.length
    : 0;
  $: avgPathos = aiMessages.length > 0
    ? aiMessages.reduce((sum, m) => sum + (m.elp?.pathos || 0), 0) / aiMessages.length
    : 0;
  $: avgConfidence = aiMessages.length > 0
    ? aiMessages.reduce((sum, m) => sum + (m.confidence || 0), 0) / aiMessages.length
    : 0;
  $: sacredCount = aiMessages.filter(m => [3, 6, 9].includes(m.flux_position || 0)).length;
</script>

<aside class="analytics-panel">
  <div class="panel-header">
    <h3>📊 Analytics</h3>
  </div>
  
  <div class="panel-content">
    {#if aiMessages.length === 0}
      <div class="empty-analytics">
        <p>Start chatting to see analytics</p>
      </div>
    {:else}
      <div class="stat-card">
        <div class="stat-label">Average ELP</div>
        <div class="elp-avg">
          <div class="avg-item">
            <span class="avg-label ethos">Ethos</span>
            <span class="avg-value">{avgEthos.toFixed(1)}</span>
          </div>
          <div class="avg-item">
            <span class="avg-label logos">Logos</span>
            <span class="avg-value">{avgLogos.toFixed(1)}</span>
          </div>
          <div class="avg-item">
            <span class="avg-label pathos">Pathos</span>
            <span class="avg-value">{avgPathos.toFixed(1)}</span>
          </div>
        </div>
      </div>
      
      <div class="stat-card">
        <div class="stat-label">Confidence</div>
        <div class="confidence-display">
          <div class="confidence-bar">
            <div 
              class="confidence-fill" 
              style="width: {(avgConfidence * 100).toFixed(0)}%"
            ></div>
          </div>
          <div class="confidence-value">{(avgConfidence * 100).toFixed(1)}%</div>
        </div>
      </div>
      
      <div class="stat-card">
        <div class="stat-label">Messages</div>
        <div class="message-stats">
          <div class="stat-row">
            <span>Total</span>
            <span class="stat-num">{messages.length}</span>
          </div>
          <div class="stat-row">
            <span>Sacred Positions</span>
            <span class="stat-num sacred">{sacredCount}</span>
          </div>
        </div>
      </div>
      
      <div class="stat-card">
        <div class="stat-label">Position Distribution</div>
        <div class="position-grid">
          {#each [0,1,2,3,4,5,6,7,8,9] as pos}
            {@const count = aiMessages.filter(m => m.flux_position === pos).length}
            {@const isSacred = [3,6,9].includes(pos)}
            <div 
              class="position-cell"
              class:sacred={isSacred}
              class:has-data={count > 0}
              title="Position {pos}: {count} messages"
            >
              <div class="pos-num">{pos}</div>
              <div class="pos-count">{count || ''}</div>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </div>
</aside>

<style>
  .analytics-panel {
    width: 320px;
    background: linear-gradient(180deg, #1a1a2e 0%, #16161f 100%);
    border-left: 1px solid rgba(255, 255, 255, 0.06);
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow-y: auto;
  }
  
  .panel-header {
    padding: 1.5rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }
  
  .panel-header h3 {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 600;
    color: #e4e4e7;
  }
  
  .panel-content {
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  
  .empty-analytics {
    padding: 3rem 1rem;
    text-align: center;
    color: #71717a;
    font-size: 0.9rem;
  }
  
  .stat-card {
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 12px;
    padding: 1rem;
  }
  
  .stat-label {
    font-size: 0.75rem;
    font-weight: 600;
    color: #71717a;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 0.75rem;
  }
  
  .elp-avg {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  
  .avg-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  
  .avg-label {
    font-size: 0.85rem;
    font-weight: 500;
  }
  
  .avg-label.ethos {
    color: #ef4444;
  }
  
  .avg-label.logos {
    color: #3b82f6;
  }
  
  .avg-label.pathos {
    color: #10b981;
  }
  
  .avg-value {
    font-size: 1rem;
    font-weight: 600;
    color: #e4e4e7;
  }
  
  .confidence-display {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }
  
  .confidence-bar {
    flex: 1;
    height: 8px;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 4px;
    overflow: hidden;
  }
  
  .confidence-fill {
    height: 100%;
    background: linear-gradient(90deg, #60a5fa 0%, #3b82f6 100%);
    border-radius: 4px;
    transition: width 0.3s ease;
  }
  
  .confidence-value {
    font-weight: 600;
    color: #e4e4e7;
    font-size: 0.9rem;
    min-width: 50px;
    text-align: right;
  }
  
  .message-stats {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  
  .stat-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.85rem;
    color: #a1a1aa;
  }
  
  .stat-num {
    font-weight: 600;
    color: #e4e4e7;
  }
  
  .stat-num.sacred {
    color: #60a5fa;
  }
  
  .position-grid {
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    gap: 0.5rem;
  }
  
  .position-cell {
    aspect-ratio: 1;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 8px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.25rem;
    transition: all 0.2s;
  }
  
  .position-cell.has-data {
    background: rgba(96, 165, 250, 0.1);
    border-color: rgba(96, 165, 250, 0.2);
  }
  
  .position-cell.sacred {
    background: rgba(96, 165, 250, 0.1);
    border-color: rgba(96, 165, 250, 0.4);
  }
  
  .position-cell.sacred.has-data {
    background: rgba(96, 165, 250, 0.15);
    border-color: rgba(96, 165, 250, 0.4);
  }
  
  .pos-num {
    font-size: 0.9rem;
    font-weight: 600;
    color: #a1a1aa;
  }
  
  .position-cell.sacred .pos-num {
    color: #60a5fa;
  }
  
  .pos-count {
    font-size: 0.7rem;
    color: #71717a;
    font-weight: 500;
  }
  
  .position-cell.has-data .pos-count {
    color: #60a5fa;
  }
  
  .analytics-panel::-webkit-scrollbar {
    width: 6px;
  }
  
  .analytics-panel::-webkit-scrollbar-track {
    background: transparent;
  }
  
  .analytics-panel::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 3px;
  }
</style>
