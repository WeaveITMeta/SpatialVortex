//! Meta-Learning Module
//! 
//! Implements learning from reasoning chains to improve future performance.
//! Stores successful reasoning patterns in PostgreSQL and retrieves them
//! to accelerate future queries.

use crate::ai::flux_reasoning::{FluxReasoningChain, FluxThought};
use crate::error::Result;
use crate::storage::SpatialDatabase;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// A learned reasoning pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningPattern {
    pub id: Uuid,
    pub domain: String,
    pub complexity: f32,
    pub keywords: Vec<String>,
    pub initial_state: crate::models::ELPTensor,
    pub successful_steps: Vec<ReasoningStepPattern>,
    pub final_state: crate::models::ELPTensor,
    pub success_rate: f32,
    pub created_at: DateTime<Utc>,
}

/// A condensed step in a reasoning pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningStepPattern {
    pub vortex_position: u8,
    pub action: String, // "Internal" or "Oracle"
    pub transformation: String, // Description of what changed
}

/// Meta-learning engine
pub struct MetaLearningEngine {
    db: SpatialDatabase,
}

impl MetaLearningEngine {
    pub fn new(db: SpatialDatabase) -> Self {
        Self { db }
    }

    /// Extract and store a pattern from a successful reasoning chain
    pub async fn learn_from_chain(&self, chain: &FluxReasoningChain) -> Result<Uuid> {
        let id = Uuid::new_v4();
        
        // Extract keywords from query (simple heuristic for now)
        let keywords: Vec<String> = chain.query
            .split_whitespace()
            .map(|s| s.to_lowercase())
            .filter(|s| s.len() > 4)
            .collect();
            
        // Simplify steps into a pattern
        let steps: Vec<ReasoningStepPattern> = chain.thoughts.iter().map(|t| {
            ReasoningStepPattern {
                vortex_position: t.vortex_position,
                action: if !t.oracle_contributions.is_empty() { 
                    "Oracle".to_string() 
                } else { 
                    "Internal".to_string() 
                },
                transformation: t.reasoning_trace.clone(),
            }
        }).collect();
        
        let initial = chain.thoughts.first().unwrap().elp_state;
        let final_state = chain.thoughts.last().unwrap().elp_state;
        
        let pattern = ReasoningPattern {
            id,
            domain: "general".to_string(), // TODO: Classify domain
            complexity: 0.5, // TODO: Calculate complexity
            keywords,
            initial_state: initial,
            successful_steps: steps,
            final_state,
            success_rate: chain.chain_confidence,
            created_at: Utc::now(),
        };
        
        // Store in DB
        self.store_pattern(&pattern).await?;
        
        Ok(id)
    }
    
    async fn store_pattern(&self, pattern: &ReasoningPattern) -> Result<()> {
        let client = self.db.get_pool().get().await
            .map_err(|e| crate::error::SpatialVortexError::Database(e.to_string()))?;
            
        let data = serde_json::to_value(pattern)?;
        
        // We'll use the generic flux_matrices table for now, but with a special subject prefix
        // In a real impl, we'd make a dedicated table
        client.execute(
            "INSERT INTO flux_matrices (id, subject, data, created_at, updated_at) VALUES ($1, $2, $3, $4, $4)",
            &[&pattern.id, &format!("pattern:{}", pattern.id), &data, &pattern.created_at]
        ).await.map_err(|e| crate::error::SpatialVortexError::Database(e.to_string()))?;
        
        Ok(())
    }
    
    /// Retrieve relevant patterns for a new query
    pub async fn retrieve_relevant_patterns(&self, query: &str) -> Result<Vec<ReasoningPattern>> {
        // Simple keyword matching
        let query_words: Vec<String> = query.split_whitespace()
            .map(|s| s.to_lowercase())
            .collect();
            
        // In a real implementation, we would use pgvector or full-text search
        // Here we just grab recent patterns and filter in memory (prototype)
        let client = self.db.get_pool().get().await
            .map_err(|e| crate::error::SpatialVortexError::Database(e.to_string()))?;
            
        let rows = client.query(
            "SELECT data FROM flux_matrices WHERE subject LIKE 'pattern:%' ORDER BY created_at DESC LIMIT 50",
            &[]
        ).await.map_err(|e| crate::error::SpatialVortexError::Database(e.to_string()))?;
        
        let mut relevant = Vec::new();
        for row in rows {
            let data: serde_json::Value = row.get(0);
            if let Ok(pattern) = serde_json::from_value::<ReasoningPattern>(data) {
                // Check overlap
                let overlap = pattern.keywords.iter().filter(|k| query_words.contains(k)).count();
                if overlap > 0 {
                    relevant.push(pattern);
                }
            }
        }
        
        Ok(relevant)
    }
}
