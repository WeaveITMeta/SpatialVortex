<script lang="ts">
  export let freshnessScore: number; // 0.0 to 1.0
  export let publishedDate: string | undefined = undefined;
  
  // Determine freshness level and appearance
  $: freshnessInfo = getFreshnessInfo(freshnessScore);
  
  // Format date for display
  $: formattedDate = publishedDate ? formatDate(publishedDate) : null;
  
  function getFreshnessInfo(score: number) {
    if (score >= 0.9) {
      return {
        color: '#a6e3a1', // Green
        bg: 'rgba(166, 227, 161, 0.15)',
        label: 'Very Fresh',
        icon: '🔥',
        description: 'Recent content (< 1 month)'
      };
    } else if (score >= 0.7) {
      return {
        color: '#89b4fa', // Blue
        bg: 'rgba(137, 180, 250, 0.15)',
        label: 'Fresh',
        icon: '✨',
        description: 'Recent content (< 3 months)'
      };
    } else if (score >= 0.5) {
      return {
        color: '#f9e2af', // Yellow
        bg: 'rgba(249, 226, 175, 0.15)',
        label: 'Moderate',
        icon: '📅',
        description: 'Content from this year'
      };
    } else if (score >= 0.3) {
      return {
        color: '#fab387', // Orange
        bg: 'rgba(250, 179, 135, 0.15)',
        label: 'Aging',
        icon: '📆',
        description: 'Older content'
      };
    } else {
      return {
        color: '#f38ba8', // Red
        bg: 'rgba(243, 139, 168, 0.15)',
        label: 'Old',
        icon: '🕰️',
        description: 'Dated content (> 1 year)'
      };
    }
  }
  
  function formatDate(dateStr: string): string {
    try {
      const date = new Date(dateStr);
      const now = new Date();
      const diffDays = Math.floor((now.getTime() - date.getTime()) / (1000 * 60 * 60 * 24));
      
      if (diffDays === 0) return 'Today';
      if (diffDays === 1) return 'Yesterday';
      if (diffDays < 7) return `${diffDays} days ago`;
      if (diffDays < 30) return `${Math.floor(diffDays / 7)} weeks ago`;
      if (diffDays < 365) return `${Math.floor(diffDays / 30)} months ago`;
      return `${Math.floor(diffDays / 365)} years ago`;
    } catch {
      return dateStr;
    }
  }
</script>

<div 
  class="freshness-badge"
  style="
    color: {freshnessInfo.color};
    background: {freshnessInfo.bg};
    border-color: {freshnessInfo.color};
  "
  title="{freshnessInfo.description}{formattedDate ? ` • Published ${formattedDate}` : ''}"
>
  <span class="icon">{freshnessInfo.icon}</span>
  <span class="label">{freshnessInfo.label}</span>
</div>

<style>
  .freshness-badge {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.2rem 0.5rem;
    border-radius: 12px;
    border: 1px solid;
    font-size: 0.7rem;
    font-weight: 600;
    transition: all 0.2s;
    cursor: help;
    white-space: nowrap;
  }
  
  .freshness-badge:hover {
    transform: scale(1.05);
  }
  
  .icon {
    font-size: 0.8rem;
    line-height: 1;
  }
  
  .label {
    line-height: 1;
  }
</style>
