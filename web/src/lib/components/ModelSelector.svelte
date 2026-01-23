<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api/client';
  
  // Component props
  interface Props {
    selectedModel?: string;
    onModelChange?: (modelId: string) => void;
  }
  
  let { selectedModel = 'llama2', onModelChange }: Props = $props();
  
  // State
  interface Model {
    id: string;
    name: string;
    size: string;
  }
  
  let models = $state<Model[]>([]);
  let isLoading = $state<boolean>(true);
  let error = $state<string>('');
  let currentModel = $state<string>(selectedModel);
  
  onMount(async (): Promise<void> => {
    try {
      models = await api.listModels();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to load models';
    } finally {
      isLoading = false;
    }
  });
  
  function handleChange(event: Event): void {
    const target = event.target as HTMLSelectElement;
    currentModel = target.value;
    onModelChange?.(currentModel);
  }
</script>

<div class="model-selector">
  <label for="model-select">AI Model</label>
  {#if isLoading}
    <select class="md-input" disabled>
      <option>Loading models...</option>
    </select>
  {:else if error}
    <div class="error">{error}</div>
  {:else}
    <select 
      id="model-select"
      class="md-input"
      value={currentModel}
      onchange={handleChange}
    >
      {#each models as model (model.id)}
        <option value={model.id}>
          {model.name} ({model.size})
        </option>
      {/each}
    </select>
  {/if}
</div>

<style>
  .model-selector {
    display: flex;
    align-items: center;
    gap: var(--md-spacing-md);
    padding: var(--md-spacing-sm) var(--md-spacing-md);
    background-color: var(--md-surface-variant);
    border-radius: var(--md-radius-sm);
  }
  
  label {
    font-weight: 500;
    color: var(--md-on-surface);
  }
  
  select {
    flex: 1;
    cursor: pointer;
    font-family: var(--md-font-family);
  }
  
  select:disabled {
    opacity: 0.38;
    cursor: not-allowed;
  }
  
  .error {
    color: var(--md-error);
    font-size: 0.9rem;
  }
</style>
