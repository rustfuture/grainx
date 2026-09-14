use crate::error::{GrainxError, Result};
use crate::export::StatsSnapshot;
use crate::monitor::SystemMonitor;
use axum::{Json, Router, extract::State, routing::get};
use parking_lot::Mutex;
use std::net::{IpAddr, SocketAddr};
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

/// Whether `ip` addresses only the local host.
///
/// An IPv4-mapped IPv6 loopback address (`::ffff:127.0.0.1`) is loopback too, so it is
/// unwrapped before the check rather than rejected by `Ipv6Addr::is_loopback`.
fn is_loopback(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => v4.is_loopback(),
        IpAddr::V6(v6) => {
            v6.is_loopback() || v6.to_ipv4_mapped().is_some_and(|v4| v4.is_loopback())
        }
    }
}

/// Reject any address that is reachable from outside the local host.
///
/// The agent serves host metrics without authentication, TLS, or rate limiting. Binding it
/// to a routable interface would expose those metrics to the network, so the localhost-only
/// boundary promised by the README and `--bind --help` is enforced here at runtime instead of
/// relying on the operator to follow the documentation.
fn ensure_loopback(addr: SocketAddr) -> Result<()> {
    if is_loopback(addr.ip()) {
        return Ok(());
    }
    Err(GrainxError::NonLoopbackBind(addr.to_string()))
}

/// Build the socket address for `bind`, which must be an IP literal.
///
/// Joining `"host:port"` as a string cannot represent IPv6: `::1` would become `::1:9090`,
/// which does not parse. Taking an `IpAddr` first keeps both families working.
fn parse_bind(bind: &str, port: u16) -> Result<SocketAddr> {
    let ip: IpAddr = bind
        .parse()
        .map_err(|_| GrainxError::InvalidBind(bind.to_string()))?;
    Ok(SocketAddr::new(ip, port))
}

pub async fn run(bind: &str, port: u16) -> Result<()> {
    let addr = parse_bind(bind, port)?;

    ensure_loopback(addr)?;

    let app = router();
    let listener = tokio::net::TcpListener::bind(addr).await?;
    eprintln!("grainx agent listening on http://{addr}");
    eprintln!("  GET /health  — liveness");
    eprintln!("  GET /metrics — JSON snapshot");

    tokio::select! {
        result = axum::serve(listener, app) => {
            result.map_err(GrainxError::from)?;
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

    #[test]
    fn loopback_addresses_are_accepted() {
        for addr in ["127.0.0.1:9090", "[::1]:9090", "[::ffff:127.0.0.1]:9090"] {
            let parsed: SocketAddr = addr.parse().unwrap();
            assert!(
                ensure_loopback(parsed).is_ok(),
                "{addr} should be accepted as loopback"
            );
        }
    }

    #[test]
    fn ipv6_bind_addresses_are_parsed_and_formatted_correctly() {
        // The bind argument is an IP literal, so the port must be attached with
        // SocketAddr::new rather than by formatting "host:port".
        let addr = parse_bind("::1", 9090).expect("::1 should parse");
        assert_eq!(addr.to_string(), "[::1]:9090");
        assert!(ensure_loopback(addr).is_ok());

        assert!(matches!(
            parse_bind("localhost", 9090),
            Err(GrainxError::InvalidBind(_))
        ));
    }

    #[test]
    fn routable_addresses_are_rejected() {
        // 0.0.0.0 and :: are the wildcard binds; the rest are ordinary interface addresses.
        for addr in [
            "0.0.0.0:9090",
            "[::]:9090",
            "192.168.1.10:9090",
            "10.0.0.5:9090",
            "[2001:db8::1]:9090",
        ] {
            let parsed: SocketAddr = addr.parse().unwrap();
            assert!(
                matches!(
                    ensure_loopback(parsed),
                    Err(GrainxError::NonLoopbackBind(_))
                ),
                "{addr} must be refused"
            );
        }
    }

    #[tokio::test]
    async fn run_refuses_a_routable_bind_address() {
        // run() checks the address before binding, so nothing is ever listened on here.
        let result = run("0.0.0.0", 9090).await;
        match result {
            Err(GrainxError::NonLoopbackBind(message)) => {
                assert!(
                    message.contains("0.0.0.0:9090"),
                    "unexpected message: {message}"
                );
            }
            other => panic!("expected NonLoopbackBind, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn run_reports_an_unparseable_bind_address() {
        match run("not-an-address", 9090).await {
            Err(GrainxError::InvalidBind(_)) => {}
            other => panic!("expected InvalidBind, got {other:?}"),
        }
    }
}
