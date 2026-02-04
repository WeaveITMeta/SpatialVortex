//! Dual Response API Module
//! 
//! Stub module for dual response API functionality.

use actix_web::{web, HttpResponse};
use crate::ai::api::AppState;

/// Dual response endpoint stub
pub async fn dual_response(_data: web::Data<AppState>) -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::Ok().json("dual response"))
}
