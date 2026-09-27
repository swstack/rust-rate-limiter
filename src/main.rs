use pura_coding_interview::api::{AppState, router};

#[tokio::main]
async fn main() {
    let addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("listening on {addr}");
    axum::serve(listener, router(AppState::default()))
        .await
        .unwrap();
}
