<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  
  export let rating: number | undefined = undefined;
  export let readonly: boolean = false;
  export let size: 'small' | 'medium' | 'large' = 'medium';
  
  const dispatch = createEventDispatcher();
  
  let hoverRating: number | null = null;
  
  $: displayRating = hoverRating !== null ? hoverRating : rating || 0;
  $: sizeClass = `size-${size}`;
  
  function handleClick(star: number) {
    if (readonly) return;
    dispatch('rate', star);
  }
  
  function handleMouseEnter(star: number) {
    if (readonly) return;
    hoverRating = star;
  }
  
  function handleMouseLeave() {
    if (readonly) return;
    hoverRating = null;
  }
  
  function getStarIcon(position: number): string {
    if (displayRating >= position) {
      return '★'; // Filled star
    } else if (displayRating >= position - 0.5) {
      return '⯪'; // Half star
    } else {
      return '☆'; // Empty star
    }
  }
</script>

<div 
  class="star-rating {sizeClass}" 
  class:readonly
  class:interactive={!readonly}
>
  {#each [1, 2, 3, 4, 5] as star}
    <button
      class="star"
      class:filled={displayRating >= star}
      class:half={displayRating >= star - 0.5 && displayRating < star}
      disabled={readonly}
      on:click={() => handleClick(star)}
      on:mouseenter={() => handleMouseEnter(star)}
      on:mouseleave={handleMouseLeave}
      title={readonly ? `${rating || 0} stars` : `Rate ${star} stars`}
    >
      {getStarIcon(star)}
    </button>
  {/each}
  
  {#if rating !== undefined}
    <span class="rating-text">{rating.toFixed(1)}</span>
  {/if}
</div>

<style>
  .star-rating {
    display: inline-flex;
    align-items: center;
    gap: 0.15rem;
  }
  
  .star {
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    color: #71717a;
    transition: all 0.2s;
    line-height: 1;
  }
  
  .star:hover:not(:disabled) {
    transform: scale(1.15);
  }
  
  .star:disabled {
    cursor: default;
  }
  
  .star.filled {
    color: #f9e2af;
    text-shadow: 0 0 4px rgba(249, 226, 175, 0.5);
  }
  
  .star.half {
    color: #f9e2af;
    opacity: 0.7;
  }
  
  .interactive .star:hover:not(:disabled) {
    color: #f9e2af;
  }
  
  /* Size variants */
  .size-small .star {
    font-size: 0.875rem;
  }
  
  .size-medium .star {
    font-size: 1rem;
  }
  
  .size-large .star {
    font-size: 1.25rem;
  }
  
  .rating-text {
    margin-left: 0.375rem;
    font-size: 0.75rem;
    color: #a1a1aa;
    font-weight: 600;
  }
  
  .size-small .rating-text {
    font-size: 0.65rem;
  }
  
  .size-large .rating-text {
    font-size: 0.85rem;
  }
</style>
