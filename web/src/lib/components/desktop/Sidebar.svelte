<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import SessionContextMenu from './SessionContextMenu.svelte';
  import ShareModal from './ShareModal.svelte';
  
  export let sessions: string[] = [];
  export let currentSession: string;
  export let showAnalytics = true;
  
  const dispatch = createEventDispatcher();
  
  // Context menu state
  let contextMenuX = 0;
  let contextMenuY = 0;
  let showContextMenu = false;
  let contextMenuSession = '';
  
  // Share modal state
  let showShareModal = false;
  let shareSessionTitle = '';
  let shareSessionId = '';
  
  function handleContextMenu(event: MouseEvent, session: string) {
    event.preventDefault();
    contextMenuX = event.clientX;
    contextMenuY = event.clientY;
    contextMenuSession = session;
    showContextMenu = true;
  }
  
  function handleRename() {
    const newName = prompt('Enter new session name:', contextMenuSession);
    if (newName && newName.trim()) {
      dispatch('renameSession', { oldName: contextMenuSession, newName: newName.trim() });
    }
  }
  
  function handleShare() {
    shareSessionTitle = contextMenuSession;
    // Dispatch to get session ID from parent
    dispatch('getSessionId', contextMenuSession);
    showShareModal = true;
  }
  
  function handleDelete() {
    if (confirm(`Delete session "${contextMenuSession}"?`)) {
      dispatch('deleteSession', contextMenuSession);
    }
  }
  
  export function setShareSessionId(id: string) {
    shareSessionId = id;
  }
</script>

<aside class="sidebar">
  <div class="sidebar-header">
    <div class="logo">
      <span class="logo-icon">🌀</span>
      <span class="logo-text">The Vortex</span>
    </div>
    <button class="new-chat-btn" on:click={() => dispatch('newSession')}>
      <span>+</span> New Chat
    </button>
  </div>
  
  <nav class="sessions">
    <div class="sessions-header">
      <span>Recent Sessions</span>
      <span class="sessions-count">{sessions.length}</span>
    </div>
    {#each sessions as session}
      <button
        class="session-item"
        class:active={session === currentSession}
        on:click={() => dispatch('switchSession', session)}
        on:contextmenu={(e) => handleContextMenu(e, session)}
        title="Right-click for options"
      >
        <span class="session-icon">💬</span>
        <div class="session-info">
          <span class="session-name">{session}</span>
          <span class="session-hint">Right-click for options</span>
        </div>
      </button>
    {/each}
    
    {#if sessions.length === 0}
      <div class="empty-state">
        <span class="empty-icon">🌀</span>
        <p>No sessions yet</p>
        <p class="empty-hint">Click "New Chat" to start</p>
      </div>
    {/if}
  </nav>
  
  <SessionContextMenu
    bind:show={showContextMenu}
    x={contextMenuX}
    y={contextMenuY}
    sessionTitle={contextMenuSession}
    on:rename={handleRename}
    on:share={handleShare}
    on:delete={handleDelete}
  />
  
  <ShareModal
    bind:show={showShareModal}
    sessionTitle={shareSessionTitle}
    sessionId={shareSessionId}
  />
  
  <div class="sidebar-footer">
    <button
      class="control-btn"
      class:active={showAnalytics}
      on:click={() => dispatch('toggleAnalytics')}
      title="Toggle Analytics Panel"
    >
      <span>📊</span> Analytics
    </button>
    <button
      class="control-btn"
      on:click={() => dispatch('toggleSettings')}
      title="Settings"
    >
      <span>⚙️</span> Settings
    </button>
  </div>
</aside>

<style>
  .sidebar {
    background: linear-gradient(180deg, #1a1a2e 0%, #16161f 100%);
    border-right: 1px solid rgba(255, 255, 255, 0.06);
    display: flex;
    flex-direction: column;
    height: 100vh;
  }
  
  .sidebar-header {
    padding: 1.5rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }
  
  .logo {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 1.5rem;
    padding: 0 0.25rem;
  }
  
  .logo-icon {
    font-size: 28px;
    animation: rotate 8s linear infinite;
  }
  
  @keyframes rotate {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
  
  .logo-text {
    font-size: 32px;
    font-weight: 700;
    background: linear-gradient(135deg, #60a5fa 0%, #3b82f6 100%);
    -webkit-background-clip: text;
    background-clip: text;
    -webkit-text-fill-color: transparent;
    letter-spacing: -0.5px;
  }
  
  .new-chat-btn {
    width: 100%;
    padding: 0.75rem 1rem;
    background: linear-gradient(135deg, #60a5fa 0%, #3b82f6 100%);
    color: white;
    border: none;
    border-radius: 8px;
    font-weight: 600;
    font-size: 0.9rem;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
  }
  
  .new-chat-btn:hover {
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
  }
  
  .new-chat-btn span:first-child {
    font-size: 1.2rem;
  }
  
  .sessions {
    flex: 1;
    overflow-y: auto;
    padding: 0.5rem;
  }
  
  .sessions-header {
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    color: #71717a;
    padding: 0.75rem 0.75rem 0.5rem;
    letter-spacing: 0.5px;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  
  .sessions-count {
    background: rgba(96, 165, 250, 0.15);
    color: #60a5fa;
    padding: 0.125rem 0.5rem;
    border-radius: 12px;
    font-size: 0.7rem;
    font-weight: 700;
  }
  
  .session-item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem;
    background: transparent;
    border: none;
    border-radius: 8px;
    color: #a1a1aa;
    cursor: pointer;
    transition: all 0.2s;
    text-align: left;
    margin-bottom: 0.25rem;
  }
  
  .session-item:hover {
    background: rgba(255, 255, 255, 0.05);
    color: #e4e4e7;
  }
  
  .session-item.active {
    background: rgba(96, 165, 250, 0.15);
    color: #60a5fa;
    font-weight: 500;
  }
  
  .session-icon {
    font-size: 1.1rem;
    opacity: 0.7;
  }
  
  .session-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
    min-width: 0;
  }
  
  .session-name {
    font-size: 0.9rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  
  .session-hint {
    font-size: 0.7rem;
    color: #52525b;
    opacity: 0;
    transition: opacity 0.2s;
  }
  
  .session-item:hover .session-hint {
    opacity: 1;
  }
  
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 3rem 1.5rem;
    text-align: center;
    color: #71717a;
  }
  
  .empty-icon {
    font-size: 3rem;
    margin-bottom: 1rem;
    opacity: 0.3;
  }
  
  .empty-state p {
    margin: 0.25rem 0;
    font-size: 0.875rem;
  }
  
  .empty-hint {
    font-size: 0.75rem !important;
    color: #52525b;
  }
  
  .sidebar-footer {
    padding: 1rem;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  
  .control-btn {
    width: 100%;
    padding: 0.75rem 1rem;
    background: transparent;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    color: #a1a1aa;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    font-size: 0.9rem;
  }
  
  .control-btn:hover {
    background: rgba(255, 255, 255, 0.05);
    border-color: rgba(255, 255, 255, 0.15);
    color: #e4e4e7;
  }
  
  .control-btn.active {
    background: rgba(96, 165, 250, 0.15);
    border-color: rgba(96, 165, 250, 0.3);
    color: #60a5fa;
  }
  
  .control-btn span:first-child {
    font-size: 1.1rem;
  }
  
  .sessions::-webkit-scrollbar {
    width: 6px;
  }
  
  .sessions::-webkit-scrollbar-track {
    background: transparent;
  }
  
  .sessions::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 3px;
  }
  
  .sessions::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.15);
  }
</style>
