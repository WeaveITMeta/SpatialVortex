<script lang="ts">
  export let onSave: (instructions: CustomInstructions) => void;
  
  interface CustomInstructions {
    responseStyle: string;
    codePreferences: string;
    outputFormat: string;
    customRules: string;
  }
  
  let instructions: CustomInstructions = {
    responseStyle: 'Clear, well-formatted responses with proper markdown',
    codePreferences: '',
    outputFormat: 'Use proper markdown formatting: headers (###), lists, code blocks. NO task checkboxes or TODO lists in responses.',
    customRules: 'Always use clean markdown formatting. Do not include task management checkboxes or implementation TODO lists in your responses to users.'
  };
  
  // Load from localStorage
  if (typeof window !== 'undefined') {
    const saved = localStorage.getItem('customInstructions');
    if (saved) {
      instructions = JSON.parse(saved);
    }
  }
  
  function saveInstructions() {
    localStorage.setItem('customInstructions', JSON.stringify(instructions));
    onSave(instructions);
    showSavedMessage = true;
    setTimeout(() => showSavedMessage = false, 2000);
  }
  
  function resetInstructions() {
    instructions = {
      responseStyle: '',
      codePreferences: '',
      outputFormat: '',
      customRules: ''
    };
    localStorage.removeItem('customInstructions');
  }
  
  let showSavedMessage = false;
  
  const examples = {
    responseStyle: [
      'Be concise and technical',
      'Explain like I\'m 5',
      'Use analogies and examples',
      'Professional and formal'
    ],
    codePreferences: [
      'Use TypeScript over JavaScript',
      'Prefer functional programming',
      'Include error handling',
      'Follow SOLID principles'
    ],
    outputFormat: [
      'Use tables for comparisons',
      'Include code examples',
      'Add diagrams when helpful',
      'Provide step-by-step guides'
    ]
  };
</script>

<div class="custom-instructions">
  <div class="instructions-header">
    <h3>⚙️ Custom Instructions</h3>
    <p>Customize how AI responds to you</p>
  </div>
  
  <div class="instructions-form">
    <div class="form-section">
      <label for="responseStyle">
        <span class="label-text">Response Style</span>
        <span class="label-hint">How should I communicate?</span>
      </label>
      <textarea
        id="responseStyle"
        bind:value={instructions.responseStyle}
        placeholder="e.g., Be concise and technical, avoid jargon"
        rows="3"
      ></textarea>
      <div class="examples">
        <span class="examples-label">Examples:</span>
        {#each examples.responseStyle as example}
          <button 
            class="example-btn"
            on:click={() => instructions.responseStyle = example}
          >
            {example}
          </button>
        {/each}
      </div>
    </div>
    
    <div class="form-section">
      <label for="codePreferences">
        <span class="label-text">Code Preferences</span>
        <span class="label-hint">Your coding style and standards</span>
      </label>
      <textarea
        id="codePreferences"
        bind:value={instructions.codePreferences}
        placeholder="e.g., Always use TypeScript, include error handling, follow SOLID principles"
        rows="3"
      ></textarea>
      <div class="examples">
        <span class="examples-label">Examples:</span>
        {#each examples.codePreferences as example}
          <button 
            class="example-btn"
            on:click={() => instructions.codePreferences = example}
          >
            {example}
          </button>
        {/each}
      </div>
    </div>
    
    <div class="form-section">
      <label for="outputFormat">
        <span class="label-text">Output Format</span>
        <span class="label-hint">How should I structure responses?</span>
      </label>
      <textarea
        id="outputFormat"
        bind:value={instructions.outputFormat}
        placeholder="e.g., Use tables for comparisons, include code examples"
        rows="3"
      ></textarea>
      <div class="examples">
        <span class="examples-label">Examples:</span>
        {#each examples.outputFormat as example}
          <button 
            class="example-btn"
            on:click={() => instructions.outputFormat = example}
          >
            {example}
          </button>
        {/each}
      </div>
    </div>
    
    <div class="form-section">
      <label for="customRules">
        <span class="label-text">Custom Rules</span>
        <span class="label-hint">Additional preferences or requirements</span>
      </label>
      <textarea
        id="customRules"
        bind:value={instructions.customRules}
        placeholder="e.g., Always cite sources, ask clarifying questions, suggest alternatives"
        rows="4"
      ></textarea>
    </div>
    
    <div class="form-actions">
      <button class="reset-btn" on:click={resetInstructions}>
        Reset to Default
      </button>
      <button class="save-btn" on:click={saveInstructions}>
        💾 Save Instructions
      </button>
    </div>
    
    {#if showSavedMessage}
      <div class="saved-message">
        ✅ Instructions saved successfully!
      </div>
    {/if}
  </div>
</div>

<style>
  .custom-instructions {
    padding: 1.5rem;
    max-width: 800px;
    margin: 0 auto;
  }
  
  .instructions-header {
    margin-bottom: 2rem;
  }
  
  .instructions-header h3 {
    font-size: 1.5rem;
    margin-bottom: 0.5rem;
    color: #e4e4e7;
  }
  
  .instructions-header p {
    color: #a1a1aa;
  }
  
  .instructions-form {
    display: flex;
    flex-direction: column;
    gap: 2rem;
  }
  
  .form-section {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  
  label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  
  .label-text {
    font-size: 1rem;
    font-weight: 600;
    color: #e4e4e7;
  }
  
  .label-hint {
    font-size: 0.875rem;
    color: #a1a1aa;
  }
  
  textarea {
    padding: 0.75rem 1rem;
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    color: #e4e4e7;
    font-size: 0.875rem;
    font-family: inherit;
    line-height: 1.5;
    resize: vertical;
  }
  
  textarea:focus {
    outline: none;
    border-color: #60a5fa;
  }
  
  .examples {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    align-items: center;
  }
  
  .examples-label {
    font-size: 0.75rem;
    color: #71717a;
    font-weight: 600;
  }
  
  .example-btn {
    padding: 0.25rem 0.75rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 4px;
    color: #a1a1aa;
    font-size: 0.75rem;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .example-btn:hover {
    background: rgba(96, 165, 250, 0.15);
    border-color: rgba(96, 165, 250, 0.3);
    color: #89b4fa;
  }
  
  .form-actions {
    display: flex;
    gap: 1rem;
    justify-content: flex-end;
    margin-top: 1rem;
  }
  
  .reset-btn, .save-btn {
    padding: 0.75rem 1.5rem;
    border-radius: 8px;
    font-size: 0.875rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .reset-btn {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: #a1a1aa;
  }
  
  .reset-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #e4e4e7;
  }
  
  .save-btn {
    background: rgba(96, 165, 250, 0.2);
    border: 1px solid rgba(96, 165, 250, 0.3);
    color: #60a5fa;
  }
  
  .save-btn:hover {
    background: rgba(96, 165, 250, 0.3);
  }
  
  .saved-message {
    padding: 1rem;
    background: rgba(166, 227, 161, 0.15);
    border: 1px solid rgba(166, 227, 161, 0.3);
    border-radius: 8px;
    color: #a6e3a1;
    text-align: center;
    font-size: 0.875rem;
    font-weight: 600;
    animation: slideIn 0.3s ease-out;
  }
  
  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateY(-10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
