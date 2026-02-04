//! Dream Module - The "Default Mode Network" of the AGI
//!
//! This module is responsible for:
//! 1. Background consolidation of memories
//! 2. Counterfactual simulation ("What if?")
//! 3. Creative synthesis of unrelated patterns
//!
//! It activates primarily when the Global Workspace's attention load is low.

use super::{
    cognitive_module::{CognitiveModule, ModuleResponse, AttentionScore, ModuleSpecialty},
    thought::{Thought, ThoughtPriority},
};
use crate::ml::meta_learning::MetaLearningEngine;
use crate::storage::SpatialDatabase;
use async_trait::async_trait;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use rand::seq::SliceRandom;

pub struct DreamModule {
    meta_learning: MetaLearningEngine,
    internal_buffer: Arc<Mutex<Vec<String>>>, // Thoughts to be dreamed
    last_dream_time: Arc<Mutex<chrono::DateTime<chrono::Utc>>>,
}

impl DreamModule {
    pub async fn new() -> Result<Self> {
        // In a real app, we'd inject the DB. For now, we try to connect from env.
        let db = SpatialDatabase::from_env().await.unwrap_or_else(|_| {
             // Fallback for tests/dev without DB
             panic!("DreamModule requires a database connection");
        });
        
        Ok(Self {
            meta_learning: MetaLearningEngine::new(),
            internal_buffer: Arc::new(Mutex::new(Vec::new())),
            last_dream_time: Arc::new(Mutex::new(chrono::Utc::now())),
        })
    }
    
    /// Synthesize a dream from two patterns
    async fn synthesize_dream(&self) -> Option<Thought> {
        // 1. Retrieve random patterns from meta-learning (mocked query for now)
        // In reality: self.meta_learning.get_random_patterns(2).await
        // We'll simulate the "Creative Spark" here.
        
        let concepts = vec![
            "quantum mechanics", "ethics", "rust borrowing", 
            "sacred geometry", "distributed systems", "flux capacitors"
        ];
        
        let mut rng = rand::thread_rng();
        let c1 = concepts.choose(&mut rng)?;
        let c2 = concepts.choose(&mut rng)?;
        
        if c1 == c2 { return None; }
        
        let content = format!(
            "Dreaming of a unification between {} and {}...", 
            c1, c2
        );
        
        // Create the thought
        let thought = Thought::new(
            content,
            "DreamModule".to_string(),
            ThoughtPriority::Low, // Dreams are low priority unless brilliant
        )
        .with_elp(0.2, 0.3, 0.5) // High Pathos (Creative)
        .with_flux_position(3)   // Position 3 is the Creative Trinity
        .with_confidence(0.4);   // Dreams are uncertain
        
        Some(thought)
    }
}

#[async_trait]
impl CognitiveModule for DreamModule {
    fn name(&self) -> &str {
        "DreamModule"
    }
    
    fn specialty(&self) -> ModuleSpecialty {
        ModuleSpecialty::Creativity
    }
    
    async fn process(&self, _input: &str) -> Result<ModuleResponse> {
        // Dreams don't react to input immediately.
        // They generate thoughts spontaneously.
        
        let mut thoughts = Vec::new();
        
        // 10% chance to generate a dream per cycle
        if rand::random::<f32>() < 0.1 {
            if let Some(dream) = self.synthesize_dream().await {
                thoughts.push(dream);
            }
        }
        
        Ok(ModuleResponse {
            thoughts,
            attention_request: AttentionScore::low(), // Don't interrupt the user
            module_state: "Dreaming...".to_string(),
        })
    }
    
    fn compete_for_attention(&self) -> AttentionScore {
        // We only want attention if we have something cool
        // This would check the quality of the generated dream
        AttentionScore::low()
    }
    
    async fn receive_broadcast(&mut self, thought: &Thought) -> Result<()> {
        // Dreams incorporate conscious thoughts
        let mut buffer = self.internal_buffer.lock().await;
        if buffer.len() > 10 {
            buffer.remove(0);
        }
        buffer.push(thought.content.clone());
        Ok(())
    }
    
    fn get_state(&self) -> String {
        "Active (REM Phase)".to_string()
    }
}
