<script lang="ts">
  import { onMount } from 'svelte';
  import { marked } from 'marked';
  import DOMPurify from 'dompurify';
  import mermaid from 'mermaid';
  import katex from 'katex';
  import hljs from 'highlight.js/lib/core';
  // Register 24 common languages
  import javascript from 'highlight.js/lib/languages/javascript';
  import typescript from 'highlight.js/lib/languages/typescript';
  import python from 'highlight.js/lib/languages/python';
  import rust from 'highlight.js/lib/languages/rust';
  import go from 'highlight.js/lib/languages/go';
  import java from 'highlight.js/lib/languages/java';
  import c from 'highlight.js/lib/languages/c';
  import cpp from 'highlight.js/lib/languages/cpp';
  import csharp from 'highlight.js/lib/languages/csharp';
  import ruby from 'highlight.js/lib/languages/ruby';
  import php from 'highlight.js/lib/languages/php';
  import swift from 'highlight.js/lib/languages/swift';
  import kotlin from 'highlight.js/lib/languages/kotlin';
  import scala from 'highlight.js/lib/languages/scala';
  import sql from 'highlight.js/lib/languages/sql';
  import bash from 'highlight.js/lib/languages/bash';
  import powershell from 'highlight.js/lib/languages/powershell';
  import json from 'highlight.js/lib/languages/json';
  import yaml from 'highlight.js/lib/languages/yaml';
  import ini from 'highlight.js/lib/languages/ini'; // used for INI/TOML
  import xml from 'highlight.js/lib/languages/xml'; // HTML/XML/SVG
  import css from 'highlight.js/lib/languages/css';
  import markdownLang from 'highlight.js/lib/languages/markdown';
  
  // Import required CSS
  import 'highlight.js/styles/github-dark.min.css';
  import 'katex/dist/katex.min.css';
  
  export let content: string;
  
  let container: HTMLDivElement;
  let renderedHtml = '';
  
  // Initialize Mermaid
  mermaid.initialize({
    startOnLoad: false,
    theme: 'dark',
    themeVariables: {
      primaryColor: '#60a5fa',
      primaryTextColor: '#fff',
      primaryBorderColor: '#3b82f6',
      lineColor: '#60a5fa',
      secondaryColor: '#3b82f6',
      tertiaryColor: '#1e40af',
    }
  });

  // Register highlight.js languages once
  hljs.registerLanguage('javascript', javascript);
  hljs.registerLanguage('typescript', typescript);
  hljs.registerLanguage('python', python);
  hljs.registerLanguage('rust', rust);
  hljs.registerLanguage('go', go);
  hljs.registerLanguage('java', java);
  hljs.registerLanguage('c', c);
  hljs.registerLanguage('cpp', cpp);
  hljs.registerLanguage('csharp', csharp);
  hljs.registerLanguage('ruby', ruby);
  hljs.registerLanguage('php', php);
  hljs.registerLanguage('swift', swift);
  hljs.registerLanguage('kotlin', kotlin);
  hljs.registerLanguage('scala', scala);
  hljs.registerLanguage('sql', sql);
  hljs.registerLanguage('bash', bash);
  hljs.registerLanguage('powershell', powershell);
  hljs.registerLanguage('json', json);
  hljs.registerLanguage('yaml', yaml);
  hljs.registerLanguage('ini', ini);
  hljs.registerLanguage('toml', ini);
  hljs.registerLanguage('xml', xml);
  hljs.registerLanguage('html', xml);
  hljs.registerLanguage('css', css);
  hljs.registerLanguage('markdown', markdownLang);
  
  // Configure marked for better rendering
  marked.setOptions({
    breaks: true,   // Convert single newlines to <br> for proper line breaks
    gfm: true,      // GitHub Flavored Markdown for better list/table support
    mangle: false,
    highlight(code: string, lang?: string) {
      try {
        if (lang && hljs.getLanguage(lang)) {
          return hljs.highlight(code, { language: lang }).value;
        }
        return hljs.highlightAuto(code).value;
      } catch {
        return code;
      }
    }
  });
  
  // Render markdown with Mermaid and LaTeX support
  function renderMarkdown(text: string): string {
    let processed = text;
    
    // Track mermaid and math blocks to protect them
    const mermaidBlocks: string[] = [];
    const mathBlocks: string[] = [];
    const inlineMath: string[] = [];
    
    // Extract and protect Mermaid diagrams
    processed = processed.replace(/```mermaid([\s\S]*?)```/g, (_match, code) => {
      const id = `MERMAID_${mermaidBlocks.length}`;
      mermaidBlocks.push(code.trim());
      return `<div class="mermaid-placeholder" data-id="${id}"></div>`;
    });
    
    // Extract and protect LaTeX math blocks ($$...$$)
    processed = processed.replace(/\$\$([\s\S]*?)\$\$/g, (_match, math) => {
      const id = `MATH_${mathBlocks.length}`;
      mathBlocks.push(math.trim());
      return `<div class="math-placeholder" data-id="${id}"></div>`;
    });
    
    // Extract and protect inline LaTeX ($...$)
    processed = processed.replace(/\$([^\$\n]+?)\$/g, (_match, math) => {
      const id = `INLINE_${inlineMath.length}`;
      inlineMath.push(math.trim());
      return `<span class="inline-math-placeholder" data-id="${id}"></span>`;
    });
    
    // Render markdown
    let html = marked.parse(processed) as string;
    
    // Sanitize
    html = DOMPurify.sanitize(html, {
      ADD_TAGS: ['div', 'span'],
      ADD_ATTR: ['class', 'data-id'],
    });
    
    // Replace mermaid placeholders
    mermaidBlocks.forEach((code, index) => {
      const id = `mermaid-${Date.now()}-${index}`;
      html = html.replace(
        `<div class="mermaid-placeholder" data-id="MERMAID_${index}"></div>`,
        `<div class="mermaid" id="${id}">${code}</div>`
      );
    });
    
    // Replace math placeholders
    mathBlocks.forEach((math, index) => {
      try {
        const rendered = katex.renderToString(math, {
          displayMode: true,
          throwOnError: false,
        });
        html = html.replace(
          `<div class="math-placeholder" data-id="MATH_${index}"></div>`,
          `<div class="math-block">${rendered}</div>`
        );
      } catch (err) {
        console.error('KaTeX error:', err);
        html = html.replace(
          `<div class="math-placeholder" data-id="MATH_${index}"></div>`,
          `<div class="math-error">$$${math}$$</div>`
        );
      }
    });
    
    // Replace inline math placeholders
    inlineMath.forEach((math, index) => {
      try {
        const rendered = katex.renderToString(math, {
          displayMode: false,
          throwOnError: false,
        });
        html = html.replace(
          `<span class="inline-math-placeholder" data-id="INLINE_${index}"></span>`,
          `<span class="inline-math">${rendered}</span>`
        );
      } catch (err) {
        console.error('KaTeX error:', err);
        html = html.replace(
          `<span class="inline-math-placeholder" data-id="INLINE_${index}"></span>`,
          `<span class="math-error">$${math}$</span>`
        );
      }
    });
    
    return html;
  }
  
  // Render mermaid diagrams after DOM update
  async function renderMermaidDiagrams() {
    if (!container) return;
    
    const mermaidElements = container.querySelectorAll('.mermaid');
    for (const element of Array.from(mermaidElements)) {
      const htmlElement = element as HTMLElement;
      if (htmlElement.getAttribute('data-processed') !== 'true') {
        try {
          const { svg } = await mermaid.render(htmlElement.id, htmlElement.textContent || '');
          htmlElement.innerHTML = svg;
          htmlElement.setAttribute('data-processed', 'true');
        } catch (err) {
          console.error('Mermaid error:', err);
          htmlElement.innerHTML = `<div class="mermaid-error">Failed to render diagram</div>`;
        }
      }
    }
  }
  
  // Highlight any code blocks that were not handled during markdown render
  function highlightCodeBlocks() {
    if (!container) return;
    const blocks = container.querySelectorAll('pre code');
    blocks.forEach((el) => {
      try {
        hljs.highlightElement(el as HTMLElement);
      } catch {
        // no-op
      }
    });
  }

  // Update when content changes
  $: {
    renderedHtml = renderMarkdown(content);
    // Render mermaid diagrams after DOM update
    setTimeout(() => {
      renderMermaidDiagrams();
      highlightCodeBlocks();
    }, 0);
  }
  
  onMount(() => {
    renderMermaidDiagrams();
    highlightCodeBlocks();
  });
</script>

<div class="rich-markdown" bind:this={container}>
  {@html renderedHtml}
</div>

<style>
  .rich-markdown {
    color: #e4e4e7;
    line-height: 1.6;
  }
  
  /* Headings */
  .rich-markdown :global(h1) {
    font-size: 2rem;
    font-weight: 700;
    margin: 1.5rem 0 1rem 0;
    color: #f4f4f5;
  }
  
  .rich-markdown :global(h2) {
    font-size: 1.5rem;
    font-weight: 600;
    margin: 1.25rem 0 0.75rem 0;
    color: #f4f4f5;
  }
  
  .rich-markdown :global(h3) {
    font-size: 1.25rem;
    font-weight: 600;
    margin: 1rem 0 0.5rem 0;
    color: #f4f4f5;
  }
  
  /* Paragraphs */
  .rich-markdown :global(p) {
    margin: 0.4rem 0;
  }
  
  /* Lists */
  .rich-markdown :global(ul),
  .rich-markdown :global(ol) {
    margin: 0.75rem 0;
    padding-left: 2rem;
  }
  
  .rich-markdown :global(li) {
    margin: 0.25rem 0;
  }
  
  /* Code blocks */
  .rich-markdown :global(pre) {
    background: #18181b;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    padding: 1rem;
    overflow-x: auto;
    margin: 1rem 0;
  }
  
  .rich-markdown :global(code) {
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
    font-size: 0.9em;
  }
  
  .rich-markdown :global(pre code) {
    background: none;
    border: none;
    padding: 0;
  }
  
  .rich-markdown :global(:not(pre) > code) {
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 4px;
    padding: 0.125rem 0.375rem;
    font-size: 0.875em;
  }
  
  /* Tables */
  .rich-markdown :global(table) {
    width: 100%;
    border-collapse: collapse;
    margin: 1rem 0;
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 8px;
    overflow: hidden;
  }
  
  .rich-markdown :global(thead) {
    background: rgba(96, 165, 250, 0.1);
  }
  
  .rich-markdown :global(th) {
    padding: 0.75rem 1rem;
    text-align: left;
    font-weight: 600;
    color: #60a5fa;
    border-bottom: 2px solid rgba(96, 165, 250, 0.3);
  }
  
  .rich-markdown :global(td) {
    padding: 0.75rem 1rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }
  
  .rich-markdown :global(tr:last-child td) {
    border-bottom: none;
  }
  
  .rich-markdown :global(tbody tr:hover) {
    background: rgba(255, 255, 255, 0.05);
  }
  
  /* Blockquotes */
  .rich-markdown :global(blockquote) {
    border-left: 4px solid #60a5fa;
    padding-left: 1rem;
    margin: 1rem 0;
    color: #a1a1aa;
    font-style: italic;
  }
  
  /* Links */
  .rich-markdown :global(a) {
    color: #60a5fa;
    text-decoration: none;
    transition: color 0.2s;
  }
  
  .rich-markdown :global(a:hover) {
    color: #3b82f6;
    text-decoration: underline;
  }
  
  /* Horizontal rule */
  .rich-markdown :global(hr) {
    border: none;
    border-top: 1px solid rgba(255, 255, 255, 0.2);
    margin: 1.5rem 0;
  }
  
  /* Mermaid diagrams */
  .rich-markdown :global(.mermaid) {
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    padding: 1rem;
    margin: 1rem 0;
    display: flex;
    justify-content: center;
  }
  
  .rich-markdown :global(.mermaid svg) {
    max-width: 100%;
    height: auto;
  }
  
  .rich-markdown :global(.mermaid-error) {
    color: #ef4444;
    padding: 1rem;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: 8px;
    margin: 1rem 0;
  }
  
  /* Math blocks */
  .rich-markdown :global(.math-block) {
    overflow-x: auto;
    padding: 1rem;
    margin: 1rem 0;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    text-align: center;
  }
  
  .rich-markdown :global(.inline-math) {
    display: inline-block;
    vertical-align: middle;
  }
  
  .rich-markdown :global(.math-error) {
    color: #ef4444;
    background: rgba(239, 68, 68, 0.1);
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    font-family: monospace;
  }
  
  /* KaTeX styling adjustments for dark theme */
  .rich-markdown :global(.katex) {
    color: #e4e4e7;
  }
  
  .rich-markdown :global(.katex .mord) {
    color: #e4e4e7;
  }
</style>
