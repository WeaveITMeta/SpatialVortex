// Chat-related types for SpatialVortex

export interface ELPValues {
  ethos: number;    // -13 to +13
  logos: number;    // -13 to +13
  pathos: number;   // -13 to +13
}

export interface CodeBlock {
  language: string;
  code: string;
  filename?: string;
  reasoning_steps?: number;
  complexity_score?: number;
}

export interface WebSourceMeta {
  url: string;
  title: string;
  domain: string;
  credibility_score: number;
  source_type: string;
  search_engine: string;
  published_date?: string;
  freshness_score: number;
  user_rating?: number;
  is_bookmarked: boolean;
}

export interface SourceAttribution {
  doc_id: string;
  chunk_id: string;
  relevance: number;
  content_snippet: string;
  web_source?: WebSourceMeta;
}

export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'native' | 'vortex';  // 'vortex' for Vortex consensus
  content: string;
  timestamp: Date;
  model_name?: string;  // For Ollama models (llama3.2, mixtral, codellama)
  elp?: ELPValues;
  flux_position?: number;
  confidence?: number;
  // Color ML fields
  semantic_color?: string;  // Hex color code
  primary_meaning?: string;
  related_meanings?: string[];
  color_confidence?: number;
  // Code generation fields
  code_blocks?: CodeBlock[];
  is_code_response?: boolean;
  generation_time_ms?: number;
  // Streaming state
  is_streaming?: boolean;
  // RAG sources (Phase 2)
  sources?: SourceAttribution[];
}

export interface ChatRequest {
  message: string;
  user_id: string;
  context?: string[];
}

export interface ChatResponse {
  response: string;
  elp_values: ELPValues;
  flux_position: number;
  confidence: number;
  processing_time_ms?: number;
  // Color ML fields
  semantic_color?: string;
  primary_meaning?: string;
  related_meanings?: string[];
  color_confidence?: number;
}

export interface MultimodalInput {
  text?: string;
  image?: File;
  audio?: File;
  pointcloud?: File;
  fusion_config?: FusionConfig;
}

export interface FusionConfig {
  strategy: 'average' | 'cross_attention' | 'hierarchical';
  weights?: ModalityWeights;
}

export interface ModalityWeights {
  text_weight: number;
  image_weight: number;
  audio_weight: number;
  pointcloud_weight: number;
}

export interface ModalityType {
  id: string;
  name: string;
  icon: string;
  enabled: boolean;
  description: string;
}

export const MODALITIES: ModalityType[] = [
  {
    id: 'text',
    name: 'Text',
    icon: '📝',
    enabled: true,
    description: 'Natural language input',
  },
  {
    id: 'voice',
    name: 'Voice',
    icon: '🎤',
    enabled: false,
    description: 'Speech-to-text with voice characteristics',
  },
  {
    id: 'image',
    name: 'Image',
    icon: '🖼️',
    enabled: false,
    description: 'Visual understanding with CLIP',
  },
  {
    id: 'audio',
    name: 'Audio',
    icon: '🎵',
    enabled: false,
    description: 'Acoustic embeddings with wav2vec2',
  },
  {
    id: '3d',
    name: '3D',
    icon: '🎲',
    enabled: false,
    description: '3D point cloud analysis',
  },
  {
    id: 'multimodal',
    name: 'Multimodal',
    icon: '🎭',
    enabled: false,
    description: 'Combined multi-modal inputs',
  },
];

// v0.8.4: ParallelFusion Unified API Types
export interface UnifiedRequest {
  input: string;
  mode?: 'Fast' | 'Balanced' | 'Thorough';
  strategy?: string;
  sacred_only?: boolean;
  min_confidence?: number;
  min_confidence?: number;
  enable_consensus?: boolean;
  auto_store?: boolean;
  client_id?: string;
  request_id?: string;
}

export interface UnifiedResponse {
  result: string;
  confidence: number;
  flux_position: number;
  elp: ELPValues;
  confidence: number;
  sacred_boost: boolean;
  metadata: ResponseMetadata;
  metrics: ResponseMetrics;
  api_version: string;
}

export interface ResponseMetadata {
  mode?: string;
  strategy: string;
  orchestrators_used: string;
  vortex_cycles: number;
  models_used: string[];
  confidence_lake_hit: boolean;
  consensus_achieved: boolean;
  stored_to_lake: boolean;
}

export interface ResponseMetrics {
  duration_ms: number;
  inference_ms?: number;
  consensus_ms?: number;
  lake_query_ms?: number;
  tokens_used?: number;
  cpu_usage?: number;
  memory_bytes?: number;
}

export interface Conversation {
  id: string;
  title: string;
  messages: ChatMessage[];
  created_at: number;
  updated_at: number;
}
