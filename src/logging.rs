use crate::metrics::MetricBackend;
use chrono::Utc;
use std::fs::OpenOptions;
use std::io::{self, Write};

pub struct MetricLogger {
    enabled: bool,
    log_path: String,
    log_interval_iterations: u32,
}

impl MetricLogger {
    pub fn new(enabled: bool, log_path: String, log_interval_iterations: u32) -> Self {
        MetricLogger {
            enabled,
            log_path,
            log_interval_iterations,
        }
    }

    pub fn from_config(config: &crate::config::DashboardConfig) -> Self {
        Self::new(
            config.log_enabled,
            config.log_path.clone(),
            config.log_interval_iterations,
        )
    }

    pub fn should_log(&self, iteration: i32) -> bool {
        if !self.enabled || self.log_interval_iterations == 0 {
            return false;
        }
        iteration > 0 && (iteration as u32).is_multiple_of(self.log_interval_iterations)
    }

    pub fn format_log_line(
        cpu_percent: f32,
        memory_used: u64,
        memory_total: u64,
        network_rx: u64,
        network_tx: u64,
    ) -> String {
        let timestamp = Utc::now().to_rfc3339();
        format!(
            "{},{},{},{},{},{}",
            timestamp, cpu_percent, memory_used, memory_total, network_rx, network_tx
        )
    }

    pub fn maybe_log(&mut self, iteration: i32, backend: &mut MetricBackend) -> io::Result<()> {
        if !self.should_log(iteration) {
            return Ok(());
        }

        let metrics = backend.refresh()?;
        let cpu_percent = metrics.cpu_usage;
        let memory_used = metrics.memory_used;
        let memory_total = metrics.memory_total;
        let network_rx = metrics.network_rx;
        let network_tx = metrics.network_tx;

        let line = Self::format_log_line(
            cpu_percent,
            memory_used,
            memory_total,
            network_rx,
            network_tx,
        );

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;

        writeln!(file, "{}", line)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn temp_log_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("grainx_{}_{}", name, std::process::id()))
    }

    #[test]
    fn test_log_line_format() {
        let line = MetricLogger::format_log_line(42.5, 1024, 8192, 100, 200);

        let parts: Vec<&str> = line.split(',').collect();
        assert_eq!(parts.len(), 6);
        assert!(parts[0].contains('T'), "timestamp should be ISO-8601");
        assert_eq!(parts[1], "42.5");
        assert_eq!(parts[2], "1024");
        assert_eq!(parts[3], "8192");
        assert_eq!(parts[4], "100");
        assert_eq!(parts[5], "200");
    }

    #[test]
    fn test_interval_gating() {
        let logger = MetricLogger::new(true, "grainx_metrics.log".to_string(), 10);

        assert!(!logger.should_log(0));
        assert!(!logger.should_log(1));
        assert!(!logger.should_log(9));
        assert!(logger.should_log(10));
        assert!(!logger.should_log(11));
        assert!(logger.should_log(20));
    }

    #[test]
    fn test_interval_gating_when_disabled() {
        let logger = MetricLogger::new(false, "grainx_metrics.log".to_string(), 10);
        assert!(!logger.should_log(10));
    }

    #[test]
    fn test_maybe_log_writes_on_interval() {
        let path = temp_log_path("maybe_log");
        fs::remove_file(&path).ok();

        let mut logger = MetricLogger::new(true, path.to_string_lossy().to_string(), 1);
        let mut backend = MetricBackend::local();

        logger.maybe_log(1, &mut backend).unwrap();

        let contents = fs::read_to_string(&path).unwrap();
        let line = contents.lines().next().expect("expected one log line");
        let parts: Vec<&str> = line.split(',').collect();
        assert_eq!(parts.len(), 6);

        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_maybe_log_skips_off_interval() {
        let path = temp_log_path("skip_log");
        fs::remove_file(&path).ok();

        let mut logger = MetricLogger::new(true, path.to_string_lossy().to_string(), 10);
        let mut backend = MetricBackend::local();

        logger.maybe_log(5, &mut backend).unwrap();
        assert!(!path.exists());

        fs::remove_file(&path).ok();
    }
}
