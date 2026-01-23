<script lang="ts">
  import { onMount } from 'svelte';
  import { createEventDispatcher } from 'svelte';
  
  const dispatch = createEventDispatcher();
  
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
  
  interface SessionStats {
    total_sessions: number;
    active_sessions: number;
    archived_sessions: number;
    total_messages: number;
  }
  
  let sessions: Session[] = [];
  let filteredSessions: Session[] = [];
  let stats: SessionStats | null = null;
  let searchQuery = '';
  let isLoading = false;
  let showArchived = false;
  let selectedTag: string | null = null;
  
  onMount(() => {
    loadSessions();
    loadStats();
  });
  
  async function loadSessions() {
    isLoading = true;
    try {
      const params = new URLSearchParams({
        include_archived: showArchived.toString()
      });
      
      const response = await fetch(`http://localhost:7000/api/v1/sessions/list?${params}`);
      const data = await response.json();
      sessions = data.sessions || [];
      filterSessions();
    } catch (error) {
      console.error('Failed to load sessions:', error);
    } finally {
      isLoading = false;
    }
  }
  
  async function loadStats() {
    try {
      const response = await fetch('http://localhost:7000/api/v1/sessions/stats');
      stats = await response.json();
    } catch (error) {
      console.error('Failed to load stats:', error);
    }
  }
  
  async function searchSessions() {
    if (!searchQuery.trim()) {
      filterSessions();
      return;
    }
    
    isLoading = true;
    try {
      const response = await fetch('http://localhost:7000/api/v1/sessions/search', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ query: searchQuery })
      });
      
      const data = await response.json();
      sessions = data.sessions || [];
      filterSessions();
    } catch (error) {
      console.error('Failed to search sessions:', error);
    } finally {
      isLoading = false;
    }
  }
  
  function filterSessions() {
    filteredSessions = sessions.filter(s => {
      if (selectedTag && !s.tags.includes(selectedTag)) {
        return false;
      }
      if (!showArchived && s.is_archived) {
        return false;
      }
      return true;
    });
  }
  
  function resumeSession(session: Session) {
    dispatch('resume', session);
  }
  
  async function deleteSession(sessionId: string) {
    if (!confirm('Delete this session? This cannot be undone.')) {
      return;
    }
    
    try {
      await fetch(`http://localhost:7000/api/v1/sessions/${sessionId}`, {
        method: 'DELETE'
      });
      
      await loadSessions();
      await loadStats();
    } catch (error) {
      console.error('Failed to delete session:', error);
    }
  }
  
  async function archiveSession(sessionId: string) {
    try {
      await fetch(`http://localhost:7000/api/v1/sessions/${sessionId}/archive`, {
        method: 'PUT'
      });
      
      await loadSessions();
      await loadStats();
    } catch (error) {
      console.error('Failed to archive session:', error);
    }
  }
  
  function formatDate(dateString: string): string {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);
    
    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;
    
    return date.toLocaleDateString();
  }
  
  function getAllTags(): string[] {
    const tagSet = new Set<string>();
    sessions.forEach(s => s.tags.forEach(t => tagSet.add(t)));
    return Array.from(tagSet).sort();
  }
  
  $: {
    // Reactively filter when showArchived or selectedTag changes
    filterSessions();
  }
</script>

<div class="session-history">
  <!-- Header -->
  <div class="history-header">
    <h2 class="history-title">💬 Chat History</h2>
    <button class="new-chat-btn" on:click={() => dispatch('newChat')}>
      ➕ New Chat
    </button>
  </div>
  
  <!-- Stats -->
  {#if stats}
    <div class="stats-bar">
      <div class="stat">
        <span class="stat-value">{stats.active_sessions}</span>
        <span class="stat-label">Active</span>
      </div>
      <div class="stat">
        <span class="stat-value">{stats.total_messages}</span>
        <span class="stat-label">Messages</span>
      </div>
      <div class="stat">
        <span class="stat-value">{stats.archived_sessions}</span>
        <span class="stat-label">Archived</span>
      </div>
    </div>
  {/if}
  
  <!-- Search -->
  <div class="search-box">
    <input
      type="text"
      bind:value={searchQuery}
      on:keydown={(e) => e.key === 'Enter' && searchSessions()}
      placeholder="Search conversations..."
      class="search-input"
    />
    <button class="search-btn" on:click={searchSessions}>
      🔍
    </button>
  </div>
  
  <!-- Filters -->
  <div class="filters">
    <label class="filter-checkbox">
      <input type="checkbox" bind:checked={showArchived} />
      <span>Show Archived</span>
    </label>
    
    {#if getAllTags().length > 0}
      <select bind:value={selectedTag} class="tag-filter">
        <option value={null}>All Tags</option>
        {#each getAllTags() as tag}
          <option value={tag}>{tag}</option>
        {/each}
      </select>
    {/if}
  </div>
  
  <!-- Sessions List -->
  <div class="sessions-list">
    {#if isLoading}
      <div class="loading">
        <div class="spinner"></div>
        <span>Loading...</span>
      </div>
    {:else if filteredSessions.length === 0}
      <div class="empty-state">
        <span class="empty-icon">💭</span>
        <p>No conversations yet</p>
        <p class="empty-hint">Start a new chat to begin</p>
      </div>
    {:else}
      {#each filteredSessions as session (session.id)}
        <div class="session-card" class:archived={session.is_archived}>
          <div class="session-main" on:click={() => resumeSession(session)}>
            <h3 class="session-title">{session.title}</h3>
            
            {#if session.summary}
              <p class="session-summary">{session.summary}</p>
            {/if}
            
            <div class="session-meta">
              <span class="meta-item">
                💬 {session.message_count} {session.message_count === 1 ? 'message' : 'messages'}
              </span>
              <span class="meta-item">
                🕐 {formatDate(session.updated_at)}
              </span>
            </div>
            
            {#if session.tags.length > 0}
              <div class="session-tags">
                {#each session.tags as tag}
                  <span class="tag">{tag}</span>
                {/each}
              </div>
            {/if}
          </div>
          
          <div class="session-actions">
            {#if !session.is_archived}
              <button 
                class="action-btn archive-btn" 
                on:click|stopPropagation={() => archiveSession(session.id)}
                title="Archive"
              >
                📦
              </button>
            {/if}
            <button 
              class="action-btn delete-btn" 
              on:click|stopPropagation={() => deleteSession(session.id)}
              title="Delete"
            >
              🗑️
            </button>
          </div>
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .session-history {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #18181b;
    color: #e4e4e7;
  }
  
  .history-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }
  
  .history-title {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 700;
  }
  
  .new-chat-btn {
    padding: 0.5rem 1rem;
    background: linear-gradient(135deg, #60a5fa 0%, #3b82f6 100%);
    border: none;
    border-radius: 6px;
    color: white;
    font-weight: 600;
    font-size: 0.9rem;
    cursor: pointer;
    transition: transform 0.2s;
  }
  
  .new-chat-btn:hover {
    transform: translateY(-2px);
  }
  
  .stats-bar {
    display: flex;
    gap: 1rem;
    padding: 1rem;
    background: rgba(96, 165, 250, 0.05);
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }
  
  .stat {
    display: flex;
    flex-direction: column;
    align-items: center;
    flex: 1;
  }
  
  .stat-value {
    font-size: 1.5rem;
    font-weight: 700;
    color: #60a5fa;
  }
  
  .stat-label {
    font-size: 0.75rem;
    color: #a1a1aa;
    margin-top: 0.25rem;
  }
  
  .search-box {
    display: flex;
    gap: 0.5rem;
    padding: 1rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }
  
  .search-input {
    flex: 1;
    padding: 0.625rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    color: #e4e4e7;
    font-size: 0.9rem;
  }
  
  .search-input:focus {
    outline: none;
    border-color: #60a5fa;
  }
  
  .search-btn {
    padding: 0.625rem 1rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    color: #e4e4e7;
    font-size: 1rem;
    cursor: pointer;
    transition: background 0.2s;
  }
  
  .search-btn:hover {
    background: rgba(255, 255, 255, 0.1);
  }
  
  .filters {
    display: flex;
    gap: 1rem;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    align-items: center;
  }
  
  .filter-checkbox {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.9rem;
    cursor: pointer;
  }
  
  .tag-filter {
    padding: 0.375rem 0.75rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    color: #e4e4e7;
    font-size: 0.85rem;
    cursor: pointer;
  }
  
  .sessions-list {
    flex: 1;
    overflow-y: auto;
    padding: 0.5rem;
  }
  
  .loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    padding: 3rem;
    color: #a1a1aa;
  }
  
  .spinner {
    width: 30px;
    height: 30px;
    border: 3px solid rgba(96, 165, 250, 0.3);
    border-top-color: #60a5fa;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }
  
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 3rem 1rem;
    text-align: center;
    color: #a1a1aa;
  }
  
  .empty-icon {
    font-size: 3rem;
    margin-bottom: 1rem;
    opacity: 0.5;
  }
  
  .empty-hint {
    font-size: 0.85rem;
    margin-top: 0.5rem;
  }
  
  .session-card {
    display: flex;
    gap: 0.5rem;
    padding: 1rem;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    margin-bottom: 0.5rem;
    transition: all 0.2s;
    cursor: pointer;
  }
  
  .session-card:hover {
    background: rgba(255, 255, 255, 0.05);
    border-color: #60a5fa;
    transform: translateX(4px);
  }
  
  .session-card.archived {
    opacity: 0.6;
  }
  
  .session-main {
    flex: 1;
  }
  
  .session-title {
    margin: 0 0 0.5rem 0;
    font-size: 1rem;
    font-weight: 600;
    color: #e4e4e7;
    overflow: hidden;
    text-overflow: ellipsis;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
  }
  
  .session-summary {
    margin: 0 0 0.5rem 0;
    font-size: 0.85rem;
    color: #a1a1aa;
    overflow: hidden;
    text-overflow: ellipsis;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
  }
  
  .session-meta {
    display: flex;
    gap: 1rem;
    font-size: 0.75rem;
    color: #71717a;
    margin-bottom: 0.5rem;
  }
  
  .meta-item {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }
  
  .session-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 0.375rem;
  }
  
  .tag {
    padding: 0.25rem 0.5rem;
    background: rgba(96, 165, 250, 0.1);
    border: 1px solid rgba(96, 165, 250, 0.3);
    border-radius: 4px;
    font-size: 0.7rem;
    color: #60a5fa;
  }
  
  .session-actions {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  
  .action-btn {
    width: 2rem;
    height: 2rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 4px;
    font-size: 1rem;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .action-btn:hover {
    background: rgba(255, 255, 255, 0.1);
  }
  
  .archive-btn:hover {
    background: rgba(255, 193, 7, 0.2);
    border-color: rgba(255, 193, 7, 0.5);
  }
  
  .delete-btn:hover {
    background: rgba(239, 68, 68, 0.2);
    border-color: rgba(239, 68, 68, 0.5);
  }
</style>
