use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use http_body_util::BodyExt;
use ratelimiter::api::{AppState, router};
use serde_json::{Value, json};
use tower::ServiceExt;

async fn post_json(app: Router, uri: &str, body: Value) -> (StatusCode, Value) {
    let req = Request::post(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

#[tokio::test]
async fn configure_echoes_config() {
    let app = router(AppState::default());
    let body = json!({ "window_seconds": 10, "request_per_window": 5 });
    let (status, resp) = post_json(app, "/api/v1/configure", body.clone()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp, body);
}

#[tokio::test]
async fn ratelimit_returns_decision() {
    let app = router(AppState::default());
    let body = json!({ "client_id": "abc", "resource": "/foo" });
    let (status, resp) = post_json(app, "/api/v1/ratelimit", body).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["allowed"], json!(true));
    assert!(resp.get("retry_after").is_none());
}

#[tokio::test]
async fn ratelimit_rejects_malformed_body() {
    let app = router(AppState::default());
    let (status, _) = post_json(app, "/api/v1/ratelimit", json!({ "nope": 1 })).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn configure_persists_config() {
    let app = router(AppState::default());
    let body = json!({ "window_seconds": 10, "request_per_window": 5 });
    post_json(app.clone(), "/api/v1/configure", body.clone()).await;

    let req = Request::get("/api/v1/config").body(Body::empty()).unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(serde_json::from_slice::<Value>(&bytes).unwrap(), body);
}
