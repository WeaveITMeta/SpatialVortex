<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  
  export let text: string = '';
  export let autoSpeak: boolean = false;
  export let rate: number = 1.0; // 0.1 to 10
  export let pitch: number = 1.0; // 0 to 2
  export let volume: number = 1.0; // 0 to 1
  
  let isSpeaking = false;
  let isPaused = false;
  let isSupported = false;
  let voices: SpeechSynthesisVoice[] = [];
  let selectedVoice: SpeechSynthesisVoice | null = null;
  let utterance: SpeechSynthesisUtterance | null = null;
  let progress = 0;
  
  onMount(() => {
    if ('speechSynthesis' in window) {
      isSupported = true;
      loadVoices();
      
      // Voices may load async
      speechSynthesis.onvoiceschanged = loadVoices;
    }
  });
  
  onDestroy(() => {
    stopSpeaking();
  });
  
  function loadVoices() {
    voices = speechSynthesis.getVoices();
    
    // Try to select a good default voice
    if (voices.length > 0 && !selectedVoice) {
      // Prefer English voices
      selectedVoice = voices.find(v => v.lang.startsWith('en')) || voices[0];
    }
  }
  
  export function speak(textToSpeak?: string) {
    const content = textToSpeak || text;
    
    if (!isSupported || !content.trim()) {
      return;
    }
    
    // Stop any ongoing speech
    stopSpeaking();
    
    utterance = new SpeechSynthesisUtterance(content);
    utterance.rate = rate;
    utterance.pitch = pitch;
    utterance.volume = volume;
    
    if (selectedVoice) {
      utterance.voice = selectedVoice;
    }
    
    utterance.onstart = () => {
      isSpeaking = true;
      isPaused = false;
      progress = 0;
    };
    
    utterance.onend = () => {
      isSpeaking = false;
      isPaused = false;
      progress = 100;
    };
    
    utterance.onerror = (event) => {
      console.error('Speech synthesis error:', event);
      isSpeaking = false;
      isPaused = false;
    };
    
    utterance.onboundary = (event) => {
      // Update progress based on character position
      if (content.length > 0) {
        progress = (event.charIndex / content.length) * 100;
      }
    };
    
    speechSynthesis.speak(utterance);
  }
  
  export function stopSpeaking() {
    if (isSupported) {
      speechSynthesis.cancel();
      isSpeaking = false;
      isPaused = false;
      progress = 0;
    }
  }
  
  function pauseSpeaking() {
    if (isSupported && isSpeaking) {
      speechSynthesis.pause();
      isPaused = true;
    }
  }
  
  function resumeSpeaking() {
    if (isSupported && isPaused) {
      speechSynthesis.resume();
      isPaused = false;
    }
  }
  
  function togglePlayPause() {
    if (isSpeaking && !isPaused) {
      pauseSpeaking();
    } else if (isPaused) {
      resumeSpeaking();
    } else {
      speak();
    }
  }
  
  // Auto-speak when text changes
  $: {
    if (autoSpeak && text && isSupported) {
      speak(text);
    }
  }
</script>

<div class="voice-output">
  {#if !isSupported}
    <div class="not-supported">
      <span class="warning-icon">⚠️</span>
      <p>Text-to-speech is not supported in your browser.</p>
    </div>
  {:else}
    <div class="controls">
      <!-- Play/Pause Button -->
      <button 
        class="control-btn play-pause-btn"
        class:speaking={isSpeaking}
        on:click={togglePlayPause}
        disabled={!text.trim()}
        title={isSpeaking ? (isPaused ? 'Resume' : 'Pause') : 'Speak'}
      >
        {#if isSpeaking && !isPaused}
          ⏸️
        {:else if isPaused}
          ▶️
        {:else}
          🔊
        {/if}
      </button>
      
      <!-- Stop Button -->
      {#if isSpeaking || isPaused}
        <button 
          class="control-btn stop-btn"
          on:click={stopSpeaking}
          title="Stop"
        >
          ⏹️
        </button>
      {/if}
      
      <!-- Voice Selector -->
      <select 
        class="voice-select" 
        bind:value={selectedVoice}
        disabled={isSpeaking}
      >
        {#each voices as voice}
          <option value={voice}>
            {voice.name} ({voice.lang})
          </option>
        {/each}
      </select>
    </div>
    
    <!-- Progress Bar -->
    {#if isSpeaking || isPaused}
      <div class="progress-container">
        <div class="progress-bar" style="width: {progress}%"></div>
      </div>
    {/if}
    
    <!-- Settings -->
    <div class="settings">
      <div class="setting">
        <label>Speed: {rate.toFixed(1)}x</label>
        <input 
          type="range" 
          min="0.5" 
          max="2" 
          step="0.1" 
          bind:value={rate}
          disabled={isSpeaking}
        />
      </div>
      
      <div class="setting">
        <label>Pitch: {pitch.toFixed(1)}</label>
        <input 
          type="range" 
          min="0.5" 
          max="2" 
          step="0.1" 
          bind:value={pitch}
          disabled={isSpeaking}
        />
      </div>
      
      <div class="setting">
        <label>Volume: {Math.round(volume * 100)}%</label>
        <input 
          type="range" 
          min="0" 
          max="1" 
          step="0.1" 
          bind:value={volume}
          disabled={isSpeaking}
        />
      </div>
    </div>
  {/if}
</div>

<style>
  .voice-output {
    padding: 1rem;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
  }
  
  .not-supported {
    text-align: center;
    color: #a1a1aa;
  }
  
  .warning-icon {
    font-size: 2rem;
    display: block;
    margin-bottom: 0.5rem;
  }
  
  .controls {
    display: flex;
    gap: 0.75rem;
    align-items: center;
    margin-bottom: 1rem;
  }
  
  .control-btn {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    border: none;
    background: rgba(255, 255, 255, 0.05);
    color: #e4e4e7;
    font-size: 1.5rem;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  
  .control-btn:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.1);
    transform: scale(1.05);
  }
  
  .control-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  
  .play-pause-btn.speaking {
    background: linear-gradient(135deg, #22c55e 0%, #16a34a 100%);
    color: white;
    animation: pulse-glow 2s infinite;
  }
  
  @keyframes pulse-glow {
    0%, 100% {
      box-shadow: 0 0 0 0 rgba(34, 197, 94, 0.4);
    }
    50% {
      box-shadow: 0 0 20px 5px rgba(34, 197, 94, 0.2);
    }
  }
  
  .stop-btn {
    background: rgba(239, 68, 68, 0.2);
  }
  
  .stop-btn:hover {
    background: rgba(239, 68, 68, 0.3);
  }
  
  .voice-select {
    flex: 1;
    padding: 0.625rem 1rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    color: #e4e4e7;
    font-size: 0.9rem;
    cursor: pointer;
  }
  
  .voice-select:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  
  .progress-container {
    width: 100%;
    height: 4px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 2px;
    margin-bottom: 1rem;
    overflow: hidden;
  }
  
  .progress-bar {
    height: 100%;
    background: linear-gradient(90deg, #22c55e 0%, #10b981 100%);
    transition: width 0.3s ease-out;
    border-radius: 2px;
  }
  
  .settings {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  
  .setting {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }
  
  .setting label {
    font-size: 0.85rem;
    color: #a1a1aa;
    font-weight: 500;
  }
  
  .setting input[type="range"] {
    width: 100%;
    height: 6px;
    border-radius: 3px;
    background: rgba(255, 255, 255, 0.1);
    outline: none;
    -webkit-appearance: none;
  }
  
  .setting input[type="range"]::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #60a5fa;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .setting input[type="range"]::-webkit-slider-thumb:hover {
    background: #3b82f6;
    transform: scale(1.2);
  }
  
  .setting input[type="range"]::-moz-range-thumb {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #60a5fa;
    cursor: pointer;
    border: none;
    transition: all 0.2s;
  }
  
  .setting input[type="range"]::-moz-range-thumb:hover {
    background: #3b82f6;
    transform: scale(1.2);
  }
  
  .setting input[type="range"]:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
