/**
 * Typed API client for SpatialVortex backend
 * v0.8.4 - ParallelFusion API
 */

import type {
  ChatRequest,
  ChatResponse,
  Conversation,
} from '$lib/types/chat';
import type {
  CompressionRequest,
  CompressionResponse,
  DecompressionResult,
} from '$lib/types/compression';

// SpatialVortex version
export const SPATIALVORTEX_VERSION = '0.8.4';
export const SPATIALVORTEX_CODENAME = 'Ensemble Fusion';

export class SpatialVortexAPI {
  private baseUrl: string;
  private defaultHeaders: HeadersInit;

  constructor(baseUrl?: string) {
    // v0.8.4: ParallelFusion API Server runs on port 7000
    this.baseUrl = baseUrl || import.meta.env.VITE_API_URL || 'http://localhost:7000';
    this.defaultHeaders = {
      'Content-Type': 'application/json',
    };
  }

  /**
   * Send a chat message and get AI response
   */
  async chat(request: ChatRequest): Promise<ChatResponse> {
    const response = await fetch(`${this.baseUrl}/api/chat`, {
      method: 'POST',
      headers: this.defaultHeaders,
      body: JSON.stringify(request),
    });

    if (!response.ok) {
      throw new Error(`Chat failed: ${response.status} ${response.statusText}`);
    }

    return response.json();
  }

  /**
   * Compress text to 12-byte hash
   */
  async compress(request: CompressionRequest): Promise<CompressionResponse> {
    const response = await fetch(`${this.baseUrl}/api/compress`, {
      method: 'POST',
      headers: this.defaultHeaders,
      body: JSON.stringify(request),
    });

    if (!response.ok) {
      throw new Error(`Compression failed: ${response.status}`);
    }

    return response.json();
  }

  /**
   * Decompress hash back to structured data
   */
  async decompress(hash: string): Promise<DecompressionResult> {
    const response = await fetch(`${this.baseUrl}/api/decompress`, {
      method: 'POST',
      headers: this.defaultHeaders,
      body: JSON.stringify({ hash }),
    });

    if (!response.ok) {
      throw new Error(`Decompression failed: ${response.status}`);
    }

    return response.json();
  }

  /**
   * List available AI models
   */
  async listModels(): Promise<Array<{ id: string; name: string; size: string }>> {
    const response = await fetch(`${this.baseUrl}/api/models`, {
      headers: this.defaultHeaders,
    });

    if (!response.ok) {
      throw new Error(`Failed to list models: ${response.status}`);
    }

    return response.json();
  }

  /**
   * Get conversation history
   */
  async getConversations(): Promise<Conversation[]> {
    const response = await fetch(`${this.baseUrl}/api/conversations`, {
      headers: this.defaultHeaders,
    });

    if (!response.ok) {
      throw new Error(`Failed to get conversations: ${response.status}`);
    }

    return response.json();
  }

  /**
   * Save a conversation
   */
  async saveConversation(conversation: Conversation): Promise<{ id: string }> {
    const response = await fetch(`${this.baseUrl}/api/conversations`, {
      method: 'POST',
      headers: this.defaultHeaders,
      body: JSON.stringify(conversation),
    });

    if (!response.ok) {
      throw new Error(`Failed to save conversation: ${response.status}`);
    }

    return response.json();
  }

  /**
   * Health check
   */
  async health(): Promise<{ status: string; backend: string; version: string }> {
    const response = await fetch(`${this.baseUrl}/health`, {
      headers: this.defaultHeaders,
    });

    if (!response.ok) {
      throw new Error(`Health check failed: ${response.status}`);
    }

    return response.json();
  }

  /**
   * v0.8.4: ParallelFusion Unified API
   * Use this for the new 97-99% accuracy Ensemble fusion
   */
  async processUnified(request: Partial<import('$lib/types/chat').UnifiedRequest>): Promise<import('$lib/types/chat').UnifiedResponse> {
    const fullRequest: import('$lib/types/chat').UnifiedRequest = {
      input: request.input || '',
      mode: request.mode || 'Balanced',
      sacred_only: request.sacred_only ?? false,
      min_confidence: request.min_confidence || 0.6,
      ...request,
    };

    const response = await fetch(`${this.baseUrl}/api/v1/process`, {
      method: 'POST',
      headers: this.defaultHeaders,
      body: JSON.stringify(fullRequest),
    });

    if (!response.ok) {
      throw new Error(`ParallelFusion processing failed: ${response.status} ${response.statusText}`);
    }

    return response.json();
  }
}

// Export singleton instance
export const api = new SpatialVortexAPI();
