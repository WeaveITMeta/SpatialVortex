<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  
  export let show = false;
  export let sessionTitle = '';
  export let sessionId = '';
  
  const dispatch = createEventDispatcher();
  
  let shareMode: 'anyone' | 'restricted' = 'anyone';
  let allowedEmails: string[] = [];
  let newEmail = '';
  let shareLink = '';
  let isGenerating = false;
  let copied = false;
  
  async function generateShareLink() {
    isGenerating = true;
    
    try {
      const response = await fetch('http://localhost:7000/api/v1/chat/share', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          session_id: sessionId,
          share_mode: shareMode,
          allowed_emails: shareMode === 'restricted' ? allowedEmails : [],
        }),
      });
      
      if (response.ok) {
        const data = await response.json();
        shareLink = `${window.location.origin}/share/${data.share_token}`;
      } else {
        alert('Failed to generate share link');
      }
    } catch (err) {
      console.error('Share error:', err);
      alert('Failed to generate share link');
    } finally {
      isGenerating = false;
    }
  }
  
  function addEmail() {
    if (newEmail && newEmail.includes('@')) {
      allowedEmails = [...allowedEmails, newEmail];
      newEmail = '';
    }
  }
  
  function removeEmail(email: string) {
    allowedEmails = allowedEmails.filter(e => e !== email);
  }
  
  async function copyLink() {
    if (shareLink) {
      await navigator.clipboard.writeText(shareLink);
      copied = true;
      setTimeout(() => copied = false, 2000);
    }
  }
  
  function close() {
    show = false;
    dispatch('close');
  }
  
  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      close();
    }
  }
  
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      close();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if show}
  <div class="modal-backdrop" on:click={handleBackdropClick} on:keydown={handleKeydown} role="button" tabindex="-1">
    <div class="modal">
      <div class="modal-header">
        <h2>Share "{sessionTitle}"</h2>
        <button class="close-btn" on:click={close}>✕</button>
      </div>
      
      <div class="modal-content">
        <!-- Share Mode Selection -->
        <div class="section">
          <h3>Who can access?</h3>
          <div class="radio-group">
            <label class="radio-option" class:selected={shareMode === 'anyone'}>
              <input type="radio" bind:group={shareMode} value="anyone" />
              <div class="radio-content">
                <div class="radio-icon">🌐</div>
                <div class="radio-text">
                  <div class="radio-title">Anyone with the link</div>
                  <div class="radio-desc">Anyone who has the link can view</div>
                </div>
              </div>
            </label>
            
            <label class="radio-option" class:selected={shareMode === 'restricted'}>
              <input type="radio" bind:group={shareMode} value="restricted" />
              <div class="radio-content">
                <div class="radio-icon">🔒</div>
                <div class="radio-text">
                  <div class="radio-title">Restricted</div>
                  <div class="radio-desc">Only people with access can view</div>
                </div>
              </div>
            </label>
          </div>
        </div>
        
        <!-- Email List (if restricted) -->
        {#if shareMode === 'restricted'}
          <div class="section">
            <h3>Add people</h3>
            <div class="email-input-group">
              <input
                type="email"
                bind:value={newEmail}
                placeholder="Enter email address"
                on:keydown={(e) => e.key === 'Enter' && addEmail()}
              />
              <button class="add-btn" on:click={addEmail}>Add</button>
            </div>
            
            {#if allowedEmails.length > 0}
              <div class="email-list">
                {#each allowedEmails as email}
                  <div class="email-item">
                    <span>👤 {email}</span>
                    <button class="remove-btn" on:click={() => removeEmail(email)}>✕</button>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {/if}
        
        <!-- Generate Link -->
        {#if !shareLink}
          <button class="generate-btn" on:click={generateShareLink} disabled={isGenerating}>
            {isGenerating ? '⏳ Generating...' : '🔗 Generate Share Link'}
          </button>
        {:else}
          <div class="section">
            <h3>Share link</h3>
            <div class="link-container">
              <input type="text" readonly value={shareLink} class="link-input" />
              <button class="copy-btn" on:click={copyLink}>
                {copied ? '✓ Copied!' : '📋 Copy'}
              </button>
            </div>
            <p class="link-hint">🔗 This link never expires and can be shared freely</p>
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2000;
    animation: fadeIn 0.2s ease-out;
  }
  
  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }
  
  .modal {
    background: #1e1e2e;
    border: 1px solid #313244;
    border-radius: 12px;
    width: 90%;
    max-width: 500px;
    max-height: 80vh;
    overflow-y: auto;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
    animation: slideUp 0.3s ease-out;
  }
  
  @keyframes slideUp {
    from {
      opacity: 0;
      transform: translateY(20px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
  
  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5rem;
    border-bottom: 1px solid #313244;
  }
  
  .modal-header h2 {
    margin: 0;
    font-size: 1.25rem;
    color: #e4e4e7;
  }
  
  .close-btn {
    background: transparent;
    border: none;
    color: #a1a1aa;
    font-size: 1.5rem;
    cursor: pointer;
    padding: 0.25rem;
    line-height: 1;
    transition: color 0.2s;
  }
  
  .close-btn:hover {
    color: #e4e4e7;
  }
  
  .modal-content {
    padding: 1.5rem;
  }
  
  .section {
    margin-bottom: 1.5rem;
  }
  
  .section h3 {
    margin: 0 0 1rem 0;
    font-size: 0.875rem;
    color: #a1a1aa;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  
  .radio-group {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  
  .radio-option {
    display: block;
    cursor: pointer;
  }
  
  .radio-option input {
    display: none;
  }
  
  .radio-content {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1rem;
    background: rgba(255, 255, 255, 0.03);
    border: 2px solid #313244;
    border-radius: 8px;
    transition: all 0.2s;
  }
  
  .radio-option:hover .radio-content {
    background: rgba(255, 255, 255, 0.05);
    border-color: #45475a;
  }
  
  .radio-option.selected .radio-content {
    background: rgba(96, 165, 250, 0.1);
    border-color: #60a5fa;
  }
  
  .radio-icon {
    font-size: 1.5rem;
  }
  
  .radio-text {
    flex: 1;
  }
  
  .radio-title {
    font-weight: 600;
    color: #e4e4e7;
    margin-bottom: 0.25rem;
  }
  
  .radio-desc {
    font-size: 0.875rem;
    color: #a1a1aa;
  }
  
  .email-input-group {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1rem;
  }
  
  .email-input-group input {
    flex: 1;
    padding: 0.75rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid #313244;
    border-radius: 8px;
    color: #e4e4e7;
    font-size: 0.875rem;
  }
  
  .email-input-group input:focus {
    outline: none;
    border-color: #60a5fa;
    box-shadow: 0 0 0 3px rgba(96, 165, 250, 0.1);
  }
  
  .add-btn {
    padding: 0.75rem 1.5rem;
    background: #60a5fa;
    color: white;
    border: none;
    border-radius: 8px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .add-btn:hover {
    background: #3b82f6;
    transform: translateY(-1px);
  }
  
  .email-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  
  .email-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid #313244;
    border-radius: 8px;
  }
  
  .email-item span {
    color: #cdd6f4;
    font-size: 0.875rem;
  }
  
  .remove-btn {
    background: transparent;
    border: none;
    color: #a1a1aa;
    cursor: pointer;
    padding: 0.25rem 0.5rem;
    transition: color 0.2s;
  }
  
  .remove-btn:hover {
    color: #f38ba8;
  }
  
  .generate-btn {
    width: 100%;
    padding: 1rem;
    background: linear-gradient(135deg, #60a5fa 0%, #3b82f6 100%);
    color: white;
    border: none;
    border-radius: 8px;
    font-weight: 600;
    font-size: 1rem;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .generate-btn:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: 0 8px 20px rgba(96, 165, 250, 0.3);
  }
  
  .generate-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  
  .link-container {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
  }
  
  .link-input {
    flex: 1;
    padding: 0.75rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid #313244;
    border-radius: 8px;
    color: #e4e4e7;
    font-size: 0.875rem;
    font-family: 'Courier New', monospace;
  }
  
  .copy-btn {
    padding: 0.75rem 1.5rem;
    background: #60a5fa;
    color: white;
    border: none;
    border-radius: 8px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
    white-space: nowrap;
  }
  
  .copy-btn:hover {
    background: #3b82f6;
  }
  
  .link-hint {
    margin: 0;
    font-size: 0.75rem;
    color: #71717a;
  }
</style>
