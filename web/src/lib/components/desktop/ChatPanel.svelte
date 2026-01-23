<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { ChatMessage } from '$lib/types/chat';
  import MessageBubble from './MessageBubble.svelte';
  import CustomInstructions from './CustomInstructions.svelte';
  import PromptTemplates from './PromptTemplates.svelte';
  import DocumentUpload from './DocumentUpload.svelte';
  import CodeExecutor from './CodeExecutor.svelte';
  import VoiceControls from './VoiceControls.svelte';
  import CollaborationPanel from './CollaborationPanel.svelte';
  import TaskList from './TaskList.svelte';
  import { taskStore } from '$lib/stores/taskStore';
  
  const dispatch = createEventDispatcher();
  
  export let messages: ChatMessage[] = [];
  export let isGenerating = false; // External state from parent
  
  let inputText = '';
  let isLoading = false;
  let chatContainer: HTMLDivElement;
  let textareaElement: HTMLTextAreaElement;
  let previousMessagesLength = 0;
  let showSettings = false;
  let showTemplates = false;
  let showDocuments = false;
  let showCodeExecutor = false;
  let showVoiceControls = false;
  let showCollaboration = false;
  let showTaskList = false;
  let isThinking = false;
  let thinkingStatus = 'Analyzing your question...';
  let uploadedDocuments: any[] = [];
  let isDragOver = false;
  let fileInputElement: HTMLInputElement;
  let isRecording = false;
  let mediaRecorder: MediaRecorder | null = null;
  let audioChunks: Blob[] = [];
  
  // Reset loading state when messages array is cleared (new session)
  $: {
    if (messages.length === 0 && previousMessagesLength > 0) {
      isLoading = false;
    }
    previousMessagesLength = messages.length;
  }
  
  function handleFollowUp(suggestion: string) {
    inputText = suggestion;
    sendMessage();
  }
  
  function startConversation(prompt: string) {
    inputText = prompt;
    sendMessage();
  }
  
  function handleSaveInstructions(instructions: any) {
    // Store in localStorage (already done by component)
    // Could also send to backend API here
    console.log('Custom instructions saved:', instructions);
    showSettings = false;
  }
  
  function handleUseTemplate(event: CustomEvent<string>) {
    inputText = event.detail;
    showTemplates = false;
  }
  
  function handleDocumentUpload(event: CustomEvent<any>) {
    const doc = event.detail;
    uploadedDocuments = [...uploadedDocuments, doc];
    showDocuments = false;
    
    // Optionally add a message about the uploaded document
    const userMessage: ChatMessage = {
      id: Date.now().toString(),
      role: 'user',
      content: `📄 Uploaded document: ${doc.filename} (${doc.chunks_created} chunks)`,
      timestamp: new Date(),
    };
    messages = [...messages, userMessage];
    dispatch('newMessage', userMessage);
  }
  
  function exportToMarkdown() {
    if (messages.length === 0) {
      alert('No messages to export');
      return;
    }
    
    let markdown = '# Chat Export\n\n';
    markdown += `**Date**: ${new Date().toLocaleString()}\n\n`;
    markdown += '---\n\n';
    
    messages.forEach((msg) => {
      const role = msg.role === 'user' ? '**You**' : (msg.role === 'vortex' ? '**Vortex**' : (msg.model_name ? `**${msg.model_name}**` : '**AI**'));
      const time = msg.timestamp.toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit' });
      
      markdown += `### ${role} (${time})\n\n`;
      markdown += `${msg.content}\n\n`;
      
      // Add sources if present
      if (msg.sources && msg.sources.length > 0) {
        markdown += `**Sources:**\n\n`;
        msg.sources.forEach((source, idx) => {
          if (source.web_source) {
            markdown += `${idx + 1}. [${source.web_source.title}](${source.web_source.url})\n`;
          }
        });
        markdown += '\n';
      }
      
      markdown += '---\n\n';
    });
    
    // Download file
    const blob = new Blob([markdown], { type: 'text/markdown' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `chat-export-${Date.now()}.md`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }
  
  async function sendMessage() {
    if (!inputText.trim() || isLoading || isGenerating) return;
    
    const text = inputText;
    inputText = '';
    isLoading = true;
    
    // Show thinking indicator
    isThinking = true;
    thinkingStatus = 'Analyzing your question...';
    
    // Simulate thinking phases
    setTimeout(() => {
      if (isThinking) thinkingStatus = 'Structuring response...';
    }, 500);
    
    setTimeout(() => {
      if (isThinking) thinkingStatus = 'Gathering context...';
    }, 1000);
    
    // Add user message immediately
    const userMessage: ChatMessage = {
      id: Date.now().toString(),
      role: 'user',
      content: text,
      timestamp: new Date(),
    };
    messages = [...messages, userMessage];
    dispatch('newMessage', userMessage);
    
    try {
      // Use multi-model endpoint
      const response = await fetch('http://localhost:7000/api/v1/chat/dual-response', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          message: text,
        }),
      });
      
      if (!response.ok) throw new Error(`API error: ${response.status}`);
      
      // Parse multi-model response
      const data = await response.json();
      console.log('🔍 ChatPanel received data:', data);
      
      // Hide thinking indicator
      isThinking = false;
      
      // Add each individual model response as separate bubbles
      if (data.model_responses && Array.isArray(data.model_responses)) {
        data.model_responses.forEach((modelResp: any, index: number) => {
          const modelMessage: ChatMessage = {
            id: (Date.now() + index + 1).toString(),
            role: 'assistant',
            model_name: modelResp.model_name,
            content: modelResp.text,
            timestamp: new Date(),
            confidence: modelResp.confidence / 100,
          };
          messages = [...messages, modelMessage];
        });
      }
      
      // Add Vortex consensus (orange bubble)
      if (data.vortex_consensus) {
        const vortexMessage: ChatMessage = {
          id: (Date.now() + (data.model_responses?.length || 0) + 1).toString(),
          role: 'vortex',
          content: data.vortex_consensus.text,
          timestamp: new Date(),
          confidence: data.vortex_consensus.confidence / 100,
          flux_position: data.vortex_consensus.flux_position,
        };
        messages = [...messages, vortexMessage];
      }
      
      scrollToBottom();
      
      // Dispatch events for all new messages
      const newMessages = messages.slice(-((data.model_responses?.length || 0) + 1));
      newMessages.forEach(msg => {
        dispatch('newMessage', msg);
      });
    } catch (err) {
      console.error('Chat error:', err);
      isThinking = false;
    } finally {
      isLoading = false;
      isThinking = false;
      scrollToBottom();
    }
  }
  
  function scrollToBottom() {
    setTimeout(() => {
      if (chatContainer) {
        chatContainer.scrollTop = chatContainer.scrollHeight;
      }
    }, 0);
  }
  
  // Auto-resize textarea based on content
  function autoResizeTextarea() {
    if (textareaElement) {
      // Reset height to get accurate scrollHeight
      textareaElement.style.height = 'auto';
      
      // Calculate new height (max 200px)
      const newHeight = Math.min(textareaElement.scrollHeight, 200);
      textareaElement.style.height = `${newHeight}px`;
    }
  }
  
  // Auto-resize on input change
  $: if (inputText !== undefined) {
    autoResizeTextarea();
  }
  
  // Auto-scroll to bottom when messages change
  $: {
    if (messages.length > 0) {
      scrollToBottom();
    }
  }
  // Auto-scroll effect for isThinking
  $: if (isThinking) {
    scrollToBottom();
  }
  $: if (!isThinking) {
    setTimeout(() => {
      scrollToBottom();
    }, 100);
  }
  
  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  }
  
  // File upload handlers
  function handleAttachClick() {
    fileInputElement?.click();
  }
  
  function handleFileSelect(event: Event) {
    const input = event.target as HTMLInputElement;
    if (input.files && input.files[0]) {
      processFile(input.files[0]);
    }
  }
  
  function handleDrop(event: DragEvent) {
    event.preventDefault();
    isDragOver = false;
    
    if (event.dataTransfer?.files && event.dataTransfer.files[0]) {
      processFile(event.dataTransfer.files[0]);
    }
  }
  
  function handleDragOver(event: DragEvent) {
    event.preventDefault();
    isDragOver = true;
  }
  
  function handleDragLeave(event: DragEvent) {
    event.preventDefault();
    isDragOver = false;
  }
  
  async function processFile(file: File) {
    const maxSize = 50 * 1024 * 1024; // 50MB total safety
    const inlineMax = 1024 * 1024;     // 1MB for inline embedding
    
    if (file.size > maxSize) {
      alert('File too large. Maximum size is 50MB');
      return;
    }
    
    const extension = (file.name.split('.').pop() || '').toLowerCase();
    const textLike = new Set([
      'txt','md','markdown','json','yaml','yml','toml','ini','csv',
      'js','ts','jsx','tsx','py','rs','go','java','c','cpp','h','hpp','cs','rb','php','swift','kt','kts','scala','sql','html','css','sh','bash','ps1'
    ]);
    
    const extToLang: Record<string,string> = {
      txt: 'plaintext', md: 'markdown', markdown: 'markdown', json: 'json', yaml: 'yaml', yml: 'yaml', toml: 'toml', ini: 'ini', csv: 'plaintext',
      js: 'javascript', jsx: 'javascript', ts: 'typescript', tsx: 'typescript', py: 'python', rs: 'rust', go: 'go', java: 'java', c: 'c', cpp: 'cpp',
      h: 'cpp', hpp: 'cpp', cs: 'csharp', rb: 'ruby', php: 'php', swift: 'swift', kt: 'kotlin', kts: 'kotlin', scala: 'scala', sql: 'sql',
      html: 'html', css: 'css', sh: 'bash', bash: 'bash', ps1: 'powershell'
    };
    
    // Always keep an entry of the uploaded document
    const doc = { name: file.name, size: file.size, type: file.type, file, uploadedAt: new Date() };
    uploadedDocuments = [...uploadedDocuments, doc];
    
    // If it's text-like and small enough, embed inline as fenced code block
    if (textLike.has(extension) && file.size <= inlineMax) {
      const reader = new FileReader();
      reader.onload = () => {
        const text = typeof reader.result === 'string' ? reader.result : '';
        const lang = extToLang[extension] || 'plaintext';
        // Escape triple backticks to avoid fence termination inside content
        const safe = text.replace(/```/g, '\u200B```');
        const snippet = `\n\n**File:** ${file.name}\n\n\`\`\`${lang}\n${safe}\n\`\`\``;
        inputText = inputText ? `${inputText}${snippet}` : snippet.trimStart();
        autoResizeTextarea();
      };
      reader.readAsText(file);
      return;
    }
    
    // Otherwise, just note attachment inline
    inputText = inputText ? `${inputText}\n\n📎 Attached: ${file.name}` : `📎 Attached: ${file.name}`;
    autoResizeTextarea();
  }
  
  // Voice recording functions
  async function toggleVoiceRecording() {
    if (isRecording) {
      stopRecording();
    } else {
      startRecording();
    }
  }
  
  async function startRecording() {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      mediaRecorder = new MediaRecorder(stream);
      audioChunks = [];
      
      mediaRecorder.ondataavailable = (event) => {
        if (event.data.size > 0) {
          audioChunks.push(event.data);
        }
      };
      
      mediaRecorder.onstop = async () => {
        const audioBlob = new Blob(audioChunks, { type: 'audio/wav' });
        await transcribeAudio(audioBlob);
        
        // Stop all tracks
        stream.getTracks().forEach(track => track.stop());
      };
      
      mediaRecorder.start();
      isRecording = true;
    } catch (error) {
      console.error('Error accessing microphone:', error);
      alert('Could not access microphone. Please check permissions.');
    }
  }
  
  function stopRecording() {
    if (mediaRecorder && isRecording) {
      mediaRecorder.stop();
      isRecording = false;
    }
  }
  
  async function transcribeAudio(audioBlob: Blob) {
    try {
      // Convert blob to base64
      const reader = new FileReader();
      reader.readAsDataURL(audioBlob);
      
      reader.onloadend = async () => {
        const base64Audio = reader.result?.toString().split(',')[1];
        
        if (!base64Audio) {
          throw new Error('Failed to encode audio');
        }
        
        // Send to backend
        const response = await fetch('http://localhost:7000/api/v1/voice/transcribe', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            audio_data: base64Audio,
            language: 'en',
            timestamps: false
          })
        });
        
        if (!response.ok) {
          throw new Error(`Transcription failed: ${response.statusText}`);
        }
        
        const result = await response.json();
        
        // Add transcribed text to input
        if (result.text) {
          inputText = inputText ? `${inputText} ${result.text}` : result.text;
          autoResizeTextarea();
        }
      };
    } catch (error) {
      console.error('Transcription error:', error);
      alert('Failed to transcribe audio. Please try again.');
    }
  }
</script>

<main class="chat-panel">
  <header class="chat-header">
    <div>
      <h1 class="chat-title">Chat</h1>
      <p class="chat-subtitle">AI powered by advanced algorithms and machine learning</p>
    </div>
    <div class="header-actions">
      <button class="icon-btn" on:click={() => showTaskList = true} title="Task Tracker">
        🧠
      </button>
      <button class="icon-btn" on:click={() => showCollaboration = true} title="Collaboration">
        👥
      </button>
      <button class="icon-btn" on:click={() => showVoiceControls = true} title="Voice Controls">
        🎤
      </button>
      <button class="icon-btn" on:click={() => showCodeExecutor = true} title="Code Executor">
        💻
      </button>
      <button class="icon-btn" on:click={() => showDocuments = true} title="Upload Document">
        📄
      </button>
      <button class="icon-btn" on:click={exportToMarkdown} title="Export to Markdown" disabled={messages.length === 0}>
        📥
      </button>
      <button class="icon-btn" on:click={() => showTemplates = true} title="Prompt Templates">
        📋
      </button>
      <button class="icon-btn" on:click={() => showSettings = true} title="Settings">
        ⚙️
      </button>
    </div>
  </header>
  
  <!-- Task List Modal -->
  {#if showTaskList}
    <div class="modal-overlay" on:click={() => showTaskList = false}>
      <div class="modal-content" on:click|stopPropagation>
        <button class="modal-close" on:click={() => showTaskList = false}>✕</button>
        <TaskList 
          tasks={$taskStore}
          on:statusChange={(e) => taskStore.updateStatus(e.detail.id, e.detail.status)}
        />
      </div>
    </div>
  {/if}
  
  <!-- Collaboration Modal -->
  {#if showCollaboration}
    <div class="modal-overlay" on:click={() => showCollaboration = false}>
      <div class="modal-content" on:click|stopPropagation>
        <button class="modal-close" on:click={() => showCollaboration = false}>✕</button>
        <CollaborationPanel 
          sessionId=""
          username="User"
        />
      </div>
    </div>
  {/if}
  
  <!-- Voice Controls Modal -->
  {#if showVoiceControls}
    <div class="modal-overlay" on:click={() => showVoiceControls = false}>
      <div class="modal-content" on:click|stopPropagation>
        <button class="modal-close" on:click={() => showVoiceControls = false}>✕</button>
        <VoiceControls 
          lastAIResponse={messages.filter(m => m.role === 'assistant').slice(-1)[0]?.content || ''}
          on:voiceMessage={(e) => { inputText = e.detail.content; sendMessage(); }}
        />
      </div>
    </div>
  {/if}
  
  <!-- Code Executor Modal -->
  {#if showCodeExecutor}
    <div class="modal-overlay" on:click={() => showCodeExecutor = false}>
      <div class="modal-content modal-large" on:click|stopPropagation>
        <button class="modal-close" on:click={() => showCodeExecutor = false}>✕</button>
        <CodeExecutor />
      </div>
    </div>
  {/if}
  
  <!-- Documents Upload Modal -->
  {#if showDocuments}
    <div class="modal-overlay" on:click={() => showDocuments = false}>
      <div class="modal-content" on:click|stopPropagation>
        <button class="modal-close" on:click={() => showDocuments = false}>✕</button>
        <DocumentUpload on:upload={handleDocumentUpload} />
      </div>
    </div>
  {/if}
  
  <!-- Settings Modal -->
  {#if showSettings}
    <div class="modal-overlay" on:click={() => showSettings = false}>
      <div class="modal-content" on:click|stopPropagation>
        <button class="modal-close" on:click={() => showSettings = false}>✕</button>
        <CustomInstructions onSave={handleSaveInstructions} />
      </div>
    </div>
  {/if}
  
  <!-- Templates Modal -->
  {#if showTemplates}
    <div class="modal-overlay" on:click={() => showTemplates = false}>
      <div class="modal-content" on:click|stopPropagation>
        <button class="modal-close" on:click={() => showTemplates = false}>✕</button>
        <PromptTemplates on:use={handleUseTemplate} />
      </div>
    </div>
  {/if}
  
  <div class="messages" bind:this={chatContainer}>
    {#if messages.length === 0}
      <div class="empty-state">
        <div class="empty-icon">✨</div>
        <h2>Welcome to The Vortex</h2>
        <p>Experience AI powered chat with advancements in natural language processing, machine learning, reinforcement learning and neural networks.</p>
        <div class="quick-starts">
          <button on:click={() => startConversation("What is consciousness?")}>
            🧠 What is consciousness?
          </button>
          <button on:click={() => startConversation("Technical deep dive.")}>
            🔬 Technical Deep Dive
          </button>
          <button on:click={() => startConversation("Compare Wikipedia to Grokipedia, show a table.")}>
            📊 Compare Wikipedia to Grokipedia
          </button>
          <button on:click={() => startConversation("Create a response demonstrating ALL your formatting capabilities: tables, code blocks, citations, images, callouts, and task lists.")}>
            ✨ Formatting Showcase
          </button>
          <button on:click={() => startConversation("Maintain context across a long conversation with 10+ topics, then recall details from the beginning.")}>
            🎯 Context Preservation Test
          </button>
          <button on:click={() => startConversation("What can you do for me?")}>
            💬 What can you do for me?
          </button>
        </div>
      </div>
    {:else}
      {#each messages as message, index (message.id)}
        <MessageBubble 
          {message} 
          showFollowUps={index === messages.length - 1 && message.role === 'assistant'}
          on:followup={(e) => handleFollowUp(e.detail)}
        />
      {/each}
      
      {#if isThinking}
        <div class="thinking-indicator">
          <div class="thinking-bubble">
            <div class="thinking-header">
              <span class="thinking-icon">🧠</span>
              <span class="thinking-label">Thinking...</span>
            </div>
            <div class="thinking-status">{thinkingStatus}</div>
            <div class="thinking-dots">
              <span class="dot"></span>
              <span class="dot"></span>
              <span class="dot"></span>
            </div>
          </div>
        </div>
      {/if}
    {/if}
  </div>
  
  <div 
    class="input-container"
    class:drag-over={isDragOver}
    on:drop={handleDrop}
    on:dragover={handleDragOver}
    on:dragleave={handleDragLeave}
  >
    <input
      type="file"
      bind:this={fileInputElement}
      on:change={handleFileSelect}
      accept=".pdf,.docx,.xlsx,.xls,.txt,.md,.markdown,.json,.yaml,.yml,.toml,.ini,.csv,.js,.jsx,.ts,.tsx,.py,.rs,.go,.java,.c,.cpp,.h,.hpp,.cs,.rb,.php,.swift,.kt,.kts,.scala,.sql,.html,.css,.sh,.bash,.ps1"
      style="display: none;"
    />
    
    <button 
      class="attach-btn" 
      on:click={handleAttachClick}
      title="Attach document (PDF, DOCX, Excel, Text)"
      disabled={isLoading || isGenerating}
    >
      📎
    </button>
    
    <textarea
      bind:this={textareaElement}
      bind:value={inputText}
      on:keydown={handleKeyDown}
      on:input={autoResizeTextarea}
      placeholder={isDragOver ? "Drop file here..." : isGenerating ? "Generating response..." : "Ask me anything..."}
      rows="1"
      disabled={isLoading || isGenerating}
      class="message-input"
    ></textarea>
    
    <button 
      class="voice-btn" 
      class:recording={isRecording}
      on:click={toggleVoiceRecording}
      title={isRecording ? "Stop recording" : "Voice input"}
      disabled={isLoading || isGenerating}
    >
      {#if isRecording}
        ⏹️
      {:else}
        🎤
      {/if}
    </button>
    
    <button
      on:click={sendMessage}
      disabled={!inputText.trim() || isLoading || isGenerating}
      class="send-btn"
      class:generating={isGenerating}
      title={isGenerating ? "Generating response..." : "Send message (Enter)"}
    >
      {#if isLoading || isGenerating}
        <span class="spinner">⚡</span>
      {:else}
        ➤
      {/if}
    </button>
  </div>
</main>

<style>
  .chat-panel {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: linear-gradient(180deg, #0f0f1a 0%, #141420 100%);
  }
  
  .chat-header {
    padding: 1.5rem 2rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  
  .chat-title {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 700;
    background: linear-gradient(135deg, #60a5fa 0%, #3b82f6 100%);
    -webkit-background-clip: text;
    background-clip: text;
    -webkit-text-fill-color: transparent;
  }
  
  .chat-subtitle {
    margin: 0.25rem 0 0 0;
    font-size: 0.875rem;
    color: #71717a;
  }
  
  .header-actions {
    display: flex;
    gap: 0.5rem;
  }
  
  .icon-btn {
    width: 2.5rem;
    height: 2.5rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    font-size: 1.25rem;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  
  .icon-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    transform: translateY(-1px);
  }
  
  .messages {
    flex: 1;
    overflow-y: auto;
    padding: 2rem;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }
  
  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: 3rem;
  }
  
  .empty-icon {
    font-size: 4rem;
    margin-bottom: 1.5rem;
    animation: float 3s ease-in-out infinite;
  }
  
  @keyframes float {
    0%, 100% { transform: translateY(0px); }
    50% { transform: translateY(-10px); }
  }
  
  .empty-state h2 {
    font-size: 1.75rem;
    margin: 0 0 0.75rem 0;
    color: #e4e4e7;
    font-weight: 600;
  }
  
  .empty-state p {
    color: #a1a1aa;
    margin: 0 0 2rem 0;
    max-width: 600px;
    line-height: 1.6;
  }
  
  .quick-starts {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 1rem;
    max-width: 900px;
    width: 100%;
  }
  
  .quick-starts button {
    padding: 1rem 1.5rem;
    background: rgba(96, 165, 250, 0.1);
    border: 1px solid rgba(96, 165, 250, 0.2);
    border-radius: 12px;
    color: #e4e4e7;
    cursor: pointer;
    transition: all 0.2s;
    font-size: 0.9rem;
    font-weight: 500;
    text-align: left;
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }
  
  .quick-starts button:hover {
    background: linear-gradient(135deg, rgba(96, 165, 250, 0.15) 0%, rgba(59, 130, 246, 0.15) 100%);
    border-color: rgba(96, 165, 250, 0.4);
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(96, 165, 250, 0.2);
  }
  
  .input-container {
    padding: 1.5rem 2rem;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
    display: flex;
    gap: 1rem;
    align-items: flex-end;
    background: rgba(255, 255, 255, 0.02);
    transition: background 0.3s;
  }
  
  .input-container.drag-over {
    background: rgba(96, 165, 250, 0.1);
    border-color: rgba(96, 165, 250, 0.4);
  }
  
  .attach-btn {
    flex-shrink: 0;
    width: 52px;
    height: 52px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    color: #e4e4e7;
    font-size: 1.5rem;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .attach-btn:hover:not(:disabled) {
    background: rgba(96, 165, 250, 0.15);
    border-color: #60a5fa;
    transform: scale(1.05);
  }
  
  .attach-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  
  .voice-btn {
    flex-shrink: 0;
    width: 52px;
    height: 52px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    color: #e4e4e7;
    font-size: 1.5rem;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .voice-btn:hover:not(:disabled) {
    background: rgba(96, 165, 250, 0.15);
    border-color: #60a5fa;
    transform: scale(1.05);
  }
  
  .voice-btn.recording {
    background: rgba(239, 68, 68, 0.2);
    border-color: #ef4444;
    animation: pulse 1.5s ease-in-out infinite;
  }
  
  .voice-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  
  @keyframes pulse {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.6;
    }
  }
  
  .message-input {
    flex: 1;
    height: 52px;
    min-height: 52px;
    max-height: 200px;
    padding: 1rem 1.25rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    color: #e4e4e7;
    font-family: inherit;
    font-size: 0.95rem;
    line-height: 1.5;
    resize: none;
    transition: border-color 0.2s, background 0.2s, box-shadow 0.2s;
    overflow-y: auto;
  }
  
  .message-input:focus {
    outline: none;
    border-color: #60a5fa;
    background: rgba(255, 255, 255, 0.08);
    box-shadow: 0 0 0 3px rgba(96, 165, 250, 0.1);
  }
  
  .message-input::placeholder {
    color: #52525b;
  }
  
  .message-input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  
  .send-btn {
    width: 52px;
    height: 52px;
    background: linear-gradient(135deg, #60a5fa 0%, #3b82f6 100%);
    border: none;
    border-radius: 12px;
    color: white;
    font-size: 1.25rem;
    cursor: pointer;
    transition: all 0.2s;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  
  .send-btn:hover:not(:disabled) {
    transform: translateY(-2px) scale(1.05);
    box-shadow: 0 8px 20px rgba(96, 165, 250, 0.3);
  }
  
  .send-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
    transform: none;
  }
  
  .send-btn.generating {
    background: linear-gradient(135deg, #3b82f6 0%, #60a5fa 100%);
  }
  
  .spinner {
    animation: spin 1s linear infinite;
    display: inline-block;
  }
  
  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }
  
  .messages::-webkit-scrollbar {
    width: 8px;
  }
  
  .messages::-webkit-scrollbar-track {
    background: transparent;
  }
  
  .messages::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 4px;
  }
  
  .messages::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.15);
  }
  
  /* Modal Styles */
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    animation: fadeIn 0.2s ease-out;
  }
  
  .modal-content {
    background: linear-gradient(180deg, #1a1a2e 0%, #16161f 100%);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 16px;
    max-width: 600px;
    width: 90%;
    max-height: 80vh;
    overflow-y: auto;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
    position: relative;
    animation: slideUp 0.3s ease-out;
  }
  
  .modal-content.modal-large {
    max-width: 1000px;
    width: 95%;
    max-height: 90vh;
    padding: 0;
    overflow: hidden;
  }
  
  .modal-close {
    position: absolute;
    top: 1rem;
    right: 1rem;
    width: 2.5rem;
    height: 2.5rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    color: #a1a1aa;
    font-size: 1.25rem;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1;
  }
  
  .modal-close:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #e4e4e7;
    transform: rotate(90deg);
  }
  
  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
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
  
  .modal-content::-webkit-scrollbar {
    width: 8px;
  }
  
  .modal-content::-webkit-scrollbar-track {
    background: transparent;
  }
  
  .modal-content::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 4px;
  }
  
  /* Thinking Indicator */
  .thinking-indicator {
    display: flex;
    justify-content: flex-start;
    padding: 1rem 2rem;
    animation: fadeIn 0.3s ease-out;
  }
  
  .thinking-bubble {
    background: linear-gradient(135deg, rgba(96, 165, 250, 0.1) 0%, rgba(59, 130, 246, 0.05) 100%);
    border: 1px solid rgba(96, 165, 250, 0.2);
    border-radius: 16px;
    padding: 1rem 1.5rem;
    max-width: 400px;
    box-shadow: 0 4px 12px rgba(96, 165, 250, 0.1);
  }
  
  .thinking-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
  }
  
  .thinking-icon {
    font-size: 1.25rem;
    animation: pulse 2s ease-in-out infinite;
  }
  
  .thinking-label {
    font-weight: 600;
    color: #60a5fa;
    font-size: 0.9rem;
  }
  
  .thinking-status {
    color: #a1a1aa;
    font-size: 0.85rem;
    margin-bottom: 0.75rem;
  }
  
  .thinking-dots {
    display: flex;
    gap: 0.375rem;
    justify-content: center;
  }
  
  .dot {
    width: 6px;
    height: 6px;
    background: #60a5fa;
    border-radius: 50%;
    animation: bounce 1.4s ease-in-out infinite;
  }
  
  .dot:nth-child(1) {
    animation-delay: 0s;
  }
  
  .dot:nth-child(2) {
    animation-delay: 0.2s;
  }
  
  .dot:nth-child(3) {
    animation-delay: 0.4s;
  }
  
  @keyframes pulse {
    0%, 100% {
      transform: scale(1);
      opacity: 1;
    }
    50% {
      transform: scale(1.1);
      opacity: 0.8;
    }
  }
  
  @keyframes bounce {
    0%, 80%, 100% {
      transform: translateY(0);
    }
    40% {
      transform: translateY(-8px);
    }
  }
</style>
