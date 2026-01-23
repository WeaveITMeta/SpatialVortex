//! Causal Reasoning Engine for AGI
//!
//! Enables causal understanding through:
//! - Causal graph construction (cause -> effect relationships)
//! - Counterfactual reasoning ("what if X had been different?")
//! - Intervention planning (predict outcomes of actions)

use crate::data::models::ELPTensor;
use anyhow::Result;
use chrono::Utc;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalNode {
    pub id: Uuid,
    pub name: String,
    pub node_type: NodeType,
    pub value: Option<CausalValue>,
    pub elp_profile: ELPTensor,
    pub confidence: f32,
    pub observed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType { Observable, Latent, Intervention, Outcome, Confounder }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CausalValue {
    Boolean(bool),
    Numeric(f64),
    Categorical(String),
    ELP(ELPTensor),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalEdge {
    pub id: Uuid,
    pub from_node: Uuid,
    pub to_node: Uuid,
    pub edge_type: EdgeType,
    pub strength: f32,
    pub mechanism: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EdgeType { Direct, Mediated, Bidirectional, Confounding, Temporal }

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CausalGraph {
    pub nodes: HashMap<Uuid, CausalNode>,
    pub edges: Vec<CausalEdge>,
    pub adjacency: HashMap<Uuid, Vec<Uuid>>,
    pub reverse_adjacency: HashMap<Uuid, Vec<Uuid>>,
}

impl CausalGraph {
    pub fn new() -> Self { Self::default() }
    
    pub fn add_node(&mut self, node: CausalNode) {
        let id = node.id;
        self.nodes.insert(id, node);
        self.adjacency.entry(id).or_default();
        self.reverse_adjacency.entry(id).or_default();
    }
    
    pub fn add_edge(&mut self, edge: CausalEdge) {
        self.adjacency.entry(edge.from_node).or_default().push(edge.to_node);
        self.reverse_adjacency.entry(edge.to_node).or_default().push(edge.from_node);
        self.edges.push(edge);
    }
    
    pub fn get_causes(&self, node_id: Uuid) -> Vec<&CausalNode> {
        self.reverse_adjacency.get(&node_id)
            .map(|ids| ids.iter().filter_map(|id| self.nodes.get(id)).collect())
            .unwrap_or_default()
    }
    
    pub fn get_effects(&self, node_id: Uuid) -> Vec<&CausalNode> {
        self.adjacency.get(&node_id)
            .map(|ids| ids.iter().filter_map(|id| self.nodes.get(id)).collect())
            .unwrap_or_default()
    }
    
    pub fn is_acyclic(&self) -> bool {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        for &node_id in self.nodes.keys() {
            if self.has_cycle(node_id, &mut visited, &mut rec_stack) { return false; }
        }
        true
    }
    
    fn has_cycle(&self, node: Uuid, visited: &mut HashSet<Uuid>, stack: &mut HashSet<Uuid>) -> bool {
        if stack.contains(&node) { return true; }
        if visited.contains(&node) { return false; }
        visited.insert(node);
        stack.insert(node);
        if let Some(neighbors) = self.adjacency.get(&node) {
            for &n in neighbors { if self.has_cycle(n, visited, stack) { return true; } }
        }
        stack.remove(&node);
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intervention {
    pub id: Uuid,
    pub target_node: Uuid,
    pub new_value: CausalValue,
    pub intervention_type: InterventionType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum InterventionType { Do, Observe, Soft { strength: f32 } }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Counterfactual {
    pub id: Uuid,
    pub description: String,
    pub intervention: Intervention,
    pub query_variable: Uuid,
    pub factual_value: Option<CausalValue>,
    pub counterfactual_value: Option<CausalValue>,
    pub confidence: f32,
}

pub struct CausalWorldModel {
    pub causal_graph: CausalGraph,
    pub world_state: HashMap<String, CausalValue>,
    pub stats: CausalStats,
}

#[derive(Debug, Clone, Default)]
pub struct CausalStats {
    pub nodes_created: u64,
    pub edges_created: u64,
    pub interventions: u64,
    pub counterfactuals: u64,
}

impl Default for CausalWorldModel {
    fn default() -> Self { Self::new() }
}

impl CausalWorldModel {
    pub fn new() -> Self {
        Self { causal_graph: CausalGraph::new(), world_state: HashMap::new(), stats: CausalStats::default() }
    }
    
    pub fn learn_from_observation(&mut self, cause: &str, effect: &str, strength: f32, elp: &ELPTensor) {
        let cause_id = self.get_or_create_node(cause, NodeType::Observable, elp);
        let effect_id = self.get_or_create_node(effect, NodeType::Outcome, elp);
        let edge = CausalEdge {
            id: Uuid::new_v4(), from_node: cause_id, to_node: effect_id,
            edge_type: EdgeType::Direct, strength, mechanism: format!("{} causes {}", cause, effect),
        };
        self.causal_graph.add_edge(edge);
        self.stats.edges_created += 1;
    }
    
    fn get_or_create_node(&mut self, name: &str, node_type: NodeType, elp: &ELPTensor) -> Uuid {
        for (id, node) in &self.causal_graph.nodes {
            if node.name == name { return *id; }
        }
        let node = CausalNode {
            id: Uuid::new_v4(), name: name.to_string(), node_type,
            value: None, elp_profile: elp.clone(), confidence: 0.5, observed: false,
        };
        let id = node.id;
        self.causal_graph.add_node(node);
        self.stats.nodes_created += 1;
        id
    }
    
    pub fn simulate_intervention(&mut self, target: &str, value: CausalValue) -> Result<HashMap<String, CausalValue>> {
        let target_id = self.causal_graph.nodes.iter()
            .find(|(_, n)| n.name == target).map(|(id, _)| *id)
            .ok_or_else(|| anyhow::anyhow!("Target not found"))?;
        
        let mut results = HashMap::new();
        results.insert(target.to_string(), value.clone());
        
        let mut queue = VecDeque::new();
        queue.push_back(target_id);
        
        while let Some(current) = queue.pop_front() {
            if let Some(children) = self.causal_graph.adjacency.get(&current) {
                for &child in children {
                    if let Some(node) = self.causal_graph.nodes.get(&child) {
                        let parents = self.causal_graph.get_causes(child);
                        let vals: Vec<f64> = parents.iter()
                            .filter_map(|p| results.get(&p.name))
                            .filter_map(|v| match v { CausalValue::Numeric(n) => Some(*n), _ => None })
                            .collect();
                        if !vals.is_empty() {
                            results.insert(node.name.clone(), CausalValue::Numeric(vals.iter().sum::<f64>() / vals.len() as f64));
                            queue.push_back(child);
                        }
                    }
                }
            }
        }
        self.stats.interventions += 1;
        Ok(results)
    }
    
    pub fn explain(&self, outcome: &str) -> Result<String> {
        let outcome_id = self.causal_graph.nodes.iter()
            .find(|(_, n)| n.name == outcome).map(|(id, _)| *id)
            .ok_or_else(|| anyhow::anyhow!("Outcome not found"))?;
        let causes = self.causal_graph.get_causes(outcome_id);
        if causes.is_empty() { return Ok(format!("'{}' has no known causes.", outcome)); }
        let mut exp = format!("'{}' is caused by:\n", outcome);
        for c in causes {
            let s = self.causal_graph.edges.iter().find(|e| e.from_node == c.id && e.to_node == outcome_id).map(|e| e.strength).unwrap_or(0.5);
            exp.push_str(&format!("  - {} (strength: {:.2})\n", c.name, s));
        }
        Ok(exp)
    }
    
    /// Ask a counterfactual question
    pub fn ask_counterfactual(
        &mut self,
        description: &str,
        target: &str,
        new_value: CausalValue,
        query: &str,
    ) -> Result<Counterfactual> {
        let target_id = self.causal_graph.nodes.iter()
            .find(|(_, n)| n.name == target).map(|(id, _)| *id)
            .ok_or_else(|| anyhow::anyhow!("Target node not found: {}", target))?;
        
        let query_id = self.causal_graph.nodes.iter()
            .find(|(_, n)| n.name == query).map(|(id, _)| *id)
            .ok_or_else(|| anyhow::anyhow!("Query node not found: {}", query))?;
        
        // Get factual value
        let factual_value = self.causal_graph.nodes.get(&query_id).and_then(|n| n.value.clone());
        
        // Simulate intervention to get counterfactual
        let results = self.simulate_intervention(target, new_value.clone())?;
        let counterfactual_value = results.get(query).cloned();
        
        // Compute confidence based on path length and edge strengths
        let confidence = self.compute_counterfactual_confidence(target_id, query_id);
        
        self.stats.counterfactuals += 1;
        
        Ok(Counterfactual {
            id: Uuid::new_v4(),
            description: description.to_string(),
            intervention: Intervention {
                id: Uuid::new_v4(),
                target_node: target_id,
                new_value,
                intervention_type: InterventionType::Do,
            },
            query_variable: query_id,
            factual_value,
            counterfactual_value,
            confidence,
        })
    }
    
    fn compute_counterfactual_confidence(&self, from: Uuid, to: Uuid) -> f32 {
        // Simple heuristic: confidence decreases with path length
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back((from, 1.0_f32));
        
        while let Some((current, conf)) = queue.pop_front() {
            if current == to { return conf; }
            if visited.contains(&current) { continue; }
            visited.insert(current);
            
            if let Some(children) = self.causal_graph.adjacency.get(&current) {
                for &child in children {
                    let edge_strength = self.causal_graph.edges.iter()
                        .find(|e| e.from_node == current && e.to_node == child)
                        .map(|e| e.strength)
                        .unwrap_or(0.5);
                    queue.push_back((child, conf * edge_strength * 0.9));
                }
            }
        }
        0.3 // Default low confidence if no path found
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_causal_learning() {
        let mut model = CausalWorldModel::new();
        let elp = ELPTensor { ethos: 5.0, logos: 7.0, pathos: 3.0 };
        model.learn_from_observation("Smoking", "Cancer", 0.8, &elp);
        assert_eq!(model.causal_graph.nodes.len(), 2);
        assert_eq!(model.causal_graph.edges.len(), 1);
    }
    
    #[test]
    fn test_intervention() {
        let mut model = CausalWorldModel::new();
        let elp = ELPTensor { ethos: 5.0, logos: 7.0, pathos: 3.0 };
        model.learn_from_observation("Exercise", "Health", 0.9, &elp);
        let results = model.simulate_intervention("Exercise", CausalValue::Numeric(1.0)).unwrap();
        assert!(results.contains_key("Exercise"));
    }
}
