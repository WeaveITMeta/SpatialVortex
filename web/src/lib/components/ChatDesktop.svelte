<script lang="ts">
  import Sidebar from './desktop/Sidebar.svelte';
  import ChatPanel from './desktop/ChatPanel.svelte';
  import AnalyticsPanel from './desktop/AnalyticsPanel.svelte';
  import type { ChatMessage } from '../types/chat';
  import { sessionStore } from '../stores/sessionStore';
  import { onMount } from 'svelte';
  
  let showAnalytics = true;
  let showSettings = false;
  let isGenerating = false;
  
  // Subscribe to individual stores
  let currentSession: any;
  let messages: ChatMessage[] = [];
  let sessionId = '';
  let sessions: string[] = [];
  
  // React to store changes
  sessionStore.currentSession.subscribe(session => {
    currentSession = session;
    messages = session?.messages || [];
    sessionId = session?.id || '';
  });
  
  sessionStore.getSortedSessions.subscribe(sorted => {
    sessions = sorted.map((s: any) => s.title);
  });
  
  // Initialize: Load sessions from backend, then create first session if none exist
  onMount(() => {
    // Load sessions from backend first
    sessionStore.loadSessions().then(() => {
      sessionStore.sessions.subscribe(sessionsMap => {
        if (sessionsMap.size === 0) {
          sessionStore.createSession();
        } else {
          sessionStore.getSortedSessions.subscribe(sorted => {
            if (sorted.length > 0 && sorted[0]) {
              sessionStore.switchSession(sorted[0].id);
            }
          });
        }
      });
    });
  });
  
  // API configuration
  const API_BASE = 'http://localhost:7000';
  
  async function sendMessage(text: string) {
    if (!sessionId) return;
    
    isGenerating = true;
    const userMessage: ChatMessage = {
      id: Date.now().toString(),
      role: 'user',
      content: text,
      timestamp: new Date(),
    };
    
    // Add to store
    sessionStore.addMessage(userMessage);
    
    try {
      // Call dual-response endpoint to get both consensus and native AI responses
      const response = await fetch(`${API_BASE}/api/v1/chat/dual-response`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          message: text,
        }),
      });
      
      if (!response.ok) throw new Error(`API error: ${response.status}`);
      
      // Parse multi-model response
      const data = await response.json();
      console.log('🔍 Received data:', data);
      console.log('📊 Model responses:', data.model_responses);
      console.log('🌀 Vortex consensus:', data.vortex_consensus);
      
      // Add each individual model response as separate bubbles
      if (data.model_responses && Array.isArray(data.model_responses)) {
        data.model_responses.forEach((modelResp: any, index: number) => {
        const modelMessage: ChatMessage = {
          id: (Date.now() + index + 1).toString(),
          role: 'assistant',  // Each model is 'assistant' role
          model_name: modelResp.model_name,  // Store model name for display
          content: modelResp.text,
          timestamp: new Date(),
          confidence: modelResp.confidence / 100,
        };
        sessionStore.addMessage(modelMessage);
        });
      } else {
        console.warn('⚠️ No model responses in data');
      }
      
      // Final message: Vortex consensus (orange bubble)
      if (data.vortex_consensus) {
        const vortexMessage: ChatMessage = {
          id: (Date.now() + (data.model_responses?.length || 0) + 1).toString(),
          role: 'vortex',  // Vortex consensus role
          content: data.vortex_consensus.text,
          timestamp: new Date(),
          confidence: data.vortex_consensus.confidence / 100,
          flux_position: data.vortex_consensus.flux_position,
        };
        
        sessionStore.addMessage(vortexMessage);
      } else {
        console.warn('⚠️ No vortex consensus in data');
      }
      
      // Auto-generate title after first exchange
      // Count user messages to determine if this is the first exchange
      const userMessageCount = messages.filter(m => m.role === 'user').length;
      if (userMessageCount === 1) {
        // First user message - generate title
        setTimeout(() => {
          sessionStore.generateTitle(sessionId);
        }, 100);
      }
    } catch (err) {
      console.error('Chat error:', err);
      throw err;
    } finally {
      isGenerating = false;
    }
  }
  
  function newSession() {
    sessionStore.createSession();
  }
  
  function switchSession(sessionTitle: string) {
    // Find session by title
    sessionStore.getSortedSessions.subscribe(sorted => {
      const session = sorted.find((s: any) => s.title === sessionTitle);
      if (session) {
        sessionStore.switchSession(session.id);
      }
    });
  }
  
  function renameSession(event: CustomEvent<{oldName: string, newName: string}>) {
    const { oldName, newName } = event.detail;
    sessionStore.getSortedSessions.subscribe(sorted => {
      const session = sorted.find((s: any) => s.title === oldName);
      if (session) {
        sessionStore.updateTitle(session.id, newName);
      }
    });
  }
  
  function shareSession(event: CustomEvent<string>) {
    const sessionTitle = event.detail;
    sessionStore.getSortedSessions.subscribe(sorted => {
      const session = sorted.find((s: any) => s.title === sessionTitle);
      if (session) {
        // Export as JSON to clipboard
        const exportData = JSON.stringify(session, null, 2);
        navigator.clipboard.writeText(exportData);
        console.log('Session exported:', session);
      }
    });
  }
  
  function deleteSession(event: CustomEvent<string>) {
    const sessionTitle = event.detail;
    sessionStore.getSortedSessions.subscribe(sorted => {
      const session = sorted.find((s: any) => s.title === sessionTitle);
      if (session) {
        sessionStore.deleteSession(session.id);
      }
    });
  }
  
  let sidebarRef: any;
  
  function getSessionId(event: CustomEvent<string>) {
    const sessionTitle = event.detail;
    sessionStore.getSortedSessions.subscribe(sorted => {
      const session = sorted.find((s: any) => s.title === sessionTitle);
      if (session && sidebarRef) {
        sidebarRef.setShareSessionId(session.id);
      }
    });
  }
</script>

<div class="desktop-app">
  <Sidebar
    bind:this={sidebarRef}
    {sessions}
    currentSession={currentSession?.title || 'New Chat'}
    on:newSession={newSession}
    on:switchSession={(e) => switchSession(e.detail)}
    on:renameSession={renameSession}
    on:shareSession={shareSession}
    on:deleteSession={deleteSession}
    on:getSessionId={getSessionId}
    on:toggleAnalytics={() => showAnalytics = !showAnalytics}
    on:toggleSettings={() => showSettings = !showSettings}
    bind:showAnalytics
  />
  
  <ChatPanel
    {messages}
    {isGenerating}
    on:send={(e) => sendMessage(e.detail)}
  />
  
  {#if showAnalytics}
    <AnalyticsPanel {messages} />
  {/if}
</div>

<style>
  .desktop-app {
    display: grid;
    grid-template-columns: 280px 1fr auto;
    height: 100vh;
    background: #0f0f1a;
    color: #e4e4e7;
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    overflow: hidden;
  }
  
  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
  }
</style>
