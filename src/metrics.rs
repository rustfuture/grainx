use crate::error::{GrainxError, Result};
use crate::export::StatsSnapshot;
use crate::monitor::{SystemMetrics, SystemMonitor};
use std::time::Duration;

const REMOTE_TIMEOUT: Duration = Duration::from_secs(5);

pub enum MetricBackend {
    Local(Box<SystemMonitor>),
    Remote(Box<RemoteMetricsClient>),
}

impl MetricBackend {
    pub fn local() -> Self {
        MetricBackend::Local(Box::default())
    }

    pub fn remote(url: impl Into<String>) -> Self {
        MetricBackend::Remote(Box::new(RemoteMetricsClient::new(url)))
    }

    pub fn refresh(&mut self) -> Result<SystemMetrics> {
        match self {
            MetricBackend::Local(monitor) => Ok(monitor.refresh().clone()),
            MetricBackend::Remote(client) => client.fetch(),
        }
    }

    pub fn take_alerts(&mut self) -> Vec<String> {
        match self {
            MetricBackend::Local(monitor) => monitor.take_alerts(),
            MetricBackend::Remote(_) => Vec::new(),
        }
    }

    pub fn get_processes(&mut self) -> Result<Vec<(usize, String, f32, u64)>> {
        match self {
            MetricBackend::Local(monitor) => Ok(monitor.get_processes()),
            MetricBackend::Remote(client) => Ok(client.processes()),
        }
    }

    pub fn kill_process(&mut self, pid: usize) -> bool {
        match self {
            MetricBackend::Local(monitor) => monitor.kill_process(pid),
            MetricBackend::Remote(_) => false,
        }
    }

    pub fn last_cpu_usage(&self) -> f32 {
        match self {
            MetricBackend::Local(monitor) => monitor.last_cpu_usage,
            MetricBackend::Remote(client) => client.last_cpu_usage(),
        }
    }
}

pub struct RemoteMetricsClient {
    metrics_url: String,
    last_metrics: Option<SystemMetrics>,
    last_error: Option<String>,
}

impl RemoteMetricsClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        let base = base_url.into().trim_end_matches('/').to_string();

        RemoteMetricsClient {
            metrics_url: format!("{base}/metrics"),
            last_metrics: None,
            last_error: None,
        }
    }

    pub fn fetch(&mut self) -> Result<SystemMetrics> {
        let url = self.metrics_url.clone();
        let result = std::thread::spawn(move || fetch_snapshot(&url))
            .join()
            .unwrap_or_else(|_| Err("remote metrics request thread panicked".to_string()));

        match result {
            Ok(snapshot) => {
                let metrics = system_metrics_from_snapshot(snapshot);
                self.last_metrics = Some(metrics.clone());
                self.last_error = None;
                Ok(metrics)
            }
            Err(message) => {
                self.last_error = Some(message.clone());
                if let Some(metrics) = self.last_metrics.clone() {
                    Ok(metrics)
                } else {
                    Err(GrainxError::RemoteMetrics(message))
                }
            }
        }
    }

    pub fn processes(&self) -> Vec<(usize, String, f32, u64)> {
        self.last_metrics
            .as_ref()
            .map(|m| m.processes.clone())
            .unwrap_or_default()
    }

    pub fn last_cpu_usage(&self) -> f32 {
        self.last_metrics
            .as_ref()
            .map(|m| m.cpu_usage)
            .unwrap_or(0.0)
    }
}

/// Performs one blocking HTTP request on a dedicated OS thread.
///
/// `reqwest::blocking` creates and drops an internal Tokio runtime, so it must
/// not run inside the Tokio runtime that drives the commands. Running the
/// request on a plain thread keeps client construction and use outside any
/// async context.
fn fetch_snapshot(url: &str) -> std::result::Result<StatsSnapshot, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(REMOTE_TIMEOUT)
        .build()
        .map_err(|error| format!("could not create HTTP client: {error}"))?;
    let response = client
        .get(url)
        .send()
        .map_err(|error| format!("failed to fetch remote metrics from {url}: {error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "remote metrics request failed with status {}",
            response.status()
        ));
    }
    response
        .json::<StatsSnapshot>()
        .map_err(|error| format!("invalid metrics JSON: {error}"))
}

fn system_metrics_from_snapshot(snapshot: StatsSnapshot) -> SystemMetrics {
    SystemMetrics {
        cpu_usage: snapshot.cpu_usage_percent,
        memory_used: snapshot.memory_used_bytes,
        memory_total: snapshot.memory_total_bytes,
        network_rx: snapshot.network_rx_bytes,
        network_tx: snapshot.network_tx_bytes,
        cpu_cores: snapshot.cpu_cores,
        disks: snapshot
            .disks
            .into_iter()
            .map(|d| (d.name, d.total_bytes, d.available_bytes, d.used_percent))
            .collect(),
        os_name: snapshot.os_name,
        kernel_version: snapshot.kernel_version,
        uptime_seconds: snapshot.uptime_seconds,
        processes: snapshot
            .processes
            .into_iter()
            .map(|p| (p.pid, p.name, p.cpu_usage, p.memory_bytes))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn converts_snapshot_to_system_metrics() {
        let snapshot = StatsSnapshot {
            timestamp: Utc::now(),
            cpu_usage_percent: 42.0,
            memory_used_bytes: 100,
            memory_total_bytes: 1000,
            memory_usage_percent: 10.0,
            network_rx_bytes: 50,
            network_tx_bytes: 25,
            cpu_cores: vec![10.0, 20.0],
            disks: vec![],
            os_name: "Linux".to_string(),
            kernel_version: "6.1".to_string(),
            uptime_seconds: 3600,
            processes: vec![],
        };

        let metrics = system_metrics_from_snapshot(snapshot);
        assert_eq!(metrics.cpu_usage, 42.0);
        assert_eq!(metrics.memory_total, 1000);
        assert_eq!(metrics.cpu_cores.len(), 2);
    }

    #[test]
    fn local_backend_refresh_returns_metrics() {
        let mut backend = MetricBackend::local();
        let metrics = backend.refresh().unwrap();
        assert!(metrics.memory_total > 0);
    }

    #[test]
    fn remote_backend_errors_when_agent_unreachable() {
        let mut backend = MetricBackend::remote("http://127.0.0.1:59999");
        let err = backend.refresh().unwrap_err();
        assert!(matches!(err, GrainxError::RemoteMetrics(_)));
    }

    #[tokio::test]
    async fn remote_backend_is_usable_inside_async_context() {
        // Regression: building reqwest::blocking inside the Tokio runtime panicked
        // with "Cannot drop a runtime in a context where blocking is not allowed".
        let mut backend = MetricBackend::remote("http://127.0.0.1:59999");
        let err = backend.refresh().unwrap_err();
        assert!(matches!(err, GrainxError::RemoteMetrics(_)));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn remote_backend_fetches_from_a_live_agent_inside_async_context() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, crate::agent::router()).await.unwrap();
        });

        let mut backend = MetricBackend::remote(format!("http://{address}"));
        let metrics = backend.refresh().unwrap();
        assert!(metrics.memory_total > 0);
        server.abort();
    }

    #[tokio::test]
    async fn agent_metrics_json_is_remote_compatible() {
        use axum::body::Body;
        use axum::http::{Request, StatusCode};
        use tower::ServiceExt;

        let app = crate::agent::router();
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

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let snapshot: StatsSnapshot = serde_json::from_slice(&body).unwrap();
        let metrics = system_metrics_from_snapshot(snapshot);
        assert!(metrics.memory_total > 0);
    }
}
