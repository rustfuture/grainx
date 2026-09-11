//! End-to-end tests for `grainx export --remote <url>` at the real CLI boundary.
//!
//! The binary is executed with `CARGO_BIN_EXE_grainx` against a minimal HTTP
//! server built on `std::net::TcpListener`. No network access is required.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use grainx::export::StatsSnapshot;

const BIN: &str = env!("CARGO_BIN_EXE_grainx");

const VALID_METRICS: &str = r#"{
  "timestamp": "2026-09-11T00:00:00Z",
  "cpu_usage_percent": 12.5,
  "memory_used_bytes": 536870912,
  "memory_total_bytes": 1073741824,
  "memory_usage_percent": 50.0,
  "network_rx_bytes": 4096,
  "network_tx_bytes": 2048,
  "cpu_cores": [10.0, 20.0],
  "disks": [{"name": "test-disk", "total_bytes": 1024, "available_bytes": 512, "used_percent": 50.0}],
  "os_name": "TestOS",
  "kernel_version": "0.0.1-test",
  "uptime_seconds": 42,
  "processes": [{"pid": 7, "name": "grainx-test", "cpu_usage": 1.5, "memory_bytes": 1048576}]
}"#;

const MALFORMED_METRICS: &str = r#"{"cpu_usage_percent": "not a snapshot"}"#;

fn unique_temp_dir(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "grainx-cli-remote-export-{tag}-{}-{nanos}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

/// Serves exactly one `GET` with `body` as the response payload.
fn spawn_metrics_server(body: &'static str) -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
    let address = listener.local_addr().expect("read test server address");
    let _handle = thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut request = [0u8; 1024];
            let _ = stream.read(&mut request);
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        }
    });
    address
}

fn unused_loopback_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("bind probe port")
        .local_addr()
        .expect("read probe port")
        .port()
}

fn run_export(remote: &str, json: &Path, csv: &Path) -> Output {
    Command::new(BIN)
        .arg("export")
        .arg("--remote")
        .arg(remote)
        .arg("--json")
        .arg(json)
        .arg("--csv")
        .arg(csv)
        .output()
        .expect("run grainx export")
}

fn assert_controlled_remote_failure(output: &Output, expected_error: &str) {
    assert_eq!(
        output.status.code(),
        Some(4),
        "expected controlled remote-metrics exit code 4, got {:?}\nstdout: {}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains(expected_error),
        "expected controlled error containing {expected_error:?}, got: {combined}"
    );
    assert!(
        !combined.contains("panicked"),
        "panic text in CLI output: {combined}"
    );
    assert!(
        !combined.contains("Cannot drop a runtime"),
        "tokio runtime-drop panic text in CLI output: {combined}"
    );
}

#[test]
fn remote_export_succeeds_against_a_live_http_server() {
    let dir = unique_temp_dir("success");
    let json_path = dir.join("r.json");
    let csv_path = dir.join("r.csv");
    let address = spawn_metrics_server(VALID_METRICS);

    let output = run_export(&format!("http://{address}"), &json_path, &csv_path);
    assert!(
        output.status.success(),
        "expected exit 0, got {:?}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    let json_text = std::fs::read_to_string(&json_path).expect("json output written");
    let snapshot: StatsSnapshot = serde_json::from_str(&json_text).expect("json output parses");
    assert_eq!(snapshot.os_name, "TestOS");
    assert_eq!(snapshot.network_rx_bytes, 4096);
    assert_eq!(snapshot.network_tx_bytes, 2048);
    assert_eq!(snapshot.processes[0].name, "grainx-test");

    let raw: serde_json::Value = serde_json::from_str(&json_text).expect("json output parses");
    assert!(
        raw.get("network_rx_rate").is_none() && raw.get("network_tx_rate").is_none(),
        "network fields must stay byte counts, not rates: {json_text}"
    );

    let csv_text = std::fs::read_to_string(&csv_path).expect("csv output written");
    assert!(csv_text.starts_with("section,field,value"));
    assert!(csv_text.contains("system,network_rx_bytes,4096"));
    assert!(csv_text.contains("system,network_tx_bytes,2048"));
    assert!(csv_text.contains("pid,name,cpu_usage_percent,memory_bytes"));
    assert!(csv_text.contains("7,grainx-test,1.50,1048576"));

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn remote_export_reports_controlled_error_when_endpoint_unreachable() {
    let dir = unique_temp_dir("unreachable");
    let json_path = dir.join("r.json");
    let csv_path = dir.join("r.csv");
    let port = unused_loopback_port();

    let output = run_export(&format!("http://127.0.0.1:{port}"), &json_path, &csv_path);
    assert_controlled_remote_failure(&output, "failed to fetch remote metrics");
    assert!(
        !json_path.exists(),
        "no json output on unreachable endpoint"
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn remote_export_reports_controlled_error_on_malformed_response() {
    let dir = unique_temp_dir("malformed");
    let json_path = dir.join("r.json");
    let csv_path = dir.join("r.csv");
    let address = spawn_metrics_server(MALFORMED_METRICS);

    let output = run_export(&format!("http://{address}"), &json_path, &csv_path);
    assert_controlled_remote_failure(&output, "invalid metrics JSON");
    assert!(!json_path.exists(), "no json output on malformed response");
    assert!(!csv_path.exists(), "no csv output on malformed response");

    std::fs::remove_dir_all(&dir).ok();
}

/// Port 0 is the exact historical repro of the tokio runtime-drop panic.
#[test]
fn remote_export_to_port_zero_reports_controlled_error() {
    let dir = unique_temp_dir("port-zero");
    let json_path = dir.join("r.json");
    let csv_path = dir.join("r.csv");

    let output = run_export("http://127.0.0.1:0", &json_path, &csv_path);
    assert_controlled_remote_failure(&output, "failed to fetch remote metrics");
    assert!(!json_path.exists(), "no json output on port zero");

    std::fs::remove_dir_all(&dir).ok();
}
