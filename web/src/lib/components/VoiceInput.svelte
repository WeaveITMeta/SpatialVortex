<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';
  
  const dispatch = createEventDispatcher();
  
  let isListening = false;
  let transcript = '';
  let interimTranscript = '';
  let recognition: any = null;
  let isSupported = false;
  let error = '';
  let confidence = 0;
  let audioLevel = 0;
  let animationFrame: number;
  
  // Check for browser support
  onMount(() => {
    const SpeechRecognition = (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;
    
    if (SpeechRecognition) {
      isSupported = true;
      
      recognition = new SpeechRecognition();
      recognition.continuous = true;
      recognition.interimResults = true;
      recognition.lang = 'en-US';
      
      recognition.onstart = () => {
        isListening = true;
        error = '';
        console.log('Voice recognition started');
      };
      
      recognition.onresult = (event: any) => {
        let interim = '';
        let final = '';
        
        for (let i = event.resultIndex; i < event.results.length; i++) {
          const transcript = event.results[i][0].transcript;
          
          if (event.results[i].isFinal) {
            final += transcript + ' ';
            confidence = event.results[i][0].confidence;
          } else {
            interim += transcript;
          }
        }
        
        if (final) {
          transcript += final;
          dispatch('transcript', { text: final.trim(), confidence });
        }
        
        interimTranscript = interim;
      };
      
      recognition.onerror = (event: any) => {
        console.error('Speech recognition error:', event.error);
        
        switch (event.error) {
          case 'no-speech':
            error = 'No speech detected. Please try again.';
            break;
          case 'audio-capture':
            error = 'Microphone not available.';
            break;
          case 'not-allowed':
            error = 'Microphone permission denied.';
            break;
          default:
            error = `Error: ${event.error}`;
        }
        
        isListening = false;
      };
      
      recognition.onend = () => {
        isListening = false;
        console.log('Voice recognition ended');
      };
    }
  });
  
  onDestroy(() => {
    if (recognition && isListening) {
      recognition.stop();
    }
    if (animationFrame) {
      cancelAnimationFrame(animationFrame);
    }
  });
  
  export function startListening() {
    if (!isSupported) {
      error = 'Speech recognition not supported in this browser.';
      return;
    }
    
    if (!isListening && recognition) {
      transcript = '';
      interimTranscript = '';
      error = '';
      
      try {
        recognition.start();
        simulateAudioLevel(); // Start audio visualization
      } catch (err) {
        console.error('Failed to start recognition:', err);
        error = 'Failed to start microphone.';
      }
    }
  }
  
  export function stopListening() {
    if (isListening && recognition) {
      recognition.stop();
      if (animationFrame) {
        cancelAnimationFrame(animationFrame);
      }
      audioLevel = 0;
    }
  }
  
  function toggleListening() {
    if (isListening) {
      stopListening();
    } else {
      startListening();
    }
  }
  
  function clearTranscript() {
    transcript = '';
    interimTranscript = '';
  }
  
  function sendTranscript() {
    if (transcript.trim()) {
      dispatch('submit', { text: transcript.trim() });
      clearTranscript();
      stopListening();
    }
  }
  
  // Simulate audio level for visualization
  // In production, you'd use Web Audio API for real audio analysis
  function simulateAudioLevel() {
    if (!isListening) return;
    
    // Simulate audio level changes
    audioLevel = Math.random() * 0.5 + (isListening ? 0.3 : 0);
    
    animationFrame = requestAnimationFrame(simulateAudioLevel);
  }
</script>

<div class="voice-input">
  {#if !isSupported}
    <div class="not-supported">
      <span class="warning-icon">⚠️</span>
      <p>Speech recognition is not supported in your browser.</p>
      <p class="hint">Try Chrome, Edge, or Safari.</p>
    </div>
  {:else}
    <!-- Voice Button -->
    <button 
      class="voice-btn" 
      class:listening={isListening}
      on:click={toggleListening}
      title={isListening ? 'Stop listening' : 'Start voice input'}
    >
      <div class="mic-icon">
        {#if isListening}
          <div class="pulse-ring"></div>
        {/if}
        🎤
      </div>
    </button>
    
    <!-- Audio Level Indicator -->
    {#if isListening}
      <div class="audio-level-container">
        <div class="audio-level" style="width: {audioLevel * 100}%"></div>
      </div>
    {/if}
    
    <!-- Transcript Display -->
    {#if transcript || interimTranscript || error}
      <div class="transcript-container">
        {#if error}
          <div class="error-message">
            <span class="error-icon">❌</span>
            {error}
          </div>
        {:else}
          <div class="transcript-text">
            {#if transcript}
              <span class="final-transcript">{transcript}</span>
            {/if}
            {#if interimTranscript}
              <span class="interim-transcript">{interimTranscript}</span>
            {/if}
          </div>
          
          <div class="transcript-actions">
            <button class="action-btn clear-btn" on:click={clearTranscript}>
              Clear
            </button>
            <button 
              class="action-btn send-btn" 
              on:click={sendTranscript}
              disabled={!transcript.trim()}
            >
              Send
            </button>
          </div>
          
          {#if confidence > 0}
            <div class="confidence-indicator">
              Confidence: {Math.round(confidence * 100)}%
            </div>
          {/if}
        {/if}
      </div>
    {/if}
  {/if}
</div>

<style>
  .voice-input {
    position: relative;
  }
  
  .not-supported {
    padding: 1rem;
    background: rgba(255, 193, 7, 0.1);
    border: 1px solid rgba(255, 193, 7, 0.3);
    border-radius: 8px;
    text-align: center;
    color: #ffc107;
  }
  
  .warning-icon {
    font-size: 2rem;
    display: block;
    margin-bottom: 0.5rem;
  }
  
  .hint {
    font-size: 0.85rem;
    color: #d4d4d4;
    margin-top: 0.5rem;
  }
  
  .voice-btn {
    width: 60px;
    height: 60px;
    border-radius: 50%;
    border: none;
    background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
    color: white;
    font-size: 2rem;
    cursor: pointer;
    transition: all 0.3s;
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 4px 12px rgba(239, 68, 68, 0.3);
  }
  
  .voice-btn:hover {
    transform: scale(1.05);
    box-shadow: 0 6px 16px rgba(239, 68, 68, 0.4);
  }
  
  .voice-btn:active {
    transform: scale(0.95);
  }
  
  .voice-btn.listening {
    background: linear-gradient(135deg, #22c55e 0%, #16a34a 100%);
    box-shadow: 0 4px 12px rgba(34, 197, 94, 0.3);
    animation: glow 2s infinite;
  }
  
  @keyframes glow {
    0%, 100% {
      box-shadow: 0 4px 12px rgba(34, 197, 94, 0.3);
    }
    50% {
      box-shadow: 0 4px 20px rgba(34, 197, 94, 0.6);
    }
  }
  
  .mic-icon {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  
  .pulse-ring {
    position: absolute;
    width: 100%;
    height: 100%;
    border: 3px solid rgba(34, 197, 94, 0.6);
    border-radius: 50%;
    animation: pulse 2s infinite;
  }
  
  @keyframes pulse {
    0% {
      transform: scale(1);
      opacity: 1;
    }
    100% {
      transform: scale(1.5);
      opacity: 0;
    }
  }
  
  .audio-level-container {
    width: 100%;
    height: 4px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 2px;
    margin-top: 0.75rem;
    overflow: hidden;
  }
  
  .audio-level {
    height: 100%;
    background: linear-gradient(90deg, #22c55e 0%, #10b981 100%);
    transition: width 0.1s ease-out;
    border-radius: 2px;
  }
  
  .transcript-container {
    margin-top: 1rem;
    padding: 1rem;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
  }
  
  .error-message {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: #ef4444;
    font-size: 0.9rem;
  }
  
  .error-icon {
    font-size: 1.25rem;
  }
  
  .transcript-text {
    color: #e4e4e7;
    font-size: 1rem;
    line-height: 1.5;
    margin-bottom: 1rem;
    min-height: 3rem;
  }
  
  .final-transcript {
    color: #e4e4e7;
  }
  
  .interim-transcript {
    color: #a1a1aa;
    font-style: italic;
  }
  
  .transcript-actions {
    display: flex;
    gap: 0.5rem;
    justify-content: flex-end;
  }
  
  .action-btn {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 6px;
    font-size: 0.9rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .clear-btn {
    background: rgba(255, 255, 255, 0.05);
    color: #d4d4d4;
    border: 1px solid rgba(255, 255, 255, 0.1);
  }
  
  .clear-btn:hover {
    background: rgba(255, 255, 255, 0.1);
  }
  
  .send-btn {
    background: linear-gradient(135deg, #60a5fa 0%, #3b82f6 100%);
    color: white;
  }
  
  .send-btn:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(96, 165, 250, 0.3);
  }
  
  .send-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  
  .confidence-indicator {
    margin-top: 0.75rem;
    font-size: 0.75rem;
    color: #a1a1aa;
    text-align: right;
  }
</style>
