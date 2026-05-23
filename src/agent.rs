use crate::export::StatsSnapshot;
use crate::monitor::SystemMonitor;
use axum::{Json, Router, extract::State, routing::get};
use parking_lot::Mutex;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Clone)]
struct AgentState {
    monitor: Arc<Mutex<SystemMonitor>>,
}

async fn health() -> &'static str {
    "ok"
}

async fn metrics(State(state): State<AgentState>) -> Json<StatsSnapshot> {
    let monitor = Arc::clone(&state.monitor);
    let snapshot = tokio::task::spawn_blocking(move || {
        let mut monitor = monitor.lock();
        StatsSnapshot::capture(&mut monitor)
    })
    .await
    .expect("metrics task panicked");
    Json(snapshot)
}

pub fn router() -> Router {
    let state = AgentState {
        monitor: Arc::new(Mutex::new(SystemMonitor::new())),
    };
    Router::new()
        .route("/health", get(health))
        .route("/metrics", get(metrics))
        .with_state(state)
}

pub async fn run(bind: &str, port: u16) -> io::Result<()> {
    let addr: SocketAddr = format!("{bind}:{port}")
        .parse()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    let app = router();
    let listener = tokio::net::TcpListener::bind(addr).await?;
    eprintln!("grainx agent listening on http://{addr}");
    eprintln!("  GET /health  — liveness");
    eprintln!("  GET /metrics — JSON snapshot");

    tokio::select! {
        result = axum::serve(listener, app) => {
            result.map_err(io::Error::other)?;
        }
        _ = tokio::signal::ctrl_c() => {
            eprintln!("grainx agent shutting down");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_endpoint_returns_ok() {
        let app = router();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn metrics_endpoint_returns_json() {
        let app = router();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/metrics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response.headers().get("content-type").unwrap();
        assert!(content_type.to_str().unwrap().contains("application/json"));
    }
}
