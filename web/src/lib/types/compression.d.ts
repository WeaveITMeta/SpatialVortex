/**
 * Compression and hashing type definitions for 12-byte thought compression
 */

import type { ELPChannels } from './chat';

export interface CompressionHash {
  hash: string;  // Hex encoded 12-byte hash
  size: number;  // Always 12
  who: [number, number];      // 2 bytes - User ID
  what: [number, number];     // 2 bytes - Subject seed
  where: [number, number];    // 2 bytes - Position & depth
  tensor: [number, number];   // 2 bytes - ELP tensor
  color: number;              // 1 byte - RGB color
  attributes: [number, number, number];  // 3 bytes - Metadata
}

export interface CompressionRequest {
  text: string;
  context?: string;
  user_id?: string;
}

export interface CompressionResponse {
  hash: string;
  size: number;
  original_size: number;
  compression_ratio: number;
  elp_channels?: ELPChannels;
  flux_position?: number;
}

export interface DecompressionResult {
  user_id?: string;
  subject: string;
  position: number;
  depth: number;
  elp_channels: ELPChannels;
  confidence: number;
  color: { r: number; g: number; b: number };
  attributes: {
    can_replicate: boolean;
    is_diamond_moment: boolean;
    spin_rate: number;
  };
}

export interface HashComponent {
  name: 'who' | 'what' | 'where' | 'tensor' | 'color' | 'attributes';
  bytes: number[];
  description: string;
  value: string | number | object;
}

export interface HashVisualization {
  hash: string;
  components: HashComponent[];
  totalSize: number;
  compressionRatio: number;
}
