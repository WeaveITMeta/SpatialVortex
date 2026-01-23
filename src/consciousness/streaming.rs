//! WebTransport streaming server for real-time consciousness analytics
//!
//! Provides high-performance bidirectional streaming using WebTransport
//! with support for:
//! - Real-time analytics broadcasting
//! - Selection analysis requests
//! - Word-level insight streaming
//! - Low-latency updates (<50ms)

use super::analytics::{AnalyticsBroadcaster, StreamingEvent, SelectionAnalysisResult, WordLevelInsights};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use std::collections::HashMap;

/// WebTransport streaming server for consciousness analytics
pub struct ConsciousnessStreamingServer {
    /// Broadcast channel for streaming events
    event_tx: broadcast::Sender<StreamingEvent>,
    
    /// Current session broadcaster
    broadcaster: Arc<RwLock<AnalyticsBroadcaster>>,
    
    /// Word-level tracking for selection analysis
    word_tracker: Arc<RwLock<WordTracker>>,
    
    /// Active subscriptions
    subscriptions: Arc<RwLock<HashMap<String, SubscriptionInfo>>>,
}

/// Information about an active subscription
#[derive(Debug, Clone)]
struct SubscriptionInfo {
    #[allow(dead_code)]
    client_id: String,
    #[allow(dead_code)]
    filter: EventFilter,
    #[allow(dead_code)]
    last_event_time: u64,
}

/// Filter for streaming events
#[derive(Debug, Clone)]
pub struct EventFilter {
    pub include_snapshots: bool,
    pub include_thoughts: bool,
    pub include_words: bool,
    pub include_patterns: bool,
    pub include_phi: bool,
    pub include_states: bool,
}

impl Default for EventFilter {
    fn default() -> Self {
        Self {
            include_snapshots: true,
            include_thoughts: true,
            include_words: false, // Off by default (high volume)
            include_patterns: true,
            include_phi: true,
            include_states: true,
        }
    }
}

/// Tracks words and their associated insights
pub struct WordTracker {
    /// Map of word position → insights
    words: Vec<WordEntry>,
    
    /// Current text being analyzed
    current_text: String,
    
    /// Agent attribution for each word
    agent_map: Vec<String>,
}

#[derive(Debug, Clone)]
struct WordEntry {
    word: String,
    position: usize,
    insights: WordLevelInsights,
}

impl WordTracker {
    pub fn new() -> Self {
        Self {
            words: Vec::new(),
            current_text: String::new(),
            agent_map: Vec::new(),
        }
    }
    
    /// Add word with insights
    pub fn add_word(&mut self, word: String, agent: String, insights: WordLevelInsights) {
        let position = self.words.len();
        self.words.push(WordEntry {
            word: word.clone(),
            position,
            insights,
        });
        self.current_text.push_str(&word);
        self.current_text.push(' ');
        self.agent_map.push(agent);
    }
    
    /// Get insights for a specific word position
    pub fn get_word_insights(&self, position: usize) -> Option<&WordLevelInsights> {
        self.words.get(position).map(|e| &e.insights)
    }
    
    /// Analyze selected text range
    pub fn analyze_selection(&self, start_pos: usize, end_pos: usize) -> SelectionAnalysisResult {
        let selected_words: Vec<&WordEntry> = self.words.iter()
            .filter(|w| w.position >= start_pos && w.position < end_pos)
            .collect();
        
        if selected_words.is_empty() {
            return SelectionAnalysisResult::empty();
        }
        
        // Calculate aggregate metrics
        let mut total_e = 0.0;
        let mut total_l = 0.0;
        let mut total_p = 0.0;
        let mut total_confidence = 0.0;
        let mut patterns = Vec::new();
        let mut agent_counts: HashMap<String, usize> = HashMap::new();
        
        for word_entry in &selected_words {
            let (e, l, p) = word_entry.insights.elp_influence;
            total_e += e;
            total_l += l;
            total_p += p;
            total_confidence += word_entry.insights.confidence;
            
            // Track agent attribution
            if let Some(agent) = self.agent_map.get(word_entry.position) {
                *agent_counts.entry(agent.clone()).or_insert(0) += 1;
            }
            
            // Collect unique patterns
            for pattern in &word_entry.insights.related_patterns {
                if !patterns.contains(pattern) {
                    patterns.push(pattern.clone());
                }
            }
        }
        
        let word_count = selected_words.len();
        let avg_e = total_e / word_count as f64;
        let avg_l = total_l / word_count as f64;
        let avg_p = total_p / word_count as f64;
        
        // Determine dominant agent
        let dominant_agent = agent_counts.iter()
            .max_by_key(|(_, count)| *count)
            .map(|(agent, _)| agent.clone())
            .unwrap_or_else(|| "Unknown".to_string());
        
        // Calculate emotional tone
        let avg_valence: f64 = selected_words.iter()
            .map(|w| w.insights.valence)
            .sum::<f64>() / word_count as f64;
        
        let emotional_tone = if avg_valence > 0.3 {
            "Positive"
        } else if avg_valence < -0.3 {
            "Negative"
        } else {
            "Neutral"
        }.to_string();
        
        // Calculate logical coherence
        let logical_coherence: f64 = selected_words.iter()
            .map(|w| w.insights.logical_strength)
            .sum::<f64>() / word_count as f64;
        
        // Estimate Φ contribution (simplified)
        let phi_contribution = (logical_coherence + total_confidence / word_count as f64) / 2.0;
        
        // Gather ethical implications
        let ethical_implications = selected_words.iter()
            .filter(|w| w.insights.ethical_weight > 0.6)
            .map(|w| format!("High ethical weight on '{}'", w.word))
            .collect();
        
        SelectionAnalysisResult {
            elp_balance: (avg_e, avg_l, avg_p),
            avg_confidence: total_confidence / word_count as f64,
            dominant_agent,
            patterns,
            emotional_tone,
            logical_coherence,
            ethical_implications,
            phi_contribution,
            word_count,
            word_insights: selected_words.iter()
                .map(|w| w.insights.clone())
                .collect(),
        }
    }
    
    /// Clear all tracked words
    pub fn clear(&mut self) {
        self.words.clear();
        self.current_text.clear();
        self.agent_map.clear();
    }
    
    /// Get current full text
    pub fn current_text(&self) -> &str {
        &self.current_text
    }
}

impl SelectionAnalysisResult {
    fn empty() -> Self {
        Self {
            elp_balance: (0.33, 0.33, 0.34),
            avg_confidence: 0.0,
            dominant_agent: "None".to_string(),
            patterns: Vec::new(),
            emotional_tone: "Neutral".to_string(),
            logical_coherence: 0.0,
            ethical_implications: Vec::new(),
            phi_contribution: 0.0,
            word_count: 0,
            word_insights: Vec::new(),
        }
    }
}

impl ConsciousnessStreamingServer {
    /// Create new streaming server
    pub fn new(session_id: String) -> Self {
        let (event_tx, _) = broadcast::channel(1000); // Buffer up to 1000 events
        
        Self {
            event_tx,
            broadcaster: Arc::new(RwLock::new(AnalyticsBroadcaster::new(session_id))),
            word_tracker: Arc::new(RwLock::new(WordTracker::new())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Subscribe to events
    pub fn subscribe(&self, client_id: String, filter: EventFilter) -> broadcast::Receiver<StreamingEvent> {
        let rx = self.event_tx.subscribe();
        
        // Track subscription
        tokio::spawn({
            let subscriptions = self.subscriptions.clone();
            async move {
                let mut subs = subscriptions.write().await;
                subs.insert(client_id.clone(), SubscriptionInfo {
                    client_id,
                    filter,
                    last_event_time: 0,
                });
            }
        });
        
        rx
    }
    
    /// Broadcast event to all subscribers
    pub async fn broadcast(&self, event: StreamingEvent) -> Result<()> {
        // Send to all subscribers
        let _ = self.event_tx.send(event);
        Ok(())
    }
    
    /// Add word-level insight and broadcast
    pub async fn add_word_insight(
        &self,
        word: String,
        agent: String,
        insights: WordLevelInsights,
    ) -> Result<()> {
        let position = {
            let mut tracker = self.word_tracker.write().await;
            let pos = tracker.words.len();
            tracker.add_word(word.clone(), agent.clone(), insights.clone());
            pos
        };
        
        // Broadcast word insight event
        let broadcaster = self.broadcaster.read().await;
        let event = broadcaster.word_insight(word, position, insights);
        self.broadcast(event).await?;
        
        Ok(())
    }
    
    /// Analyze selected text and broadcast result
    pub async fn analyze_selection(
        &self,
        selected_text: String,
        start_pos: usize,
        end_pos: usize,
    ) -> Result<SelectionAnalysisResult> {
        let analysis = {
            let tracker = self.word_tracker.read().await;
            tracker.analyze_selection(start_pos, end_pos)
        };
        
        // Broadcast selection analysis
        let broadcaster = self.broadcaster.read().await;
        let event = broadcaster.selection_analysis(
            selected_text,
            start_pos,
            end_pos,
            analysis.clone(),
        );
        self.broadcast(event).await?;
        
        Ok(analysis)
    }
    
    /// Get broadcaster for creating events
    pub fn broadcaster(&self) -> Arc<RwLock<AnalyticsBroadcaster>> {
        self.broadcaster.clone()
    }
    
    /// Get word tracker for direct access
    pub fn word_tracker(&self) -> Arc<RwLock<WordTracker>> {
        self.word_tracker.clone()
    }
    
    /// Get number of active subscriptions
    pub async fn subscription_count(&self) -> usize {
        self.subscriptions.read().await.len()
    }
    
    /// Clear all word tracking
    pub async fn clear_words(&self) {
        let mut tracker = self.word_tracker.write().await;
        tracker.clear();
    }
}

/// Helper to create word insights from thought data
pub fn create_word_insights(
    word: &str,
    agent: &str,
    elp: (f64, f64, f64),
    confidence: f64,
) -> WordLevelInsights {
    // Determine category based on word characteristics
    let category = if word.chars().all(|c| c.is_alphabetic()) {
        if word.len() > 8 {
            "Complex"
        } else {
            "Simple"
        }
    } else {
        "Mixed"
    }.to_string();
    
    // Estimate valence (simplified - could use sentiment analysis)
    let valence = match agent {
        "Ethos (Moral)" => 0.2, // Slightly positive (moral tone)
        "Logos (Logic)" => 0.0,  // Neutral (factual)
        "Pathos (Emotion)" => 0.5, // Positive (empathetic)
        _ => 0.0,
    };
    
    WordLevelInsights {
        agent: agent.to_string(),
        elp_influence: elp,
        confidence,
        category,
        valence,
        logical_strength: elp.1, // Logos component
        ethical_weight: elp.0,   // Ethos component
        related_patterns: Vec::new(), // To be filled by meta-monitor
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_word_tracker() {
        let mut tracker = WordTracker::new();
        
        let insights = create_word_insights(
            "consciousness",
            "Logos (Logic)",
            (0.1, 0.8, 0.1),
            0.9,
        );
        
        tracker.add_word("consciousness".to_string(), "Logos (Logic)".to_string(), insights);
        
        assert_eq!(tracker.words.len(), 1);
        assert!(tracker.current_text().contains("consciousness"));
    }
    
    #[tokio::test]
    async fn test_streaming_server() {
        let server = ConsciousnessStreamingServer::new("test-session".to_string());
        
        let mut rx = server.subscribe("client-1".to_string(), EventFilter::default());
        
        let event = StreamingEvent::StateChanged {
            timestamp: 123456,
            from: "Idle".to_string(),
            to: "Focused".to_string(),
            reason: "Question received".to_string(),
        };
        
        server.broadcast(event).await.unwrap();
        
        let received = rx.recv().await.unwrap();
        assert!(matches!(received, StreamingEvent::StateChanged { .. }));
    }
}
