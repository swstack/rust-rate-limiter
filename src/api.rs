use std::sync::{Arc};

use axum::{Json, Router, extract::State, routing::post, routing::get};
use serde::{Deserialize, Serialize};

use crate::limiter::{RateLimitConfig, RateLimiter};


#[derive(Debug, Deserialize)]
pub struct RateLimitRequest {
    pub client_id: String,
    pub resource: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RateLimitResponse {
    pub allowed: bool,
    pub remaining: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after: Option<u64>,
}

#[derive(Clone, Default)]
pub struct AppState {
    limiter: Arc<RateLimiter>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/ratelimit", post(ratelimit))
        .route("/api/v1/configure", post(configure))
        .route("/api/v1/config", get(get_config))
        .with_state(state)
}

async fn ratelimit(
    State(state): State<AppState>,
    Json(req): Json<RateLimitRequest>,
) -> Json<RateLimitResponse> {
    eprintln!("Check rate limit {:?}", req);
    let (allowed, remaining, retry_after) = state.limiter.check(&req.client_id, &req.resource);
    Json(RateLimitResponse {
        allowed: allowed,
        remaining: remaining,
        retry_after: retry_after,
    })
}

async fn configure(
    State(state): State<AppState>,
    Json(new_config): Json<RateLimitConfig>,
) -> Json<RateLimitConfig> {
    eprintln!("Configuring {:?}", new_config);
    state.limiter.update_config(new_config);
    Json(new_config)
}

async fn get_config(
    State(state): State<AppState>
) -> Json<RateLimitConfig> {
    eprintln!("Getting config");
    let config = state.limiter.config();
    Json(config)
}
