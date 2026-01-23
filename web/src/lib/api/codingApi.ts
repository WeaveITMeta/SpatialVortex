// Coding API Client for Enhanced Code Generation

export interface CodingRequest {
  message: string;
  user_id: string;
  language?: string;
  context?: string[];
}

export interface CodeBlockResponse {
  language: string;
  code: string;
  filename?: string;
  reasoning_steps?: number;
  complexity_score?: number;
}

export interface ELPValues {
  ethos: number;
  logos: number;
  pathos: number;
}

export interface CodingResponse {
  response: string;
  code_blocks: CodeBlockResponse[];
  is_code_response: boolean;
  elp_values: ELPValues;
  confidence: number;
  flux_position: number;
  generation_time_ms?: number;
  reasoning_steps?: number;
  semantic_color?: string;
  primary_meaning?: string;
}

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:7000';

export class CodingApiClient {
  private baseUrl: string;

  constructor(baseUrl: string = API_BASE_URL) {
    this.baseUrl = baseUrl;
  }

  /**
   * Generate code using the Enhanced Coding Agent
   */
  async generateCode(request: CodingRequest): Promise<CodingResponse> {
    const response = await fetch(`${this.baseUrl}/api/v1/chat/code`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(request),
    });

    if (!response.ok) {
      throw new Error(`Code generation failed: ${response.statusText}`);
    }

    return await response.json();
  }

  /**
   * Unified chat endpoint that intelligently routes to code generation or text response
   */
  async unifiedChat(request: CodingRequest): Promise<CodingResponse> {
    const response = await fetch(`${this.baseUrl}/api/v1/chat/unified`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(request),
    });

    if (!response.ok) {
      throw new Error(`Chat failed: ${response.statusText}`);
    }

    return await response.json();
  }

  /**
   * Health check for the API
   */
  async healthCheck(): Promise<boolean> {
    try {
      const response = await fetch(`${this.baseUrl}/api/v1/health`);
      return response.ok;
    } catch {
      return false;
    }
  }
}

// Export singleton instance
export const codingApi = new CodingApiClient();
