//! Scene3D WorldState — 3D workshop scenario domain (Phase 2 stub).
//!
//! Implements WorldState for 3D scenes with objects, tools, and physics.
//! The same `solve::<Scene3D>()` function that solves ARC grids also
//! solves 3D workshop tasks — cross-domain transfer via shared CausalGraph.
//!
//! Phase 2 will flesh this out with Bevy/Rapier integration.

use eustress_vortex_core::{
    DSLOp, Delta, Domain, Property, PropertyValue, Score, WorldState,
};
use serde::{Deserialize, Serialize};

/// A 3D object in the workshop scene.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneObject {
    pub id: String,
    pub class_name: String,
    pub position: [f64; 3],
    pub rotation: [f64; 4], // quaternion
    pub scale: [f64; 3],
    pub properties: serde_json::Value,
}

/// A tool available in the workshop.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub actions: Vec<String>,
    pub parameters: serde_json::Value,
}

/// A 3D workshop scene implementing WorldState.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Scene3D {
    pub objects: Vec<SceneObject>,
    pub tools: Vec<ToolDefinition>,
    pub constraints: Vec<Constraint>,
    pub tick: u64,
}

/// Physical constraint on the scene.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Constraint {
    /// Object must reach a target position.
    PositionTarget { object_id: String, target: [f64; 3], tolerance: f64 },
    /// A property must reach a target value.
    PropertyTarget { object_id: String, property: String, target: f64, tolerance: f64 },
    /// Objects must be in a specific order (e.g., stacked).
    Ordering { object_ids: Vec<String>, axis: Axis },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Axis { X, Y, Z }

/// Workshop action — what Vortex can do in 3D.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum WorkshopAction {
    ApplyForce { object_id: String, force: [f64; 3] },
    UseTool { tool: String, target_object: String, params: serde_json::Value },
    MoveObject { object_id: String, target: [f64; 3] },
    RotateObject { object_id: String, rotation: [f64; 4] },
    SimulatePhysics { ticks: u32 },
}

/// Workshop scenario — the 3D equivalent of an ARC task.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkshopScenario {
    pub id: String,
    pub description: String,
    pub initial_state: Scene3D,
    pub goal: WorkshopGoal,
    pub available_tools: Vec<ToolDefinition>,
    pub demonstrations: Vec<(Scene3D, Vec<WorkshopAction>, Scene3D)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WorkshopGoal {
    ExactState(Scene3D),
    Predicate(String),
    ScoreFunction(String),
}

impl Scene3D {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            tools: Vec::new(),
            constraints: Vec::new(),
            tick: 0,
        }
    }

    pub fn add_object(&mut self, obj: SceneObject) {
        self.objects.push(obj);
    }

    pub fn add_tool(&mut self, tool: ToolDefinition) {
        self.tools.push(tool);
    }

    fn constraint_satisfaction(&self, goal: &Self) -> f64 {
        if goal.objects.is_empty() {
            return 0.0;
        }

        let mut total_score = 0.0;
        let mut count = 0;

        for goal_obj in &goal.objects {
            if let Some(current_obj) = self.objects.iter().find(|o| o.id == goal_obj.id) {
                let dist = euclidean_distance(&current_obj.position, &goal_obj.position);
                let max_dist = 100.0; // normalizer
                let position_score = (1.0 - dist / max_dist).max(0.0);
                total_score += position_score;
                count += 1;
            }
        }

        if count == 0 { 0.0 } else { total_score / count as f64 }
    }
}

impl Default for Scene3D {
    fn default() -> Self {
        Self::new()
    }
}

impl WorldState for Scene3D {
    fn analyze(&self) -> Vec<Property> {
        let mut props = Vec::new();

        props.push(Property {
            name: "object_count".into(),
            domain: Domain::Scene3D,
            value: PropertyValue::Int(self.objects.len() as i64),
        });

        props.push(Property {
            name: "tool_count".into(),
            domain: Domain::Scene3D,
            value: PropertyValue::Int(self.tools.len() as i64),
        });

        // Spatial relationships
        for (i, a) in self.objects.iter().enumerate() {
            for b in self.objects.iter().skip(i + 1) {
                let dist = euclidean_distance(&a.position, &b.position);
                props.push(Property {
                    name: format!("distance_{}_{}", a.id, b.id),
                    domain: Domain::Scene3D,
                    value: PropertyValue::Float(dist),
                });

                // Vertical ordering
                if a.position[1] > b.position[1] + 0.1 {
                    props.push(Property {
                        name: format!("{}_above_{}", a.id, b.id),
                        domain: Domain::Scene3D,
                        value: PropertyValue::Bool(true),
                    });
                }
            }
        }

        // Check for unsupported objects (nothing below on Y axis)
        for obj in &self.objects {
            let has_support = self.objects.iter().any(|other| {
                other.id != obj.id
                    && other.position[1] < obj.position[1]
                    && euclidean_distance_xz(&other.position, &obj.position) < 2.0
            });
            if !has_support && obj.position[1] > 0.1 {
                props.push(Property {
                    name: "has_unsupported_objects".into(),
                    domain: Domain::Scene3D,
                    value: PropertyValue::Bool(true),
                });
                break;
            }
        }

        props
    }

    fn available_actions(&self) -> Vec<DSLOp> {
        let mut ops = Vec::new();

        // Generate actions for each object × tool combination
        for obj in &self.objects {
            ops.push(DSLOp {
                name: format!("move_{}_up", obj.id),
                domain: Domain::Scene3D,
                parameters: vec![serde_json::json!(obj.id)],
            });
            ops.push(DSLOp {
                name: format!("move_{}_down", obj.id),
                domain: Domain::Scene3D,
                parameters: vec![serde_json::json!(obj.id)],
            });

            for tool in &self.tools {
                ops.push(DSLOp {
                    name: format!("use_{}_{}", tool.name, obj.id),
                    domain: Domain::Scene3D,
                    parameters: vec![
                        serde_json::json!(tool.name),
                        serde_json::json!(obj.id),
                    ],
                });
            }
        }

        ops.push(DSLOp {
            name: "simulate_physics".into(),
            domain: Domain::Scene3D,
            parameters: vec![serde_json::json!(10)],
        });

        ops
    }

    fn apply(&self, action: &DSLOp) -> Self {
        let mut scene = self.clone();
        scene.tick += 1;

        // Phase 2: integrate with Bevy/Rapier physics
        // For now, simple positional changes
        if action.name.starts_with("move_") && action.name.ends_with("_up") {
            let obj_id = action.name.strip_prefix("move_").and_then(|s| s.strip_suffix("_up"));
            if let Some(id) = obj_id {
                if let Some(obj) = scene.objects.iter_mut().find(|o| o.id == id) {
                    obj.position[1] += 1.0;
                }
            }
        } else if action.name.starts_with("move_") && action.name.ends_with("_down") {
            let obj_id = action.name.strip_prefix("move_").and_then(|s| s.strip_suffix("_down"));
            if let Some(id) = obj_id {
                if let Some(obj) = scene.objects.iter_mut().find(|o| o.id == id) {
                    obj.position[1] = (obj.position[1] - 1.0).max(0.0);
                }
            }
        } else if action.name == "simulate_physics" {
            // Gravity: unsupported objects fall
            for obj in &mut scene.objects {
                if obj.position[1] > 0.0 {
                    // Simple gravity — Phase 2 uses Rapier
                    obj.position[1] = (obj.position[1] - 0.5).max(0.0);
                }
            }
        }

        scene
    }

    fn score_against(&self, goal: &Self) -> Score {
        let accuracy = self.constraint_satisfaction(goal);
        Score {
            exact_match: accuracy > 0.99,
            accuracy,
            details: serde_json::json!({
                "object_count_match": self.objects.len() == goal.objects.len(),
                "tick": self.tick,
            }),
        }
    }

    fn diff(&self, other: &Self) -> Vec<Delta> {
        let mut deltas = Vec::new();

        for obj in &self.objects {
            if let Some(other_obj) = other.objects.iter().find(|o| o.id == obj.id) {
                let dist = euclidean_distance(&obj.position, &other_obj.position);
                if dist > 0.01 {
                    deltas.push(Delta {
                        kind: "position_change".into(),
                        description: format!(
                            "{}: moved {:.2} units",
                            obj.id, dist
                        ),
                        magnitude: dist,
                    });
                }
            }
        }

        deltas
    }

    fn summary(&self) -> String {
        format!(
            "Scene3D({} objects, {} tools, tick={})",
            self.objects.len(), self.tools.len(), self.tick
        )
    }
}

fn euclidean_distance(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn euclidean_distance_xz(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_basics() {
        let mut scene = Scene3D::new();
        scene.add_object(SceneObject {
            id: "block_a".into(),
            class_name: "cube".into(),
            position: [0.0, 5.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
            properties: serde_json::Value::Null,
        });

        let props = scene.analyze();
        assert!(props.iter().any(|p| p.name == "object_count"));
    }

    #[test]
    fn test_gravity_simulation() {
        let mut scene = Scene3D::new();
        scene.add_object(SceneObject {
            id: "block".into(),
            class_name: "cube".into(),
            position: [0.0, 10.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
            properties: serde_json::Value::Null,
        });

        let action = DSLOp {
            name: "simulate_physics".into(),
            domain: Domain::Scene3D,
            parameters: vec![],
        };

        let after = scene.apply(&action);
        assert!(after.objects[0].position[1] < 10.0, "Object should fall");
    }

    #[test]
    fn test_worldstate_trait() {
        let scene = Scene3D::new();
        let _actions = scene.available_actions();
        let _props = scene.analyze();
        let _summary = scene.summary();
    }
}
