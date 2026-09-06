use std::collections::VecDeque;

use sysinfo::{Disks, Networks, Pid, System};

const MAX_ALERTS: usize = 5;

#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub network_rx: u64,
    pub network_tx: u64,
    pub cpu_cores: Vec<f32>,
    pub disks: Vec<(String, u64, u64, f64)>,
    pub os_name: String,
    pub kernel_version: String,
    pub uptime_seconds: u64,
    pub processes: Vec<(usize, String, f32, u64)>,
}

pub struct SystemMonitor {
    sys: System,
    networks: Networks,
    disks: Disks,
    pub last_cpu_usage: f32,
    high_cpu_duration: u32,
    cpu_state_history: VecDeque<bool>,
    user_cpu_threshold: f32,
    high_cpu_count: u32,
    pending_alerts: Vec<String>,
    os_name: String,
    kernel_version: String,
    last_metrics: Option<SystemMetrics>,
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemMonitor {
    pub fn new() -> Self {
        SystemMonitor {
            sys: System::new_all(),
            networks: Networks::new_with_refreshed_list(),
            disks: Disks::new_with_refreshed_list(),
            last_cpu_usage: 0.0,
            high_cpu_duration: 0,
            cpu_state_history: VecDeque::with_capacity(5),
            user_cpu_threshold: 75.0,
            high_cpu_count: 0,
            pending_alerts: Vec::new(),
            os_name: System::name().unwrap_or_else(|| "Unknown".to_string()),
            kernel_version: System::kernel_version().unwrap_or_else(|| "Unknown".to_string()),
            last_metrics: None,
        }
    }

    pub fn refresh(&mut self) -> &SystemMetrics {
        self.sys.refresh_cpu();
        self.sys.refresh_memory();
        self.sys.refresh_processes();
        self.networks.refresh();
        self.disks.refresh();

        let cpu_usage = self.sys.global_cpu_info().cpu_usage();
        self.last_cpu_usage = cpu_usage;
        self.update_alerts(cpu_usage);

        let memory_used = self.sys.used_memory();
        let memory_total = self.sys.total_memory();

        let mut network_rx = 0;
        let mut network_tx = 0;
        for data in self.networks.values() {
            network_rx += data.received();
            network_tx += data.transmitted();
        }

        let cpu_cores = self.sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();

        let disks = self
            .disks
            .iter()
            .map(|disk| {
                let name = disk.name().to_string_lossy().to_string();
                let total = disk.total_space();
                let available = disk.available_space();
                let used_percentage = if total > 0 {
                    ((total - available) as f64 / total as f64) * 100.0
                } else {
                    0.0
                };
                (name, total, available, used_percentage)
            })
            .collect();

        let mut processes: Vec<(usize, String, f32, u64)> = self
            .sys
            .processes()
            .iter()
            .map(|(pid, p)| {
                (
                    pid.as_u32() as usize,
                    p.name().to_string(),
                    p.cpu_usage(),
                    p.memory(),
                )
            })
            .collect();
        processes.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        processes.truncate(10);

        let metrics = SystemMetrics {
            cpu_usage,
            memory_used,
            memory_total,
            network_rx,
            network_tx,
            cpu_cores,
            disks,
            os_name: self.os_name.clone(),
            kernel_version: self.kernel_version.clone(),
            uptime_seconds: System::uptime(),
            processes,
        };

        self.last_metrics = Some(metrics);
        self.last_metrics.as_ref().expect("metrics just set")
    }

    pub fn take_alerts(&mut self) -> Vec<String> {
        std::mem::take(&mut self.pending_alerts)
    }

    fn push_alert(&mut self, message: impl Into<String>) {
        let message = message.into();
        if self.pending_alerts.iter().any(|a| a == &message) {
            return;
        }
        if self.pending_alerts.len() >= MAX_ALERTS {
            self.pending_alerts.remove(0);
        }
        self.pending_alerts.push(message);
    }

    fn update_alerts(&mut self, current_cpu_usage: f32) {
        if current_cpu_usage > 90.0 {
            self.high_cpu_duration += 1;
            if self.high_cpu_duration == 5 {
                self.push_alert(
                    "WARNING: Sustained high CPU usage detected! Potential CPU bottleneck.",
                );
            }
        } else {
            self.high_cpu_duration = 0;
        }

        let is_high_cpu = current_cpu_usage > 70.0;
        self.cpu_state_history.push_back(is_high_cpu);
        if self.cpu_state_history.len() > 5 {
            self.cpu_state_history.pop_front();
        }

        if self.cpu_state_history.len() == 3
            && self.cpu_state_history[0]
            && !self.cpu_state_history[1]
            && self.cpu_state_history[2]
        {
            self.push_alert("PATTERN: CPU usage fluctuating (High-Low-High).");
            self.cpu_state_history.clear();
        }

        if current_cpu_usage > self.user_cpu_threshold {
            self.high_cpu_count += 1;
            if self.high_cpu_count == 10 {
                self.user_cpu_threshold += 5.0;
                self.push_alert(format!(
                    "LEARNED: Increased user CPU threshold to {:.1}%.",
                    self.user_cpu_threshold
                ));
                self.high_cpu_count = 0;
            }
        } else {
            self.high_cpu_count = 0;
        }
    }

    pub fn kill_process(&mut self, pid: usize) -> bool {
        if let Some(process) = self.sys.process(Pid::from(pid)) {
            process.kill()
        } else {
            false
        }
    }

    pub fn get_processes(&mut self) -> Vec<(usize, String, f32, u64)> {
        self.refresh().processes.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refresh_is_fast_without_blocking_sleeps() {
        let mut monitor = SystemMonitor::new();
        let start = std::time::Instant::now();
        let _ = monitor.refresh();
        assert!(
            start.elapsed().as_millis() < 200,
            "refresh took {}ms, expected <200ms",
            start.elapsed().as_millis()
        );
    }

    #[test]
    fn alerts_queue_instead_of_printing() {
        let mut monitor = SystemMonitor::new();
        for _ in 0..5 {
            monitor.update_alerts(95.0);
        }
        let alerts = monitor.take_alerts();
        assert!(alerts.iter().any(|a| a.contains("Sustained high CPU")));
    }
}
