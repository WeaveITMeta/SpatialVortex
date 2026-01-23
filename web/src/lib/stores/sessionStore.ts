// Session Management Store
import { writable, derived, get } from 'svelte/store';
import type { ChatMessage } from '$lib/types/chat';

export interface ChatSession {
  id: string;
  title: string;
  messages: ChatMessage[];
  created_at: Date;
  updated_at: Date;
  from_backend?: boolean;  // Track if session came from backend
}

const API_BASE = 'http://localhost:7000';
const USER_ID = 'desktop_user';

// Load sessions from BACKEND API (not localStorage)
async function loadSessionsFromBackend(): Promise<Map<string, ChatSession>> {
  if (typeof window === 'undefined') return new Map();
  
  try {
    const response = await fetch(`${API_BASE}/api/v1/chat/sessions?user_id=${USER_ID}`);
    
    // If backend has no sessions or is unavailable, merge with localStorage
    if (!response.ok) {
      if (response.status === 404) {
        console.log('No sessions found on backend, using localStorage');
        return loadSessionsFromLocalStorage();
      }
      console.warn(`Backend error ${response.status}, using localStorage fallback`);
      return loadSessionsFromLocalStorage();
    }
    
    const data = await response.json();
    const sessions = new Map<string, ChatSession>();
    
    // Convert backend format to frontend format
    for (const session of data.sessions || []) {
      sessions.set(session.session_id, {
        id: session.session_id,
        title: session.preview || 'New Chat',
        messages: [], // Will be loaded when session is opened
        created_at: new Date(session.created_at),
        updated_at: new Date(session.last_activity || session.created_at),
        from_backend: true,  // Mark as coming from backend
      });
    }
    
    // Also merge in any localStorage-only sessions (new chats not yet on backend)
    const localSessions = loadSessionsFromLocalStorage();
    for (const [id, session] of localSessions.entries()) {
      if (!sessions.has(id)) {
        // This is a new local session not yet on backend
        sessions.set(id, session);
      }
    }
    
    console.log(`Loaded ${sessions.size} sessions (${data.sessions?.length || 0} from backend, ${localSessions.size - (data.sessions?.length || 0)} from localStorage)`);
    return sessions;
  } catch (e) {
    console.error('Failed to load sessions from backend:', e);
    return loadSessionsFromLocalStorage();
  }
}

// Fallback: Load sessions from localStorage
function loadSessionsFromLocalStorage(): Map<string, ChatSession> {
  if (typeof window === 'undefined') return new Map();
  
  const stored = localStorage.getItem('chat_sessions');
  if (!stored) return new Map();
  
  try {
    const data = JSON.parse(stored);
    const sessions = new Map();
    
    for (const [id, session] of Object.entries(data)) {
      const s = session as any;
      sessions.set(id, {
        id: s.id,
        title: s.title,
        messages: Array.isArray(s.messages) ? [...s.messages] : [],  // Deep copy messages
        created_at: new Date(s.created_at),
        updated_at: new Date(s.updated_at),
      });
    }
    
    return sessions;
  } catch (e) {
    console.error('Failed to load sessions from localStorage:', e);
    return new Map();
  }
}

// Save sessions to localStorage
function saveSessions(sessions: Map<string, ChatSession>) {
  if (typeof window === 'undefined') return;
  
  const data: Record<string, ChatSession> = {};
  sessions.forEach((session, id) => {
    // Explicitly create a clean object for each session
    data[id] = {
      id: session.id,
      title: session.title,
      messages: session.messages,
      created_at: session.created_at,
      updated_at: session.updated_at,
    };
  });
  
  localStorage.setItem('chat_sessions', JSON.stringify(data));
}

// Load session history from backend
async function loadSessionHistory(sessionId: string): Promise<ChatMessage[]> {
  try {
    const response = await fetch(`${API_BASE}/api/v1/chat/history/${sessionId}`);
    
    // 404 is expected for newly created sessions that haven't been saved to backend yet
    if (response.status === 404) {
      console.log(`Session ${sessionId} not found on backend (likely new session)`);
      return [];
    }
    
    if (!response.ok) {
      console.warn(`Failed to load session history: ${response.status}`);
      return [];
    }
    
    const data = await response.json();
    
    // Convert backend message format to frontend format
    return (data.messages || []).map((msg: any) => ({
      id: msg.timestamp || Date.now().toString(),
      role: msg.role,
      content: msg.content,
      timestamp: new Date(msg.timestamp || Date.now()),
    }));
  } catch (e) {
    console.error('Failed to load session history:', e);
    return [];
  }
}

// Create store
function createSessionStore() {
  const sessions = writable<Map<string, ChatSession>>(new Map());
  const currentSessionId = writable<string | null>(null);
  const isLoading = writable<boolean>(false);
  
  // Auto-save on changes
  sessions.subscribe(($sessions) => {
    saveSessions($sessions);
  });
  
  return {
    sessions,
    currentSessionId,
    isLoading,
    
    // Get current session
    currentSession: derived(
      [sessions, currentSessionId],
      ([$sessions, $currentSessionId]) => {
        if (!$currentSessionId) return null;
        return $sessions.get($currentSessionId) || null;
      }
    ),
    
    // Load sessions from backend on startup
    loadSessions: async () => {
      isLoading.set(true);
      try {
        const loadedSessions = await loadSessionsFromBackend();
        sessions.set(loadedSessions);
      } catch (e) {
        console.error('Failed to load sessions:', e);
      } finally {
        isLoading.set(false);
      }
    },
    
    // Create new session
    createSession: () => {
      const id = `session_${Date.now()}`;
      const now = new Date();
      
      const newSession: ChatSession = {
        id,
        title: 'New Chat',
        messages: [],
        created_at: now,
        updated_at: now,
      };
      
      sessions.update($sessions => {
        // Create new Map to trigger Svelte reactivity
        const newSessions = new Map($sessions);
        newSessions.set(id, newSession);
        return newSessions;
      });
      
      currentSessionId.set(id);
      return id;
    },
    
    // Switch to session and load its history from backend
    switchSession: async (id: string) => {
      currentSessionId.set(id);
      
      // Load session history from backend
      const $sessions = get(sessions);
      const session = $sessions.get(id);
      
      // Try to load from backend if:
      // 1. Session exists
      // 2. Session came from backend (has from_backend flag)
      // 3. Session was created more than 1 second ago (to avoid loading brand new sessions)
      const now = Date.now();
      const sessionAge = now - new Date(session?.created_at || now).getTime();
      
      if (session && session.from_backend && sessionAge > 1000) {
        isLoading.set(true);
        try {
          const messages = await loadSessionHistory(id);
          
          // Always update with backend messages (even if empty)
          // This ensures we have the latest state from backend
          sessions.update($sessions => {
            const newSessions = new Map($sessions);
            const updatedSession = newSessions.get(id);
            if (updatedSession) {
              newSessions.set(id, {
                ...updatedSession,
                messages,
              });
            }
            return newSessions;
          });
        } catch (e) {
          console.error('Failed to load session history:', e);
        } finally {
          isLoading.set(false);
        }
      }
    },
    
    // Update session title
    updateTitle: async (id: string, title: string) => {
      sessions.update($sessions => {
        const session = $sessions.get(id);
        if (session) {
          // Create completely new Map with deep-copied sessions
          const newSessions = new Map<string, ChatSession>();
          $sessions.forEach((s, key) => {
            if (key === id) {
              // Update this session with new title
              newSessions.set(key, {
                id: s.id,
                title: title,
                messages: s.messages,
                created_at: s.created_at,
                updated_at: new Date(),
              });
            } else {
              // Create new object for other sessions too (prevent any reference sharing)
              newSessions.set(key, {
                id: s.id,
                title: s.title,
                messages: s.messages,
                created_at: s.created_at,
                updated_at: s.updated_at,
              });
            }
          });
          return newSessions;
        }
        return $sessions;
      });
    },
    
    // Add message to current session
    addMessage: (message: ChatMessage) => {
      const $currentSessionId = get(currentSessionId);
      if (!$currentSessionId) return;
      
      sessions.update($sessions => {
        const session = $sessions.get($currentSessionId);
        if (session) {
          // Create completely new Map with deep-copied sessions
          const newSessions = new Map<string, ChatSession>();
          $sessions.forEach((s, key) => {
            if (key === $currentSessionId) {
              // Update this session with new message
              newSessions.set(key, {
                id: s.id,
                title: s.title,
                messages: [...s.messages, message],
                created_at: s.created_at,
                updated_at: new Date(),
              });
            } else {
              // Create new object for other sessions too
              newSessions.set(key, {
                id: s.id,
                title: s.title,
                messages: s.messages,
                created_at: s.created_at,
                updated_at: s.updated_at,
              });
            }
          });
          return newSessions;
        }
        return $sessions;
      });
    },
    
    // Update message content (for streaming)
    updateMessageContent: (messageId: string, content: string) => {
      const $currentSessionId = get(currentSessionId);
      if (!$currentSessionId) return;
      
      sessions.update($sessions => {
        const session = $sessions.get($currentSessionId);
        if (session) {
          // Create completely new Map with deep-copied sessions
          const newSessions = new Map<string, ChatSession>();
          $sessions.forEach((s, key) => {
            if (key === $currentSessionId) {
              // Update this session with modified message
              const updatedMessages = s.messages.map(msg => {
                if (msg.id === messageId) {
                  return {
                    ...msg,
                    content,
                  };
                }
                return msg;
              });
              
              newSessions.set(key, {
                id: s.id,
                title: s.title,
                messages: updatedMessages,
                created_at: s.created_at,
                updated_at: new Date(),
              });
            } else {
              // Create new object for other sessions too
              newSessions.set(key, {
                id: s.id,
                title: s.title,
                messages: s.messages,
                created_at: s.created_at,
                updated_at: s.updated_at,
              });
            }
          });
          return newSessions;
        }
        return $sessions;
      });
    },
    
    // Auto-generate session title using AI
    generateTitle: async (sessionId: string) => {
      const $sessions = get(sessions);
      const session = $sessions.get(sessionId);
      
      if (!session || session.messages.length === 0) {
        console.log('No session or no messages for title generation');
        return;
      }
      
      // Get first user message
      const firstMessage = session.messages.find(m => m.role === 'user');
      if (!firstMessage) {
        console.log('No user message found for title generation');
        return;
      }
      
      console.log('Generating title for session:', sessionId, 'with message:', firstMessage.content.substring(0, 50));
      
      try {
        const response = await fetch('http://localhost:7000/api/v1/chat/generate-title', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            first_message: firstMessage.content,
            session_id: sessionId,
          }),
        });
        
        if (response.ok) {
          const data = await response.json();
          console.log('Generated title:', data.title);
          sessions.update($sessions => {
            const session = $sessions.get(sessionId);
            if (session && data.title) {
              // Create completely new Map with deep-copied sessions
              const newSessions = new Map<string, ChatSession>();
              $sessions.forEach((s, key) => {
                if (key === sessionId) {
                  newSessions.set(key, {
                    id: s.id,
                    title: data.title,
                    messages: s.messages,
                    created_at: s.created_at,
                    updated_at: new Date(),
                  });
                } else {
                  newSessions.set(key, {
                    id: s.id,
                    title: s.title,
                    messages: s.messages,
                    created_at: s.created_at,
                    updated_at: s.updated_at,
                  });
                }
              });
              return newSessions;
            }
            return $sessions;
          });
        } else {
          console.error('Title generation API error:', response.status);
          throw new Error(`API error: ${response.status}`);
        }
      } catch (e) {
        console.error('Failed to generate title:', e);
        // Fallback: use first 50 chars of first message
        const fallbackTitle = firstMessage.content.substring(0, 50).trim() + 
          (firstMessage.content.length > 50 ? '...' : '');
        
        console.log('Using fallback title:', fallbackTitle);
        sessions.update($sessions => {
          const session = $sessions.get(sessionId);
          if (session) {
            // Create completely new Map with deep-copied sessions
            const newSessions = new Map<string, ChatSession>();
            $sessions.forEach((s, key) => {
              if (key === sessionId) {
                newSessions.set(key, {
                  id: s.id,
                  title: fallbackTitle,
                  messages: s.messages,
                  created_at: s.created_at,
                  updated_at: new Date(),
                });
              } else {
                newSessions.set(key, {
                  id: s.id,
                  title: s.title,
                  messages: s.messages,
                  created_at: s.created_at,
                  updated_at: s.updated_at,
                });
              }
            });
            return newSessions;
          }
          return $sessions;
        });
      }
    },
    
    // Delete session
    deleteSession: (id: string) => {
      sessions.update($sessions => {
        // Create new Map to trigger Svelte reactivity
        const newSessions = new Map($sessions);
        newSessions.delete(id);
        return newSessions;
      });
      
      // If deleted current session, switch to another or create new
      if (get(currentSessionId) === id) {
        const remaining = Array.from(get(sessions).keys());
        if (remaining.length > 0) {
          currentSessionId.set(remaining[0] || null);
        } else {
          // No sessions left, create a new one
          const newId = `session_${Date.now()}`;
          const now = new Date();
          
          sessions.update($sessions => {
            // Create new Map to trigger Svelte reactivity
            const newSessions = new Map($sessions);
            newSessions.set(newId, {
              id: newId,
              title: 'New Chat',
              messages: [],
              created_at: now,
              updated_at: now,
            });
            return newSessions;
          });
          
          currentSessionId.set(newId);
        }
      }
    },
    
    // Get all sessions sorted by updated_at
    getSortedSessions: derived(sessions, ($sessions) => {
      return Array.from($sessions.values())
        .sort((a, b) => b.updated_at.getTime() - a.updated_at.getTime());
    }),
  };
}

export const sessionStore = createSessionStore();
