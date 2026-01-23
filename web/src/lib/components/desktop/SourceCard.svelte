<script lang="ts">
  import CredibilityBadge from './CredibilityBadge.svelte';
  import FreshnessBadge from './FreshnessBadge.svelte';
  import StarRating from './StarRating.svelte';
  
  export let source: {
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
      published_date?: string;
      freshness_score: number;
      user_rating?: number;
      is_bookmarked: boolean;
    };
  };
  
  let expanded = false;
  
  $: isWeb = source.web_source !== undefined && source.web_source !== null;
  $: credibility = isWeb ? source.web_source!.credibility_score : source.relevance;
  $: title = isWeb ? source.web_source!.title : `Document: ${source.doc_id}`;
  $: subtitle = isWeb ? source.web_source!.domain : `Chunk: ${source.chunk_id}`;
  
  // Get source type icon
  $: sourceIcon = getSourceIcon(isWeb ? source.web_source?.source_type : 'Local');
  
  function getSourceIcon(type?: string): string {
    if (!type) return '📄';
    
    const iconMap: Record<string, string> = {
      'Academic': '🎓',
      'Government': '🏛️',
      'Wikipedia': '📖',
      'Technical': '💻',
      'News': '📰',
      'Reference': '📚',
      'Commercial': '🌐',
      'Local': '📄',
      'Unknown': '❓'
    };
    
    return iconMap[type] || '📄';
  }
  
  function copyUrl() {
    if (isWeb && source.web_source) {
      navigator.clipboard.writeText(source.web_source.url);
    }
  }
  
  function openInNewTab() {
    if (isWeb && source.web_source) {
      window.open(source.web_source.url, '_blank');
    }
  }
  
  async function handleRating(event: CustomEvent<number>) {
    if (!isWeb || !source.web_source) return;
    
    const rating = event.detail;
    
    try {
      const response = await fetch('http://localhost:7000/api/v1/sources/rate', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          url: source.web_source.url,
          rating
        })
      });
      
      if (response.ok) {
        // Update local state
        if (source.web_source) {
          source.web_source.user_rating = rating;
        }
      }
    } catch (error) {
      console.error('Failed to rate source:', error);
    }
  }
  
  async function toggleBookmark() {
    if (!isWeb || !source.web_source) return;
    
    const newBookmarked = !source.web_source.is_bookmarked;
    
    try {
      const response = await fetch('http://localhost:7000/api/v1/sources/bookmark', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          url: source.web_source.url,
          bookmarked: newBookmarked
        })
      });
      
      if (response.ok) {
        // Update local state
        if (source.web_source) {
          source.web_source.is_bookmarked = newBookmarked;
        }
      }
    } catch (error) {
      console.error('Failed to bookmark source:', error);
    }
  }
</script>

<div class="source-card" class:expanded>
  <div 
    class="source-header" 
    role="button"
    tabindex="0"
    on:click={() => expanded = !expanded}
    on:keydown={(e) => e.key === 'Enter' && (expanded = !expanded)}
  >
    <div class="source-info">
      <div class="source-title">
        <span class="icon">{sourceIcon}</span>
        <span class="title-text" title={title}>{title}</span>
        <CredibilityBadge score={credibility} />
        {#if isWeb && source.web_source}
          <FreshnessBadge 
            freshnessScore={source.web_source.freshness_score} 
            publishedDate={source.web_source.published_date}
          />
        {/if}
      </div>
      <div class="source-subtitle">
        {subtitle}
        {#if isWeb && source.web_source}
          <span class="metadata">
            {source.web_source.source_type} • {source.web_source.search_engine}
          </span>
        {/if}
      </div>
    </div>
    
    {#if isWeb && source.web_source}
      <button 
        class="bookmark-btn" 
        class:bookmarked={source.web_source.is_bookmarked}
        on:click|stopPropagation={toggleBookmark}
        title={source.web_source.is_bookmarked ? 'Remove bookmark' : 'Bookmark this source'}
      >
        {source.web_source.is_bookmarked ? '🔖' : '🏷️'}
      </button>
    {/if}
    
    <button class="expand-btn" title={expanded ? 'Collapse' : 'Expand'}>
      {expanded ? '▲' : '▼'}
    </button>
  </div>
  
  {#if expanded}
    <div class="source-content">
      <div class="snippet">
        {source.content_snippet}
      </div>
      
      {#if isWeb && source.web_source}
        <div class="source-rating">
          <span class="rating-label">Rate this source:</span>
          <StarRating 
            rating={source.web_source.user_rating} 
            on:rate={handleRating}
            size="medium"
          />
        </div>
        
        <div class="source-actions">
          <button class="action-btn" on:click={copyUrl} title="Copy URL">
            📋 Copy URL
          </button>
          <button class="action-btn" on:click={openInNewTab} title="Open in new tab">
            🔗 Open
          </button>
        </div>
        
        <div class="source-url">
          <a href={source.web_source.url} target="_blank" rel="noopener noreferrer">
            {source.web_source.url}
          </a>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .source-card {
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 8px;
    overflow: hidden;
    transition: all 0.2s;
  }
  
  .source-card:hover {
    border-color: rgba(96, 165, 250, 0.3);
    background: rgba(255, 255, 255, 0.05);
  }
  
  .source-card.expanded {
    border-color: rgba(96, 165, 250, 0.4);
  }
  
  .source-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem;
    cursor: pointer;
    user-select: none;
  }
  
  .source-info {
    flex: 1;
    min-width: 0;
  }
  
  .source-title {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.25rem;
    flex-wrap: wrap;
  }
  
  .icon {
    font-size: 1rem;
    flex-shrink: 0;
  }
  
  .title-text {
    color: #e4e4e7;
    font-weight: 500;
    font-size: 0.875rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    min-width: 0;
  }
  
  .source-subtitle {
    color: #a1a1aa;
    font-size: 0.75rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
  }
  
  .metadata {
    color: #71717a;
    font-size: 0.7rem;
  }
  
  .expand-btn {
    padding: 0.25rem 0.5rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 4px;
    color: #a1a1aa;
    cursor: pointer;
    transition: all 0.2s;
    font-size: 0.75rem;
    flex-shrink: 0;
  }
  
  .expand-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #e4e4e7;
  }
  
  .source-content {
    padding: 0 0.75rem 0.75rem;
    animation: slideDown 0.2s ease-out;
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
  
  .snippet {
    padding: 0.75rem;
    background: rgba(0, 0, 0, 0.2);
    border-radius: 6px;
    color: #d4d4d8;
    font-size: 0.85rem;
    line-height: 1.5;
    margin-bottom: 0.5rem;
    border-left: 3px solid #60a5fa;
  }
  
  .source-actions {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
  }
  
  .action-btn {
    padding: 0.375rem 0.75rem;
    background: rgba(96, 165, 250, 0.1);
    border: 1px solid rgba(96, 165, 250, 0.2);
    border-radius: 6px;
    color: #60a5fa;
    cursor: pointer;
    transition: all 0.2s;
    font-size: 0.8rem;
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }
  
  .action-btn:hover {
    background: rgba(96, 165, 250, 0.2);
    border-color: rgba(96, 165, 250, 0.4);
  }
  
  .source-url {
    font-size: 0.75rem;
    color: #71717a;
    word-break: break-all;
  }
  
  .source-url a {
    color: #89b4fa;
    text-decoration: none;
    transition: color 0.2s;
  }
  
  .source-url a:hover {
    color: #60a5fa;
    text-decoration: underline;
  }
  
  /* Bookmark button */
  .bookmark-btn {
    padding: 0.25rem 0.5rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 4px;
    color: #a1a1aa;
    cursor: pointer;
    transition: all 0.2s;
    font-size: 1rem;
    flex-shrink: 0;
  }
  
  .bookmark-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    transform: scale(1.1);
  }
  
  .bookmark-btn.bookmarked {
    background: rgba(249, 226, 175, 0.15);
    border-color: rgba(249, 226, 175, 0.3);
    color: #f9e2af;
  }
  
  /* Rating section */
  .source-rating {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem;
    background: rgba(96, 165, 250, 0.05);
    border-radius: 6px;
    margin-bottom: 0.5rem;
    border: 1px solid rgba(96, 165, 250, 0.15);
  }
  
  .rating-label {
    font-size: 0.85rem;
    color: #a1a1aa;
    font-weight: 500;
  }
</style>
