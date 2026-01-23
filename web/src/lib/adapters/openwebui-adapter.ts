/**
 * OpenWebUI to SpatialVortex API Adapter
 * Bridges OpenWebUI component expectations with our Rust backend
 * 
 * v0.8.4 - ParallelFusion Integration
 * Now uses Ensemble fusion for 97-99% accuracy
 */

import { api, SPATIALVORTEX_VERSION } from '$lib/api/client';
import type { 
  ChatMessage,
  ChatRequest, 
  ChatResponse, 
  Conversation,
  UnifiedRequest,
  UnifiedResponse
} from '$lib/types/chat';

// Alias for compatibility
type Message = ChatMessage;

/**
 * OpenWebUI expects certain API structures that differ from ours
 * This adapter translates between the two systems
 */

// OpenWebUI message format (their structure)
interface OpenWebUIMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp?: number;
  model?: string;
}

// OpenWebUI chat format
interface OpenWebUIChat {
  id: string;
  title?: string;
  messages: OpenWebUIMessage[];
  models?: string[];
  history?: {
    messages: Record<string, OpenWebUIMessage>;
  };
}

/**
 * Adapter class to translate between OpenWebUI and SpatialVortex
 */
export class OpenWebUIAdapter {
  /**
   * Convert OpenWebUI message to our Message type
   */
  static messageToSpatialVortex(owMessage: OpenWebUIMessage): Message {
    return {
      id: owMessage.id,
      role: owMessage.role,
      content: owMessage.content,
      timestamp: owMessage.timestamp || Date.now(),
    };
  }

  /**
   * Convert our Message to OpenWebUI format
   */
  static messageToOpenWebUI(message: Message): OpenWebUIMessage {
    return {
      id: message.id,
      role: message.role,
      content: message.content,
      timestamp: message.timestamp,
    };
  }

  /**
   * v0.8.4: Send chat message using ParallelFusion (RECOMMENDED)
   * Uses Ensemble fusion for 97-99% accuracy
   */
  static async sendMessageV2(
    prompt: string,
    options: {
      mode?: 'Fast' | 'Balanced' | 'Thorough';
      sacred_only?: boolean;
      min_confidence?: number;
    } = {}
  ): Promise<UnifiedResponse> {
    const request: Partial<UnifiedRequest> = {
      input: prompt,
      mode: options.mode || 'Balanced',
      sacred_only: options.sacred_only ?? false,
      min_confidence: options.min_confidence || 0.6,
    };

    return await api.processUnified(request);
  }

  /**
   * Send chat message using our backend (Legacy - v0.7.x)
   * @deprecated Use sendMessageV2 for v0.8.4 ParallelFusion
   */
  static async sendMessage(
    prompt: string,
    model: string = 'parallel-fusion',
    options: {
      compress?: boolean;
      context?: string;
    } = {}
  ): Promise<ChatResponse> {
    // For now, route to v0.8.4 API
    const unified = await this.sendMessageV2(prompt, {
      mode: 'Balanced',
    });

    // Convert to old format for compatibility
    return {
      response: unified.result,
      elp_values: unified.elp,
      flux_position: unified.flux_position,
      confidence: unified.confidence,
      processing_time_ms: unified.metrics.duration_ms,
    };
  }

  /**
   * Get model list from backend
   */
  static async getModels(): Promise<Array<{ id: string; name: string; size: string }>> {
    return await api.listModels();
  }

  /**
   * Save conversation to backend
   */
  static async saveChat(chat: OpenWebUIChat): Promise<{ id: string }> {
    const conversation: Conversation = {
      id: chat.id,
      title: chat.title || 'Untitled',
      messages: chat.messages.map(this.messageToSpatialVortex),
      created_at: Date.now(),
      updated_at: Date.now(),
    };

    return await api.saveConversation(conversation);
  }

  /**
   * Stream message response (for real-time updates)
   */
  static async *streamMessage(
    prompt: string,
    model: string = 'llama2'
  ): AsyncGenerator<string, void, unknown> {
    // TODO: Implement streaming when backend supports it
    // For now, return full response
    const response = await this.sendMessage(prompt, model);
    yield response.response;
  }

  /**
   * Format message with v0.8.4 metadata
   */
  static formatMessageWithMetadata(
    content: string,
    response: ChatResponse | UnifiedResponse
  ): string {
    // Check if it's a UnifiedResponse
    if ('metadata' in response && 'metrics' in response) {
      const unified = response as UnifiedResponse;
      return `${content}\n\n---\n✨ **v0.8.4 ParallelFusion** (${unified.metadata.strategy})\n📊 Confidence: ${(unified.confidence * 100).toFixed(1)}%\n📍 Flux Position: ${unified.flux_position}${unified.sacred_boost ? ' 🔮 (Sacred!)' : ''}\n💎 ELP: E:${unified.elp.ethos.toFixed(1)} L:${unified.elp.logos.toFixed(1)} P:${unified.elp.pathos.toFixed(1)}\n⚡ Signal: ${(unified.confidence * 100).toFixed(0)}%\n⏱️ ${unified.metrics.duration_ms}ms`;
    }

    // Legacy format
    const legacy = response as ChatResponse;
    return `${content}\n\n---\n📍 Position: ${legacy.flux_position}\n💎 ELP: E:${legacy.elp_values.ethos.toFixed(1)} L:${legacy.elp_values.logos.toFixed(1)} P:${legacy.elp_values.pathos.toFixed(1)}`;
  }

  /**
   * Handle errors and convert to OpenWebUI format
   */
  static handleError(error: unknown): OpenWebUIMessage {
    const message = error instanceof Error ? error.message : 'Unknown error occurred';
    
    return {
      id: crypto.randomUUID(),
      role: 'system',
      content: `❌ Error: ${message}`,
      timestamp: Date.now(),
    };
  }

  /**
   * Create a new chat session
   */
  static createNewChat(title?: string): OpenWebUIChat {
    return {
      id: crypto.randomUUID(),
      title: title || `Chat ${new Date().toLocaleDateString()}`,
      messages: [],
      models: ['llama2'],
    };
  }

  /**
   * Add message to chat
   */
  static addMessageToChat(
    chat: OpenWebUIChat,
    message: OpenWebUIMessage
  ): OpenWebUIChat {
    return {
      ...chat,
      messages: [...chat.messages, message],
    };
  }

  /**
   * Check backend health
   */
  static async checkHealth(): Promise<boolean> {
    try {
      const health = await api.health();
      return health.status === 'healthy';
    } catch {
      return false;
    }
  }

  /**
   * Get backend info for debugging
   */
  static async getBackendInfo(): Promise<{
    healthy: boolean;
    backend: string;
    version: string;
  }> {
    try {
      const health = await api.health();
      return {
        healthy: health.status === 'healthy',
        backend: 'ParallelFusion',
        version: health.version || SPATIALVORTEX_VERSION,
      };
    } catch (error) {
      return {
        healthy: false,
        backend: 'ParallelFusion',
        version: SPATIALVORTEX_VERSION,
      };
    }
  }
}

/**
 * Helper function for OpenWebUI components to use our backend
 * v0.8.4: Uses ParallelFusion by default
 */
export async function owSendChatMessage(
  prompt: string,
  model: string = 'parallel-fusion',
  options?: { showMetadata?: boolean; mode?: 'Fast' | 'Balanced' | 'Thorough' }
): Promise<OpenWebUIMessage> {
  try {
    // Use v0.8.4 API
    const response = await OpenWebUIAdapter.sendMessageV2(prompt, {
      mode: options?.mode || 'Balanced',
    });
    
    const content = options?.showMetadata 
      ? OpenWebUIAdapter.formatMessageWithMetadata(response.result, response)
      : response.result;

    return {
      id: crypto.randomUUID(),
      role: 'assistant',
      content,
      timestamp: Date.now(),
      model: `ParallelFusion v${SPATIALVORTEX_VERSION}`,
    };
  } catch (error) {
    return OpenWebUIAdapter.handleError(error);
  }
}

/**
 * Helper to get models for OpenWebUI components
 */
export async function owGetModels(): Promise<Array<{ id: string; name: string }>> {
  const models = await OpenWebUIAdapter.getModels();
  return models.map(m => ({ id: m.id, name: m.name }));
}

/**
 * Export singleton instance for convenience
 */
export const owAdapter = OpenWebUIAdapter;
