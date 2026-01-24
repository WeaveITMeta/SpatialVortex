//! Flux Reasoning Chain - Sacred geometry reasoning

use crate::data::attributes::Attributes;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Default)]
pub struct FluxReasoningChain {
    thoughts: Vec<FluxThought>,
    current_position: u8,
}

impl FluxReasoningChain {
    pub fn new() -> Self { Self { thoughts: Vec::new(), current_position: 1 } }

    pub fn add_thought(&mut self, content: String, confidence: f32) -> &FluxThought {
        let pos = self.current_position;
        let is_sacred = matches!(pos, 3 | 6 | 9);
        
        self.thoughts.push(FluxThought {
            content,
            position: pos,
            confidence: if is_sacred { (confidence * 1.15).min(1.0) } else { confidence },
            is_sacred,
            timestamp: Utc::now(),
            attributes: Attributes::new(),
        });

        self.current_position = match self.current_position {
            1 => 2, 2 => 4, 4 => 8, 8 => 7, 7 => 5, 5 => 1, _ => 1,
        };

        self.thoughts.last().unwrap()
    }

    pub fn thoughts(&self) -> &[FluxThought] { &self.thoughts }
    
    pub fn chain_confidence(&self) -> f32 {
        if self.thoughts.is_empty() { return 0.0; }
        self.thoughts.iter().map(|t| t.confidence).sum::<f32>() / self.thoughts.len() as f32
    }

    pub fn synthesize(&self) -> String {
        self.thoughts.iter().map(|t| t.content.as_str()).collect::<Vec<_>>().join(" → ")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluxThought {
    pub content: String,
    pub position: u8,
    pub confidence: f32,
    pub is_sacred: bool,
    pub timestamp: DateTime<Utc>,
    pub attributes: Attributes,
}
