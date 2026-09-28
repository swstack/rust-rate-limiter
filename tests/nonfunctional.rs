use std::future::IntoFuture;
use std::time::{Duration, Instant};

use ratelimiter::api::{AppState, router};
use reqwest::StatusCode;
use serde_json::{Value, json};
use tokio::task::JoinSet;

/// Serves the real app over TCP on a random free port and returns the ratelimit URL.
async fn spawn_server() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/api/v1/ratelimit", listener.local_addr().unwrap());
    tokio::spawn(axum::serve(listener, router(AppState::default())).into_future());
    url
}

/// Sends one ratelimit request and asserts it was answered and allowed.
async fn check_allowed(client: &reqwest::Client, url: &str, body: &Value) {
    let res = client.post(url).json(body).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let decision: Value = res.json().await.unwrap();
    assert_eq!(decision["allowed"], json!(true));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn one_thousand_rps_with_ten_ms_latency() {
    const TOTAL: usize = 1000;
    const CONCURRENCY: usize = 50;

    let url = spawn_server().await;
    let client = reqwest::Client::new();

    // Notes: Currently, 1000 requests for unique clients and the same client have the
    // same latency, but once we move to a sharded lock, this will no longer be the case.
    // Every request uses a unique client_id so none are rejected by the limiter.
    let mut workers = JoinSet::new();
    let start = Instant::now();
    for w in 0..CONCURRENCY {
        let client = client.clone();
        let url = url.clone();
        workers.spawn(async move {
            // Untimed warm-up so connection setup isn't counted in the latencies.
            let warmup = json!({ "client_id": format!("warmup-{w}"), "resource": "/foo" });
            check_allowed(&client, &url, &warmup).await;

            let mut latencies = Vec::with_capacity(TOTAL / CONCURRENCY);
            for i in 0..TOTAL / CONCURRENCY {
                let body = json!({ "client_id": format!("client-{w}-{i}"), "resource": "/foo" });
                let sent = Instant::now();
                check_allowed(&client, &url, &body).await;
                latencies.push(sent.elapsed());
            }
            latencies
        });
    }
    let mut latencies: Vec<Duration> = workers.join_all().await.into_iter().flatten().collect();
    // Includes the warm-ups, so the throughput figure is slightly pessimistic.
    let elapsed = start.elapsed();

    latencies.sort();
    let p50 = latencies[latencies.len() / 2];
    let p99 = latencies[latencies.len() * 99 / 100];
    let rps = TOTAL as f64 / elapsed.as_secs_f64();
    eprintln!("{TOTAL} requests in {elapsed:?} ({rps:.0} rps), p50 {p50:?}, p99 {p99:?}");

    assert_eq!(latencies.len(), TOTAL);
    assert!(rps >= 1000.0, "only {rps:.0} rps");
    assert!(p99 < Duration::from_millis(10), "p99 was {p99:?}");
}

#[tokio::test]
async fn single_request_sub_10_ms() {
    let url = spawn_server().await;
    let client = reqwest::Client::new();
    let body = json!({ "client_id": "latency-client", "resource": "/foo" });

    // Warm-up request: absorbs one-time costs (connection setup, server task start, first-touch
    // allocations) so the timed request measures steady-state latency.
    check_allowed(&client, &url, &body).await;

    let start = Instant::now();
    check_allowed(&client, &url, &body).await;
    let elapsed = start.elapsed();

    assert!(elapsed < Duration::from_millis(10), "took {elapsed:?}");
}
