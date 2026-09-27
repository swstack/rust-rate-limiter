use std::sync::{Arc, RwLock};

use axum::{Json, Router, extract::State, routing::post, routing::get};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub window_seconds: u64,
    pub request_per_window: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            window_seconds: 60,
            request_per_window: 100,
        }
    }
}

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
    config: RateLimitConfig,
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
    Json(_req): Json<RateLimitRequest>,
) -> Json<RateLimitResponse> {
    // TODO: plug in the actual rate limiter keyed on client_id/resource.
    // let config = *state.config.read().unwrap();
    eprintln!("Check rate limit {:?}", _req);
    Json(RateLimitResponse {
        allowed: true,
        remaining: state.config.request_per_window,
        retry_after: None,
    })
}

async fn configure(
    State(mut state): State<AppState>,
    Json(new_config): Json<RateLimitConfig>,
) -> Json<RateLimitConfig> {
    eprintln!("Configuring {:?}", new_config);
    state.config = new_config;
    Json(new_config)
}

async fn get_config(
    State(state): State<AppState>
) -> Json<RateLimitConfig> {
    eprintln!("Getting config {:?}", state.config);
    Json(state.config)
}
