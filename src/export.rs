use crate::error::Result;
use crate::monitor::SystemMonitor;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ProcessInfo {
    pub pid: usize,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_bytes: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DiskInfo {
    pub name: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_percent: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct StatsSnapshot {
    pub timestamp: DateTime<Utc>,
    pub cpu_usage_percent: f32,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
    pub memory_usage_percent: f64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub cpu_cores: Vec<f32>,
    pub disks: Vec<DiskInfo>,
    pub os_name: String,
    pub kernel_version: String,
    pub uptime_seconds: u64,
    pub processes: Vec<ProcessInfo>,
}

impl StatsSnapshot {
    pub fn capture(monitor: &mut SystemMonitor) -> Self {
        let metrics = monitor.refresh();
        Self::from_system_metrics(metrics)
    }

    pub fn capture_backend(backend: &mut crate::metrics::MetricBackend) -> Result<Self> {
        let metrics = backend.refresh()?;
        Ok(Self::from_system_metrics(&metrics))
    }

    fn from_system_metrics(metrics: &crate::monitor::SystemMetrics) -> Self {
        let memory_percentage = if metrics.memory_total > 0 {
            (metrics.memory_used as f64 / metrics.memory_total as f64) * 100.0
        } else {
            0.0
        };

        StatsSnapshot {
            timestamp: Utc::now(),
            cpu_usage_percent: metrics.cpu_usage,
            memory_used_bytes: metrics.memory_used,
            memory_total_bytes: metrics.memory_total,
            memory_usage_percent: memory_percentage,
            network_rx_bytes: metrics.network_rx,
            network_tx_bytes: metrics.network_tx,
            cpu_cores: metrics.cpu_cores.clone(),
            disks: metrics
                .disks
                .iter()
                .map(|(name, total, available, used_percent)| DiskInfo {
                    name: name.clone(),
                    total_bytes: *total,
                    available_bytes: *available,
                    used_percent: *used_percent,
                })
                .collect(),
            os_name: metrics.os_name.clone(),
            kernel_version: metrics.kernel_version.clone(),
            uptime_seconds: metrics.uptime_seconds,
            processes: metrics
                .processes
                .iter()
                .map(|(pid, name, cpu_usage, memory_bytes)| ProcessInfo {
                    pid: *pid,
                    name: name.clone(),
                    cpu_usage: *cpu_usage,
                    memory_bytes: *memory_bytes,
                })
                .collect(),
        }
    }

    pub fn save_json(&self, path: &str) -> io::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)
    }

    pub fn save_csv(&self, path: &str) -> io::Result<()> {
        let mut lines = Vec::new();
        lines.push("section,field,value".to_string());
        lines.push(format!("system,timestamp,{}", self.timestamp.to_rfc3339()));
        lines.push(format!(
            "system,cpu_usage_percent,{:.2}",
            self.cpu_usage_percent
        ));
        lines.push(format!(
            "system,memory_used_bytes,{}",
            self.memory_used_bytes
        ));
        lines.push(format!(
            "system,memory_total_bytes,{}",
            self.memory_total_bytes
        ));
        lines.push(format!(
            "system,memory_usage_percent,{:.2}",
            self.memory_usage_percent
        ));
        lines.push(format!("system,network_rx_bytes,{}", self.network_rx_bytes));
        lines.push(format!("system,network_tx_bytes,{}", self.network_tx_bytes));
        lines.push(format!("system,os_name,{}", csv_escape(&self.os_name)));
        lines.push(format!(
            "system,kernel_version,{}",
            csv_escape(&self.kernel_version)
        ));
        lines.push(format!("system,uptime_seconds,{}", self.uptime_seconds));
        lines.push(format!(
            "system,cpu_cores,{}",
            csv_escape(
                &self
                    .cpu_cores
                    .iter()
                    .map(|c| format!("{c:.1}"))
                    .collect::<Vec<_>>()
                    .join("|")
            )
        ));

        for disk in &self.disks {
            lines.push(format!("disk,name,{}", csv_escape(&disk.name)));
            lines.push(format!("disk,total_bytes,{}", disk.total_bytes));
            lines.push(format!("disk,available_bytes,{}", disk.available_bytes));
            lines.push(format!("disk,used_percent,{:.2}", disk.used_percent));
        }

        lines.push(String::new());
        lines.push("pid,name,cpu_usage_percent,memory_bytes".to_string());
        for process in &self.processes {
            lines.push(format!(
                "{},{},{:.2},{}",
                process.pid,
                csv_escape(&process.name),
                process.cpu_usage,
                process.memory_bytes
            ));
        }

        fs::write(path, lines.join("\n"))
    }

    pub fn save_both(
        json_path: &str,
        csv_path: &str,
        backend: &mut crate::metrics::MetricBackend,
    ) -> Result<()> {
        let snapshot = Self::capture_backend(backend)?;
        snapshot.save_json(json_path)?;
        snapshot.save_csv(csv_path)?;
        Ok(())
    }
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_snapshot() -> StatsSnapshot {
        StatsSnapshot {
            timestamp: Utc::now(),
            cpu_usage_percent: 42.5,
            memory_used_bytes: 4_000_000_000,
            memory_total_bytes: 8_000_000_000,
            memory_usage_percent: 50.0,
            network_rx_bytes: 1000,
            network_tx_bytes: 2000,
            cpu_cores: vec![10.0, 20.0],
            disks: vec![DiskInfo {
                name: "sda".to_string(),
                total_bytes: 500_000_000_000,
                available_bytes: 100_000_000_000,
                used_percent: 80.0,
            }],
            os_name: "Linux".to_string(),
            kernel_version: "6.1.0".to_string(),
            uptime_seconds: 3600,
            processes: vec![ProcessInfo {
                pid: 1234,
                name: "grainx".to_string(),
                cpu_usage: 5.5,
                memory_bytes: 8_000_000,
            }],
        }
    }

    #[test]
    fn test_save_json_roundtrip() {
        let snapshot = sample_snapshot();
        let path = "test_grainx_stats.json";

        snapshot.save_json(path).unwrap();
        let content = fs::read_to_string(path).unwrap();
        let loaded: StatsSnapshot = serde_json::from_str(&content).unwrap();

        assert_eq!(loaded.cpu_usage_percent, snapshot.cpu_usage_percent);
        assert_eq!(loaded.processes.len(), 1);
        assert_eq!(loaded.processes[0].name, "grainx");

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_save_csv_contains_sections() {
        let snapshot = sample_snapshot();
        let path = "test_grainx_stats.csv";

        snapshot.save_csv(path).unwrap();
        let content = fs::read_to_string(path).unwrap();

        assert!(content.contains("section,field,value"));
        assert!(content.contains("system,cpu_usage_percent,42.50"));
        assert!(content.contains("pid,name,cpu_usage_percent,memory_bytes"));
        assert!(content.contains("1234,grainx,5.50,8000000"));

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_csv_escape_commas() {
        assert_eq!(csv_escape("hello"), "hello");
        assert_eq!(csv_escape("a,b"), "\"a,b\"");
        assert_eq!(csv_escape("say \"hi\""), "\"say \"\"hi\"\"\"");
    }
}
