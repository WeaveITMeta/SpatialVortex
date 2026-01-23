<script lang="ts">
  import { api } from '$lib/api/client';
  import type { CompressionHash, DecompressionResult } from '$lib/types';
  
  // Component props
  interface Props {
    hash?: string;
    showDetails?: boolean;
  }
  
  let { hash = '', showDetails = false }: Props = $props();
  
  // Component state
  let decompressed = $state<DecompressionResult | null>(null);
  let isDecompressing = $state<boolean>(false);
  let error = $state<string>('');
  
  // Parse hash into components
  let hashComponents = $derived(hash ? parseHash(hash) : null);
  
  function parseHash(hashString: string): CompressionHash | null {
    if (hashString.length !== 24) return null; // 12 bytes = 24 hex chars
    
    try {
      const bytes = hexToBytes(hashString);
      return {
        hash: hashString,
        size: 12,
        who: [bytes[0]!, bytes[1]!],
        what: [bytes[2]!, bytes[3]!],
        where: [bytes[4]!, bytes[5]!],
        tensor: [bytes[6]!, bytes[7]!],
        color: bytes[8]!,
        attributes: [bytes[9]!, bytes[10]!, bytes[11]!],
      };
    } catch {
      return null;
    }
  }
  
  function hexToBytes(hex: string): number[] {
    const bytes: number[] = [];
    for (let i = 0; i < hex.length; i += 2) {
      bytes.push(parseInt(hex.slice(i, i + 2), 16));
    }
    return bytes;
  }
  
  function byteToHex(byte: number): string {
    return byte.toString(16).padStart(2, '0');
  }
  
  async function decompress(): Promise<void> {
    if (!hash) return;
    
    isDecompressing = true;
    error = '';
    
    try {
      decompressed = await api.decompress(hash);
    } catch (err) {
      error = err instanceof Error ? err.message : 'Decompression failed';
      decompressed = null;
    } finally {
      isDecompressing = false;
    }
  }
  
  // Calculate color from ELP if decompressed
  let elpColor = $derived(decompressed ? {
    r: Math.round(decompressed.elp_channels.pathos * 255 / 9),
    g: Math.round(decompressed.elp_channels.logos * 255 / 9),
    b: Math.round(decompressed.elp_channels.ethos * 255 / 9),
  } : null);
</script>

<div class="compression-display">
  <div class="hash-header">
    <span class="hash-label">Compression Hash</span>
    {#if hash}
      <span class="hash-size">12 bytes (833:1 compression)</span>
    {/if}
  </div>
  
  {#if hash}
    <div class="hash-value" title={hash}>
      {hash}
    </div>
    
    {#if showDetails && hashComponents}
      <div class="hash-breakdown">
        <div class="component-row">
          <span class="component-label">WHO (2B):</span>
          <span class="component-value">
            {byteToHex(hashComponents.who[0])} {byteToHex(hashComponents.who[1])}
          </span>
          <span class="component-desc">User ID</span>
        </div>
        <div class="component-row">
          <span class="component-label">WHAT (2B):</span>
          <span class="component-value">
            {byteToHex(hashComponents.what[0])} {byteToHex(hashComponents.what[1])}
          </span>
          <span class="component-desc">Subject seed</span>
        </div>
        <div class="component-row">
          <span class="component-label">WHERE (2B):</span>
          <span class="component-value">
            {byteToHex(hashComponents.where[0])} {byteToHex(hashComponents.where[1])}
          </span>
          <span class="component-desc">Position & depth</span>
        </div>
        <div class="component-row">
          <span class="component-label">TENSOR (2B):</span>
          <span class="component-value">
            {byteToHex(hashComponents.tensor[0])} {byteToHex(hashComponents.tensor[1])}
          </span>
          <span class="component-desc">ELP channels</span>
        </div>
        <div class="component-row">
          <span class="component-label">COLOR (1B):</span>
          <span class="component-value">
            {byteToHex(hashComponents.color)}
          </span>
          <span class="component-desc">RGB color</span>
        </div>
        <div class="component-row">
          <span class="component-label">ATTRS (3B):</span>
          <span class="component-value">
            {byteToHex(hashComponents.attributes[0])} 
            {byteToHex(hashComponents.attributes[1])} 
            {byteToHex(hashComponents.attributes[2])}
          </span>
          <span class="component-desc">Metadata</span>
        </div>
      </div>
      
      <button 
        onclick={() => void decompress()}
        disabled={isDecompressing}
        class="decompress-btn"
      >
        {isDecompressing ? 'Decompressing...' : 'Decompress'}
      </button>
    {/if}
    
    {#if error}
      <div class="error-message">{error}</div>
    {/if}
    
    {#if decompressed}
      <div class="decompressed-result">
        <h4>Decompressed Data:</h4>
        <div class="result-grid">
          <div class="result-item">
            <span class="result-label">Subject:</span>
            <span class="result-value">{decompressed.subject}</span>
          </div>
          <div class="result-item">
            <span class="result-label">Position:</span>
            <span class="result-value">{decompressed.position}</span>
          </div>
          <div class="result-item">
            <span class="result-label">Depth:</span>
            <span class="result-value">{decompressed.depth}</span>
          </div>
          <div class="result-item">
            <span class="result-label">Confidence:</span>
            <span class="result-value">{(decompressed.confidence * 100).toFixed(1)}%</span>
          </div>
          {#if elpColor}
            <div 
              class="result-item elp-display"
              style:background-color="rgb({elpColor.r}, {elpColor.g}, {elpColor.b})"
            >
              <span class="result-label">ELP:</span>
              <span class="result-value">
                E:{decompressed.elp_channels.ethos.toFixed(1)} 
                L:{decompressed.elp_channels.logos.toFixed(1)} 
                P:{decompressed.elp_channels.pathos.toFixed(1)}
              </span>
            </div>
          {/if}
        </div>
      </div>
    {/if}
  {:else}
    <div class="no-hash">
      <span>No hash to display</span>
    </div>
  {/if}
</div>

<style>
  .compression-display {
    background-color: var(--md-surface);
    border-radius: var(--md-radius-md);
    padding: var(--md-spacing-lg);
    box-shadow: var(--md-elevation-2);
    font-family: var(--md-font-family-mono);
    transition: box-shadow var(--md-transition-base);
  }
  
  .compression-display:hover {
    box-shadow: var(--md-elevation-3);
  }
  
  .hash-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--md-spacing-md);
    padding-bottom: var(--md-spacing-sm);
    border-bottom: 1px solid rgba(255, 255, 255, 0.12);
  }
  
  .hash-label {
    font-weight: 500;
    color: var(--md-primary);
    font-size: 1.125rem;
    letter-spacing: 0.0125em;
  }
  
  .hash-size {
    font-size: 0.875rem;
    color: var(--md-on-surface-variant);
  }
  
  .hash-value {
    padding: var(--md-spacing-md);
    background-color: var(--md-surface-variant);
    border-radius: var(--md-radius-sm);
    word-break: break-all;
    color: var(--md-secondary);
    font-size: 0.9rem;
    letter-spacing: 0.05em;
    border: 1px solid rgba(3, 218, 198, 0.3);
    box-shadow: var(--md-elevation-1);
  }
  
  .hash-breakdown {
    margin-top: var(--md-spacing-lg);
    display: flex;
    flex-direction: column;
    gap: var(--md-spacing-xs);
  }
  
  .component-row {
    display: grid;
    grid-template-columns: 100px 150px 1fr;
    gap: var(--md-spacing-md);
    padding: var(--md-spacing-sm);
    background-color: rgba(255, 255, 255, 0.03);
    border-radius: var(--md-radius-sm);
    font-size: 0.875rem;
    transition: background-color var(--md-transition-fast);
  }
  
  .component-row:hover {
    background-color: var(--md-surface-variant);
  }
  
  .component-label {
    color: var(--md-on-surface-variant);
    font-weight: 500;
  }
  
  .component-value {
    color: var(--md-secondary);
    font-weight: 500;
  }
  
  .component-desc {
    color: var(--md-on-surface-variant);
    opacity: 0.7;
  }
  
  .decompress-btn {
    width: 100%;
    margin-top: var(--md-spacing-lg);
    padding: 0 var(--md-spacing-md);
    height: 48px;
    background-color: var(--md-primary);
    color: var(--md-on-primary);
    border: none;
    border-radius: var(--md-radius-sm);
    font-family: var(--md-font-family);
    font-size: 0.875rem;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.0892857143em;
    cursor: pointer;
    transition: all var(--md-transition-fast);
    box-shadow: var(--md-elevation-2);
  }
  
  .decompress-btn:hover:not(:disabled) {
    background-color: var(--md-primary-variant);
    box-shadow: var(--md-elevation-4);
  }
  
  .decompress-btn:active:not(:disabled) {
    box-shadow: var(--md-elevation-1);
  }
  
  .decompress-btn:disabled {
    opacity: 0.38;
    cursor: not-allowed;
  }
  
  .error-message {
    margin-top: var(--md-spacing-md);
    padding: var(--md-spacing-md);
    background-color: rgba(207, 102, 121, 0.1);
    border: 1px solid var(--md-error);
    border-radius: var(--md-radius-sm);
    color: var(--md-error);
    font-size: 0.875rem;
  }
  
  .decompressed-result {
    margin-top: var(--md-spacing-lg);
    padding: var(--md-spacing-md);
    background-color: rgba(0, 200, 83, 0.05);
    border: 1px solid var(--md-success);
    border-radius: var(--md-radius-md);
    box-shadow: var(--md-elevation-1);
  }
  
  .decompressed-result h4 {
    margin: 0 0 var(--md-spacing-md) 0;
    color: var(--md-success);
    font-weight: 500;
    font-size: 1rem;
  }
  
  .result-grid {
    display: grid;
    gap: var(--md-spacing-sm);
  }
  
  .result-item {
    display: flex;
    justify-content: space-between;
    padding: var(--md-spacing-sm) var(--md-spacing-md);
    background-color: var(--md-surface-variant);
    border-radius: var(--md-radius-sm);
    transition: background-color var(--md-transition-fast);
  }
  
  .result-item:hover {
    background-color: rgba(255, 255, 255, 0.08);
  }
  
  .result-label {
    color: var(--md-on-surface-variant);
    font-weight: 500;
  }
  
  .result-value {
    color: var(--md-on-surface);
    font-weight: 500;
  }
  
  .elp-display {
    color: #000;
    font-weight: 500;
  }
  
  .elp-display .result-label,
  .elp-display .result-value {
    color: #000;
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
  }
  
  .no-hash {
    padding: var(--md-spacing-2xl);
    text-align: center;
    color: var(--md-on-surface-variant);
    font-size: 0.875rem;
  }
</style>
