//! `spatial`: route planning, clearance, assembly and kinematics (008).

use serde::Deserialize;
use serde_json::{Value, json};
use vortex_kernel::{KinematicsQuery, SpatialChallenge};

use super::{Module, challenge_schema, method, object, parse, query_schema, to_value};
use crate::json::{ApiError, Method};
use crate::vortex::Vortex;

pub struct Spatial;

impl Module for Spatial {
    fn id(&self) -> &str {
        "spatial"
    }

    fn methods(&self) -> Vec<Method> {
        vec![
            method("spatial.solve", "Solve a spatial challenge: collision-free route for a robot among rectangular obstacles, passable portals, assembly placement.", object(json!({ "challenge": challenge_schema() }), &["challenge"])),
            method("spatial.kinematics", "Closed-form answer to a 3D-plus-time kinematics query. `values` is null, with a `reason`, when there is no solution.", object(json!({ "query": query_schema() }), &["query"])),
        ]
    }

    fn call(&self, _vortex: &Vortex, m: &str, params: Value) -> Result<Value, ApiError> {
        match m {
            "spatial.solve" => {
                #[derive(Deserialize)]
                struct P {
                    challenge: SpatialChallenge,
                }
                let p: P = parse(m, params)?;
                let (submission, confidence) = crate::spatial::solve_challenge(&p.challenge);
                let route_length = crate::spatial::plan::length(&submission.route);
                Ok(json!({ "submission": to_value(m, &submission)?, "confidence": confidence, "route_length": route_length }))
            }
            "spatial.kinematics" => {
                #[derive(Deserialize)]
                struct P {
                    query: KinematicsQuery,
                }
                let p: P = parse(m, params)?;
                Ok(match crate::spatial::kinematics::answer(&p.query) {
                    Some(values) => json!({ "values": values }),
                    None => json!({ "values": null, "reason": "no solution in range" }),
                })
            }
            other => Err(ApiError::UnknownMethod(other.to_string())),
        }
    }
}
