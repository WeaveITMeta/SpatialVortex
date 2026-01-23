<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api/client';
  import type { 
    ChatResponse, 
    ELPChannels, 
    Message 
  } from '$lib/types';
  import type { BeamRenderParams } from '$lib/types';
  
  // Component props
  interface Props {
    initialMessages?: Message[];
    autoCompress?: boolean;
    modelId?: string;
  }
  
  let { 
    initialMessages = [], 
    autoCompress = true,
    modelId = 'llama2'
  }: Props = $props();
  
  // Component state
  let prompt = $state<string>('');
  let compressedHash = $state<string>('');
  let beamPosition = $state<number>(0);
  let elpChannels = $state<ELPChannels>({ ethos: 0, logos: 0, pathos: 0 });
  let messages = $state<Message[]>(initialMessages);
  let isLoading = $state<boolean>(false);
  let thinkingTime = $state<number>(0);
  
  // WASM module reference
  let wasmLoaded = $state<boolean>(false);
  
  // Initialize 3D WASM visualization
  onMount(async (): Promise<void> => {
    try {
      const script = document.createElement('script');
      script.type = 'module';
      script.textContent = `
        import init, { render_beam } from '/bevy/vortex_view.js';
        await init({ module_or_path: '/bevy/vortex_view_bg.wasm' });
        
        // Expose render function globally
        window.renderBeam = render_beam;
      `;
      document.body.appendChild(script);
      
      // Wait for WASM to load
      await new Promise<void>(resolve => {
        const checkLoaded = setInterval(() => {
          if (window.renderBeam) {
            clearInterval(checkLoaded);
            wasmLoaded = true;
            resolve();
          }
        }, 100);
      });
    } catch (error) {
      console.error('Failed to load WASM module:', error);
    }
  });
  
  async function sendMessage(): Promise<void> {
    if (!prompt.trim() || isLoading) return;
    
    const userMessage: Message = {
      id: crypto.randomUUID(),
      role: 'user',
      content: prompt,
      timestamp: Date.now(),
    };
    
    messages = [...messages, userMessage];
    const currentPrompt = prompt;
    prompt = '';
    isLoading = true;
    
    try {
      const startTime = performance.now();
      
      const data: ChatResponse = await api.chat({
        prompt: currentPrompt,
        compress: autoCompress,
        model: modelId,
      });
      
      thinkingTime = (performance.now() - startTime) / 1000;
      
      // Update state
      compressedHash = data.compressed_hash ?? '';
      beamPosition = data.beam_position ?? 0;
      elpChannels = data.elp_channels ?? { ethos: 0, logos: 0, pathos: 0 };
      
      // Add AI message
      const aiMessage: Message = {
        id: crypto.randomUUID(),
        role: 'assistant',
        content: data.response,
        timestamp: Date.now(),
        ...(data.compressed_hash && { compressed_hash: data.compressed_hash }),
        ...(data.beam_position !== undefined && { beam_position: data.beam_position }),
        ...(data.elp_channels && { elp_channels: data.elp_channels }),
      };
      
      messages = [...messages, aiMessage];
      
      // Render beam in 3D if WASM is loaded
      if (wasmLoaded && window.renderBeam && data.confidence !== undefined) {
        const params: BeamRenderParams = {
          position: beamPosition,
          ethos: elpChannels.ethos,
          logos: elpChannels.logos,
          pathos: elpChannels.pathos,
          word: currentPrompt,
          confidence: data.confidence,
        };
        window.renderBeam(params);
      }
    } catch (error) {
      console.error('Chat error:', error);
      const errorMessage: Message = {
        id: crypto.randomUUID(),
        role: 'system',
        content: `Error: ${error instanceof Error ? error.message : 'Failed to get response'}`,
        timestamp: Date.now(),
      };
      messages = [...messages, errorMessage];
    } finally {
      isLoading = false;
    }
  }
  
  function handleKeyDown(event: KeyboardEvent): void {
    if (event.key === 'Enter' && event.ctrlKey) {
      event.preventDefault();
      void sendMessage();
    }
  }
  
  // Calculate RGB color from ELP channels
  let beamColor = $derived({
    r: Math.round(elpChannels.pathos * 255 / 9),
    g: Math.round(elpChannels.logos * 255 / 9),
    b: Math.round(elpChannels.ethos * 255 / 9),
  });
</script>

<div class="chat-container">
  <div class="canvas-3d">
    <canvas id="beam-canvas"></canvas>
    <div class="beam-info">
      <div class="info-row">
        <span class="label">Position:</span>
        <span class="value">{beamPosition}</span>
      </div>
      <div 
        class="info-row elp-display"
        style:color="rgb({beamColor.r}, {beamColor.g}, {beamColor.b})"
      >
        <span class="label">ELP:</span>
        <span class="value">
          E:{elpChannels.ethos.toFixed(1)} 
          L:{elpChannels.logos.toFixed(1)} 
          P:{elpChannels.pathos.toFixed(1)}
        </span>
      </div>
      {#if compressedHash}
        <div class="info-row hash-display">
          <span class="label">Hash:</span>
          <span class="value hash-mono">{compressedHash.slice(0, 16)}...</span>
        </div>
      {/if}
      {#if thinkingTime > 0}
        <div class="info-row">
          <span class="label">Time:</span>
          <span class="value">{thinkingTime.toFixed(2)}s</span>
        </div>
      {/if}
      {#if !wasmLoaded}
        <div class="info-row warning">
          <span>Loading 3D visualization...</span>
        </div>
      {/if}
    </div>
  </div>
  
  <div class="chat-interface">
    <div class="messages">
      {#each messages as message (message.id)}
        <div class="message {message.role}-message">
          <div class="message-content">{message.content}</div>
          {#if message.compressed_hash}
            <div class="message-meta">
              <span class="hash-badge">{message.compressed_hash.slice(0, 8)}</span>
              {#if message.beam_position !== undefined}
                <span class="position-badge">Pos {message.beam_position}</span>
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
    </div>
    
    <div class="input-container">
      <textarea 
        bind:value={prompt}
        onkeydown={handleKeyDown}
        placeholder="Type your message... (Ctrl+Enter to send)"
        disabled={isLoading}
        rows="3"
      ></textarea>
      <button 
        onclick={() => void sendMessage()}
        disabled={isLoading || !prompt.trim()}
      >
        {isLoading ? 'Sending...' : 'Send'}
      </button>
    </div>
  </div>
</div>

<style>
  .chat-container {
    display: grid;
    grid-template-columns: 1fr 2fr;
    height: 100vh;
    gap: var(--md-spacing-md);
    padding: var(--md-spacing-md);
    background-color: var(--md-background);
    color: var(--md-on-background);
  }
  
  .canvas-3d {
    position: relative;
    background-color: var(--md-surface);
    border-radius: var(--md-radius-lg);
    overflow: hidden;
    box-shadow: var(--md-elevation-3);
  }
  
  #beam-canvas {
    width: 100%;
    height: 70vh;
    display: block;
  }
  
  .beam-info {
    padding: var(--md-spacing-lg);
    background-color: var(--md-surface-variant);
    backdrop-filter: blur(10px);
    border-top: 1px solid rgba(255, 255, 255, 0.12);
  }
  
  .info-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--md-spacing-sm) 0;
    font-family: var(--md-font-family-mono);
    font-size: 0.875rem;
  }
  
  .label {
    color: var(--md-on-surface-variant);
    font-weight: 500;
  }
  
  .value {
    color: var(--md-on-surface);
    font-weight: 500;
  }
  
  .elp-display {
    font-weight: 500;
    padding: var(--md-spacing-md);
    background-color: rgba(255, 255, 255, 0.05);
    border-radius: var(--md-radius-sm);
    margin: var(--md-spacing-sm) 0;
  }
  
  .hash-display {
    border-top: 1px solid rgba(255, 255, 255, 0.12);
    padding-top: var(--md-spacing-md);
    margin-top: var(--md-spacing-sm);
  }
  
  .hash-mono {
    font-family: var(--md-font-family-mono);
    color: var(--md-secondary);
    font-size: 0.8rem;
  }
  
  .warning {
    color: var(--md-warning);
    justify-content: center;
    font-size: 0.875rem;
  }
  
  .chat-interface {
    display: flex;
    flex-direction: column;
    background-color: var(--md-surface);
    border-radius: var(--md-radius-lg);
    overflow: hidden;
    box-shadow: var(--md-elevation-3);
  }
  
  .messages {
    flex: 1;
    overflow-y: auto;
    padding: var(--md-spacing-lg);
    display: flex;
    flex-direction: column;
    gap: var(--md-spacing-md);
  }
  
  .message {
    padding: var(--md-spacing-md) var(--md-spacing-lg);
    border-radius: var(--md-radius-md);
    animation: slideInUp var(--md-transition-base);
  }
  
  .user-message {
    background-color: var(--md-primary);
    color: var(--md-on-primary);
    align-self: flex-end;
    max-width: 80%;
    box-shadow: var(--md-elevation-2);
  }
  
  .assistant-message {
    background-color: var(--md-surface-variant);
    color: var(--md-on-surface);
    align-self: flex-start;
    max-width: 80%;
    border-left: 3px solid var(--md-secondary);
  }
  
  .system-message {
    background-color: rgba(255, 152, 0, 0.1);
    color: var(--md-warning);
    align-self: center;
    border: 1px solid var(--md-warning);
  }
  
  .loading {
    opacity: 0.7;
  }
  
  .loading-dots::after {
    content: '...';
    animation: dots 1.5s steps(4, end) infinite;
  }
  
  @keyframes dots {
    0%, 20% { content: '.'; }
    40% { content: '..'; }
    60%, 100% { content: '...'; }
  }
  
  .message-content {
    line-height: 1.6;
  }
  
  .message-meta {
    display: flex;
    gap: var(--md-spacing-sm);
    margin-top: var(--md-spacing-sm);
    font-size: 0.75rem;
    opacity: 0.7;
  }
  
  .hash-badge, .position-badge {
    padding: var(--md-spacing-xs) var(--md-spacing-sm);
    background-color: rgba(0, 0, 0, 0.3);
    border-radius: var(--md-radius-sm);
    font-family: var(--md-font-family-mono);
  }
  
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
  
  textarea::placeholder {
    color: var(--md-on-surface-variant);
    opacity: 0.6;
  }
  
  textarea:disabled {
    opacity: 0.38;
    cursor: not-allowed;
  }
  
  button {
    padding: 0 var(--md-spacing-xl);
    height: 56px;
    background-color: var(--md-primary);
    color: var(--md-on-primary);
    border: none;
    border-radius: var(--md-radius-sm);
    font-family: var(--md-font-family);
    font-size: 0.875rem;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.0892857143em;
    cursor: pointer;
    transition: all var(--md-transition-fast);
    white-space: nowrap;
    box-shadow: var(--md-elevation-2);
  }
  
  button:hover:not(:disabled) {
    background-color: var(--md-primary-variant);
    box-shadow: var(--md-elevation-4);
  }
  
  button:active:not(:disabled) {
    box-shadow: var(--md-elevation-1);
  }
  
  button:disabled {
    opacity: 0.38;
    cursor: not-allowed;
  }
</style>
