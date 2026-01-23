<script lang="ts">
  import { onMount } from 'svelte';
  
  export let leftMinWidth = 300;
  export let rightMinWidth = 400;
  export let initialSplit = 50; // percentage
  
  let container: HTMLDivElement;
  let divider: HTMLDivElement;
  let isDragging = false;
  let splitPercent = initialSplit;
  
  function startDrag(e: MouseEvent) {
    isDragging = true;
    e.preventDefault();
  }
  
  function drag(e: MouseEvent) {
    if (!isDragging || !container) return;
    
    const containerRect = container.getBoundingClientRect();
    const offsetX = e.clientX - containerRect.left;
    const percent = (offsetX / containerRect.width) * 100;
    
    // Enforce minimum widths
    const leftPx = (percent / 100) * containerRect.width;
    const rightPx = containerRect.width - leftPx;
    
    if (leftPx >= leftMinWidth && rightPx >= rightMinWidth) {
      splitPercent = Math.max(20, Math.min(80, percent));
    }
  }
  
  function stopDrag() {
    isDragging = false;
  }
  
  onMount(() => {
    document.addEventListener('mousemove', drag);
    document.addEventListener('mouseup', stopDrag);
    
    return () => {
      document.removeEventListener('mousemove', drag);
      document.removeEventListener('mouseup', stopDrag);
    };
  });
</script>

<div class="split-pane" bind:this={container}>
  <div class="pane left-pane" style="width: {splitPercent}%">
    <slot name="left" />
  </div>
  
  <div 
    class="divider" 
    class:dragging={isDragging}
    bind:this={divider}
    on:mousedown={startDrag}
    role="separator"
    aria-orientation="vertical"
    tabindex="0"
  >
    <div class="divider-handle">
      <div class="handle-bar"></div>
    </div>
  </div>
  
  <div class="pane right-pane" style="width: {100 - splitPercent}%">
    <slot name="right" />
  </div>
</div>

<style>
  .split-pane {
    display: flex;
    width: 100%;
    height: 100%;
    overflow: hidden;
    position: relative;
  }
  
  .pane {
    height: 100%;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
  
  .left-pane {
    border-right: 1px solid rgba(255, 255, 255, 0.1);
  }
  
  .divider {
    width: 8px;
    height: 100%;
    background: transparent;
    cursor: col-resize;
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.2s;
    flex-shrink: 0;
  }
  
  .divider:hover,
  .divider.dragging {
    background: rgba(96, 165, 250, 0.1);
  }
  
  .divider-handle {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  
  .handle-bar {
    width: 2px;
    height: 60px;
    background: rgba(96, 165, 250, 0.5);
    border-radius: 2px;
    transition: all 0.2s;
  }
  
  .divider:hover .handle-bar,
  .divider.dragging .handle-bar {
    background: #60a5fa;
    height: 80px;
    width: 3px;
  }
  
  .divider:active {
    cursor: col-resize;
  }
  
  /* Prevent text selection while dragging */
  .dragging {
    user-select: none;
  }
  
  .split-pane:has(.divider.dragging) {
    user-select: none;
  }
</style>
