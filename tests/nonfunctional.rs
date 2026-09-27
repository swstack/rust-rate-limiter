use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use http_body_util::BodyExt;
use pura_coding_interview::api::{AppState, router};
use serde_json::{Value, json};
use tower::ServiceExt;

#[tokio::test]
async fn one_thousand_rps_with_ten_ms_latency() {
    let app = router(AppState::default());

    for i in 1..=1000 {
        let body = Body::from(json!({
            "client_id": format!("test-client-{}", i),
            "resource": "string"
        }).to_string());

        let req = Request::builder()
            .uri("/api/v1/ratelimit")
            .header(header::CONTENT_TYPE, "application/json")
            .body(body)
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

    }
}
