<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { collaborationService, type SessionState, type CollaborationUser } from '$lib/services/collaboration';
  
  export let sessionId: string = '';
  export let username: string = 'User';
  
  let isConnected = false;
  let currentSession: SessionState | null = null;
  let activeUsers: CollaborationUser[] = [];
  let shareLink = '';
  let showCopyNotification = false;
  
  onMount(async () => {
    // Generate session ID if not provided
    if (!sessionId) {
      sessionId = generateSessionId();
    }
    
    // Join session
    try {
      currentSession = await collaborationService.joinSession(sessionId, username);
      isConnected = true;
      updateActiveUsers();
      
      // Generate share link
      shareLink = `${window.location.origin}?session=${sessionId}`;
      
      // Listen for session updates
      collaborationService.on('session_update', handleSessionUpdate);
    } catch (error) {
      console.error('Failed to join collaboration session:', error);
    }
  });
  
  onDestroy(() => {
    collaborationService.off('session_update', handleSessionUpdate);
    collaborationService.leaveSession();
  });
  
  function generateSessionId(): string {
    return Math.random().toString(36).substring(2, 10);
  }
  
  function handleSessionUpdate(session: SessionState) {
    currentSession = session;
    updateActiveUsers();
  }
  
  function updateActiveUsers() {
    if (!currentSession) return;
    
    activeUsers = Object.values(currentSession.active_users)
      .filter(user => user.user_id !== collaborationService.getCurrentUser().user_id);
  }
  
  async function copyShareLink() {
    try {
      await navigator.clipboard.writeText(shareLink);
      showCopyNotification = true;
      setTimeout(() => {
        showCopyNotification = false;
      }, 2000);
    } catch (error) {
      console.error('Failed to copy link:', error);
    }
  }
  
  function getUserInitials(username: string): string {
    return username
      .split(' ')
      .map(word => word[0])
      .join('')
      .toUpperCase()
      .substring(0, 2);
  }
  
  function getTimeAgo(timestamp: number): string {
    const now = Date.now() / 1000;
    const diff = now - timestamp;
    
    if (diff < 60) return 'just now';
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    return `${Math.floor(diff / 86400)}d ago`;
  }
</script>

<div class="collaboration-panel">
  <div class="panel-header">
    <div class="header-left">
      <h3 class="panel-title">👥 Collaboration</h3>
      <div class="connection-status" class:connected={isConnected}>
        <span class="status-dot"></span>
        {isConnected ? 'Connected' : 'Connecting...'}
      </div>
    </div>
  </div>
  
  {#if isConnected}
    <!-- Share Section -->
    <div class="share-section">
      <label class="section-label">Share Session</label>
      <div class="share-input-group">
        <input 
          type="text" 
          readonly 
          value={shareLink}
          class="share-input"
        />
        <button class="copy-btn" on:click={copyShareLink}>
          📋
        </button>
      </div>
      {#if showCopyNotification}
        <div class="copy-notification">
          ✅ Copied to clipboard!
        </div>
      {/if}
      <p class="share-hint">
        Share this link to collaborate in real-time
      </p>
    </div>
    
    <!-- Active Users -->
    <div class="users-section">
      <label class="section-label">
        Active Users ({activeUsers.length + 1})
      </label>
      
      <div class="users-list">
        <!-- Current User -->
        <div class="user-card current-user">
          <div class="user-avatar" style="background: #60a5fa;">
            {getUserInitials(username)}
          </div>
          <div class="user-info">
            <div class="user-name">{username} (You)</div>
            <div class="user-status">Online</div>
          </div>
        </div>
        
        <!-- Other Users -->
        {#each activeUsers as user}
          <div class="user-card">
            <div class="user-avatar" style="background: {user.color};">
              {getUserInitials(user.username)}
            </div>
            <div class="user-info">
              <div class="user-name">{user.username}</div>
              <div class="user-status">
                Active {getTimeAgo(user.last_seen)}
              </div>
            </div>
            {#if user.cursor}
              <div class="cursor-indicator" title="Cursor active">
                🖱️
              </div>
            {/if}
          </div>
        {/each}
        
        {#if activeUsers.length === 0}
          <div class="empty-state">
            <span class="empty-icon">👤</span>
            <p>No other users yet</p>
            <p class="empty-hint">Share the link to invite collaborators</p>
          </div>
        {/if}
      </div>
    </div>
    
    <!-- Session Info -->
    <div class="session-info">
      <div class="info-row">
        <span class="info-label">Session ID:</span>
        <span class="info-value">{sessionId}</span>
      </div>
      <div class="info-row">
        <span class="info-label">Started:</span>
        <span class="info-value">
          {currentSession ? new Date(currentSession.created_at * 1000).toLocaleTimeString() : '-'}
        </span>
      </div>
    </div>
  {:else}
    <div class="connecting-state">
      <div class="spinner"></div>
      <p>Connecting to collaboration server...</p>
    </div>
  {/if}
</div>

<style>
  .collaboration-panel {
    padding: 1.5rem;
    background: linear-gradient(180deg, #1a1a2e 0%, #16161f 100%);
    border-radius: 12px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    max-height: 80vh;
    overflow-y: auto;
  }
  
  .panel-header {
    display: flex;
    justify-content: flex-start;
    align-items: center;
    margin-bottom: 1.5rem;
  }
  
  .header-left {
    display: flex;
    align-items: center;
    gap: 1rem;
    flex-wrap: wrap;
  }
  
  .panel-title {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
    color: #e4e4e7;
  }
  
  .connection-status {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.85rem;
    color: #a1a1aa;
    padding: 0.375rem 0.75rem;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 20px;
  }
  
  .connection-status.connected {
    color: #22c55e;
  }
  
  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #71717a;
    animation: pulse-dot 2s infinite;
  }
  
  .connection-status.connected .status-dot {
    background: #22c55e;
  }
  
  @keyframes pulse-dot {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
  }
  
  .share-section {
    margin-bottom: 1.5rem;
    padding-bottom: 1.5rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }
  
  .section-label {
    display: block;
    font-size: 0.9rem;
    font-weight: 500;
    color: #d4d4d4;
    margin-bottom: 0.75rem;
  }
  
  .share-input-group {
    display: flex;
    gap: 0.5rem;
  }
  
  .share-input {
    flex: 1;
    padding: 0.625rem 1rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    color: #e4e4e7;
    font-size: 0.875rem;
    font-family: monospace;
  }
  
  .copy-btn {
    padding: 0.625rem 1rem;
    background: rgba(96, 165, 250, 0.1);
    border: 1px solid rgba(96, 165, 250, 0.3);
    border-radius: 6px;
    color: #60a5fa;
    font-size: 1.25rem;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .copy-btn:hover {
    background: rgba(96, 165, 250, 0.2);
    transform: scale(1.05);
  }
  
  .copy-notification {
    margin-top: 0.5rem;
    padding: 0.5rem;
    background: rgba(34, 197, 94, 0.1);
    border: 1px solid rgba(34, 197, 94, 0.3);
    border-radius: 6px;
    color: #22c55e;
    font-size: 0.85rem;
    text-align: center;
    animation: slideDown 0.3s ease-out;
  }
  
  @keyframes slideDown {
    from {
      opacity: 0;
      transform: translateY(-10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
  
  .share-hint {
    margin-top: 0.5rem;
    font-size: 0.8rem;
    color: #71717a;
  }
  
  .users-section {
    margin-bottom: 1.5rem;
  }
  
  .users-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  
  .user-card {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    transition: all 0.2s;
  }
  
  .user-card:hover {
    background: rgba(255, 255, 255, 0.05);
  }
  
  .user-card.current-user {
    border-color: rgba(96, 165, 250, 0.3);
    background: rgba(96, 165, 250, 0.05);
  }
  
  .user-avatar {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
    font-weight: 600;
    font-size: 0.9rem;
  }
  
  .user-info {
    flex: 1;
  }
  
  .user-name {
    font-weight: 500;
    color: #e4e4e7;
    font-size: 0.9rem;
  }
  
  .user-status {
    font-size: 0.75rem;
    color: #71717a;
    margin-top: 0.125rem;
  }
  
  .cursor-indicator {
    font-size: 1.25rem;
  }
  
  .empty-state {
    text-align: center;
    padding: 2rem 1rem;
    color: #71717a;
  }
  
  .empty-icon {
    font-size: 3rem;
    display: block;
    margin-bottom: 0.5rem;
  }
  
  .empty-hint {
    font-size: 0.8rem;
    margin-top: 0.5rem;
  }
  
  .session-info {
    padding: 1rem;
    background: rgba(255, 255, 255, 0.03);
    border-radius: 8px;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  
  .info-row {
    display: flex;
    justify-content: space-between;
    font-size: 0.85rem;
  }
  
  .info-label {
    color: #a1a1aa;
  }
  
  .info-value {
    color: #d4d4d4;
    font-family: monospace;
  }
  
  .connecting-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 3rem 1rem;
    color: #a1a1aa;
  }
  
  .spinner {
    width: 40px;
    height: 40px;
    border: 4px solid rgba(255, 255, 255, 0.1);
    border-top-color: #60a5fa;
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-bottom: 1rem;
  }
  
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
