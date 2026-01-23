<script lang="ts">
  import SourceCard from './SourceCard.svelte';
  
  export let sources: Array<{
    doc_id: string;
    chunk_id: string;
    relevance: number;
    content_snippet: string;
    web_source?: {
      url: string;
      title: string;
      domain: string;
      credibility_score: number;
      source_type: string;
      search_engine: string;
    };
  }>;
  
  let filterType: string = 'All';
  let sortBy: 'credibility' | 'type' | 'relevance' = 'credibility';
  let collapsed = false;
  
  // Count sources by type
  $: sourceCounts = {
    total: sources.length,
    web: sources.filter(s => s.web_source).length,
    local: sources.filter(s => !s.web_source).length,
  };
  
  // Get unique source types for filter
  $: sourceTypes = ['All', ...new Set(sources.map(s => 
    s.web_source ? s.web_source.source_type : 'Local'
  ))];
  
  // Filter sources
  $: filteredSources = sources.filter(source => {
    if (filterType === 'All') return true;
    const type = source.web_source ? source.web_source.source_type : 'Local';
    return type === filterType;
  });
  
  // Sort sources
  $: sortedSources = [...filteredSources].sort((a, b) => {
    switch (sortBy) {
      case 'credibility': {
        const scoreA = a.web_source ? a.web_source.credibility_score : a.relevance;
        const scoreB = b.web_source ? b.web_source.credibility_score : b.relevance;
        return scoreB - scoreA; // Descending
      }
      case 'type': {
        const typeA = a.web_source ? a.web_source.source_type : 'Local';
        const typeB = b.web_source ? b.web_source.source_type : 'Local';
        return typeA.localeCompare(typeB);
      }
      case 'relevance': {
        return b.relevance - a.relevance; // Descending
      }
      default:
        return 0;
    }
  });
</script>

{#if sources.length > 0}
  <div class="sources-panel" class:collapsed>
    <div 
      class="panel-header" 
      role="button"
      tabindex="0"
      on:click={() => collapsed = !collapsed}
      on:keydown={(e) => e.key === 'Enter' && (collapsed = !collapsed)}
    >
      <div class="header-content">
        <h3 class="panel-title">
          📚 Sources ({sourceCounts.total})
        </h3>
        <div class="source-counts">
          {#if sourceCounts.web > 0}
            <span class="count-badge web" title="Web sources">
              🌐 {sourceCounts.web}
            </span>
          {/if}
          {#if sourceCounts.local > 0}
            <span class="count-badge local" title="Local sources">
              📄 {sourceCounts.local}
            </span>
          {/if}
        </div>
      </div>
      
      <button class="collapse-btn" title={collapsed ? 'Expand' : 'Collapse'}>
        {collapsed ? '▼' : '▲'}
      </button>
    </div>
    
    {#if !collapsed}
      <div class="panel-controls">
        <div class="control-group">
          <label for="filter-type">Filter:</label>
          <select id="filter-type" bind:value={filterType}>
            {#each sourceTypes as type}
              <option value={type}>{type}</option>
            {/each}
          </select>
        </div>
        
        <div class="control-group">
          <label for="sort-by">Sort:</label>
          <select id="sort-by" bind:value={sortBy}>
            <option value="credibility">Credibility</option>
            <option value="type">Type</option>
            <option value="relevance">Relevance</option>
          </select>
        </div>
      </div>
      
      <div class="sources-list">
        {#if sortedSources.length === 0}
          <div class="empty-state">
            No sources match the current filter
          </div>
        {:else}
          {#each sortedSources as source (source.chunk_id)}
            <SourceCard {source} />
          {/each}
        {/if}
      </div>
    {/if}
  </div>
{/if}

<style>
  .sources-panel {
    margin-top: 1rem;
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 12px;
    overflow: hidden;
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
  
  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.875rem 1rem;
    background: rgba(96, 165, 250, 0.08);
    border-bottom: 1px solid rgba(96, 165, 250, 0.15);
    cursor: pointer;
    user-select: none;
    transition: background 0.2s;
  }
  
  .panel-header:hover {
    background: rgba(96, 165, 250, 0.12);
  }
  
  .sources-panel.collapsed .panel-header {
    border-bottom: none;
  }
  
  .header-content {
    display: flex;
    align-items: center;
    gap: 1rem;
    flex: 1;
  }
  
  .panel-title {
    margin: 0;
    font-size: 0.9rem;
    font-weight: 600;
    color: #60a5fa;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  
  .source-counts {
    display: flex;
    gap: 0.5rem;
  }
  
  .count-badge {
    padding: 0.25rem 0.5rem;
    border-radius: 12px;
    font-size: 0.7rem;
    font-weight: 600;
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }
  
  .count-badge.web {
    background: rgba(137, 180, 250, 0.15);
    color: #89b4fa;
    border: 1px solid rgba(137, 180, 250, 0.3);
  }
  
  .count-badge.local {
    background: rgba(166, 227, 161, 0.15);
    color: #a6e3a1;
    border: 1px solid rgba(166, 227, 161, 0.3);
  }
  
  .collapse-btn {
    padding: 0.25rem 0.5rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 4px;
    color: #a1a1aa;
    cursor: pointer;
    transition: all 0.2s;
    font-size: 0.75rem;
  }
  
  .collapse-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #e4e4e7;
  }
  
  .panel-controls {
    display: flex;
    gap: 1rem;
    padding: 0.75rem 1rem;
    background: rgba(0, 0, 0, 0.2);
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }
  
  .control-group {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  
  .control-group label {
    font-size: 0.8rem;
    color: #a1a1aa;
    font-weight: 500;
  }
  
  .control-group select {
    padding: 0.375rem 0.75rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    color: #e4e4e7;
    font-size: 0.8rem;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .control-group select:hover {
    background: rgba(255, 255, 255, 0.08);
    border-color: rgba(96, 165, 250, 0.3);
  }
  
  .control-group select:focus {
    outline: none;
    border-color: rgba(96, 165, 250, 0.5);
  }
  
  .sources-list {
    padding: 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    max-height: 500px;
    overflow-y: auto;
  }
  
  .sources-list::-webkit-scrollbar {
    width: 8px;
  }
  
  .sources-list::-webkit-scrollbar-track {
    background: rgba(0, 0, 0, 0.2);
    border-radius: 4px;
  }
  
  .sources-list::-webkit-scrollbar-thumb {
    background: rgba(96, 165, 250, 0.3);
    border-radius: 4px;
  }
  
  .sources-list::-webkit-scrollbar-thumb:hover {
    background: rgba(96, 165, 250, 0.5);
  }
  
  .empty-state {
    padding: 2rem;
    text-align: center;
    color: #71717a;
    font-size: 0.875rem;
  }
</style>
