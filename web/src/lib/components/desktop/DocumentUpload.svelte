<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  
  const dispatch = createEventDispatcher();
  
  let selectedFile: File | null = null;
  let uploading = false;
  let uploadProgress = 0;
  let uploadedDocument: any = null;
  let error: string | null = null;
  let dragOver = false;
  
  const acceptedTypes = '.pdf,.docx,.xlsx,.xls';
  const maxSize = 50 * 1024 * 1024; // 50MB
  
  function handleFileSelect(event: Event) {
    const input = event.target as HTMLInputElement;
    if (input.files && input.files[0]) {
      selectFile(input.files[0]);
    }
  }
  
  function selectFile(file: File) {
    error = null;
    
    // Check file size
    if (file.size > maxSize) {
      error = `File too large. Maximum size is 50MB`;
      return;
    }
    
    // Check file type
    const extension = file.name.split('.').pop()?.toLowerCase();
    if (!['pdf', 'docx', 'xlsx', 'xls'].includes(extension || '')) {
      error = `Unsupported file type. Please upload PDF, DOCX, or Excel files.`;
      return;
    }
    
    selectedFile = file;
  }
  
  async function uploadDocument() {
    if (!selectedFile) return;
    
    uploading = true;
    uploadProgress = 0;
    error = null;
    
    try {
      const formData = new FormData();
      formData.append('file', selectedFile);
      
      const response = await fetch('http://localhost:7000/api/v1/rag/documents/upload', {
        method: 'POST',
        body: formData,
      });
      
      if (!response.ok) {
        throw new Error(`Upload failed: ${response.statusText}`);
      }
      
      const result = await response.json();
      
      if (result.success) {
        uploadedDocument = result;
        dispatch('upload', result);
        
        // Reset after success
        setTimeout(() => {
          selectedFile = null;
          uploadedDocument = null;
        }, 3000);
      } else {
        throw new Error(result.error || 'Upload failed');
      }
    } catch (err) {
      error = err instanceof Error ? err.message : 'Upload failed';
    } finally {
      uploading = false;
      uploadProgress = 0;
    }
  }
  
  function handleDrop(event: DragEvent) {
    event.preventDefault();
    dragOver = false;
    
    if (event.dataTransfer?.files && event.dataTransfer.files[0]) {
      selectFile(event.dataTransfer.files[0]);
    }
  }
  
  function handleDragOver(event: DragEvent) {
    event.preventDefault();
    dragOver = true;
  }
  
  function handleDragLeave() {
    dragOver = false;
  }
  
  function clearFile() {
    selectedFile = null;
    error = null;
    uploadedDocument = null;
  }
</script>

<div class="document-upload">
  <h3 class="title">📄 Upload Document</h3>
  <p class="subtitle">Chat with PDF, Word, or Excel files</p>
  
  {#if !selectedFile && !uploadedDocument}
    <div 
      class="drop-zone"
      class:drag-over={dragOver}
      on:drop={handleDrop}
      on:dragover={handleDragOver}
      on:dragleave={handleDragLeave}
    >
      <div class="drop-icon">📁</div>
      <p class="drop-text">Drag and drop your file here</p>
      <p class="drop-or">or</p>
      <label class="file-button">
        <input 
          type="file" 
          accept={acceptedTypes}
          on:change={handleFileSelect}
          style="display: none;"
        />
        Choose File
      </label>
      <p class="file-types">Supports: PDF, Word (.docx), Excel (.xlsx)</p>
      <p class="file-size">Maximum size: 50MB</p>
    </div>
  {/if}
  
  {#if selectedFile && !uploadedDocument}
    <div class="file-preview">
      <div class="file-info">
        <div class="file-icon">
          {#if selectedFile.name.endsWith('.pdf')}
            📕
          {:else if selectedFile.name.endsWith('.docx')}
            📘
          {:else}
            📊
          {/if}
        </div>
        <div class="file-details">
          <div class="file-name">{selectedFile.name}</div>
          <div class="file-size">{(selectedFile.size / 1024 / 1024).toFixed(2)} MB</div>
        </div>
        {#if !uploading}
          <button class="clear-btn" on:click={clearFile}>✕</button>
        {/if}
      </div>
      
      {#if !uploading}
        <button class="upload-btn" on:click={uploadDocument}>
          📤 Upload & Analyze
        </button>
      {:else}
        <div class="uploading">
          <div class="spinner">⚡</div>
          <span>Analyzing document...</span>
        </div>
      {/if}
    </div>
  {/if}
  
  {#if uploadedDocument}
    <div class="success-message">
      <div class="success-icon">✅</div>
      <div class="success-text">
        <strong>Document uploaded successfully!</strong>
        <p>{uploadedDocument.filename}</p>
        <p class="success-details">
          {uploadedDocument.chunks_created} chunks created • 
          {uploadedDocument.content_length.toLocaleString()} characters
        </p>
        {#if uploadedDocument.preview}
          <div class="preview">
            <strong>Preview:</strong>
            <p>{uploadedDocument.preview}...</p>
          </div>
        {/if}
      </div>
    </div>
  {/if}
  
  {#if error}
    <div class="error-message">
      <span class="error-icon">⚠️</span>
      {error}
    </div>
  {/if}
</div>

<style>
  .document-upload {
    padding: 1.5rem;
  }
  
  .title {
    margin: 0 0 0.5rem 0;
    font-size: 1.25rem;
    font-weight: 700;
    color: #e4e4e7;
  }
  
  .subtitle {
    margin: 0 0 1.5rem 0;
    font-size: 0.9rem;
    color: #a1a1aa;
  }
  
  .drop-zone {
    border: 2px dashed rgba(96, 165, 250, 0.3);
    border-radius: 12px;
    padding: 3rem 2rem;
    text-align: center;
    background: rgba(96, 165, 250, 0.05);
    transition: all 0.3s;
    cursor: pointer;
  }
  
  .drop-zone.drag-over {
    border-color: #60a5fa;
    background: rgba(96, 165, 250, 0.1);
    transform: scale(1.02);
  }
  
  .drop-icon {
    font-size: 3rem;
    margin-bottom: 1rem;
    animation: float 3s ease-in-out infinite;
  }
  
  @keyframes float {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-10px); }
  }
  
  .drop-text {
    font-size: 1.1rem;
    color: #e4e4e7;
    margin: 0 0 0.5rem 0;
  }
  
  .drop-or {
    color: #71717a;
    margin: 1rem 0;
  }
  
  .file-button {
    display: inline-block;
    padding: 0.75rem 2rem;
    background: linear-gradient(135deg, #60a5fa 0%, #3b82f6 100%);
    border-radius: 8px;
    color: white;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .file-button:hover {
    transform: translateY(-2px);
    box-shadow: 0 8px 20px rgba(96, 165, 250, 0.3);
  }
  
  .file-types {
    margin: 1rem 0 0.25rem 0;
    font-size: 0.85rem;
    color: #a1a1aa;
  }
  
  .file-size {
    font-size: 0.8rem;
    color: #71717a;
    margin: 0;
  }
  
  .file-preview {
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    padding: 1.5rem;
  }
  
  .file-info {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 1rem;
  }
  
  .file-icon {
    font-size: 2.5rem;
  }
  
  .file-details {
    flex: 1;
  }
  
  .file-name {
    font-weight: 600;
    color: #e4e4e7;
    margin-bottom: 0.25rem;
  }
  
  .file-size {
    color: #a1a1aa;
    font-size: 0.9rem;
  }
  
  .clear-btn {
    width: 2rem;
    height: 2rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    color: #a1a1aa;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .clear-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #e4e4e7;
  }
  
  .upload-btn {
    width: 100%;
    padding: 0.875rem;
    background: linear-gradient(135deg, #60a5fa 0%, #3b82f6 100%);
    border: none;
    border-radius: 8px;
    color: white;
    font-weight: 600;
    font-size: 1rem;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .upload-btn:hover {
    transform: translateY(-2px);
    box-shadow: 0 8px 20px rgba(96, 165, 250, 0.3);
  }
  
  .uploading {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.75rem;
    padding: 1rem;
    color: #60a5fa;
    font-weight: 600;
  }
  
  .spinner {
    animation: spin 1s linear infinite;
    font-size: 1.25rem;
  }
  
  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
  
  .success-message {
    display: flex;
    gap: 1rem;
    padding: 1.5rem;
    background: rgba(34, 197, 94, 0.1);
    border: 1px solid rgba(34, 197, 94, 0.3);
    border-radius: 12px;
    animation: slideIn 0.3s ease-out;
  }
  
  .success-icon {
    font-size: 2rem;
  }
  
  .success-text {
    flex: 1;
  }
  
  .success-text strong {
    color: #22c55e;
    display: block;
    margin-bottom: 0.5rem;
  }
  
  .success-text p {
    margin: 0.25rem 0;
    color: #a1a1aa;
    font-size: 0.9rem;
  }
  
  .success-details {
    font-size: 0.85rem !important;
    color: #71717a !important;
  }
  
  .preview {
    margin-top: 1rem;
    padding: 1rem;
    background: rgba(0, 0, 0, 0.2);
    border-radius: 8px;
    font-size: 0.85rem;
  }
  
  .preview strong {
    color: #e4e4e7;
    display: block;
    margin-bottom: 0.5rem;
  }
  
  .preview p {
    color: #a1a1aa;
    line-height: 1.5;
  }
  
  .error-message {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1rem;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: 8px;
    color: #ef4444;
    margin-top: 1rem;
  }
  
  .error-icon {
    font-size: 1.25rem;
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
</style>
