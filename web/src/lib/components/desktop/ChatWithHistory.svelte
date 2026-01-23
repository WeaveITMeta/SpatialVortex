<script lang="ts">
  import ChatPanel from './ChatPanel.svelte';
  import SessionHistory from './SessionHistory.svelte';
  import type { ChatMessage } from '$lib/types/chat';
  
  let messages: ChatMessage[] = [];
  let isGenerating = false;
  let showHistory = true;
  let currentSessionId: string | null = null;
  
  interface Session {
    id: string;
    title: string;
    summary?: string;
    created_at: string;
    updated_at: string;
    message_count: number;
    last_message_at?: string;
    tags: string[];
    is_archived: boolean;
  }
  
  // Create new session when user sends first message
  async function createSession(firstMessage: string) {
    try {
      const response = await fetch('http://localhost:7000/api/v1/sessions/create', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          title: 'New Conversation',
          tags: []
        })
      });
      
      const data = await response.json();
      if (data.success) {
        currentSessionId = data.session.id;
        
        // Add first message to session
        await addMessageToSession(currentSessionId, 'user', firstMessage);
        
        return currentSessionId;
      }
    } catch (error) {
      console.error('Failed to create session:', error);
    }
    return null;
  }
  
  // Add message to current session
  async function addMessageToSession(sessionId: string, role: string, content: string) {
    try {
      await fetch(`http://localhost:7000/api/v1/sessions/${sessionId}/messages`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          role,
          content,
          token_count: content.split(' ').length
        })
      });
    } catch (error) {
      console.error('Failed to add message to session:', error);
    }
  }
  
  // Handle new message from chat panel
  async function handleNewMessage(event: CustomEvent<ChatMessage>) {
    const message = event.detail;
    
    // Create session if this is the first message
    if (!currentSessionId && message.role === 'user') {
      await createSession(message.content);
    }
    
    // Add message to session
    if (currentSessionId) {
      await addMessageToSession(currentSessionId, message.role, message.content);
    }
  }
  
  // Resume existing session
  async function handleResumeSession(event: CustomEvent<Session>) {
    const session = event.detail;
    
    try {
      // Load messages from session
      const response = await fetch(`http://localhost:7000/api/v1/sessions/${session.id}/messages`);
      const data = await response.json();
      
      // Convert session messages to chat messages
      messages = data.messages.map((msg: any) => ({
        id: msg.id,
        role: msg.role,
        content: msg.content,
        timestamp: new Date(msg.timestamp)
      }));
      
      currentSessionId = session.id;
      
    } catch (error) {
      console.error('Failed to resume session:', error);
    }
  }
  
  // Start new chat
  function handleNewChat() {
    messages = [];
    currentSessionId = null;
  }
  
  function toggleHistory() {
    showHistory = !showHistory;
  }
</script>

<div class="chat-with-history">
  {#if showHistory}
    <div class="history-panel">
      <SessionHistory 
        on:resume={handleResumeSession}
        on:newChat={handleNewChat}
      />
    </div>
  {/if}
  
  <div class="chat-panel-wrapper">
    <button class="toggle-history-btn" on:click={toggleHistory}>
      {showHistory ? '◀' : '▶'} {showHistory ? 'Hide' : 'Show'} History
    </button>
    
    <ChatPanel 
      bind:messages 
      bind:isGenerating
      on:newMessage={handleNewMessage}
    />
  </div>
</div>

<style>
  .chat-with-history {
    display: flex;
    width: 100%;
    height: 100vh;
    background: #18181b;
  }
  
  .history-panel {
    width: 320px;
    height: 100%;
    border-right: 1px solid rgba(255, 255, 255, 0.1);
    flex-shrink: 0;
    animation: slideIn 0.3s ease-out;
  }
  
  @keyframes slideIn {
    from {
      transform: translateX(-100%);
      opacity: 0;
    }
    to {
      transform: translateX(0);
      opacity: 1;
    }
  }
  
  .chat-panel-wrapper {
    flex: 1;
    position: relative;
    overflow: hidden;
  }
  
  .toggle-history-btn {
    position: absolute;
    top: 1rem;
    left: 1rem;
    z-index: 10;
    padding: 0.5rem 1rem;
    background: rgba(255, 255, 255, 0.05);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    color: #e4e4e7;
    font-size: 0.9rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .toggle-history-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    border-color: #60a5fa;
  }
</style>
