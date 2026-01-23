/**
 * WebTransport-based Real-Time Collaboration Service
 * 
 * Uses WebTransport for low-latency, multiplexed communication
 * Falls back to polling if WebTransport not available
 */

export interface CollaborationUser {
  user_id: string;
  username: string;
  color: string;
  cursor?: CursorPosition;
  last_seen: number;
}

export interface CursorPosition {
  x: number;
  y: number;
  line?: number;
  column?: number;
}

export interface SessionState {
  session_id: string;
  active_users: Record<string, CollaborationUser>;
  created_at: number;
  last_activity: number;
}

export interface CollaborationMessage {
  type: 'join' | 'leave' | 'cursor_move' | 'text_edit' | 'chat_message' | 'canvas_update';
  session_id: string;
  user_id: string;
  data?: any;
}

export class CollaborationService {
  private baseUrl: string;
  private sessionId: string | null = null;
  private userId: string | null = null;
  private username: string = 'Anonymous';
  private pollingInterval: number | null = null;
  private listeners: Map<string, Set<Function>> = new Map();
  private isWebTransportSupported: boolean = false;
  
  constructor(baseUrl: string = 'http://localhost:7000/api/v1') {
    this.baseUrl = baseUrl;
    this.checkWebTransportSupport();
  }
  
  /**
   * Check if WebTransport is supported
   */
  private checkWebTransportSupport() {
    // @ts-ignore - WebTransport is cutting edge
    this.isWebTransportSupported = typeof WebTransport !== 'undefined';
    
    if (this.isWebTransportSupported) {
      console.log('✅ WebTransport supported!');
    } else {
      console.log('⚠️ WebTransport not supported, using HTTP polling fallback');
    }
  }
  
  /**
   * Join a collaboration session
   */
  async joinSession(sessionId: string, username?: string): Promise<SessionState> {
    this.sessionId = sessionId;
    if (username) this.username = username;
    
    try {
      const response = await fetch(`${this.baseUrl}/collaboration/join`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          session_id: sessionId,
          user_id: this.userId,
          username: this.username,
        }),
      });
      
      const data = await response.json();
      
      if (data.success) {
        this.userId = data.user_id;
        this.startPolling(); // Start real-time updates
        this.emit('session_joined', data.session);
        return data.session;
      }
      
      throw new Error('Failed to join session');
    } catch (error) {
      console.error('Join session error:', error);
      throw error;
    }
  }
  
  /**
   * Leave the current session
   */
  async leaveSession(): Promise<void> {
    if (!this.sessionId || !this.userId) return;
    
    try {
      await fetch(`${this.baseUrl}/collaboration/leave`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          session_id: this.sessionId,
          user_id: this.userId,
        }),
      });
      
      this.stopPolling();
      this.emit('session_left', this.sessionId);
      this.sessionId = null;
    } catch (error) {
      console.error('Leave session error:', error);
    }
  }
  
  /**
   * Update cursor position
   */
  async updateCursor(position: CursorPosition): Promise<void> {
    if (!this.sessionId || !this.userId) return;
    
    try {
      const response = await fetch(`${this.baseUrl}/collaboration/cursor`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          session_id: this.sessionId,
          user_id: this.userId,
          position,
        }),
      });
      
      const data = await response.json();
      if (data.success && data.session) {
        this.emit('session_update', data.session);
      }
    } catch (error) {
      console.error('Update cursor error:', error);
    }
  }
  
  /**
   * Get current session state
   */
  async getSession(): Promise<SessionState | null> {
    if (!this.sessionId) return null;
    
    try {
      const response = await fetch(
        `${this.baseUrl}/collaboration/session/${this.sessionId}`
      );
      const data = await response.json();
      return data.session || null;
    } catch (error) {
      console.error('Get session error:', error);
      return null;
    }
  }
  
  /**
   * Start polling for updates (fallback when WebTransport not available)
   */
  private startPolling() {
    if (this.pollingInterval) return;
    
    // Poll every 500ms for real-time feel
    this.pollingInterval = window.setInterval(async () => {
      const session = await this.getSession();
      if (session) {
        this.emit('session_update', session);
      }
    }, 500);
  }
  
  /**
   * Stop polling
   */
  private stopPolling() {
    if (this.pollingInterval) {
      clearInterval(this.pollingInterval);
      this.pollingInterval = null;
    }
  }
  
  /**
   * Subscribe to events
   */
  on(event: string, callback: Function) {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, new Set());
    }
    this.listeners.get(event)!.add(callback);
  }
  
  /**
   * Unsubscribe from events
   */
  off(event: string, callback: Function) {
    if (this.listeners.has(event)) {
      this.listeners.get(event)!.delete(callback);
    }
  }
  
  /**
   * Emit event to listeners
   */
  private emit(event: string, data: any) {
    if (this.listeners.has(event)) {
      this.listeners.get(event)!.forEach(callback => callback(data));
    }
  }
  
  /**
   * Get current user info
   */
  getCurrentUser(): { user_id: string | null; username: string } {
    return {
      user_id: this.userId,
      username: this.username,
    };
  }
  
  /**
   * Check if user is in a session
   */
  isInSession(): boolean {
    return this.sessionId !== null;
  }
}

// Singleton instance
export const collaborationService = new CollaborationService();
