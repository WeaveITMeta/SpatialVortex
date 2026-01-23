<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  
  const dispatch = createEventDispatcher();
  
  interface PromptTemplate {
    id: string;
    name: string;
    icon: string;
    prompt: string;
    category: string;
  }
  
  const templates: PromptTemplate[] = [
    {
      id: 'code-review',
      name: 'Code Review',
      icon: '🔍',
      prompt: 'Review this code for:\n• Bugs and errors\n• Performance issues\n• Security vulnerabilities\n• Best practices\n\nProvide specific suggestions for improvement.',
      category: 'Development'
    },
    {
      id: 'tech-doc',
      name: 'Technical Documentation',
      icon: '📚',
      prompt: 'Create comprehensive technical documentation for this code/system including:\n• Overview and purpose\n• Architecture\n• API reference\n• Usage examples\n• Installation/setup',
      category: 'Development'
    },
    {
      id: 'bug-analysis',
      name: 'Bug Analysis',
      icon: '🐛',
      prompt: 'Analyze this bug/error:\n• Root cause identification\n• Step-by-step debugging\n• Potential fixes\n• Prevention strategies',
      category: 'Development'
    },
    {
      id: 'explain-code',
      name: 'Explain Code',
      icon: '💡',
      prompt: 'Explain this code in detail:\n• What it does (high-level)\n• How it works (step-by-step)\n• Key concepts used\n• Potential improvements',
      category: 'Learning'
    },
    {
      id: 'optimize',
      name: 'Performance Optimization',
      icon: '⚡',
      prompt: 'Optimize this code for:\n• Runtime performance\n• Memory usage\n• Readability\n\nProvide before/after comparisons with benchmarks.',
      category: 'Development'
    },
    {
      id: 'test-cases',
      name: 'Test Cases',
      icon: '🧪',
      prompt: 'Generate comprehensive test cases:\n• Unit tests\n• Integration tests\n• Edge cases\n• Error handling\n\nInclude code examples.',
      category: 'Development'
    },
    {
      id: 'architecture',
      name: 'System Architecture',
      icon: '🏗️',
      prompt: 'Design a system architecture for [REQUIREMENT]:\n• Components and services\n• Data flow\n• Technology stack\n• Scalability considerations\n• Security measures',
      category: 'Design'
    },
    {
      id: 'refactor',
      name: 'Code Refactoring',
      icon: '♻️',
      prompt: 'Refactor this code to:\n• Improve readability\n• Follow SOLID principles\n• Reduce complexity\n• Enhance maintainability\n\nExplain each change.',
      category: 'Development'
    },
    {
      id: 'compare',
      name: 'Compare Solutions',
      icon: '⚖️',
      prompt: 'Compare these approaches/technologies:\n• Pros and cons\n• Use cases\n• Performance\n• Cost\n\nProvide a recommendation table.',
      category: 'Analysis'
    },
    {
      id: 'research',
      name: 'Research Summary',
      icon: '📖',
      prompt: 'Research and summarize [TOPIC]:\n• Key concepts\n• Current state-of-the-art\n• Recent developments\n• Future trends\n\nInclude citations.',
      category: 'Research'
    }
  ];
  
  let selectedCategory = 'All';
  let searchQuery = '';
  
  $: categories = ['All', ...new Set(templates.map(t => t.category))];
  
  $: filteredTemplates = templates.filter(t => {
    const matchesCategory = selectedCategory === 'All' || t.category === selectedCategory;
    const matchesSearch = searchQuery === '' || 
      t.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      t.prompt.toLowerCase().includes(searchQuery.toLowerCase());
    return matchesCategory && matchesSearch;
  });
  
  function useTemplate(template: PromptTemplate) {
    dispatch('use', template.prompt);
  }
</script>

<div class="prompt-templates">
  <div class="templates-header">
    <h3>📋 Prompt Templates</h3>
    <p>Start with a pre-built prompt</p>
  </div>
  
  <div class="templates-controls">
    <input 
      type="text" 
      bind:value={searchQuery}
      placeholder="Search templates..."
      class="search-input"
    />
    
    <div class="category-tabs">
      {#each categories as category}
        <button
          class="category-tab"
          class:active={selectedCategory === category}
          on:click={() => selectedCategory = category}
        >
          {category}
        </button>
      {/each}
    </div>
  </div>
  
  <div class="templates-grid">
    {#each filteredTemplates as template}
      <div class="template-card">
        <div class="template-header">
          <span class="template-icon">{template.icon}</span>
          <div class="template-info">
            <h4>{template.name}</h4>
            <span class="template-category">{template.category}</span>
          </div>
        </div>
        
        <div class="template-preview">
          {template.prompt.slice(0, 100)}...
        </div>
        
        <button 
          class="use-template-btn"
          on:click={() => useTemplate(template)}
        >
          Use Template
        </button>
      </div>
    {/each}
  </div>
  
  {#if filteredTemplates.length === 0}
    <div class="no-results">
      No templates found matching "{searchQuery}"
    </div>
  {/if}
</div>

<style>
  .prompt-templates {
    padding: 1.5rem;
  }
  
  .templates-header {
    margin-bottom: 1.5rem;
  }
  
  .templates-header h3 {
    font-size: 1.25rem;
    margin-bottom: 0.25rem;
    color: #e4e4e7;
  }
  
  .templates-header p {
    font-size: 0.875rem;
    color: #a1a1aa;
  }
  
  .templates-controls {
    margin-bottom: 1.5rem;
  }
  
  .search-input {
    width: 100%;
    padding: 0.75rem 1rem;
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    color: #e4e4e7;
    font-size: 0.875rem;
    margin-bottom: 1rem;
  }
  
  .search-input:focus {
    outline: none;
    border-color: #60a5fa;
  }
  
  .category-tabs {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }
  
  .category-tab {
    padding: 0.5rem 1rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    color: #a1a1aa;
    font-size: 0.875rem;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .category-tab:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #e4e4e7;
  }
  
  .category-tab.active {
    background: rgba(96, 165, 250, 0.2);
    border-color: #60a5fa;
    color: #60a5fa;
  }
  
  .templates-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 1rem;
  }
  
  .template-card {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    padding: 1rem;
    transition: all 0.2s;
  }
  
  .template-card:hover {
    border-color: rgba(96, 165, 250, 0.3);
    transform: translateY(-2px);
  }
  
  .template-header {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    margin-bottom: 0.75rem;
  }
  
  .template-icon {
    font-size: 1.5rem;
  }
  
  .template-info h4 {
    font-size: 1rem;
    margin-bottom: 0.25rem;
    color: #e4e4e7;
  }
  
  .template-category {
    font-size: 0.75rem;
    color: #a1a1aa;
    background: rgba(255, 255, 255, 0.05);
    padding: 0.125rem 0.5rem;
    border-radius: 4px;
  }
  
  .template-preview {
    font-size: 0.875rem;
    color: #a1a1aa;
    line-height: 1.5;
    margin-bottom: 1rem;
    min-height: 3rem;
  }
  
  .use-template-btn {
    width: 100%;
    padding: 0.5rem;
    background: rgba(96, 165, 250, 0.2);
    border: 1px solid rgba(96, 165, 250, 0.3);
    border-radius: 6px;
    color: #60a5fa;
    font-size: 0.875rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .use-template-btn:hover {
    background: rgba(96, 165, 250, 0.3);
  }
  
  .no-results {
    text-align: center;
    padding: 3rem;
    color: #71717a;
    font-size: 0.875rem;
  }
</style>
