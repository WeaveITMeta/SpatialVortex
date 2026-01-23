<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { createWebTransportClient } from '$lib/services/webtransport';
	import type { ServerMessage } from '$lib/services/webtransport';

	let client = createWebTransportClient('https://localhost:4433/wt/chat');
	let connected = false;
	let question = 'What is consciousness?';
	let messages: ServerMessage[] = [];
	let error = '';

	onMount(async () => {
		// Set up callbacks
		client.setCallbacks({
			onInferenceStart: (msg) => {
				console.log('🚀 Inference started:', msg.message_id);
				messages = [...messages, msg];
			},
			onReasoningStep: (msg) => {
				console.log('🧠 Reasoning step:', msg.step.description);
				messages = [...messages, msg];
			},
			onVisualizationUpdate: (msg) => {
				console.log('🎨 Visualization update:', msg.flux_data);
				messages = [...messages, msg];
			},
			onAnswerComplete: (msg) => {
				console.log('✅ Answer complete:', msg.content);
				messages = [...messages, msg];
			},
			onError: (msg) => {
				console.error('❌ Error:', msg.error);
				error = msg.error;
				messages = [...messages, msg];
			}
		});
	});

	onDestroy(() => {
		if (connected) {
			client.close();
		}
	});

	async function connect() {
		try {
			error = '';
			await client.connect();
			connected = true;
		} catch (err) {
			error = `Connection failed: ${err}`;
			console.error(err);
		}
	}

	async function sendQuestion() {
		if (!connected) {
			error = 'Not connected. Click Connect first.';
			return;
		}

		try {
			error = '';
			messages = [];
			await client.sendQuestion(question, true, true);
		} catch (err) {
			error = `Send failed: ${err}`;
			console.error(err);
		}
	}

	async function disconnect() {
		await client.close();
		connected = false;
		messages = [];
	}
</script>

<div class="container">
	<h1>🌀 WebTransport (QUIC) Test</h1>
	<p class="subtitle">Testing SpatialVortex HTTP/3 + QUIC connection</p>

	<div class="info-box">
		<h3>📋 Protocol Information</h3>
		<ul>
			<li><strong>Protocol:</strong> HTTP/3 + QUIC</li>
			<li><strong>Transport:</strong> UDP (not TCP)</li>
			<li><strong>Encryption:</strong> TLS 1.3 (built-in)</li>
			<li><strong>Server:</strong> https://localhost:4433/wt/chat</li>
			<li><strong>Latency:</strong> ~20-40ms (vs 50-100ms WebSocket)</li>
			<li><strong>Streams:</strong> 100+ concurrent per connection</li>
		</ul>
	</div>

	<div class="controls">
		<button on:click={connect} disabled={connected} class="btn btn-primary">
			{connected ? '✅ Connected' : '🔌 Connect'}
		</button>

		<button on:click={disconnect} disabled={!connected} class="btn btn-secondary">
			👋 Disconnect
		</button>
	</div>

	{#if connected}
		<div class="question-box">
			<label for="question">Ask a Question:</label>
			<input
				id="question"
				type="text"
				bind:value={question}
				placeholder="What is consciousness?"
				class="input"
			/>
			<button on:click={sendQuestion} class="btn btn-success"> 📤 Send Question </button>
		</div>
	{/if}

	{#if error}
		<div class="error-box">
			<strong>❌ Error:</strong>
			{error}
		</div>
	{/if}

	{#if messages.length > 0}
		<div class="messages">
			<h3>📨 Messages ({messages.length})</h3>
			{#each messages as msg, i}
				<div class="message" class:inference={msg.type === 'inference_start'} class:reasoning={msg.type === 'reasoning_step'} class:visualization={msg.type === 'visualization_update'} class:answer={msg.type === 'answer_complete'} class:error={msg.type === 'error'}>
					<div class="message-header">
						<span class="message-type">{msg.type}</span>
						<span class="message-id">{msg.message_id}</span>
					</div>

					{#if msg.type === 'reasoning_step'}
						<div class="message-content">
							<strong>Step {msg.step.step_number}:</strong>
							{msg.step.description}
							<br />
							<small>{msg.step.details}</small>
						</div>
					{:else if msg.type === 'visualization_update'}
						<div class="message-content">
							<strong>Position:</strong>
							{msg.flux_data.position}
							<br />
							<strong>ELP:</strong> E={msg.flux_data.elp.ethos.toFixed(2)}, L={msg.flux_data.elp
								.logos.toFixed(2)}, P={msg.flux_data.elp.pathos.toFixed(2)}
						</div>
					{:else if msg.type === 'answer_complete'}
						<div class="message-content">
							<strong>Answer:</strong>
							{msg.content}
							<br />
							<small>Confidence: {(msg.metadata.confidence * 100).toFixed(0)}%</small>
						</div>
					{:else if msg.type === 'error'}
						<div class="message-content">
							{msg.error}
						</div>
					{/if}
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.container {
		max-width: 900px;
		margin: 2rem auto;
		padding: 2rem;
		font-family: 'Inter', sans-serif;
	}

	h1 {
		font-size: 2.5rem;
		margin-bottom: 0.5rem;
		color: #3b82f6;
	}

	.subtitle {
		color: #6b7280;
		margin-bottom: 2rem;
	}

	.info-box {
		background: #f9fafb;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		padding: 1.5rem;
		margin-bottom: 2rem;
	}

	.info-box h3 {
		margin-top: 0;
		color: #1f2937;
	}

	.info-box ul {
		list-style: none;
		padding: 0;
	}

	.info-box li {
		padding: 0.5rem 0;
		border-bottom: 1px solid #e5e7eb;
	}

	.info-box li:last-child {
		border-bottom: none;
	}

	.controls {
		display: flex;
		gap: 1rem;
		margin-bottom: 2rem;
	}

	.btn {
		padding: 0.75rem 1.5rem;
		border: none;
		border-radius: 6px;
		font-size: 1rem;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.2s;
	}

	.btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.btn-primary {
		background: #3b82f6;
		color: white;
	}

	.btn-primary:hover:not(:disabled) {
		background: #2563eb;
	}

	.btn-secondary {
		background: #6b7280;
		color: white;
	}

	.btn-secondary:hover:not(:disabled) {
		background: #4b5563;
	}

	.btn-success {
		background: #10b981;
		color: white;
	}

	.btn-success:hover:not(:disabled) {
		background: #059669;
	}

	.question-box {
		background: #f9fafb;
		padding: 1.5rem;
		border-radius: 8px;
		margin-bottom: 2rem;
	}

	.question-box label {
		display: block;
		margin-bottom: 0.5rem;
		font-weight: 600;
		color: #1f2937;
	}

	.input {
		width: 100%;
		padding: 0.75rem;
		border: 1px solid #d1d5db;
		border-radius: 6px;
		font-size: 1rem;
		margin-bottom: 1rem;
	}

	.error-box {
		background: #fee2e2;
		border: 1px solid #ef4444;
		color: #991b1b;
		padding: 1rem;
		border-radius: 6px;
		margin-bottom: 2rem;
	}

	.messages {
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		padding: 1.5rem;
	}

	.messages h3 {
		margin-top: 0;
		margin-bottom: 1rem;
		color: #1f2937;
	}

	.message {
		border: 1px solid #e5e7eb;
		border-radius: 6px;
		padding: 1rem;
		margin-bottom: 1rem;
	}

	.message:last-child {
		margin-bottom: 0;
	}

	.message.inference {
		border-left: 4px solid #3b82f6;
		background: #eff6ff;
	}

	.message.reasoning {
		border-left: 4px solid #8b5cf6;
		background: #f5f3ff;
	}

	.message.visualization {
		border-left: 4px solid #ec4899;
		background: #fdf2f8;
	}

	.message.answer {
		border-left: 4px solid #10b981;
		background: #f0fdf4;
	}

	.message.error {
		border-left: 4px solid #ef4444;
		background: #fef2f2;
	}

	.message-header {
		display: flex;
		justify-content: space-between;
		margin-bottom: 0.5rem;
	}

	.message-type {
		font-weight: 600;
		text-transform: uppercase;
		font-size: 0.75rem;
		color: #6b7280;
	}

	.message-id {
		font-size: 0.75rem;
		color: #9ca3af;
		font-family: monospace;
	}

	.message-content {
		color: #1f2937;
		line-height: 1.6;
	}

	.message-content small {
		color: #6b7280;
	}
</style>
