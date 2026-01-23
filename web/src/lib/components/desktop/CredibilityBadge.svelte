<script lang="ts">
  export let score: number; // 0.0 to 1.0
  
  // Convert to percentage
  $: percentage = Math.round(score * 100);
  
  // Determine color and label based on score
  $: badgeInfo = getBadgeInfo(score);
  
  function getBadgeInfo(score: number) {
    if (score >= 0.9) {
      return {
        color: '#a6e3a1', // Green
        bg: 'rgba(166, 227, 161, 0.15)',
        label: 'High',
        icon: '⭐'
      };
    } else if (score >= 0.75) {
      return {
        color: '#89b4fa', // Blue
        bg: 'rgba(137, 180, 250, 0.15)',
        label: 'Good',
        icon: '✓'
      };
    } else if (score >= 0.6) {
      return {
        color: '#f9e2af', // Yellow
        bg: 'rgba(249, 226, 175, 0.15)',
        label: 'Medium',
        icon: '•'
      };
    } else if (score >= 0.4) {
      return {
        color: '#fab387', // Orange
        bg: 'rgba(250, 179, 135, 0.15)',
        label: 'Low',
        icon: '⚠'
      };
    } else {
      return {
        color: '#f38ba8', // Red
        bg: 'rgba(243, 139, 168, 0.15)',
        label: 'Poor',
        icon: '⚠'
      };
    }
  }
</script>

<div 
  class="credibility-badge"
  style="
    color: {badgeInfo.color};
    background: {badgeInfo.bg};
    border-color: {badgeInfo.color};
  "
  title="{percentage}% credible - {badgeInfo.label} credibility"
>
  <span class="icon">{badgeInfo.icon}</span>
  <span class="percentage">{percentage}%</span>
</div>

<style>
  .credibility-badge {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.25rem 0.5rem;
    border-radius: 12px;
    border: 1px solid;
    font-size: 0.75rem;
    font-weight: 600;
    transition: all 0.2s;
    cursor: help;
  }
  
  .credibility-badge:hover {
    transform: scale(1.05);
  }
  
  .icon {
    font-size: 0.875rem;
    line-height: 1;
  }
  
  .percentage {
    line-height: 1;
  }
</style>
