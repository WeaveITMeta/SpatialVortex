<script lang="ts">
  import { onMount } from 'svelte';
  import ELPVisualization from './ELPVisualization.svelte';
  import type { ChatMessage } from '../types/chat';
  
  let messages: ChatMessage[] = [];
  let inputText = '';
  let isLoading = false;
  let error: string | null = null;
  let chatContainer: HTMLDivElement;
  
  const API_BASE = 'http://localhost:7000';
  
  async function sendMessage() {
    if (!inputText.trim() || isLoading) return;
    
    const userMessage: ChatMessage = {
      id: Date.now().toString(),
      role: 'user',
      content: inputText,
      timestamp: new Date(),
    };
    
    messages = [...messages, userMessage];
    const currentInput = inputText;
    inputText = '';
    isLoading = true;
    error = null;
    scrollToBottom();
    
    try {
      // Call dual-response endpoint to get both consensus and native AI responses
      const response = await fetch(`${API_BASE}/api/v1/chat/dual-response`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          message: currentInput,
        }),
      });
      
      if (!response.ok) {
        throw new Error(`API error: ${response.status}`);
      }
      
      const data = await response.json();
      
      // First message: Consensus AI
      const consensusMessage: ChatMessage = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: data.consensus.text,
        timestamp: new Date(),
        confidence: data.consensus.confidence / 100,
      };
      
      // Second message: Native Rust AI (orange bubble)
      const nativeMessage: ChatMessage = {
        id: (Date.now() + 2).toString(),
        role: 'native',  // Special role for styling
        content: data.native.text,
        timestamp: new Date(),
        confidence: data.native.confidence / 100,
        flux_position: data.native.flux_position,
      };
      
      // Add both messages
      messages = [...messages, consensusMessage, nativeMessage];
      scrollToBottom();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to send message';
      console.error('Chat error:', err);
    } finally {
      isLoading = false;
    }
  }
  
  function scrollToBottom() {
    setTimeout(() => {
      if (chatContainer) {
        chatContainer.scrollTop = chatContainer.scrollHeight;
      }
    }, 100);
  }
  
  function handleKeyPress(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  }
  
  onMount(() => {
    scrollToBottom();
  });
</script>

<div class="chat-container">
  <div class="chat-header">
    <h2>🌀 SpatialVortex Chat</h2>
    <p class="subtitle">Sacred Geometry AI with ELP Channels</p>
  </div>
  
  <div class="messages" bind:this={chatContainer}>
    {#if messages.length === 0}
      <div class="empty-state">
        <div class="sacred-icon">✨</div>
        <h3>Welcome to SpatialVortex</h3>
        <p>Ask me anything and I'll analyze it through sacred geometry</p>
        <div class="example-prompts">
          <button on:click={() => inputText = "What is consciousness?"}>
            💭 What is consciousness?
          </button>
          <button on:click={() => inputText = "Explain the 3-6-9 pattern"}>
            🔢 Explain 3-6-9
          </button>
          <button on:click={() => inputText = "Tell me about sacred geometry"}>
            📐 Sacred geometry
          </button>
        </div>
      </div>
    {:else}
      {#each messages as message (message.id)}
        <div class="message {message.role}">
          <div class="message-avatar">
            {message.role === 'user' ? '👤' : (message.role === 'native' ? '⚡' : '🌀')}
          </div>
          <div class="message-content">
            <div class="message-text">{message.content}</div>
            {#if message.primary_meaning}
              <div class="color-badge" style="background-color: {message.semantic_color || '#888'}">
                <span class="mood-emoji">🎨</span>
                <span class="mood-text">{message.primary_meaning}</span>
                {#if message.color_confidence}
                  <span class="confidence-pill">{Math.round(message.color_confidence * 100)}%</span>
                {/if}
              </div>
              {#if message.related_meanings && message.related_meanings.length > 0}
                <div class="related-moods">
                  <span class="related-label">Related:</span>
                  {#each message.related_meanings as related}
                    <span class="related-tag">{related}</span>
                  {/each}
                </div>
              {/if}
            {/if}
            {#if message.elp}
              <ELPVisualization
                elp={message.elp}
                position={message.flux_position}
                confidence={message.confidence}
              />
            {/if}
            <div class="message-time">
              {message.timestamp.toLocaleTimeString()}
            </div>
          </div>
        </div>
      {/each}
      
      {#if isLoading}
        <div class="message assistant loading">
          <div class="message-avatar">🌀</div>
          <div class="message-content">
            <div class="typing-indicator">
              <span></span><span></span><span></span>
            </div>
          </div>
        </div>
      {/if}
    {/if}
  </div>
  
  {#if error}
    <div class="error-banner">
      ⚠️ {error}
    </div>
  {/if}
  
  <div class="input-area">
    <textarea
      bind:value={inputText}
      on:keypress={handleKeyPress}
      placeholder="Type your message... (Enter to send, Shift+Enter for new line)"
      rows="3"
      disabled={isLoading}
    ></textarea>
    <button
      on:click={sendMessage}
      disabled={!inputText.trim() || isLoading}
      class="send-button"
    >
      {isLoading ? '⏳' : '🚀'} Send
    </button>
  </div>
</div>

<style>
  .chat-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    max-width: 1200px;
    margin: 0 auto;
    background: linear-gradient(135deg, #0a0a1a 0%, #1a1a2e 100%);
  }
  
  .chat-header {
    padding: 1.5rem;
    background: rgba(255, 255, 255, 0.03);
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    text-align: center;
  }
  
  .chat-header h2 {
    margin: 0;
    font-size: 2em;
    background: linear-gradient(90deg, #ff4444 0%, #4444ff 50%, #44ff44 100%);
    background-clip: text;
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }
  
  .subtitle {
    margin: 0.5rem 0 0 0;
    color: #888;
    font-size: 0.9em;
  }
  
  .messages {
    flex: 1;
    overflow-y: auto;
    padding: 2rem;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }
  
  .empty-state {
    text-align: center;
    padding: 4rem 2rem;
    color: #aaa;
  }
  
  .sacred-icon {
    font-size: 4em;
    margin-bottom: 1rem;
    animation: pulse 2s infinite;
  }
  
  @keyframes pulse {
    0%, 100% { transform: scale(1); opacity: 1; }
    50% { transform: scale(1.1); opacity: 0.8; }
  }
  
  .empty-state h3 {
    color: white;
    margin: 1rem 0;
  }
  
  .example-prompts {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    max-width: 400px;
    margin: 2rem auto 0;
  }
  
  .example-prompts button {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: white;
    padding: 0.75rem 1rem;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s;
    text-align: left;
  }
  
  .example-prompts button:hover {
    background: rgba(255, 255, 255, 0.1);
    border-color: #4444ff;
    transform: translateX(4px);
  }
  
  .message {
    display: flex;
    gap: 1rem;
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
  
  .message.user {
    flex-direction: row-reverse;
  }
  
  .message-avatar {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.5em;
    flex-shrink: 0;
    background: rgba(255, 255, 255, 0.05);
  }
  
  .message.user .message-avatar {
    background: linear-gradient(135deg, #ff4444 0%, #ff6644 100%);
  }
  
  .message.assistant .message-avatar {
    background: linear-gradient(135deg, #4444ff 0%, #6644ff 100%);
  }
  
  .message-content {
    flex: 1;
    max-width: 70%;
  }
  
  .message.user .message-content {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
  }
  
  .message-text {
    padding: 1rem 1.25rem;
    border-radius: 12px;
    line-height: 1.5;
    word-wrap: break-word;
  }
  
  .message.user .message-text {
    background: linear-gradient(135deg, #ff4444 0%, #ff6644 100%);
    color: white;
    border-bottom-right-radius: 4px;
  }
  
  .message.assistant .message-text {
    background: rgba(255, 255, 255, 0.08);
    color: white;
    border-bottom-left-radius: 4px;
  }
  
  /* Native Rust AI - Darker orange bubble with white text for better readability */
  .message.native .message-avatar {
    background: linear-gradient(135deg, #c55020 0%, #d66020 100%);
  }
  
  .message.native .message-text {
    background: linear-gradient(135deg, #c55020 0%, #d66020 100%);
    color: white;
    border-bottom-left-radius: 4px;
  }
  
  .message-time {
    font-size: 0.75em;
    color: #666;
    margin-top: 0.5rem;
    padding: 0 0.25rem;
  }
  
  .typing-indicator {
    padding: 1rem 1.25rem;
    display: flex;
    gap: 4px;
  }
  
  .typing-indicator span {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #4444ff;
    animation: bounce 1.4s infinite ease-in-out;
  }
  
  .typing-indicator span:nth-child(1) {
    animation-delay: -0.32s;
  }
  
  .typing-indicator span:nth-child(2) {
    animation-delay: -0.16s;
  }
  
  @keyframes bounce {
    0%, 80%, 100% {
      transform: scale(0);
      opacity: 0.5;
    }
    40% {
      transform: scale(1);
      opacity: 1;
    }
  }
  
  .error-banner {
    padding: 1rem;
    background: rgba(255, 68, 68, 0.1);
    border: 1px solid rgba(255, 68, 68, 0.3);
    color: #ff4444;
    text-align: center;
    margin: 0 1rem;
  }
  
  .input-area {
    padding: 1.5rem;
    background: rgba(255, 255, 255, 0.03);
    border-top: 1px solid rgba(255, 255, 255, 0.1);
    display: flex;
    gap: 1rem;
  }
  
  textarea {
    flex: 1;
    padding: 1rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    color: white;
    font-family: inherit;
    font-size: 1em;
    resize: none;
    transition: all 0.2s;
  }
  
  textarea:focus {
    outline: none;
    border-color: #4444ff;
    background: rgba(255, 255, 255, 0.08);
  }
  
  textarea::placeholder {
    color: #666;
  }
  
  textarea:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  
  .send-button {
    padding: 1rem 2rem;
    background: linear-gradient(135deg, #4444ff 0%, #6644ff 100%);
    color: white;
    border: none;
    border-radius: 8px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
    white-space: nowrap;
  }
  
  .send-button:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(68, 68, 255, 0.4);
  }
  
  .send-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    transform: none;
  }
  
  /* Scrollbar styling */
  .messages::-webkit-scrollbar {
    width: 8px;
  }
  
  .messages::-webkit-scrollbar-track {
    background: rgba(255, 255, 255, 0.05);
  }
  
  .messages::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.2);
    border-radius: 4px;
  }
  
  .messages::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.3);
  }
  
  /* Color ML Visualization */
  .color-badge {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    border-radius: 20px;
    margin-top: 0.75rem;
    font-size: 0.9em;
    font-weight: 500;
    color: white;
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
    backdrop-filter: blur(10px);
  }
  
  .mood-emoji {
    font-size: 1.2em;
  }
  
  .mood-text {
    text-transform: capitalize;
  }
  
  .confidence-pill {
    background: rgba(255, 255, 255, 0.2);
    padding: 0.2rem 0.5rem;
    border-radius: 10px;
    font-size: 0.85em;
    margin-left: 0.25rem;
  }
  
  .related-moods {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin-top: 0.5rem;
    align-items: center;
  }
  
  .related-label {
    font-size: 0.85em;
    color: #888;
    font-weight: 500;
  }
  
  .related-tag {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    padding: 0.25rem 0.75rem;
    border-radius: 12px;
    font-size: 0.8em;
    color: #aaa;
    transition: all 0.2s;
  }
  
  .related-tag:hover {
    background: rgba(255, 255, 255, 0.1);
    color: white;
    transform: translateY(-1px);
  }
</style>
