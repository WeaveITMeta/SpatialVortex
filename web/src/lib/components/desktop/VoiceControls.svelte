<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import VoiceInput from '../VoiceInput.svelte';
  import VoiceOutput from '../VoiceOutput.svelte';
  
  const dispatch = createEventDispatcher();
  
  export let lastAIResponse: string = '';
  export let autoSpeakResponses: boolean = false;
  
  let voiceInput: VoiceInput;
  let voiceOutput: VoiceOutput;
  let activeTab: 'input' | 'output' = 'input';
  
  function handleTranscript(event: CustomEvent) {
    const { text, confidence } = event.detail;
    console.log(`Transcript: ${text} (confidence: ${confidence})`);
  }
  
  function handleSubmit(event: CustomEvent) {
    const { text } = event.detail;
    dispatch('voiceMessage', { content: text });
  }
  
  function speakResponse() {
    if (voiceOutput && lastAIResponse) {
      voiceOutput.speak(lastAIResponse);
    }
  }
  
  // Auto-speak AI responses when enabled
  $: {
    if (autoSpeakResponses && lastAIResponse && voiceOutput) {
      voiceOutput.speak(lastAIResponse);
    }
  }
</script>

<div class="voice-controls">
  <div class="voice-header">
    <h3 class="voice-title">🎤 Voice Controls</h3>
    
    <label class="auto-speak-toggle">
      <input type="checkbox" bind:checked={autoSpeakResponses} />
      <span>Auto-speak responses</span>
    </label>
  </div>
  
  <!-- Tab Navigation -->
  <div class="tab-nav">
    <button 
      class="tab-btn" 
      class:active={activeTab === 'input'}
      on:click={() => activeTab = 'input'}
    >
      🎤 Voice Input
    </button>
    <button 
      class="tab-btn" 
      class:active={activeTab === 'output'}
      on:click={() => activeTab = 'output'}
    >
      🔊 Voice Output
    </button>
  </div>
  
  <!-- Tab Content -->
  <div class="tab-content">
    {#if activeTab === 'input'}
      <div class="tab-pane">
        <p class="tab-description">
          Click the microphone to start voice input. Speak your message and click "Send" when done.
        </p>
        <VoiceInput 
          bind:this={voiceInput}
          on:transcript={handleTranscript}
          on:submit={handleSubmit}
        />
      </div>
    {:else}
      <div class="tab-pane">
        <p class="tab-description">
          Listen to AI responses with text-to-speech. Customize voice, speed, and pitch.
        </p>
        <VoiceOutput 
          bind:this={voiceOutput}
          text={lastAIResponse}
          autoSpeak={false}
        />
        
        {#if !lastAIResponse}
          <div class="no-response-message">
            <span class="info-icon">ℹ️</span>
            <p>No AI response to play yet. Ask a question first!</p>
          </div>
        {/if}
      </div>
    {/if}
  </div>
  
  <!-- Quick Actions -->
  <div class="quick-actions">
    <button 
      class="quick-action-btn"
      on:click={() => voiceInput?.startListening()}
      disabled={activeTab !== 'input'}
    >
      🎤 Quick Voice Input
    </button>
    <button 
      class="quick-action-btn"
      on:click={speakResponse}
      disabled={!lastAIResponse}
    >
      🔊 Speak Last Response
    </button>
  </div>
</div>

<style>
  .voice-controls {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding: 1.5rem;
    background: linear-gradient(180deg, #1a1a2e 0%, #16161f 100%);
    border-radius: 12px;
    border: 1px solid rgba(255, 255, 255, 0.1);
  }
  
  .voice-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  
  .voice-title {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
    color: #e4e4e7;
  }
  
  .auto-speak-toggle {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.9rem;
    color: #d4d4d4;
    cursor: pointer;
  }
  
  .auto-speak-toggle input[type="checkbox"] {
    width: 18px;
    height: 18px;
    cursor: pointer;
  }
  
  .tab-nav {
    display: flex;
    gap: 0.5rem;
    border-bottom: 2px solid rgba(255, 255, 255, 0.1);
    padding-bottom: 0.5rem;
  }
  
  .tab-btn {
    flex: 1;
    padding: 0.75rem 1rem;
    background: transparent;
    border: none;
    border-radius: 6px;
    color: #a1a1aa;
    font-size: 0.95rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .tab-btn:hover {
    background: rgba(255, 255, 255, 0.05);
    color: #d4d4d4;
  }
  
  .tab-btn.active {
    background: rgba(96, 165, 250, 0.1);
    color: #60a5fa;
    border-bottom: 2px solid #60a5fa;
  }
  
  .tab-content {
    min-height: 300px;
  }
  
  .tab-pane {
    animation: fadeIn 0.3s ease-out;
  }
  
  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
  
  .tab-description {
    margin: 0 0 1rem 0;
    color: #a1a1aa;
    font-size: 0.9rem;
    line-height: 1.5;
  }
  
  .no-response-message {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1rem;
    background: rgba(96, 165, 250, 0.1);
    border: 1px solid rgba(96, 165, 250, 0.3);
    border-radius: 8px;
    margin-top: 1rem;
    color: #60a5fa;
  }
  
  .info-icon {
    font-size: 1.5rem;
  }
  
  .quick-actions {
    display: flex;
    gap: 0.75rem;
    padding-top: 1rem;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
  }
  
  .quick-action-btn {
    flex: 1;
    padding: 0.75rem 1rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    color: #d4d4d4;
    font-size: 0.9rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .quick-action-btn:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.1);
    border-color: #60a5fa;
    color: #60a5fa;
  }
  
  .quick-action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
