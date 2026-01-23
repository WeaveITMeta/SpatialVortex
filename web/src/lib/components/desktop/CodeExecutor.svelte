<script lang="ts">
  import { onMount } from 'svelte';
  
  export let initialCode: string = '';
  export let initialLanguage: string = 'python';
  
  let code = initialCode;
  let language = initialLanguage;
  let output = '';
  let isExecuting = false;
  let executionTime = 0;
  let hasError = false;
  let dockerAvailable = false;
  
  interface Language {
    name: string;
    value: string;
    extension: string;
    example: string;
  }
  
  let languages: Language[] = [];
  
  onMount(async () => {
    // Fetch supported languages
    try {
      const response = await fetch('http://localhost:7000/api/v1/code/languages', {
        method: 'POST'
      });
      const data = await response.json();
      languages = data.languages;
    } catch (error) {
      console.error('Failed to fetch languages:', error);
      // Fallback languages
      languages = [
        { name: 'Python', value: 'python', extension: 'py', example: "print('Hello, World!')" },
        { name: 'JavaScript', value: 'javascript', extension: 'js', example: "console.log('Hello, World!');" },
        { name: 'TypeScript', value: 'typescript', extension: 'ts', example: "console.log('Hello, World!');" },
      ];
    }
    
    // Check Docker status
    try {
      const response = await fetch('http://localhost:7000/api/v1/code/status', {
        method: 'POST'
      });
      const data = await response.json();
      dockerAvailable = data.docker_available;
    } catch (error) {
      console.error('Failed to check Docker status:', error);
    }
  });
  
  async function executeCode() {
    if (!code.trim()) {
      output = 'Error: No code to execute';
      hasError = true;
      return;
    }
    
    isExecuting = true;
    output = '';
    hasError = false;
    executionTime = 0;
    
    try {
      const response = await fetch('http://localhost:7000/api/v1/code/execute', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          code,
          language,
          timeout_ms: 5000
        })
      });
      
      const result = await response.json();
      executionTime = result.execution_time_ms;
      
      if (result.success) {
        output = result.stdout || '(no output)';
        if (result.stderr) {
          output += '\n\nWarnings/Info:\n' + result.stderr;
        }
        hasError = false;
      } else {
        output = result.stderr || result.error || 'Execution failed';
        hasError = true;
      }
    } catch (error) {
      output = `Error: ${error instanceof Error ? error.message : 'Failed to execute code'}`;
      hasError = true;
    } finally {
      isExecuting = false;
    }
  }
  
  function clearOutput() {
    output = '';
    hasError = false;
    executionTime = 0;
  }
  
  function loadExample() {
    const lang = languages.find(l => l.value === language);
    if (lang) {
      code = lang.example;
    }
  }
  
  function handleKeyDown(e: KeyboardEvent) {
    // Ctrl/Cmd + Enter to execute
    if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
      e.preventDefault();
      executeCode();
    }
  }
</script>

<div class="code-executor">
  <!-- Toolbar -->
  <div class="executor-toolbar">
    <div class="toolbar-left">
      <select bind:value={language} class="language-select">
        {#each languages as lang}
          <option value={lang.value}>{lang.name}</option>
        {/each}
      </select>
      
      <button class="toolbar-btn secondary" on:click={loadExample}>
        📝 Load Example
      </button>
      
      {#if !dockerAvailable}
        <span class="docker-warning" title="Docker not available - running locally">
          ⚠️ Local Mode
        </span>
      {/if}
    </div>
    
    <div class="toolbar-right">
      <button class="toolbar-btn secondary" on:click={clearOutput} disabled={!output}>
        🧹 Clear Output
      </button>
      <button 
        class="toolbar-btn primary" 
        on:click={executeCode}
        disabled={isExecuting || !code.trim()}
      >
        {isExecuting ? '⏳ Running...' : '▶️ Run Code'}
      </button>
    </div>
  </div>
  
  <!-- Code Input -->
  <div class="code-input">
    <div class="input-header">
      <span class="input-label">Code</span>
      <span class="input-hint">Ctrl+Enter to run</span>
    </div>
    <textarea 
      bind:value={code}
      on:keydown={handleKeyDown}
      placeholder="Write your code here..."
      class="code-textarea"
      spellcheck="false"
    ></textarea>
  </div>
  
  <!-- Output -->
  <div class="code-output">
    <div class="output-header">
      <span class="output-label">Output</span>
      {#if executionTime > 0}
        <span class="execution-time">⚡ {executionTime}ms</span>
      {/if}
    </div>
    <div class="output-content" class:error={hasError} class:empty={!output}>
      {#if isExecuting}
        <div class="executing-indicator">
          <div class="spinner"></div>
          <span>Executing code...</span>
        </div>
      {:else if output}
        <pre>{output}</pre>
      {:else}
        <div class="empty-state">
          <span class="empty-icon">📊</span>
          <span>Output will appear here</span>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .code-executor {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #1e1e1e;
    color: #d4d4d4;
  }
  
  .executor-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    background: #252526;
    border-bottom: 1px solid #3e3e42;
  }
  
  .toolbar-left,
  .toolbar-right {
    display: flex;
    gap: 0.75rem;
    align-items: center;
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
  
  .toolbar-btn {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 4px;
    font-size: 0.9rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .toolbar-btn.primary {
    background: #0e639c;
    color: white;
  }
  
  .toolbar-btn.primary:hover:not(:disabled) {
    background: #1177bb;
  }
  
  .toolbar-btn.secondary {
    background: #3c3c3c;
    color: #d4d4d4;
  }
  
  .toolbar-btn.secondary:hover:not(:disabled) {
    background: #505050;
  }
  
  .toolbar-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  
  .docker-warning {
    padding: 0.25rem 0.75rem;
    background: rgba(255, 193, 7, 0.1);
    border: 1px solid rgba(255, 193, 7, 0.3);
    border-radius: 4px;
    color: #ffc107;
    font-size: 0.85rem;
  }
  
  .code-input {
    flex: 1;
    display: flex;
    flex-direction: column;
    border-bottom: 1px solid #3e3e42;
  }
  
  .input-header,
  .output-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 1rem;
    background: #2d2d30;
    border-bottom: 1px solid #3e3e42;
  }
  
  .input-label,
  .output-label {
    font-size: 0.85rem;
    font-weight: 600;
    color: #d4d4d4;
  }
  
  .input-hint {
    font-size: 0.75rem;
    color: #858585;
  }
  
  .execution-time {
    font-size: 0.85rem;
    color: #4fc3f7;
  }
  
  .code-textarea {
    flex: 1;
    padding: 1rem;
    background: #1e1e1e;
    border: none;
    color: #d4d4d4;
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
    font-size: 0.9rem;
    line-height: 1.5;
    resize: none;
  }
  
  .code-textarea:focus {
    outline: none;
  }
  
  .code-output {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 200px;
  }
  
  .output-content {
    flex: 1;
    padding: 1rem;
    overflow-y: auto;
    background: #1e1e1e;
  }
  
  .output-content pre {
    margin: 0;
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
    font-size: 0.9rem;
    line-height: 1.5;
    white-space: pre-wrap;
    word-wrap: break-word;
    color: #d4d4d4;
  }
  
  .output-content.error pre {
    color: #f48771;
  }
  
  .output-content.empty {
    display: flex;
    align-items: center;
    justify-content: center;
  }
  
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    color: #858585;
  }
  
  .empty-icon {
    font-size: 2rem;
    opacity: 0.5;
  }
  
  .executing-indicator {
    display: flex;
    align-items: center;
    gap: 1rem;
    color: #4fc3f7;
    font-size: 0.9rem;
  }
  
  .spinner {
    width: 20px;
    height: 20px;
    border: 2px solid rgba(79, 195, 247, 0.3);
    border-top-color: #4fc3f7;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }
  
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
