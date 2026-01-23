<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  
  export let x = 0;
  export let y = 0;
  export let show = false;
  export let sessionTitle = '';
  
  const dispatch = createEventDispatcher();
  
  function handleRename() {
    dispatch('rename');
    show = false;
  }
  
  function handleShare() {
    dispatch('share');
    show = false;
  }
  
  function handleDelete() {
    dispatch('delete');
    show = false;
  }
  
  function handleClickOutside() {
    if (show) {
      show = false;
    }
  }
</script>

<svelte:window on:click={handleClickOutside} />

{#if show}
  <div class="context-menu" style="left: {x}px; top: {y}px;">
    <div class="menu-header">{sessionTitle}</div>
    
    <button class="menu-item" on:click={handleRename}>
      <span class="menu-icon">✏️</span>
      <span>Rename</span>
    </button>
    
    <button class="menu-item" on:click={handleShare}>
      <span class="menu-icon">📤</span>
      <span>Share</span>
    </button>
    
    <div class="menu-divider"></div>
    
    <button class="menu-item danger" on:click={handleDelete}>
      <span class="menu-icon">🗑️</span>
      <span>Delete</span>
    </button>
  </div>
{/if}

<style>
  .context-menu {
    position: fixed;
    background: #1e1e2e;
    border: 1px solid #313244;
    border-radius: 8px;
    padding: 0.5rem 0;
    min-width: 180px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
    z-index: 1000;
    animation: slideIn 0.15s ease-out;
  }
  
  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateY(-8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
  
  .menu-header {
    padding: 0.5rem 1rem;
    font-size: 0.75rem;
    color: #6c7086;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 200px;
  }
  
  .menu-item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.625rem 1rem;
    background: transparent;
    border: none;
    color: #cdd6f4;
    font-size: 0.875rem;
    cursor: pointer;
    transition: all 0.15s ease;
    text-align: left;
  }
  
  .menu-item:hover {
    background: #313244;
    color: #e4e4e7;
  }
  
  .menu-item.danger {
    color: #f38ba8;
  }
  
  .menu-item.danger:hover {
    background: rgba(243, 139, 168, 0.1);
    color: #f38ba8;
  }
  
  .menu-icon {
    font-size: 1rem;
    flex-shrink: 0;
  }
  
  .menu-divider {
    height: 1px;
    background: #313244;
    margin: 0.5rem 0;
  }
</style>
