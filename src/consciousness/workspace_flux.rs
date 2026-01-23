//! Flux Workspace Orchestrator
//!
//! Table of Contents
//! 1. Purpose
//! 2. Types
//! 3. Construction
//! 4. One-tick Pipeline
//! 5. Metrics (Φ proxy, recurrence)
//! 6. Builders (5W1H, SelfModel)
//! 7. Simple Encoders
//! 8. Tests
//!
//! Purpose
//! - Numeric global workspace that aggregates subject states using projection (P_s) and broadcast (B_s)
//! - Salience-based selection (top-k) with predictive processing gating via surprise
//! - Memory buffer for recurrent context and basic metrics for instrumentation

use nalgebra::DVector;
use std::collections::VecDeque;

use super::predictive_processing::PredictiveProcessor;
use super::subject_graph::{SubjectDefinition, SubjectGraph, SubjectId};

#[derive(Clone, Debug, Default)]
pub struct FluxMetrics {
    pub selected: usize,
    pub avg_salience: f32,
    pub phi_proxy: f32,
    pub recurrence_ratio: f32,
    pub subject_occupancy: f32,
    pub workspace_occupancy: f32,
    pub pred_error: f32,
}

pub struct FluxWorkspace {
    workspace_dim: usize,
    graph: SubjectGraph,
    memory: VecDeque<DVector<f32>>, // recent workspace states
    memory_capacity: usize,
}

impl FluxWorkspace {
    pub fn new(workspace_dim: usize, memory_capacity: usize) -> Self {
        Self {
            workspace_dim,
            graph: SubjectGraph::new(workspace_dim),
            memory: VecDeque::with_capacity(memory_capacity),
            memory_capacity,
        }
    }

    pub fn graph_mut(&mut self) -> &mut SubjectGraph { &mut self.graph }
    pub fn graph(&self) -> &SubjectGraph { &self.graph }

    pub fn add_subject(&mut self, name: &str, dim: usize) -> SubjectId {
        let def = SubjectDefinition::new(name.to_string(), dim, self.workspace_dim);
        self.graph.add_subject(def)
    }

    pub fn connect(&mut self, from: SubjectId, to: SubjectId) -> bool {
        self.graph.add_edge(from, to)
    }

    /// Add canonical 5W1H subjects (Who, What, When, Where, How, Why)
    /// Returns the vector of created subject IDs in that order.
    pub fn add_5w1h(&mut self, dim_each: usize) -> Vec<SubjectId> {
        let names = ["Who", "What", "When", "Where", "How", "Why"];
        let mut ids = Vec::with_capacity(names.len());
        for n in names { ids.push(self.add_subject(n, dim_each)); }
        // Simple ring connectivity to encourage flow
        for i in 0..ids.len() {
            let from = ids[i];
            let to = ids[(i + 1) % ids.len()];
            let _ = self.connect(from, to);
        }
        ids
    }

    /// Add a SelfModel subject (introspection channel) sized to workspace.
    /// Connects all existing subjects → SelfModel for monitoring.
    pub fn add_self_model(&mut self) -> SubjectId {
        let id = self.add_subject("SelfModel", self.workspace_dim);
        let keys: Vec<SubjectId> = self.graph.subjects.keys().copied().collect();
        for k in keys {
            if k != id { let _ = self.connect(k, id); }
        }
        id
    }

    pub fn step_with_surprise(
        &mut self,
        workspace_in: &DVector<f32>,
        surprise: f64,
        top_k: usize,
    ) -> (DVector<f32>, FluxMetrics) {
        let workspace_out = self.graph.step(workspace_in, surprise, top_k);
        self.push_memory(workspace_out.clone());
        let metrics = self.compute_metrics(top_k, &workspace_out, surprise);
        (workspace_out, metrics)
    }

    pub fn step_with_predictor(
        &mut self,
        workspace_in: &DVector<f32>,
        predictor: &PredictiveProcessor,
        top_k: usize,
    ) -> (DVector<f32>, FluxMetrics) {
        let surprise = predictor.current_surprise();
        self.step_with_surprise(workspace_in, surprise, top_k)
    }

    fn push_memory(&mut self, v: DVector<f32>) {
        if self.memory.len() == self.memory_capacity { self.memory.pop_front(); }
        self.memory.push_back(v);
    }

    fn compute_metrics(&self, last_top_k: usize, last_ws: &DVector<f32>, surprise: f64) -> FluxMetrics {
        // avg salience across subjects
        let mut total = 0.0f32;
        let mut count = 0usize;
        let mut active = 0usize;
        for subj in self.graph.subjects.values() {
            total += subj.state.salience;
            if subj.state.salience > 1e-6 { active += 1; }
            count += 1;
        }
        let avg_sal = if count > 0 { total / count as f32 } else { 0.0 };
        let subject_occupancy = if count > 0 { active as f32 / count as f32 } else { 0.0 };

        // φ proxy: energy of last memory vec vs sum of subject energies
        let phi_proxy = last_ws.norm();

        // recurrence proxy: fraction of non-zero gates (assume >0 always here)
        let m = self.graph.edges.len().max(1) as f32;
        let recurrence_ratio = (self.graph.edges.iter().filter(|e| e.gate > 0.0).count() as f32) / m;

        // workspace occupancy: fraction of elements with non-trivial activation
        let mut nz = 0usize;
        for i in 0..last_ws.len() { if last_ws[i].abs() > 1e-6 { nz += 1; } }
        let workspace_occupancy = if last_ws.len() > 0 { nz as f32 / last_ws.len() as f32 } else { 0.0 };
        let pred_error = surprise as f32;

        FluxMetrics {
            selected: last_top_k,
            avg_salience: avg_sal,
            phi_proxy: phi_proxy as f32,
            recurrence_ratio,
            subject_occupancy,
            workspace_occupancy,
            pred_error,
        }
    }

    pub fn memory_len(&self) -> usize { self.memory.len() }
}

// Simple encoders for demo/testing
pub fn encode_text_to_workspace(text: &str, workspace_dim: usize) -> DVector<f32> {
    // Very simple: bag-of-chars hashed into workspace_dim, scaled
    let mut v = DVector::<f32>::zeros(workspace_dim);
    for (i, ch) in text.chars().enumerate() {
        let idx = (ch as usize + i) % workspace_dim;
        v[idx] += 1.0;
    }
    let n = v.norm();
    if n > 0.0 { v / n } else { v }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_tick() {
        let mut ws = FluxWorkspace::new(16, 4);
        let a = ws.add_subject("Ethics", 8);
        let b = ws.add_subject("Logic", 8);
        ws.connect(a, b);
        ws.connect(b, a);

        let input = encode_text_to_workspace("hello", 16);
        let (out, metrics) = ws.step_with_surprise(&input, 0.3, 1);
        assert_eq!(out.len(), 16);
        assert!(metrics.avg_salience >= 0.0);
        assert!(ws.memory_len() > 0);
    }
}
