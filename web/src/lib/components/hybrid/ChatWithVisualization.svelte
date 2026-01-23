<script lang="ts">
  import { onMount } from 'svelte';
  import ModelSelector from '../ModelSelector.svelte';
  import CompressionDisplay from '../CompressionDisplay.svelte';
  import type { Message, ChatResponse } from '$lib/types';
  import { owAdapter } from '$lib/adapters/openwebui-adapter';
  
  // Component props
  interface Props {
    initialModel?: string;
    showVisualization?: boolean;
    layout?: 'side-by-side' | 'overlay' | 'embedded';
  }
  
  let { 
    initialModel = 'llama2',
    showVisualization = true,
    layout = 'side-by-side'
  }: Props = $props();
  
  // State
  let selectedModel = $state<string>(initialModel);
  let messages = $state<Message[]>([]);
  let currentPrompt = $state<string>('');
  let isLoading = $state<boolean>(false);
  let latestResponse = $state<ChatResponse | null>(null);
  let backendHealthy = $state<boolean>(false);
  let showSettings = $state<boolean>(false);
  
  // Check backend health on mount
  onMount(async (): Promise<void> => {
    backendHealthy = await owAdapter.checkHealth();
    if (!backendHealthy) {
      console.warn('Backend not responding on port 28080');
    }
  });
  
  async function sendMessage(): Promise<void> {
    if (!currentPrompt.trim() || isLoading) return;
    
    const userMessage: Message = {
      id: crypto.randomUUID(),
      role: 'user',
      content: currentPrompt,
      timestamp: Date.now(),
    };
    
    messages = [...messages, userMessage];
    const prompt = currentPrompt;
    currentPrompt = '';
    isLoading = true;
    
    try {
      const response = await owAdapter.sendMessage(prompt, selectedModel, {
        compress: true,
      });
      
      latestResponse = response;
      
      const aiMessage: Message = {
        id: crypto.randomUUID(),
        role: 'assistant',
        content: response.response,
        timestamp: Date.now(),
        ...(response.compressed_hash && { compressed_hash: response.compressed_hash }),
        ...(response.beam_position !== undefined && { beam_position: response.beam_position }),
        ...(response.elp_channels && { elp_channels: response.elp_channels }),
      };
      
      messages = [...messages, aiMessage];
    } catch (error) {
      const errorMsg = owAdapter.handleError(error);
      messages = [...messages, owAdapter.messageToSpatialVortex(errorMsg)];
    } finally {
      isLoading = false;
    }
  }
  
  function handleKeyDown(event: KeyboardEvent): void {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      void sendMessage();
    }
  }
  
  function handleModelChange(modelId: string): void {
    selectedModel = modelId;
  }
  
  function toggleVisualization(): void {
    showVisualization = !showVisualization;
  }
</script>

<div class="hybrid-container layout-{layout}">
  <!-- Header with controls -->
  <header class="hybrid-header">
    <div class="header-left">
      <h1>SpatialVortex AGI Chat</h1>
      {#if !backendHealthy}
        <span class="md-chip status-badge offline">Backend Offline</span>
      {:else}
        <span class="md-chip status-badge online">Connected</span>
      {/if}
    </div>
    
    <div class="header-controls">
      <ModelSelector 
        {selectedModel}
        onModelChange={handleModelChange}
      />
      
      <button 
        class="md-button md-button-outlined"
        onclick={toggleVisualization}
        title={showVisualization ? 'Hide 3D' : 'Show 3D'}
      >
        {showVisualization ? 'Hide 3D' : 'Show 3D'}
      </button>
      
      <button 
        class="md-button md-button-outlined"
        onclick={() => showSettings = !showSettings}
      >
        Settings
      </button>
    </div>
  </header>
  
  <!-- Main content area -->
  <div class="hybrid-content">
    <!-- 3D Visualization Panel -->
    {#if showVisualization}
      <aside class="visualization-panel md-card">
        <div class="panel-header">
          <h3>Geometric Consciousness</h3>
        </div>
        
        <div class="canvas-container">
          <canvas id="beam-canvas"></canvas>
          <div class="canvas-overlay">
            <p class="loading-3d">WASM Loading...</p>
          </div>
        </div>
        
        {#if latestResponse}
          <div class="viz-info">
            <CompressionDisplay 
              hash={latestResponse.compressed_hash || ''}
              showDetails={true}
            />
          </div>
        {/if}
      </aside>
    {/if}
    
    <!-- Chat Panel -->
    <main class="chat-panel">
      <div class="messages-container">
        {#each messages as message (message.id)}
          <div class="message {message.role}-message">
            <div class="message-header">
              <span class="message-role">{message.role}</span>
              <span class="message-time">{new Date(message.timestamp).toLocaleTimeString()}</span>
            </div>
            <div class="message-content">{message.content}</div>
            {#if message.compressed_hash}
              <div class="message-meta">
                <span class="hash-badge" title="12-byte compression">
                  {message.compressed_hash.slice(0, 8)}...
                </span>
                {#if message.beam_position !== undefined}
                  <span class="position-badge">Position {message.beam_position}</span>
                {/if}
                {#if message.elp_channels}
                  <span class="elp-badge" style:color="rgb({message.elp_channels.pathos * 28}, {message.elp_channels.logos * 28}, {message.elp_channels.ethos * 28})">
                    E:{message.elp_channels.ethos.toFixed(1)} 
                    L:{message.elp_channels.logos.toFixed(1)} 
                    P:{message.elp_channels.pathos.toFixed(1)}
                  </span>
                {/if}
              </div>
            {/if}
          </div>
        {/each}
        
        {#if isLoading}
          <div class="message assistant-message loading">
            <div class="message-content">
              <span class="loading-dots">Thinking</span>
            </div>
          </div>
        {/if}
        
        {#if messages.length === 0}
          <div class="empty-state md-card">
            <h2 class="text-primary">Welcome to Geometric Consciousness</h2>
            <p>Your messages will be compressed to 12 bytes and visualized as colored light beams flowing through sacred geometry.</p>
          </div>
        {/if}
      </div>
      
      <div class="input-container">
        <textarea
          bind:value={currentPrompt}
          onkeydown={handleKeyDown}
          placeholder="Ask anything... (Enter to send, Shift+Enter for new line)"
          disabled={isLoading || !backendHealthy}
          rows="3"
        />
        <button
          onclick={() => void sendMessage()}
          disabled={isLoading || !currentPrompt.trim() || !backendHealthy}
          class="md-button md-button-primary"
        >
          {isLoading ? 'Sending...' : 'Send'}
        </button>
      </div>
    </main>
  </div>
  
  <!-- Settings panel -->
  {#if showSettings}
    <div class="settings-overlay" onclick={() => showSettings = false}>
      <div class="settings-panel" onclick={(e) => e.stopPropagation()}>
        <h3>Settings</h3>
        <div class="setting-group">
          <label>
            <input type="checkbox" bind:checked={showVisualization} />
            Show 3D Visualization
          </label>
        </div>
        <div class="setting-group">
          <label>
            Layout:
            <select bind:value={layout}>
              <option value="side-by-side">Side by Side</option>
              <option value="overlay">Overlay</option>
              <option value="embedded">Embedded</option>
            </select>
          </label>
        </div>
        <button class="md-button md-button-outlined" onclick={() => showSettings = false}>Close</button>
      </div>
    </div>
  {/if}
</div>

<style>
  .hybrid-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background-color: var(--md-background);
    color: var(--md-on-background);
  }
  
  .hybrid-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--md-spacing-md) var(--md-spacing-lg);
    background-color: var(--md-surface);
    box-shadow: var(--md-elevation-2);
  }
  
  .header-left {
    display: flex;
    align-items: center;
    gap: var(--md-spacing-md);
  }
  
  .header-left h1 {
    margin: 0;
    font-size: 1.5rem;
    color: var(--md-on-surface);
  }
  
  .status-badge { margin-left: var(--md-spacing-sm); }
  .status-badge.online { background-color: rgba(0, 200, 83, 0.15); color: var(--md-success); }
  .status-badge.offline { background-color: rgba(255, 152, 0, 0.15); color: var(--md-warning); }
  
  .header-controls {
    display: flex;
    gap: var(--md-spacing-md);
    align-items: center;
  }
  
  .hybrid-content {
    flex: 1;
    display: grid;
    overflow: hidden;
  }
  
  .layout-side-by-side .hybrid-content {
    grid-template-columns: 400px 1fr;
  }
  
  .layout-overlay .hybrid-content {
    grid-template-columns: 1fr;
  }
  
  .layout-embedded .hybrid-content {
    grid-template-columns: 1fr;
  }
  
  .visualization-panel {
    background-color: var(--md-surface);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  
  .panel-header {
    padding: var(--md-spacing-md);
    border-bottom: 1px solid rgba(255, 255, 255, 0.12);
  }
  
  .panel-header h3 {
    margin: 0;
    color: var(--md-on-surface);
  }
  
  .canvas-container {
    flex: 1;
    position: relative;
  }
  
  #beam-canvas {
    width: 100%;
    height: 100%;
  }
  
  .canvas-overlay {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.4);
    pointer-events: none;
  }
  
  .loading-3d {
    color: var(--md-on-surface);
    font-size: 1rem;
  }
  
  .viz-info {
    padding: var(--md-spacing-md);
    border-top: 1px solid rgba(255, 255, 255, 0.12);
    max-height: 300px;
    overflow-y: auto;
  }
  
  .chat-panel {
    display: flex;
    flex-direction: column;
    background-color: var(--md-surface);
    border-radius: var(--md-radius-md);
    box-shadow: var(--md-elevation-3);
  }
  
  .messages-container {
    flex: 1;
    overflow-y: auto;
    padding: var(--md-spacing-lg);
    display: flex;
    flex-direction: column;
    gap: var(--md-spacing-md);
  }
  
  .message {
    padding: var(--md-spacing-md);
    border-radius: var(--md-radius-md);
    max-width: 80%;
    animation: slideIn 0.3s ease-out;
  }
  
  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
  
  .user-message { background-color: var(--md-primary); color: var(--md-on-primary); align-self: flex-end; box-shadow: var(--md-elevation-2); }
  .assistant-message { background-color: var(--md-surface-variant); color: var(--md-on-surface); align-self: flex-start; border-left: 3px solid var(--md-secondary); }
  
  .system-message {
    background: rgba(255, 152, 0, 0.1);
    border: 1px solid var(--md-warning);
    align-self: center;
  }
  
  .message-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 0.5rem;
    font-size: 0.9rem;
    opacity: 0.7;
  }
  
  .message-role { font-weight: 600; text-transform: capitalize; }
  
  .message-content { line-height: 1.6; }
  
  .message-meta {
    display: flex;
    gap: var(--md-spacing-sm);
    margin-top: 0.75rem;
    flex-wrap: wrap;
    font-size: 0.75rem;
  }
  
  .hash-badge, .position-badge, .elp-badge {
    padding: var(--md-spacing-xs) var(--md-spacing-sm);
    background: rgba(255, 255, 255, 0.08);
    border-radius: var(--md-radius-sm);
    font-family: var(--md-font-family-mono);
  }
  
  .loading { opacity: 0.7; }
  
  .loading-dots::after {
    content: '...';
    animation: dots 1.5s steps(4, end) infinite;
  }
  
  @keyframes dots {
    0%, 20% { content: '.'; }
    40% { content: '..'; }
    60%, 100% { content: '...'; }
  }
  
  .empty-state { text-align: center; padding: var(--md-spacing-2xl) var(--md-spacing-xl); }
  
  .input-container {
    display: flex;
    gap: var(--md-spacing-md);
    padding: var(--md-spacing-lg);
    background-color: var(--md-surface-variant);
    border-top: 1px solid rgba(255, 255, 255, 0.12);
  }
  
  textarea {
    flex: 1;
    padding: var(--md-spacing-md);
    background-color: var(--md-surface);
    color: var(--md-on-surface);
    border: 1px solid transparent;
    border-radius: var(--md-radius-sm);
    font-family: var(--md-font-family);
    font-size: 1rem;
    resize: none;
    transition: all var(--md-transition-fast);
  }
  
  textarea:focus {
    outline: none;
    border-color: var(--md-primary);
    box-shadow: 0 0 0 3px rgba(98, 0, 238, 0.1);
  }
  
  textarea:disabled {
    opacity: 0.38;
    cursor: not-allowed;
  }
  
  .settings-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.8);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }
  
  .settings-panel {
    background-color: var(--md-surface);
    padding: var(--md-spacing-xl);
    border-radius: var(--md-radius-md);
    box-shadow: var(--md-elevation-4);
    min-width: 400px;
  }
  
  .settings-panel h3 {
    margin-top: 0;
    color: var(--md-on-surface);
  }
  
  .setting-group {
    margin: var(--md-spacing-lg) 0;
  }
  
  .settings-panel label {
    display: flex;
    align-items: center;
    gap: var(--md-spacing-sm);
  }
  
</style>
