<script lang="ts">
  import ChatPanel from './ChatPanel.svelte';
  import SplitPane from './SplitPane.svelte';
  import CanvasWorkspace from './CanvasWorkspace.svelte';
  import type { ChatMessage } from '$lib/types/chat';
  
  let messages: ChatMessage[] = [];
  let isGenerating = false;
  let canvasVisible = false;
  let canvasContent = '';
  let canvasLanguage = 'typescript';
  let canvasFileName = 'untitled.ts';
  let canvasRef: CanvasWorkspace;
  
  interface CanvasRequest {
    action: 'create' | 'update' | 'append';
    content: string;
    language?: string;
    fileName?: string;
    description?: string;
  }
  
  // Listen for canvas commands from AI responses
  function handleCanvasCommand(command: CanvasRequest) {
    canvasVisible = true;
    
    switch (command.action) {
      case 'create':
        canvasContent = command.content;
        if (command.language) canvasLanguage = command.language;
        if (command.fileName) canvasFileName = command.fileName;
        break;
      
      case 'update':
        if (canvasRef) {
          canvasRef.setContent(command.content);
        } else {
          canvasContent = command.content;
        }
        break;
      
      case 'append':
        if (canvasRef) {
          const current = canvasRef.getContent();
          canvasRef.setContent(current + '\n\n' + command.content);
        } else {
          canvasContent += '\n\n' + command.content;
        }
        break;
    }
  }
  
  function toggleCanvas() {
    canvasVisible = !canvasVisible;
  }
  
  function closeCanvas() {
    canvasVisible = false;
  }
  
  // Expose for external access
  export function openCanvas(content: string, language = 'typescript', fileName = 'untitled.ts') {
    canvasContent = content;
    canvasLanguage = language;
    canvasFileName = fileName;
    canvasVisible = true;
  }
  
  export function updateCanvas(content: string) {
    if (canvasRef) {
      canvasRef.setContent(content);
    }
  }
</script>

<div class="canvas-page">
  {#if canvasVisible}
    <SplitPane initialSplit={50} leftMinWidth={400} rightMinWidth={400}>
      <div slot="left" class="chat-section">
        <ChatPanel bind:messages bind:isGenerating />
      </div>
      
      <div slot="right" class="canvas-section">
        <div class="canvas-header">
          <div class="canvas-title">
            <span class="canvas-icon">🎨</span>
            <span>Canvas Workspace</span>
          </div>
          <button class="close-canvas-btn" on:click={closeCanvas} title="Close Canvas">
            ✕
          </button>
        </div>
        
        <CanvasWorkspace 
          bind:this={canvasRef}
          initialContent={canvasContent}
          language={canvasLanguage}
          fileName={canvasFileName}
        />
      </div>
    </SplitPane>
  {:else}
    <div class="full-chat">
      <ChatPanel bind:messages bind:isGenerating />
      
      <!-- Floating Canvas Toggle Button -->
      <button class="canvas-toggle-fab" on:click={toggleCanvas} title="Open Canvas">
        🎨 Canvas
      </button>
    </div>
  {/if}
</div>

<style>
  .canvas-page {
    width: 100%;
    height: 100vh;
    overflow: hidden;
    background: #18181b;
  }
  
  .chat-section,
  .canvas-section {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
  }
  
  .canvas-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    background: #252526;
    border-bottom: 1px solid #3e3e42;
  }
  
  .canvas-title {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 1.1rem;
    font-weight: 600;
    color: #d4d4d4;
  }
  
  .canvas-icon {
    font-size: 1.5rem;
  }
  
  .close-canvas-btn {
    width: 2.5rem;
    height: 2.5rem;
    background: transparent;
    border: none;
    border-radius: 6px;
    color: #d4d4d4;
    font-size: 1.25rem;
    cursor: pointer;
    transition: background 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  
  .close-canvas-btn:hover {
    background: rgba(255, 255, 255, 0.1);
  }
  
  .full-chat {
    width: 100%;
    height: 100%;
    position: relative;
  }
  
  .canvas-toggle-fab {
    position: fixed;
    bottom: 2rem;
    right: 2rem;
    background: linear-gradient(135deg, #60a5fa 0%, #3b82f6 100%);
    border: none;
    border-radius: 50px;
    padding: 1rem 2rem;
    color: white;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    box-shadow: 0 8px 24px rgba(96, 165, 250, 0.4);
    transition: all 0.3s;
    z-index: 100;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  
  .canvas-toggle-fab:hover {
    transform: translateY(-4px);
    box-shadow: 0 12px 32px rgba(96, 165, 250, 0.5);
  }
  
  .canvas-toggle-fab:active {
    transform: translateY(-2px);
  }
</style>
