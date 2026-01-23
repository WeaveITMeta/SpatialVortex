//! Global Workspace - The theater of consciousness
//!
//! Implements Global Workspace Theory where multiple cognitive modules
//! compete for limited attention, and winning thoughts are broadcast
//! to all modules, creating unified conscious experience.

use super::{
    cognitive_module::CognitiveModule,
    attention::AttentionMechanism,
    thought::Thought,
};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// The global workspace - theater of consciousness
pub struct GlobalWorkspace {
    /// Registered cognitive modules
    modules: Arc<RwLock<Vec<Box<dyn CognitiveModule>>>>,
    
    /// Attention mechanism (spotlight)
    attention: Arc<RwLock<AttentionMechanism>>,
    
    /// Broadcast channel for conscious thoughts
    broadcast_tx: broadcast::Sender<Thought>,
    
    /// Working memory (current conscious content)
    working_memory: Arc<RwLock<Vec<Thought>>>,
    
    /// Vortex cycle position (1-9)
    current_position: Arc<RwLock<u8>>,
}

impl GlobalWorkspace {
    /// Create new global workspace
    pub fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(100);
        
        Self {
            modules: Arc::new(RwLock::new(Vec::new())),
            attention: Arc::new(RwLock::new(AttentionMechanism::new())),
            broadcast_tx,
            working_memory: Arc::new(RwLock::new(Vec::new())),
            current_position: Arc::new(RwLock::new(1)),
        }
    }
    
    /// Register a cognitive module
    pub async fn register_module(&mut self, module: Box<dyn CognitiveModule>) {
        let mut modules = self.modules.write().await;
        modules.push(module);
    }
    
    /// Process input through the global workspace (main consciousness loop)
    pub async fn process(&self, input: &str) -> Result<Vec<Thought>> {
        // 1. All modules process input in parallel
        let candidate_thoughts = self.gather_thoughts(input).await?;
        
        // 2. Attention mechanism selects which thoughts become conscious
        let conscious_thoughts = {
            let mut attention = self.attention.write().await;
            attention.select_conscious_thoughts(candidate_thoughts)
        };
        
        // 3. Broadcast conscious thoughts to all modules
        self.broadcast_thoughts(&conscious_thoughts).await?;
        
        // 4. Update working memory
        {
            let mut memory = self.working_memory.write().await;
            *memory = conscious_thoughts.clone();
        }
        
        // 5. Advance vortex cycle
        self.advance_vortex_cycle().await;
        
        Ok(conscious_thoughts)
    }
    
    /// Gather thoughts from all cognitive modules
    async fn gather_thoughts(&self, input: &str) -> Result<Vec<Thought>> {
        let modules = self.modules.read().await;
        let mut all_thoughts = Vec::new();
        
        for module in modules.iter() {
            match module.process(input).await {
                Ok(response) => {
                    all_thoughts.extend(response.thoughts);
                }
                Err(e) => {
                    tracing::warn!("Module {} failed: {}", module.name(), e);
                }
            }
        }
        
        Ok(all_thoughts)
    }
    
    /// Broadcast conscious thoughts to all modules
    async fn broadcast_thoughts(&self, thoughts: &[Thought]) -> Result<()> {
        let mut modules = self.modules.write().await;
        
        for thought in thoughts {
            // Send via broadcast channel
            let _ = self.broadcast_tx.send(thought.clone());
            
            // Deliver to each module
            for module in modules.iter_mut() {
                if let Err(e) = module.receive_broadcast(thought).await {
                    tracing::warn!(
                        "Module {} failed to receive broadcast: {}",
                        module.name(),
                        e
                    );
                }
            }
        }
        
        Ok(())
    }
    
    /// Advance through vortex cycle (1→2→4→8→7→5→1)
    async fn advance_vortex_cycle(&self) {
        let mut pos = self.current_position.write().await;
        *pos = match *pos {
            1 => 2,
            2 => 4,
            4 => 8,
            8 => 7,
            7 => 5,
            5 => 1,
            _ => 1, // Reset if out of bounds
        };
    }
    
    /// Get current vortex position
    pub async fn get_position(&self) -> u8 {
        *self.current_position.read().await
    }
    
    /// Check if at sacred checkpoint (3, 6, or 9)
    pub async fn is_at_sacred_position(&self) -> bool {
        let pos = self.get_position().await;
        [3, 6, 9].contains(&pos)
    }
    
    /// Get current working memory contents
    pub async fn get_working_memory(&self) -> Vec<Thought> {
        self.working_memory.read().await.clone()
    }
    
    /// Get attention load (how full is consciousness)
    pub async fn get_attention_load(&self) -> f64 {
        self.attention.read().await.attention_load()
    }
    
    /// Clear all conscious thoughts (reset)
    pub async fn clear(&self) {
        self.attention.write().await.clear();
        self.working_memory.write().await.clear();
    }
    
    /// Get count of registered modules
    pub async fn module_count(&self) -> usize {
        self.modules.read().await.len()
    }
}

impl Default for GlobalWorkspace {
    fn default() -> Self {
        Self::new()
    }
}
