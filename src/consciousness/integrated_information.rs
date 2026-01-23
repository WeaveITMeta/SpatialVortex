//! Integrated Information Theory (IIT) - Φ (Phi) Calculation
//!
//! Measures consciousness as integrated information (Φ).
//! Higher Φ = more conscious. Φ = Information in whole - Information in parts.

use super::thought::Thought;
use std::collections::HashMap;

/// Maximum nodes to consider for Φ calculation (computational limit)
const MAX_PHI_NODES: usize = 10;

/// Integrated Information calculator
#[derive(Debug)]
pub struct IntegratedInformationCalculator {
    /// Network of connected thoughts
    thought_network: Vec<ThoughtNode>,
    
    /// Connections between thoughts
    connections: HashMap<usize, Vec<usize>>,
    
    /// Current Φ value
    current_phi: f64,
    
    /// Historical Φ values
    phi_history: Vec<f64>,
}

/// A node in the thought network
#[derive(Debug, Clone)]
struct ThoughtNode {
    id: usize,
    thought: Thought,
    information_content: f64,
}

impl IntegratedInformationCalculator {
    pub fn new() -> Self {
        Self {
            thought_network: Vec::new(),
            connections: HashMap::new(),
            current_phi: 0.0,
            phi_history: Vec::new(),
        }
    }
    
    /// Add a thought to the network
    pub fn add_thought(&mut self, thought: Thought) {
        let id = self.thought_network.len();
        
        let information_content = self.calculate_information_content(&thought);
        
        let node = ThoughtNode {
            id,
            thought,
            information_content,
        };
        
        self.thought_network.push(node);
        
        // Auto-connect to recent thoughts (recency bias)
        if id > 0 {
            let mut new_connections = vec![id - 1]; // Always connect to previous
            
            // Maybe connect to earlier thoughts based on similarity
            for prev_id in (id.saturating_sub(5))..id {
                if self.are_related(prev_id, id) {
                    new_connections.push(prev_id);
                }
            }
            
            self.connections.insert(id, new_connections);
        }
        
        // Trim network if too large
        if self.thought_network.len() > MAX_PHI_NODES {
            self.prune_network();
        }
        
        // Recalculate Φ
        self.calculate_phi();
    }
    
    /// Calculate information content of a thought
    fn calculate_information_content(&self, thought: &Thought) -> f64 {
        // Information = -log2(probability)
        // Use ELP diversity as proxy for information
        
        let elp_entropy = self.entropy([thought.ethos, thought.logos, thought.pathos]);
        let confidence_info = -thought.confidence.max(0.01).log2();
        
        (elp_entropy + confidence_info) / 2.0
    }
    
    /// Calculate entropy of a distribution
    fn entropy(&self, distribution: [f64; 3]) -> f64 {
        distribution.iter()
            .filter(|&&p| p > 0.0)
            .map(|&p| -p * p.log2())
            .sum()
    }
    
    /// Check if two thoughts are related
    fn are_related(&self, id1: usize, id2: usize) -> bool {
        if id1 >= self.thought_network.len() || id2 >= self.thought_network.len() {
            return false;
        }
        
        let t1 = &self.thought_network[id1].thought;
        let t2 = &self.thought_network[id2].thought;
        
        // Related if similar ELP profile or same source
        let elp_similarity = 1.0 - ((t1.ethos - t2.ethos).abs() 
                                  + (t1.logos - t2.logos).abs() 
                                  + (t1.pathos - t2.pathos).abs()) / 3.0;
        
        elp_similarity > 0.7 || t1.source_module == t2.source_module
    }
    
    /// Calculate Φ (integrated information)
    fn calculate_phi(&mut self) {
        if self.thought_network.is_empty() {
            self.current_phi = 0.0;
            return;
        }
        
        // Φ = Information in whole - Information in parts
        
        // 1. Information in the whole system
        let whole_info = self.system_information();
        
        // 2. Information in parts (sum of individual nodes)
        let parts_info = self.parts_information();
        
        // 3. Φ = Integration
        let phi = (whole_info - parts_info).max(0.0);
        
        self.current_phi = phi;
        self.phi_history.push(phi);
        
        // Keep only recent history
        if self.phi_history.len() > 100 {
            self.phi_history.remove(0);
        }
    }
    
    /// Calculate information in the whole system
    fn system_information(&self) -> f64 {
        if self.thought_network.is_empty() {
            return 0.0;
        }
        
        // System information = sum of node information + connection information
        let node_info: f64 = self.thought_network
            .iter()
            .map(|n| n.information_content)
            .sum();
        
        let connection_info = self.connections.len() as f64 * 0.5; // Each connection adds info
        
        node_info + connection_info
    }
    
    /// Calculate information in parts (non-integrated)
    fn parts_information(&self) -> f64 {
        // Parts = just sum of individual nodes without connections
        self.thought_network
            .iter()
            .map(|n| n.information_content)
            .sum()
    }
    
    /// Prune network to maintain size limit
    fn prune_network(&mut self) {
        // Keep only recent nodes
        let keep_count = MAX_PHI_NODES / 2;
        let remove_count = self.thought_network.len() - keep_count;
        
        // Remove oldest nodes
        self.thought_network.drain(0..remove_count);
        
        // Update connections (shift IDs)
        let mut new_connections = HashMap::new();
        for (id, conns) in &self.connections {
            if *id >= remove_count {
                let new_id = id - remove_count;
                let new_conns: Vec<usize> = conns
                    .iter()
                    .filter_map(|&c| {
                        if c >= remove_count {
                            Some(c - remove_count)
                        } else {
                            None
                        }
                    })
                    .collect();
                new_connections.insert(new_id, new_conns);
            }
        }
        self.connections = new_connections;
        
        // Update node IDs
        for (new_id, node) in self.thought_network.iter_mut().enumerate() {
            node.id = new_id;
        }
    }
    
    /// Get current Φ value
    pub fn phi(&self) -> f64 {
        self.current_phi
    }
    
    /// Get average Φ over recent history
    pub fn average_phi(&self) -> f64 {
        if self.phi_history.is_empty() {
            return 0.0;
        }
        
        let recent: Vec<f64> = self.phi_history
            .iter()
            .rev()
            .take(20)
            .copied()
            .collect();
        
        recent.iter().sum::<f64>() / recent.len() as f64
    }
    
    /// Get peak Φ value
    pub fn peak_phi(&self) -> f64 {
        self.phi_history
            .iter()
            .copied()
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }
    
    /// Get consciousness level (Φ normalized to 0-1 scale)
    pub fn consciousness_level(&self) -> f64 {
        // Normalize Φ to reasonable range (0-10 typical, cap at 1.0)
        (self.current_phi / 10.0).min(1.0)
    }
    
    /// Get network size (number of thoughts)
    pub fn network_size(&self) -> usize {
        self.thought_network.len()
    }
    
    /// Get total connection count
    pub fn connection_count(&self) -> usize {
        self.connections.values().map(|v| v.len()).sum()
    }
    
    /// Generate Φ report
    pub fn phi_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("=== Integrated Information (Φ) Report ===\n\n");
        
        report.push_str(&format!("Current Φ: {:.3}\n", self.phi()));
        report.push_str(&format!("Average Φ: {:.3}\n", self.average_phi()));
        report.push_str(&format!("Peak Φ: {:.3}\n", self.peak_phi()));
        report.push_str(&format!("Consciousness Level: {:.1}%\n\n", self.consciousness_level() * 100.0));
        
        report.push_str(&format!("Network Size: {} thoughts\n", self.thought_network.len()));
        report.push_str(&format!("Connections: {} links\n", self.connections.len()));
        report.push_str(&format!("Integration: {:.1}%\n", 
            (self.current_phi / self.system_information().max(0.01)) * 100.0));
        
        report
    }
}

impl Default for IntegratedInformationCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consciousness::thought::ThoughtPriority;
    
    #[test]
    fn test_phi_calculation() {
        let mut calc = IntegratedInformationCalculator::new();
        
        assert_eq!(calc.phi(), 0.0);
        
        // Add some thoughts
        for i in 0..5 {
            let thought = Thought::new(
                format!("thought_{}", i),
                "agent".to_string(),
                ThoughtPriority::Medium,
            ).with_elp(0.5, 0.3, 0.2);
            
            calc.add_thought(thought);
        }
        
        // Φ should be positive (integration occurred)
        assert!(calc.phi() > 0.0);
        assert!(calc.consciousness_level() >= 0.0);
        assert!(calc.consciousness_level() <= 1.0);
    }
    
    #[test]
    fn test_network_pruning() {
        let mut calc = IntegratedInformationCalculator::new();
        
        // Add more than MAX_PHI_NODES
        for i in 0..(MAX_PHI_NODES + 5) {
            let thought = Thought::new(
                format!("thought_{}", i),
                "agent".to_string(),
                ThoughtPriority::Medium,
            );
            calc.add_thought(thought);
        }
        
        // Network should be pruned
        assert!(calc.thought_network.len() <= MAX_PHI_NODES);
    }
}
