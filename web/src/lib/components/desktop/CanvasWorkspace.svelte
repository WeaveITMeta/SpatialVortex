<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import * as monaco from 'monaco-editor';
  
  export let initialContent: string = '';
  export let language: string = 'typescript';
  export let fileName: string = 'untitled.ts';
  export let readOnly: boolean = false;
  
  let editorContainer: HTMLDivElement;
  let editor: monaco.editor.IStandaloneCodeEditor | null = null;
  let currentVersion = 1;
  let versions: CanvasVersion[] = [];
  let showHistory = false;
  let showDiff = false;
  let diffEditor: monaco.editor.IStandaloneDiffEditor | null = null;
  let diffContainer: HTMLDivElement;
  
  interface CanvasVersion {
    id: number;
    content: string;
    timestamp: Date;
    description: string;
  }
  
  // Detect language from filename
  $: detectedLanguage = detectLanguage(fileName);
  
  function detectLanguage(name: string): string {
    const ext = name.split('.').pop()?.toLowerCase();
    const langMap: Record<string, string> = {
      'js': 'javascript',
      'ts': 'typescript',
      'jsx': 'javascript',
      'tsx': 'typescript',
      'py': 'python',
      'rs': 'rust',
      'go': 'go',
      'java': 'java',
      'html': 'html',
      'css': 'css',
      'json': 'json',
      'md': 'markdown',
      'yaml': 'yaml',
      'yml': 'yaml',
      'sh': 'shell',
      'sql': 'sql',
    };
    return langMap[ext || ''] || 'plaintext';
  }
  
  onMount(() => {
    // Initialize Monaco Editor
    if (editorContainer) {
      editor = monaco.editor.create(editorContainer, {
        value: initialContent,
        language: detectedLanguage,
        theme: 'vs-dark',
        automaticLayout: true,
        minimap: { enabled: true },
        fontSize: 14,
        lineNumbers: 'on',
        readOnly,
        scrollBeyondLastLine: false,
        wordWrap: 'on',
        tabSize: 2,
      });
      
      // Save initial version
      saveVersion('Initial version');
      
      // Auto-save on content change (debounced)
      let saveTimeout: NodeJS.Timeout;
      editor.onDidChangeModelContent(() => {
        clearTimeout(saveTimeout);
        saveTimeout = setTimeout(() => {
          // Auto-save could trigger here
        }, 1000);
      });
    }
  });
  
  onDestroy(() => {
    editor?.dispose();
    diffEditor?.dispose();
  });
  
  function saveVersion(description: string) {
    if (!editor) return;
    
    const content = editor.getValue();
    currentVersion++;
    
    versions = [...versions, {
      id: currentVersion,
      content,
      timestamp: new Date(),
      description,
    }];
  }
  
  function restoreVersion(version: CanvasVersion) {
    if (!editor) return;
    editor.setValue(version.content);
    showHistory = false;
  }
  
  function showDiffView(version: CanvasVersion) {
    if (!editor || !diffContainer) return;
    
    showDiff = true;
    showHistory = false;
    
    setTimeout(() => {
      const currentContent = editor!.getValue();
      
      diffEditor = monaco.editor.createDiffEditor(diffContainer, {
        theme: 'vs-dark',
        automaticLayout: true,
        readOnly: true,
      });
      
      const originalModel = monaco.editor.createModel(version.content, detectedLanguage);
      const modifiedModel = monaco.editor.createModel(currentContent, detectedLanguage);
      
      diffEditor.setModel({
        original: originalModel,
        modified: modifiedModel,
      });
    }, 100);
  }
  
  function closeDiff() {
    diffEditor?.dispose();
    diffEditor = null;
    showDiff = false;
  }
  
  function exportContent() {
    if (!editor) return;
    
    const content = editor.getValue();
    const blob = new Blob([content], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = fileName;
    a.click();
    URL.revokeObjectURL(url);
  }
  
  function copyToClipboard() {
    if (!editor) return;
    
    const content = editor.getValue();
    navigator.clipboard.writeText(content).then(() => {
      alert('Copied to clipboard!');
    });
  }
  
  function formatCode() {
    if (!editor) return;
    editor.getAction('editor.action.formatDocument')?.run();
  }
  
  function changeLanguage(newLang: string) {
    if (!editor) return;
    const model = editor.getModel();
    if (model) {
      monaco.editor.setModelLanguage(model, newLang);
    }
  }
  
  // Expose content getter for parent
  export function getContent(): string {
    return editor?.getValue() || '';
  }
  
  export function setContent(content: string) {
    editor?.setValue(content);
  }
</script>

<div class="canvas-workspace">
  <!-- Toolbar -->
  <div class="canvas-toolbar">
    <div class="toolbar-left">
      <input 
        type="text" 
        bind:value={fileName} 
        class="file-name-input"
        placeholder="filename.ext"
      />
      <select 
        value={detectedLanguage} 
        on:change={(e) => changeLanguage(e.currentTarget.value)}
        class="language-select"
      >
        <option value="javascript">JavaScript</option>
        <option value="typescript">TypeScript</option>
        <option value="python">Python</option>
        <option value="rust">Rust</option>
        <option value="go">Go</option>
        <option value="java">Java</option>
        <option value="html">HTML</option>
        <option value="css">CSS</option>
        <option value="json">JSON</option>
        <option value="markdown">Markdown</option>
        <option value="yaml">YAML</option>
        <option value="shell">Shell</option>
      </select>
    </div>
    
    <div class="toolbar-right">
      <button class="toolbar-btn" on:click={formatCode} title="Format Code">
        ✨ Format
      </button>
      <button class="toolbar-btn" on:click={() => showHistory = !showHistory} title="Version History">
        🕐 History ({versions.length})
      </button>
      <button class="toolbar-btn" on:click={() => saveVersion('Manual save')} title="Save Version">
        💾 Save
      </button>
      <button class="toolbar-btn" on:click={copyToClipboard} title="Copy to Clipboard">
        📋 Copy
      </button>
      <button class="toolbar-btn" on:click={exportContent} title="Download File">
        📥 Download
      </button>
    </div>
  </div>
  
  <!-- Editor Area -->
  <div class="editor-area">
    {#if !showDiff}
      <div class="editor-container" bind:this={editorContainer}></div>
    {:else}
      <div class="diff-container">
        <div class="diff-header">
          <h3>Comparing Changes</h3>
          <button class="close-diff-btn" on:click={closeDiff}>✕ Close Diff</button>
        </div>
        <div class="diff-editor" bind:this={diffContainer}></div>
      </div>
    {/if}
    
    <!-- Version History Sidebar -->
    {#if showHistory}
      <div class="history-sidebar">
        <div class="history-header">
          <h3>📜 Version History</h3>
          <button class="close-btn" on:click={() => showHistory = false}>✕</button>
        </div>
        
        <div class="history-list">
          {#each versions.slice().reverse() as version}
            <div class="history-item">
              <div class="history-info">
                <div class="version-id">v{version.id}</div>
                <div class="version-time">
                  {version.timestamp.toLocaleTimeString()}
                </div>
                <div class="version-desc">{version.description}</div>
              </div>
              <div class="history-actions">
                <button 
                  class="history-btn restore-btn" 
                  on:click={() => restoreVersion(version)}
                  title="Restore this version"
                >
                  ↺
                </button>
                <button 
                  class="history-btn diff-btn" 
                  on:click={() => showDiffView(version)}
                  title="View diff"
                >
                  ⎄
                </button>
              </div>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </div>
  
  <!-- Status Bar -->
  <div class="canvas-status">
    <div class="status-left">
      <span class="status-item">Lines: {editor?.getModel()?.getLineCount() || 0}</span>
      <span class="status-item">Language: {detectedLanguage}</span>
      <span class="status-item">Versions: {versions.length}</span>
    </div>
    <div class="status-right">
      <span class="status-item">Ln {editor?.getPosition()?.lineNumber || 1}, Col {editor?.getPosition()?.column || 1}</span>
    </div>
  </div>
</div>

<style>
  .canvas-workspace {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #1e1e1e;
    color: #d4d4d4;
  }
  
  .canvas-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    background: #252526;
    border-bottom: 1px solid #3e3e42;
  }
  
  .toolbar-left {
    display: flex;
    gap: 0.75rem;
    align-items: center;
  }
  
  .file-name-input {
    background: #3c3c3c;
    border: 1px solid #5a5a5a;
    border-radius: 4px;
    padding: 0.5rem 0.75rem;
    color: #d4d4d4;
    font-size: 0.9rem;
    font-family: monospace;
    min-width: 200px;
  }
  
  .file-name-input:focus {
    outline: none;
    border-color: #007acc;
  }
  
  .language-select {
    background: #3c3c3c;
    border: 1px solid #5a5a5a;
    border-radius: 4px;
    padding: 0.5rem 0.75rem;
    color: #d4d4d4;
    font-size: 0.9rem;
    cursor: pointer;
  }
  
  .language-select:focus {
    outline: none;
    border-color: #007acc;
  }
  
  .toolbar-right {
    display: flex;
    gap: 0.5rem;
  }
  
  .toolbar-btn {
    background: #0e639c;
    border: none;
    border-radius: 4px;
    padding: 0.5rem 1rem;
    color: white;
    font-size: 0.9rem;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.2s;
  }
  
  .toolbar-btn:hover {
    background: #1177bb;
  }
  
  .editor-area {
    flex: 1;
    display: flex;
    position: relative;
    overflow: hidden;
  }
  
  .editor-container {
    width: 100%;
    height: 100%;
  }
  
  .diff-container {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
  }
  
  .diff-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    background: #252526;
    border-bottom: 1px solid #3e3e42;
  }
  
  .diff-header h3 {
    margin: 0;
    font-size: 1rem;
    color: #d4d4d4;
  }
  
  .close-diff-btn {
    background: #3c3c3c;
    border: none;
    border-radius: 4px;
    padding: 0.5rem 1rem;
    color: #d4d4d4;
    cursor: pointer;
    transition: background 0.2s;
  }
  
  .close-diff-btn:hover {
    background: #505050;
  }
  
  .diff-editor {
    flex: 1;
    height: 100%;
  }
  
  .history-sidebar {
    position: absolute;
    right: 0;
    top: 0;
    bottom: 0;
    width: 320px;
    background: #252526;
    border-left: 1px solid #3e3e42;
    display: flex;
    flex-direction: column;
    animation: slideIn 0.3s ease-out;
  }
  
  @keyframes slideIn {
    from {
      transform: translateX(100%);
    }
    to {
      transform: translateX(0);
    }
  }
  
  .history-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    border-bottom: 1px solid #3e3e42;
  }
  
  .history-header h3 {
    margin: 0;
    font-size: 1rem;
    color: #d4d4d4;
  }
  
  .close-btn {
    background: transparent;
    border: none;
    color: #d4d4d4;
    font-size: 1.25rem;
    cursor: pointer;
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    transition: background 0.2s;
  }
  
  .close-btn:hover {
    background: #3c3c3c;
  }
  
  .history-list {
    flex: 1;
    overflow-y: auto;
    padding: 0.5rem;
  }
  
  .history-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem;
    background: #2d2d30;
    border-radius: 6px;
    margin-bottom: 0.5rem;
    transition: background 0.2s;
  }
  
  .history-item:hover {
    background: #37373d;
  }
  
  .history-info {
    flex: 1;
  }
  
  .version-id {
    font-weight: 600;
    color: #4fc3f7;
    font-size: 0.9rem;
    margin-bottom: 0.25rem;
  }
  
  .version-time {
    font-size: 0.75rem;
    color: #858585;
    margin-bottom: 0.25rem;
  }
  
  .version-desc {
    font-size: 0.85rem;
    color: #cccccc;
  }
  
  .history-actions {
    display: flex;
    gap: 0.5rem;
  }
  
  .history-btn {
    width: 2rem;
    height: 2rem;
    background: #3c3c3c;
    border: none;
    border-radius: 4px;
    color: #d4d4d4;
    font-size: 1rem;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .history-btn:hover {
    background: #505050;
  }
  
  .restore-btn:hover {
    background: #0e639c;
    color: white;
  }
  
  .diff-btn:hover {
    background: #f57c00;
    color: white;
  }
  
  .canvas-status {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 1rem;
    background: #007acc;
    color: white;
    font-size: 0.85rem;
  }
  
  .status-left, .status-right {
    display: flex;
    gap: 1.5rem;
  }
  
  .status-item {
    display: flex;
    align-items: center;
  }
</style>
