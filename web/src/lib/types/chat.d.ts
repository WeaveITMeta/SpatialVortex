/**
 * Chat-related type definitions for SpatialVortex WebUI
 */

export interface ELPChannels {
  ethos: number;   // Ethics/Stability (0-9)
  logos: number;   // Logic/Reasoning (0-9)
  pathos: number;  // Emotion/Passion (0-9)
}

export interface ChatRequest {
  prompt: string;
  model?: string;
  compress?: boolean;
  stream?: boolean;
  context?: string;
}

export interface ChatResponse {
  response: string;
  model: string;
  thinking_time: number;
  compressed_hash?: string;
  beam_position?: number;
  elp_channels?: ELPChannels;
  confidence?: number;
}

export interface Message {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: number;
  compressed_hash?: string;
  beam_position?: number;
  elp_channels?: ELPChannels;
}

export interface Conversation {
  id: string;
  title: string;
  messages: Message[];
  created_at: number;
  updated_at: number;
}
