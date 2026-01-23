use actix_web::dev::{Service, Transform, ServiceRequest, ServiceResponse};
use actix_web::{Error, HttpResponse};
use actix_web::body::EitherBody;
use futures::future::{LocalBoxFuture, Ready};
use futures::FutureExt;
use std::{collections::HashSet, task::{Context, Poll}};

pub struct ApiKeyAuth {
    required: bool,
    keys: HashSet<String>,
}

impl ApiKeyAuth {
    pub fn from_env() -> Self {
        let required = std::env::var("AUTH_REQUIRED").unwrap_or_default() == "true";
        let keys_str = std::env::var("API_KEYS").unwrap_or_default();
        let keys: HashSet<String> = keys_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        Self { required, keys }
    }

    fn is_dev() -> bool { std::env::var("DEVELOPMENT_MODE").unwrap_or_default() == "true" }

    #[allow(dead_code)]
    fn authorized(&self, req: &ServiceRequest) -> bool {
        if !self.required || Self::is_dev() { return true; }
        if self.keys.is_empty() { return true; }
        let header = req.headers().get("x-api-key").and_then(|v| v.to_str().ok());
        if let Some(h) = header { return self.keys.contains(h); }
        // Also allow Bearer token simple match
        if let Some(auth) = req.headers().get("authorization").and_then(|v| v.to_str().ok()) {
            if let Some(token) = auth.strip_prefix("Bearer ") { return self.keys.contains(token.trim()); }
        }
        false
    }
}

impl<S, B> Transform<S, ServiceRequest> for ApiKeyAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = ApiKeyAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let required = self.required;
        let keys = self.keys.clone();
        futures::future::ready(Ok(ApiKeyAuthMiddleware { service, required, keys }))
    }
}

pub struct ApiKeyAuthMiddleware<S> {
    service: S,
    required: bool,
    keys: HashSet<String>,
}

impl<S, B> Service<ServiceRequest> for ApiKeyAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let required = self.required;
        let keys = self.keys.clone();
        let authorized = if !required || ApiKeyAuth::is_dev() || keys.is_empty() {
            true
        } else {
            let header = req.headers().get("x-api-key").and_then(|v| v.to_str().ok());
            if let Some(h) = header { keys.contains(h) } else {
                if let Some(auth) = req.headers().get("authorization").and_then(|v| v.to_str().ok()) {
                    if let Some(token) = auth.strip_prefix("Bearer ") { keys.contains(token.trim()) } else { false }
                } else { false }
            }
        };

        if !authorized {
            return async move {
                let res = req.into_response(HttpResponse::Unauthorized().finish().map_into_right_body());
                Ok(res)
            }
            .boxed_local();
        }

        let fut = self.service.call(req);
        async move {
            let res = fut.await?;
            Ok(res.map_into_left_body())
        }
        .boxed_local()
    }
}
