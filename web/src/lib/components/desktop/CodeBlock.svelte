<script lang="ts">
  import type { CodeBlock } from '$lib/types/chat';
  
  export let block: CodeBlock;
  export let index: number = 0;
  
  let copied = false;
  
  function copyCode() {
    navigator.clipboard.writeText(block.code);
    copied = true;
    setTimeout(() => copied = false, 2000);
  }
  
  function downloadCode() {
    const blob = new Blob([block.code], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = block.filename || `code_${index}.${getFileExtension(block.language)}`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }
  
  function getFileExtension(language: string): string {
    const extensions: Record<string, string> = {
      'rust': 'rs',
      'python': 'py',
      'javascript': 'js',
      'typescript': 'ts',
      'java': 'java',
      'cpp': 'cpp',
      'c': 'c',
      'go': 'go',
      'ruby': 'rb',
      'php': 'php',
      'swift': 'swift',
      'kotlin': 'kt',
      'scala': 'scala',
      'sql': 'sql',
      'html': 'html',
      'css': 'css',
      'json': 'json',
      'yaml': 'yaml',
      'toml': 'toml',
      'markdown': 'md',
      'bash': 'sh',
      'shell': 'sh',
    };
    return extensions[language.toLowerCase()] || 'txt';
  }
  
  function getLanguageLabel(language: string): string {
    return language.charAt(0).toUpperCase() + language.slice(1);
  }
</script>

<div class="code-block-wrapper">
  <div class="code-header">
    <div class="code-info">
      <span class="language-badge">{getLanguageLabel(block.language)}</span>
      {#if block.filename}
        <span class="filename">📄 {block.filename}</span>
      {/if}
      {#if block.reasoning_steps}
        <span class="meta">🧠 {block.reasoning_steps} steps</span>
      {/if}
      {#if block.complexity_score}
        <span class="meta">📊 Complexity: {block.complexity_score.toFixed(1)}/10</span>
      {/if}
    </div>
    <div class="code-actions">
      <button class="action-btn" on:click={copyCode} title="Copy code">
        {copied ? '✅' : '📋'}
      </button>
      <button class="action-btn" on:click={downloadCode} title="Download">
        💾
      </button>
    </div>
  </div>
  
  <pre class="code-content"><code class="language-{block.language}">{block.code}</code></pre>
</div>

<style>
  .code-block-wrapper {
    margin: 1rem 0;
    background: rgba(0, 0, 0, 0.4);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    overflow: hidden;
  }
  
  .code-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    background: rgba(255, 255, 255, 0.03);
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  }
  
  .code-info {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex-wrap: wrap;
  }
  
  .language-badge {
    font-family: monospace;
    font-size: 0.75rem;
    font-weight: 600;
    color: #a78bfa;
    background: rgba(167, 139, 250, 0.15);
    padding: 0.25rem 0.625rem;
    border-radius: 6px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  
  .filename {
    font-size: 0.8rem;
    color: #94a3b8;
    font-family: monospace;
  }
  
  .meta {
    font-size: 0.75rem;
    color: #71717a;
  }
  
  .code-actions {
    display: flex;
    gap: 0.5rem;
  }
  
  .action-btn {
    padding: 0.375rem 0.75rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    color: #a1a1aa;
    cursor: pointer;
    transition: all 0.2s;
    font-size: 0.85rem;
  }
  
  .action-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #e4e4e7;
    transform: translateY(-1px);
  }
  
  .code-content {
    margin: 0;
    padding: 1.25rem;
    overflow-x: auto;
    font-family: 'JetBrains Mono', 'Fira Code', 'Cascadia Code', 'Consolas', monospace;
    font-size: 0.875rem;
    line-height: 1.6;
    color: #e4e4e7;
  }
  
  .code-content code {
    font-family: inherit;
  }
  
  /* Scrollbar styling */
  .code-content::-webkit-scrollbar {
    height: 8px;
  }
  
  .code-content::-webkit-scrollbar-track {
    background: rgba(0, 0, 0, 0.2);
  }
  
  .code-content::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.2);
    border-radius: 4px;
  }
  
  .code-content::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.3);
  }
</style>
