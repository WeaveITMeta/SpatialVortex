use nalgebra::{DMatrix, DVector};
use std::collections::HashMap;
use uuid::Uuid;

pub type SubjectId = Uuid;

#[derive(Clone)]
pub struct SubjectDefinition {
    pub id: SubjectId,
    pub name: String,
    pub dim: usize,
    pub workspace_dim: usize,
    pub projection: DMatrix<f32>,
    pub broadcast: DMatrix<f32>,
    pub mask: Option<DVector<f32>>,
}

impl SubjectDefinition {
    pub fn new(name: impl Into<String>, dim: usize, workspace_dim: usize) -> Self {
        let name = name.into();
        let id = Uuid::new_v4();
        let mut p = DMatrix::<f32>::zeros(dim, workspace_dim);
        let m = dim.min(workspace_dim);
        for i in 0..m {
            p[(i, i)] = 1.0;
        }
        let b = p.transpose();
        Self { id, name, dim, workspace_dim, projection: p, broadcast: b, mask: None }
    }

    pub fn with_mask(mut self, mask: DVector<f32>) -> Self {
        self.mask = Some(mask);
        self
    }
}

#[derive(Clone)]
pub struct SubjectState {
    pub id: SubjectId,
    pub x: DVector<f32>,
    pub salience: f32,
    pub last_surprise: f64,
}

impl SubjectState {
    pub fn new(id: SubjectId, dim: usize) -> Self {
        Self { id, x: DVector::zeros(dim), salience: 0.0, last_surprise: 0.0 }
    }
}

#[derive(Clone)]
pub struct FluxEdge {
    pub from: SubjectId,
    pub to: SubjectId,
    pub kernel: DMatrix<f32>,
    pub gate: f32,
    pub fact_u: Option<DMatrix<f32>>,    
    pub fact_s: Option<DVector<f32>>,    
    pub fact_vt: Option<DMatrix<f32>>,   
}

impl FluxEdge {
    pub fn new(from: SubjectId, to: SubjectId, dim_from: usize, dim_to: usize) -> Self {
        let mut k = DMatrix::<f32>::zeros(dim_to, dim_from);
        let m = dim_from.min(dim_to);
        for i in 0..m {
            k[(i, i)] = 1.0;
        }
        Self { from, to, kernel: k, gate: 1.0, fact_u: None, fact_s: None, fact_vt: None }
    }

    pub fn set_factorized(&mut self, u: DMatrix<f32>, s: DVector<f32>, vt: DMatrix<f32>) {
        self.fact_u = Some(u);
        self.fact_s = Some(s);
        self.fact_vt = Some(vt);
    }

    fn apply(&self, x: &DVector<f32>) -> DVector<f32> {
        if let (Some(u), Some(s), Some(vt)) = (&self.fact_u, &self.fact_s, &self.fact_vt) {
            let y1 = vt * x;
            let mut y2 = y1.clone();
            for i in 0..s.len().min(y2.len()) { y2[i] *= s[i]; }
            u * y2
        } else {
            &self.kernel * x
        }
    }
}

pub struct Subject {
    pub def: SubjectDefinition,
    pub state: SubjectState,
}

pub struct SubjectGraph {
    pub workspace_dim: usize,
    pub subjects: HashMap<SubjectId, Subject>,
    pub edges: Vec<FluxEdge>,
}

impl SubjectGraph {
    pub fn new(workspace_dim: usize) -> Self {
        Self { workspace_dim, subjects: HashMap::new(), edges: Vec::new() }
    }

    pub fn add_subject(&mut self, def: SubjectDefinition) -> SubjectId {
        let id = def.id;
        let state = SubjectState::new(id, def.dim);
        self.subjects.insert(id, Subject { def, state });
        id
    }

    pub fn add_edge(&mut self, from: SubjectId, to: SubjectId) -> bool {
        let (dim_from, dim_to) = match (self.subjects.get(&from), self.subjects.get(&to)) {
            (Some(a), Some(b)) => (a.def.dim, b.def.dim),
            _ => return false,
        };
        let edge = FluxEdge::new(from, to, dim_from, dim_to);
        self.edges.push(edge);
        true
    }

    pub fn set_mask_indices(&mut self, id: SubjectId, indices: &[usize]) {
        if let Some(subj) = self.subjects.get_mut(&id) {
            let mut mask = DVector::<f32>::from_element(subj.def.dim, 0.0);
            for &idx in indices { if idx < mask.len() { mask[idx] = 1.0; } }
            subj.def.mask = Some(mask);
        }
    }

    fn compute_salience(x: &DVector<f32>) -> f32 {
        x.norm()
    }

    pub fn step(&mut self, workspace_in: &DVector<f32>, surprise: f64, top_k: usize) -> DVector<f32> {
        let mut incoming: HashMap<SubjectId, DVector<f32>> = HashMap::new();
        for (id, subj) in self.subjects.iter() {
            let v = DVector::<f32>::zeros(subj.def.dim);
            incoming.insert(*id, v);
        }
        let s = (surprise as f32).clamp(0.0, 1.0);
        for e in &mut self.edges {
            e.gate = 0.9 * e.gate + 0.1 * (0.5 + 0.5 * s);
        }
        for e in &self.edges {
            if let (Some(from_subj), Some(_to_subj)) = (self.subjects.get(&e.from), self.subjects.get(&e.to)) {
                let y = e.apply(&from_subj.state.x);
                if let Some(acc) = incoming.get_mut(&e.to) {
                    *acc += y * e.gate;
                }
            }
        }
        for subj in self.subjects.values_mut() {
            let p_in = &subj.def.projection * workspace_in;
            let mut pre = &subj.state.x + p_in;
            if let Some(add) = incoming.get(&subj.state.id) {
                pre += add;
            }
            let gate = (0.5 + 0.5 * (surprise as f32).clamp(0.0, 1.0)).clamp(0.0, 1.0);
            let mut x_next = pre * gate + subj.state.x.clone() * (1.0 - gate);
            if let Some(mask) = &subj.def.mask {
                x_next.component_mul_assign(mask);
            }
            subj.state.x = x_next;
            subj.state.salience = Self::compute_salience(&subj.state.x);
            subj.state.last_surprise = surprise;
        }
        let mut ids: Vec<(SubjectId, f32)> = self.subjects.values().map(|s| (s.state.id, s.state.salience)).collect();
        ids.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let selected_pairs: Vec<(SubjectId, f32)> = ids.into_iter().take(top_k).collect();
        let sum_sal: f32 = selected_pairs.iter().map(|(_, s)| *s).sum();
        let mut workspace_out = DVector::<f32>::zeros(self.workspace_dim);
        for (id, sal) in selected_pairs {
            if let Some(subj) = self.subjects.get(&id) {
                let w = if sum_sal > 0.0 { sal / sum_sal } else { 1.0 / (top_k.max(1) as f32) };
                workspace_out += w * (&subj.def.broadcast * &subj.state.x);
            }
        }
        workspace_out
    }

    pub fn hebbian_update(&mut self, lr: f32, max_frob: f32) {
        for e in &mut self.edges {
            let (xf, xt) = match (self.subjects.get(&e.from), self.subjects.get(&e.to)) {
                (Some(a), Some(b)) => (&a.state.x, &b.state.x),
                _ => continue,
            };
            let outer = DMatrix::<f32>::from_fn(xt.len(), xf.len(), |i, j| xt[i] * xf[j]);
            e.kernel = &e.kernel + lr * outer;
            let frob = e.kernel.norm();
            if frob > max_frob {
                e.kernel *= max_frob / frob;
            }
        }
    }

    pub fn topk_subjects(&self, k: usize) -> Vec<SubjectId> {
        let mut ids: Vec<(SubjectId, f32)> = self.subjects.values().map(|s| (s.state.id, s.state.salience)).collect();
        ids.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        ids.into_iter().take(k).map(|t| t.0).collect()
    }
}
