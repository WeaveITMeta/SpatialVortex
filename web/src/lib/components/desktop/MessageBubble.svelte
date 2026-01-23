<script lang="ts">
  import type { ChatMessage } from '$lib/types/chat';
  import ELPMini from './ELPMini.svelte';
  import SourcesPanel from './SourcesPanel.svelte';
  import FollowUpSuggestions from './FollowUpSuggestions.svelte';
  import RichMarkdown from '../RichMarkdown.svelte';
  import CodeBlock from './CodeBlock.svelte';
  import { createEventDispatcher } from 'svelte';
  
  export let message: ChatMessage;
  export let showFollowUps = false; // Only show for last assistant message
  
  const dispatch = createEventDispatcher();
  
  // Parse tables from markdown
  let contentWithoutTables = '';
  
  // Format content with inline citations
  function formatWithCitations(content: string, sources: any[] | undefined): string {
    if (!sources || sources.length === 0) return content;
    
    // Add citation numbers inline where sources are referenced
    let formattedContent = content;
    
    // Add citations list at the end
    let citations = '\n\n---\n\n**References:**\n\n';
    sources.forEach((source, index) => {
      const citationNum = index + 1;
      if (source.web_source) {
        citations += `[${citationNum}] [${source.web_source.title}](${source.web_source.url}) - ${source.web_source.domain}\n\n`;
      } else {
        citations += `[${citationNum}] Document: ${source.doc_id}\n\n`;
      }
    });
    
    return formattedContent + citations;
  }
  
  // Insert a newline before an inline table header that follows a colon.
  // Example: "... comparison: | A | B |\n| --- | --- |" -> becomes a proper table block.
  function normalizeTableBlocks(text: string): string {
    try {
      return text.replace(/:\s*\|/g, ':\n|');
    } catch {
      return text;
    }
  }

  // Convert common section labels into proper markdown headings
  function autoHeadingize(text: string): string {
    const sections = [
      'Introduction',
      'How I Work',
      'Limitations',
      'Comparison with Other Sources',
      'Conclusion'
    ];

    let out = text;
    for (const s of sections) {
      // Insert a markdown heading before the section label when it starts a line
      // Preserve the text that follows the label
      const re = new RegExp(`(^|\\n)\\s*(${s})\\s+`, 'g');
      out = out.replace(re, (_m, p1, label) => `${p1}\n\n### ${label}\n\n`);
    }
    return out;
  }
  
  // Prepare content with citations for rendering
  function prepareContent(content: string): string {
    // Add citations if sources exist
    const contentWithCitations = formatWithCitations(content, message.sources);

    // Normalize cases where table header is written inline after a colon
    let normalized = normalizeTableBlocks(contentWithCitations);
    normalized = autoHeadingize(normalized);

    // Let RichMarkdown (marked with GFM) render tables natively.
    contentWithoutTables = normalized;

    return contentWithoutTables;
  }
  
  function copyMessage() {
    navigator.clipboard.writeText(message.content);
  }
  
  function formatTime(date: Date): string {
    return date.toLocaleTimeString('en-US', {
      hour: 'numeric',
      minute: '2-digit'
    });
  }
  
  // Generate contextual follow-up suggestions
  function generateFollowUps(content: string): string[] {
    const suggestions: string[] = [];
    
    // Code-related responses
    if (content.includes('```') || content.toLowerCase().includes('code')) {
      suggestions.push("Can you explain the code in more detail?");
      suggestions.push("What are potential edge cases?");
      suggestions.push("How can I optimize this?");
    }
    
    // Explanation responses
    if (content.toLowerCase().includes('because') || content.toLowerCase().includes('therefore')) {
      suggestions.push("Can you give a practical example?");
      suggestions.push("What are the alternatives?");
    }
    
    // Comparison responses
    if (content.includes('vs') || content.includes('comparison')) {
      suggestions.push("Which one would you recommend?");
      suggestions.push("What are the trade-offs?");
    }
    
    // Technical responses
    if (content.toLowerCase().includes('architecture') || content.toLowerCase().includes('system')) {
      suggestions.push("How does this scale?");
      suggestions.push("What are the security considerations?");
    }
    
    // Default suggestions
    if (suggestions.length === 0) {
      suggestions.push("Can you explain this in simpler terms?");
      suggestions.push("What else should I know?");
      suggestions.push("Show me a practical example");
    }
    
    return suggestions.slice(0, 3); // Max 3 suggestions
  }
  
  function handleFollowUpSelect(event: CustomEvent<string>) {
    dispatch('followup', event.detail);
  }
</script>

<div class="message-wrapper {message.role}">
  <div class="message-bubble">
    <div class="message-header">
      <span class="role-badge">
        {#if message.role === 'user'}
          You
        {:else if message.role === 'vortex'}
          🌀 Vortex
        {:else if message.model_name}
          🤖 {message.model_name}
        {:else}
          AI
        {/if}
      </span>
      <span class="message-time">{formatTime(message.timestamp)}</span>
    </div>
    
    <div 
      class="message-content markdown"
      class:streaming={message.is_streaming}
      style={message.semantic_color ? `color: ${message.semantic_color};` : ''}
      title={message.primary_meaning ? `Mood: ${message.primary_meaning}` : ''}
    >
      <RichMarkdown content={prepareContent(message.content)} />
    </div>
    
    
    {#if message.primary_meaning}
      <div class="semantic-badge" style="background: {message.semantic_color}">
        {message.primary_meaning}
      </div>
    {/if}
    
    {#if message.code_blocks && message.code_blocks.length > 0}
      <div class="code-blocks">
        {#each message.code_blocks as block, i}
          <CodeBlock {block} index={i} />
        {/each}
      </div>
    {/if}
    
    {#if message.generation_time_ms}
      <div class="generation-stats">
        ⚡ Generated in {(message.generation_time_ms / 1000).toFixed(1)}s
        {#if message.code_blocks?.[0]?.reasoning_steps}
          • 🧠 {message.code_blocks[0].reasoning_steps} reasoning steps
        {/if}
      </div>
    {/if}
    
    {#if message.elp}
      <ELPMini
        elp={message.elp}
        position={message.flux_position}
        confidence={message.confidence}
      />
    {/if}
    
    {#if message.sources && message.sources.length > 0}
      <SourcesPanel sources={message.sources} />
    {/if}
    
    {#if showFollowUps && message.role === 'assistant' && !message.is_streaming}
      <FollowUpSuggestions 
        suggestions={generateFollowUps(message.content)}
        on:select={handleFollowUpSelect}
      />
    {/if}
    
    <div class="message-actions">
      <button class="action-btn" on:click={copyMessage} title="Copy">
        📋
      </button>
    </div>
  </div>
</div>

<style>
  .message-wrapper {
    display: flex;
    animation: slideIn 0.3s ease-out;
  }
  
  /* Smooth text appearance during streaming */
  .message-content {
    transition: all 0.15s ease-out;
  }
  
  /* Typing cursor during streaming */
  .message-content.streaming::after {
    content: '▋';
    animation: blink 1s step-end infinite;
    color: #60a5fa;
    margin-left: 2px;
  }
  
  @keyframes blink {
    0%, 50% {
      opacity: 1;
    }
    51%, 100% {
      opacity: 0;
    }
  }
  
  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
  
  .message-wrapper.user {
    justify-content: flex-end;
  }
  
  .message-bubble {
    max-width: 70%;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 16px;
    padding: 1.25rem;
    position: relative;
  }
  
  .message-wrapper.user .message-bubble {
    background: linear-gradient(135deg, rgba(96, 165, 250, 0.15) 0%, rgba(59, 130, 246, 0.15) 100%);
    border-color: rgba(96, 165, 250, 0.2);
  }
  
  .message-wrapper.assistant .message-bubble {
    background: rgba(255, 255, 255, 0.05);
    border-color: rgba(255, 255, 255, 0.08);
  }
  
  /* Vortex Consensus - Darker orange with white text for better readability */
  .message-wrapper.vortex .message-bubble {
    background: linear-gradient(135deg, #c55020 0%, #d66020 100%);
    border-color: rgba(197, 80, 32, 0.5);
  }
  
  .message-wrapper.vortex .message-content {
    color: white !important;
  }
  
  .message-wrapper.vortex .role-badge {
    color: white !important;
    font-weight: 700;
  }
  
  .message-wrapper.vortex .message-time {
    color: rgba(255, 255, 255, 0.8) !important;
  }
  
  .message-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.75rem;
    font-size: 0.8rem;
  }
  
  .role-badge {
    font-weight: 600;
    color: #60a5fa;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    font-size: 0.75rem;
  }
  
  .message-wrapper.user .role-badge {
    color: #60a5fa;
  }
  
  .message-time {
    color: #71717a;
    font-size: 0.75rem;
  }
  
  .message-content {
    color: #e4e4e7;
    line-height: 1.6;
    font-size: 0.95rem;
  }
  
  /* HTML content styling */
  .html-content :global(h1),
  .html-content :global(h2),
  .html-content :global(h3),
  .html-content :global(h4),
  .html-content :global(h5),
  .html-content :global(h6) {
    color: #60a5fa;
    margin-top: 1.5rem;
    margin-bottom: 0.75rem;
    font-weight: 600;
    line-height: 1.3;
  }
  
  .html-content :global(h1) { font-size: 1.75rem; }
  .html-content :global(h2) { font-size: 1.5rem; }
  .html-content :global(h3) { font-size: 1.25rem; }
  
  .html-content :global(p) {
    margin: 0 0 1.5em 0;  /* Blank line between paragraphs */
    line-height: 1.75;     /* Better readability */
    text-indent: 2em;      /* First-line indent (standard typography) */
  }
  
  /* No indent for first paragraph after headings */
  .html-content :global(h1 + p),
  .html-content :global(h2 + p),
  .html-content :global(h3 + p),
  .html-content :global(h4 + p),
  .html-content :global(h5 + p),
  .html-content :global(h6 + p) {
    text-indent: 0;
  }
  
  /* No indent for first paragraph in blockquotes */
  .html-content :global(blockquote p:first-child) {
    text-indent: 0;
  }
  
  .html-content :global(ul),
  .html-content :global(ol) {
    margin: 1rem 0;
    padding-left: 2rem;
    line-height: 1.8;
  }
  
  .html-content :global(li) {
    margin: 0.5rem 0;
  }
  
  .html-content :global(strong) {
    color: #f4f4f5;
    font-weight: 600;
  }
  
  .html-content :global(em) {
    font-style: italic;
    color: #bac2de;
  }
  
  .html-content :global(a) {
    color: #89b4fa;
    text-decoration: none;
    transition: color 0.2s;
  }
  
  .html-content :global(a:hover) {
    color: #60a5fa;
    text-decoration: underline;
  }
  
  .html-content :global(blockquote) {
    border-left: 4px solid #60a5fa;
    padding-left: 1rem;
    margin: 1rem 0;
    color: #bac2de;
    font-style: italic;
    background: rgba(96, 165, 250, 0.05);
    padding: 0.75rem 1rem;
    border-radius: 4px;
  }
  
  .html-content :global(hr) {
    border: none;
    border-top: 1px solid rgba(255, 255, 255, 0.2);
    margin: 1.5rem 0;
  }
  
  .html-content :global(table) {
    width: 100%;
    border-collapse: collapse;
    margin: 1rem 0;
    background: #181825;
    border-radius: 8px;
    overflow: hidden;
  }
  
  .html-content :global(thead) {
    background: #313244;
  }
  
  .html-content :global(th) {
    color: #60a5fa;
    padding: 0.875rem 1rem;
    text-align: left;
    font-weight: 600;
    border-bottom: 2px solid #45475a;
  }
  
  .html-content :global(td) {
    padding: 0.875rem 1rem;
    border-bottom: 1px solid #313244;
    color: #cdd6f4;
  }
  
  .html-content :global(tbody tr:hover) {
    background: #1e1e2e;
  }
  
  .html-content :global(pre) {
    background: #1e1e2e;
    border: 1px solid #313244;
    border-radius: 6px;
    padding: 1rem;
    overflow-x: auto;
    margin: 1rem 0;
  }
  
  .html-content :global(code) {
    background: rgba(255, 255, 255, 0.1);
    padding: 0.2rem 0.4rem;
    border-radius: 3px;
    font-family: 'Fira Code', 'Consolas', 'Monaco', monospace;
    font-size: 0.9em;
    color: #f9e2af;
  }
  
  .html-content :global(pre code) {
    background: transparent;
    padding: 0;
    color: #cdd6f4;
  }
  
  /* Markdown rendering styles */
  .message-content.markdown :global(pre) {
    background: #1e1e2e;
    border: 1px solid #313244;
    border-radius: 6px;
    padding: 1rem;
    overflow-x: auto;
    margin: 0.5rem 0;
  }
  
  .message-content.markdown :global(code) {
    background: #1e1e2e;
    padding: 0.2rem 0.4rem;
    border-radius: 3px;
    font-family: 'Fira Code', 'Consolas', monospace;
    font-size: 0.9em;
  }
  
  .message-content.markdown :global(pre code) {
    background: transparent;
    padding: 0;
  }
  
  .message-content.markdown :global(h1),
  .message-content.markdown :global(h2),
  .message-content.markdown :global(h3) {
    color: #60a5fa;
    margin-top: 1rem;
    margin-bottom: 0.5rem;
  }
  
  .message-content.markdown :global(ul),
  .message-content.markdown :global(ol) {
    padding-left: 1.5rem;
    margin: 1rem 0;
    line-height: 1.8;
  }
  
  .message-content.markdown :global(ol li),
  .message-content.markdown :global(ul li) {
    margin-bottom: 0.75rem;
    padding-left: 0.5rem;
  }
  
  .message-content.markdown :global(ol li:last-child),
  .message-content.markdown :global(ul li:last-child) {
    margin-bottom: 0;
  }
  
  .message-content.markdown :global(p) {
    margin: 0.4rem 0;
    line-height: 1.6;
  }
  
  .message-content.markdown :global(p:first-child) {
    margin-top: 0;
  }
  
  .message-content.markdown :global(blockquote) {
    background: #1e1e2e;
    border-left: 4px solid #60a5fa;
    padding: 1rem;
    margin: 1rem 0;
    border-radius: 6px;
    color: #bac2de;
    font-style: italic;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  }
  
  .message-content.markdown :global(blockquote p) {
    margin: 0.5rem 0;
  }
  
  .message-content.markdown :global(blockquote p:first-child) {
    margin-top: 0;
  }
  
  .message-content.markdown :global(blockquote p:last-child) {
    margin-bottom: 0;
  }
  
  .message-content.markdown :global(a) {
    color: #89b4fa;
    text-decoration: none;
  }
  
  .message-content.markdown :global(a:hover) {
    text-decoration: underline;
  }
  
  /* Semantic color badge */
  .semantic-badge {
    display: inline-block;
    padding: 0.25rem 0.5rem;
    border-radius: 12px;
    font-size: 0.75rem;
    font-weight: 600;
    color: white;
    margin-top: 0.5rem;
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
    opacity: 0.9;
  }
  
  /* Callout boxes */
  .message-content :global(.callout) {
    padding: 1rem;
    margin: 1rem 0;
    border-radius: 8px;
    border-left: 4px solid;
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    font-size: 0.95rem;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  }
  
  .message-content :global(.callout-icon) {
    font-size: 1.25rem;
    line-height: 1;
    flex-shrink: 0;
  }
  
  .message-content :global(.callout-info) {
    background: #1e2030;
    border-left-color: #89b4fa;
  }
  
  .message-content :global(.callout-warning) {
    background: #2e2820;
    border-left-color: #f9e2af;
  }
  
  .message-content :global(.callout-error) {
    background: #2e1e24;
    border-left-color: #f38ba8;
  }
  
  .message-content :global(.callout-success) {
    background: #1e2e24;
    border-left-color: #a6e3a1;
  }
  
  .message-content :global(.callout-tip) {
    background: #1e2a3e;
    border-left-color: #60a5fa;
  }
  
  /* Enhanced tables */
  .message-content.markdown :global(table) {
    width: 100%;
    border-collapse: collapse;
    margin: 1rem 0;
    background: #181825;
    border-radius: 8px;
    overflow: hidden;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  }
  
  .message-content.markdown :global(thead) {
    background: #313244;
  }
  
  .message-content.markdown :global(th) {
    color: #60a5fa;
    padding: 0.875rem 1rem;
    text-align: left;
    font-weight: 600;
    font-size: 0.875rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    border-bottom: 2px solid #45475a;
  }
  
  .message-content.markdown :global(td) {
    padding: 0.875rem 1rem;
    border-bottom: 1px solid #313244;
    color: #cdd6f4;
  }
  
  .message-content.markdown :global(tbody tr) {
    transition: background 0.2s ease;
  }
  
  .message-content.markdown :global(tbody tr:hover) {
    background: #1e1e2e;
  }
  
  .message-content.markdown :global(tbody tr:last-child td) {
    border-bottom: none;
  }
  
  /* Task lists (checkboxes) */
  .message-content.markdown :global(li) {
    position: relative;
  }
  
  .message-content.markdown :global(input[type="checkbox"]) {
    margin-right: 0.75rem;
    cursor: pointer;
    width: 1.125rem;
    height: 1.125rem;
    accent-color: #89b4fa;
  }
  
  .message-content.markdown :global(li:has(input[type="checkbox"])) {
    list-style: none;
    margin-left: -1.5rem;
    padding-left: 0;
  }
  
  .message-content.markdown :global(li:has(input[type="checkbox"]:checked)) {
    opacity: 0.7;
  }
  
  .message-content.markdown :global(li:has(input[type="checkbox"]:checked) > *:not(input)) {
    text-decoration: line-through;
    color: #6c7086;
  }
  
  .message-content {
    white-space: pre-wrap;
    word-wrap: break-word;
  }
  
  .message-actions {
    margin-top: 0.75rem;
    display: flex;
    gap: 0.5rem;
    opacity: 0;
    transition: opacity 0.2s;
  }
  
  .message-bubble:hover .message-actions {
    opacity: 1;
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
  }
  
  /* Images */
  .message-content.markdown :global(img) {
    max-width: 100%;
    height: auto;
    border-radius: 8px;
    margin: 1rem 0;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    border: 1px solid rgba(255, 255, 255, 0.1);
  }
  
  .message-content.markdown :global(p img) {
    display: block;
    margin: 1rem auto;
  }
  
  /* Citations */
  .message-content.markdown :global(cite) {
    font-style: italic;
    color: #a1a1aa;
    border-left: 3px solid #60a5fa;
    padding-left: 1rem;
    display: block;
    margin: 1rem 0;
    background: rgba(96, 165, 250, 0.05);
    padding: 0.75rem 1rem;
    border-radius: 4px;
  }
  
  .message-content.markdown :global(cite::before) {
    content: '📚 ';
    margin-right: 0.5rem;
  }
  
  /* Superscript and subscript for citations */
  .message-content.markdown :global(sup) {
    color: #60a5fa;
    cursor: help;
    font-weight: 600;
  }
  
  .message-content.markdown :global(sub) {
    color: #a1a1aa;
    font-size: 0.85em;
  }
  
  .code-blocks {
    margin-top: 1rem;
  }
  
  .generation-stats {
    margin-top: 0.75rem;
    padding: 0.5rem 0.75rem;
    background: rgba(96, 165, 250, 0.1);
    border: 1px solid rgba(96, 165, 250, 0.2);
    border-radius: 8px;
    font-size: 0.8rem;
    color: #60a5fa;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
</style>
