/**
 * WebTransport Client for SpatialVortex Chat
 * 
 * Uses QUIC protocol (HTTP/3, UDP-based) for:
 * - Lower latency than WebSocket (20-40ms vs 50-100ms)
 * - Independent streams (no head-of-line blocking)
 * - Better multiplexing (100+ concurrent streams)
 * - 0-RTT reconnection
 * - Native TLS 1.3 encryption
 */

export interface QuestionRequest {
	type: 'question';
	content: string;
	include_reasoning: boolean;
	include_visualization: boolean;
}

export interface InferenceStartMessage {
	type: 'inference_start';
	message_id: string;
	stream_id: number;
}

export interface ReasoningStep {
	step_number: number;
	description: string;
	details: string;
	value?: number;
}

export interface ReasoningStepMessage {
	type: 'reasoning_step';
	message_id: string;
	stream_id: number;
	step: ReasoningStep;
}

export interface ELPValues {
	ethos: number;
	logos: number;
	pathos: number;
}

export interface FluxData {
	position: number;
	sequence: number[];
	elp: ELPValues;
}

export interface VisualizationUpdateMessage {
	type: 'visualization_update';
	message_id: string;
	stream_id: number;
	flux_data: FluxData;
}

export interface AnswerMetadata {
	confidence: number;
	position: number;
	elp: ELPValues;
	flux_sequence: number[];
	reasoning_steps: ReasoningStep[];
	latency_ms: number;
}

export interface AnswerCompleteMessage {
	type: 'answer_complete';
	message_id: string;
	stream_id: number;
	content: string;
	metadata: AnswerMetadata;
}

export interface ErrorMessage {
	type: 'error';
	message_id: string;
	error: string;
}

export type ServerMessage =
	| InferenceStartMessage
	| ReasoningStepMessage
	| VisualizationUpdateMessage
	| AnswerCompleteMessage
	| ErrorMessage;

export interface WebTransportCallbacks {
	onInferenceStart?: (msg: InferenceStartMessage) => void;
	onReasoningStep?: (msg: ReasoningStepMessage) => void;
	onVisualizationUpdate?: (msg: VisualizationUpdateMessage) => void;
	onAnswerComplete?: (msg: AnswerCompleteMessage) => void;
	onError?: (msg: ErrorMessage) => void;
}

export class SpatialVortexWebTransport {
	private transport: WebTransport | null = null;
	private stream: {
		writer: WritableStreamDefaultWriter<Uint8Array>;
		reader: ReadableStreamDefaultReader<Uint8Array>;
	} | null = null;
	private callbacks: WebTransportCallbacks = {};
	private encoder = new TextEncoder();
	private decoder = new TextDecoder();

	constructor(private url: string) {}

	/**
	 * Connect to WebTransport server
	 */
	async connect(): Promise<void> {
		try {
			console.log(`🔌 Connecting to WebTransport: ${this.url}`);

			// Create WebTransport connection (QUIC)
			this.transport = new WebTransport(this.url);

			// Wait for connection to be ready
			await this.transport.ready;
			console.log('✅ WebTransport connected via QUIC!');

			// Create bidirectional stream for messages
			const bidiStream = await this.transport.createBidirectionalStream();
			this.stream = {
				writer: bidiStream.writable.getWriter(),
				reader: bidiStream.readable.getReader()
			};

			console.log('✅ Bidirectional stream created');

			// Start reading responses
			this.startReading();
		} catch (error) {
			console.error('❌ WebTransport connection failed:', error);
			throw error;
		}
	}

	/**
	 * Send question to server
	 */
	async sendQuestion(
		content: string,
		includeReasoning: boolean = true,
		includeVisualization: boolean = true
	): Promise<void> {
		if (!this.stream) {
			throw new Error('Not connected. Call connect() first.');
		}

		const question: QuestionRequest = {
			type: 'question',
			content,
			include_reasoning: includeReasoning,
			include_visualization: includeVisualization
		};

		const json = JSON.stringify(question);
		const bytes = this.encoder.encode(json);

		console.log(`📤 Sending question (${bytes.length} bytes):`, content);
		await this.stream.writer.write(bytes);
	}

	/**
	 * Start reading server messages
	 */
	private async startReading(): Promise<void> {
		if (!this.stream) return;

		try {
			while (true) {
				const { value, done } = await this.stream.reader.read();

				if (done) {
					console.log('📭 Stream closed by server');
					break;
				}

				// Decode message
				const json = this.decoder.decode(value);
				const message: ServerMessage = JSON.parse(json);

				console.log(`📥 Received message:`, message.type);

				// Dispatch to callbacks
				this.handleMessage(message);
			}
		} catch (error) {
			console.error('❌ Error reading from stream:', error);
		}
	}

	/**
	 * Handle incoming server message
	 */
	private handleMessage(message: ServerMessage): void {
		switch (message.type) {
			case 'inference_start':
				this.callbacks.onInferenceStart?.(message);
				break;

			case 'reasoning_step':
				this.callbacks.onReasoningStep?.(message);
				break;

			case 'visualization_update':
				this.callbacks.onVisualizationUpdate?.(message);
				break;

			case 'answer_complete':
				this.callbacks.onAnswerComplete?.(message);
				break;

			case 'error':
				this.callbacks.onError?.(message);
				break;

			default:
				console.warn('⚠️ Unknown message type:', (message as any).type);
		}
	}

	/**
	 * Set event callbacks
	 */
	setCallbacks(callbacks: WebTransportCallbacks): void {
		this.callbacks = { ...this.callbacks, ...callbacks };
	}

	/**
	 * Close connection
	 */
	async close(): Promise<void> {
		if (this.stream) {
			await this.stream.writer.close();
			this.stream = null;
		}

		if (this.transport) {
			this.transport.close();
			this.transport = null;
		}

		console.log('👋 WebTransport connection closed');
	}

	/**
	 * Check if connected
	 */
	isConnected(): boolean {
		return this.transport !== null && this.stream !== null;
	}
}

/**
 * Create WebTransport client instance
 */
export function createWebTransportClient(
	serverUrl: string = 'https://localhost:4433/wt/chat'
): SpatialVortexWebTransport {
	return new SpatialVortexWebTransport(serverUrl);
}
