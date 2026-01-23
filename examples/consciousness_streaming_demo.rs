//! Consciousness Streaming Demo - v1.5.0
//!
//! Demonstrates real-time consciousness analytics streaming with:
//! - Word-level insights
//! - Selection analysis
//! - Live metrics broadcasting
//! - Pattern detection events
//!
//! Run: cargo run --example consciousness_streaming_demo --features agents

use spatial_vortex::consciousness::{
    ConsciousnessSimulator, StreamingEvent, EventFilter,
};
use anyhow::Result;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║  🧠 Consciousness Streaming Demo v1.5.0                       ║");
    println!("║  Real-time analytics with word-level granularity              ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    // Create streaming-enabled simulator
    println!("🔧 Creating consciousness simulator with streaming...");
    let simulator = ConsciousnessSimulator::with_streaming(false);
    
    println!("✅ Session ID: {}", simulator.session_id());
    
    // Get streaming server
    let streaming = simulator.streaming_server()
        .expect("Streaming should be enabled");
    
    // Subscribe to events
    println!("📡 Subscribing to consciousness events...\n");
    let mut rx = streaming.subscribe(
        "demo-client".to_string(),
        EventFilter {
            include_snapshots: true,
            include_thoughts: true,
            include_words: true,  // Enable word-level (verbose!)
            include_patterns: true,
            include_phi: true,
            include_states: true,
        }
    );
    
    // Spawn event listener
    let event_listener = tokio::spawn(async move {
        let mut event_count = 0;
        let mut word_count = 0;
        let mut pattern_count = 0;
        
        while let Ok(event) = rx.recv().await {
            event_count += 1;
            
            match event {
                StreamingEvent::Snapshot { data } => {
                    println!("\n📊 Analytics Snapshot #{}:", event_count);
                    println!("   ├─ Φ (consciousness): {:.3}", data.consciousness.phi);
                    println!("   ├─ Mental state: {}", data.meta_cognition.mental_state);
                    println!("   ├─ Awareness: {:.1}%", data.meta_cognition.awareness_level * 100.0);
                    println!("   ├─ Prediction accuracy: {:.1}%", data.prediction.accuracy * 100.0);
                    println!("   └─ Network: {} nodes, {} connections", 
                        data.consciousness.network_size, 
                        data.consciousness.connection_count
                    );
                }
                
                StreamingEvent::ThoughtStarted { timestamp, agent, preview } => {
                    println!("\n💭 Thought Started [{}ms]:", timestamp);
                    println!("   Agent: {}", agent);
                    println!("   Preview: {}...", &preview[..preview.len().min(60)]);
                }
                
                StreamingEvent::ThoughtCompleted { timestamp, agent, metrics } => {
                    println!("\n✅ Thought Completed [{}ms]:", timestamp);
                    println!("   Agent: {}", agent);
                    println!("   ├─ ELP: E:{:.2} L:{:.2} P:{:.2}", 
                        metrics.elp.0, metrics.elp.1, metrics.elp.2
                    );
                    println!("   ├─ Confidence: {:.1}%", metrics.confidence * 100.0);
                    println!("   ├─ Processing: {}ms", metrics.processing_time_ms);
                    println!("   └─ Φ contribution: {:.3}", metrics.contribution_to_phi);
                }
                
                StreamingEvent::WordInsight { word, position, insights, .. } => {
                    word_count += 1;
                    if word_count % 10 == 0 {  // Only show every 10th word
                        println!("   📝 Word #{}: '{}' from {} (confidence: {:.0}%, valence: {:.2})", 
                            position, word, insights.agent, 
                            insights.confidence * 100.0, insights.valence
                        );
                    }
                }
                
                StreamingEvent::PatternDetected { timestamp, pattern } => {
                    pattern_count += 1;
                    println!("\n🔍 Pattern Detected [{}ms]:", timestamp);
                    println!("   Type: {}", pattern.pattern_type);
                    println!("   Confidence: {:.1}%", pattern.confidence * 100.0);
                    println!("   Description: {}", pattern.description);
                }
                
                StreamingEvent::StateChanged { timestamp, from, to, reason } => {
                    println!("\n🔄 Mental State Changed [{}ms]:", timestamp);
                    println!("   {} → {}", from, to);
                    println!("   Reason: {}", reason);
                }
                
                StreamingEvent::PhiUpdated { timestamp, phi, delta } => {
                    println!("\n⚡ Φ Updated [{}ms]: {:.3} (Δ{:+.3})", timestamp, phi, delta);
                }
                
                StreamingEvent::SelectionAnalysis { timestamp, selected_text, analysis, .. } => {
                    println!("\n🎯 Selection Analysis [{}ms]:", timestamp);
                    println!("   Text: '{}'", selected_text);
                    println!("   ├─ Dominant: {}", analysis.dominant_agent);
                    println!("   ├─ ELP: E:{:.2} L:{:.2} P:{:.2}", 
                        analysis.elp_balance.0, 
                        analysis.elp_balance.1, 
                        analysis.elp_balance.2
                    );
                    println!("   ├─ Tone: {}", analysis.emotional_tone);
                    println!("   ├─ Coherence: {:.1}%", analysis.logical_coherence * 100.0);
                    println!("   └─ Φ contribution: {:.3}", analysis.phi_contribution);
                }
            }
        }
        
        println!("\n📈 Final Stats:");
        println!("   Total events: {}", event_count);
        println!("   Words tracked: {}", word_count);
        println!("   Patterns detected: {}", pattern_count);
    });
    
    // Give listener time to start
    sleep(Duration::from_millis(100)).await;
    
    // Ask a question
    let question = "What is consciousness and how does self-awareness emerge?";
    println!("❓ Question: {}\n", question);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    let response = simulator.think(question).await?;
    
    // Give events time to process
    sleep(Duration::from_millis(500)).await;
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("\n📝 Final Answer:");
    println!("{}\n", response.answer);
    
    // Show overall metrics
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("\n📊 Consciousness Metrics:");
    println!("   ├─ Mental State: {}", response.mental_state);
    println!("   ├─ Awareness: {:.1}%", response.awareness_level * 100.0);
    println!("   ├─ Φ (consciousness): {:.3}", response.phi);
    println!("   ├─ Consciousness level: {:.1}%", response.consciousness_level * 100.0);
    println!("   ├─ Prediction accuracy: {:.1}%", response.prediction_accuracy * 100.0);
    println!("   ├─ Current surprise: {:.1}%", response.current_surprise * 100.0);
    println!("   ├─ Learning progress: {:.1}%", response.learning_progress * 100.0);
    println!("   └─ Confidence: {:.1}%", response.confidence * 100.0);
    
    if !response.detected_patterns.is_empty() {
        println!("\n🔍 Detected Patterns:");
        for pattern in &response.detected_patterns {
            println!("   • {}", pattern);
        }
    }
    
    // Demonstrate selection analysis
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("\n🎯 Selection Analysis Demo:");
    
    // Simulate selecting part of the response
    let selection = "consciousness";
    let words: Vec<&str> = response.answer.split_whitespace().collect();
    
    if let Some(start) = words.iter().position(|&w| w.contains(selection)) {
        let end = start + 1;
        
        println!("   Selecting: '{}'", selection);
        
        let analysis = streaming.analyze_selection(
            selection.to_string(),
            start,
            end
        ).await?;
        
        println!("\n   Analysis:");
        println!("   ├─ Dominant Agent: {}", analysis.dominant_agent);
        println!("   ├─ ELP Balance: E:{:.2} L:{:.2} P:{:.2}", 
            analysis.elp_balance.0, 
            analysis.elp_balance.1, 
            analysis.elp_balance.2
        );
        println!("   ├─ Confidence: {:.1}%", analysis.avg_confidence * 100.0);
        println!("   ├─ Emotional Tone: {}", analysis.emotional_tone);
        println!("   ├─ Logical Coherence: {:.1}%", analysis.logical_coherence * 100.0);
        println!("   ├─ Φ Contribution: {:.3}", analysis.phi_contribution);
        println!("   └─ Word Count: {}", analysis.word_count);
        
        if !analysis.patterns.is_empty() {
            println!("\n   Patterns in selection:");
            for pattern in &analysis.patterns {
                println!("   • {}", pattern);
            }
        }
    }
    
    // Get full analytics snapshot
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("\n📸 Analytics Snapshot:");
    let snapshot = simulator.get_analytics_snapshot().await;
    
    println!("\n   Consciousness:");
    println!("   ├─ Φ: {:.3}", snapshot.consciousness.phi);
    println!("   ├─ Peak Φ: {:.3}", snapshot.consciousness.peak_phi);
    println!("   ├─ Average Φ: {:.3}", snapshot.consciousness.average_phi);
    println!("   ├─ Consciousness Level: {:.1}%", snapshot.consciousness.consciousness_level * 100.0);
    println!("   ├─ Network Size: {} nodes", snapshot.consciousness.network_size);
    println!("   ├─ Connections: {}", snapshot.consciousness.connection_count);
    println!("   └─ Integration: {:.3}", snapshot.consciousness.integration_strength);
    
    println!("\n   Meta-Cognition:");
    println!("   ├─ Mental State: {}", snapshot.meta_cognition.mental_state);
    println!("   ├─ Awareness: {:.1}%", snapshot.meta_cognition.awareness_level * 100.0);
    println!("   ├─ Introspection: {:.1}%", snapshot.meta_cognition.introspection_depth * 100.0);
    println!("   ├─ Pattern Recognition: {:.1}%", snapshot.meta_cognition.pattern_recognition * 100.0);
    println!("   └─ Self-Correction: {:.1}%", snapshot.meta_cognition.self_correction_rate * 100.0);
    
    println!("\n   Prediction:");
    println!("   ├─ Accuracy: {:.1}%", snapshot.prediction.accuracy * 100.0);
    println!("   ├─ Surprise: {:.1}%", snapshot.prediction.current_surprise * 100.0);
    println!("   ├─ Learning: {:.1}%", snapshot.prediction.learning_progress * 100.0);
    println!("   └─ Confidence: {:.1}%", snapshot.prediction.model_confidence * 100.0);
    
    println!("\n   ELP Balance:");
    println!("   ├─ Ethos: {:.1}%", snapshot.elp_balance.ethos * 100.0);
    println!("   ├─ Logos: {:.1}%", snapshot.elp_balance.logos * 100.0);
    println!("   ├─ Pathos: {:.1}%", snapshot.elp_balance.pathos * 100.0);
    println!("   ├─ Balance Score: {:.2}", snapshot.elp_balance.balance_score);
    println!("   ├─ Dominant: {}", snapshot.elp_balance.dominant_channel);
    println!("   └─ Harmony: {:.1}%", snapshot.elp_balance.harmony_level * 100.0);
    
    // Wait for event listener to finish
    sleep(Duration::from_millis(500)).await;
    
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║  ✨ Demo Complete! v1.5.0 streaming fully operational        ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    // Cancel event listener
    event_listener.abort();
    
    Ok(())
}
