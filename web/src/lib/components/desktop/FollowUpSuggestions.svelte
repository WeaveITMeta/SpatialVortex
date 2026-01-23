<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  
  export let suggestions: string[] = [];
  
  const dispatch = createEventDispatcher();
  
  function selectSuggestion(suggestion: string) {
    dispatch('select', suggestion);
  }
</script>

{#if suggestions.length > 0}
  <div class="follow-up-suggestions">
    <div class="suggestions-header">
      💡 <span>Follow-up questions:</span>
    </div>
    <div class="suggestions-grid">
      {#each suggestions as suggestion}
        <button 
          class="suggestion-btn"
          on:click={() => selectSuggestion(suggestion)}
        >
          {suggestion}
        </button>
      {/each}
    </div>
  </div>
{/if}

<style>
  .follow-up-suggestions {
    margin-top: 1rem;
    padding: 1rem;
    background: rgba(96, 165, 250, 0.05);
    border-radius: 8px;
    border: 1px solid rgba(96, 165, 250, 0.15);
  }
  
  .suggestions-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
    font-size: 0.875rem;
    color: #89b4fa;
    font-weight: 600;
  }
  
  .suggestions-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 0.5rem;
  }
  
  .suggestion-btn {
    padding: 0.75rem 1rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    color: #d4d4d8;
    font-size: 0.85rem;
    text-align: left;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .suggestion-btn:hover {
    background: rgba(96, 165, 250, 0.15);
    border-color: rgba(96, 165, 250, 0.3);
    color: #e4e4e7;
    transform: translateY(-2px);
  }
  
  .suggestion-btn:active {
    transform: translateY(0);
  }
</style>
