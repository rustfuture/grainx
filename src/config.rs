use serde::{Deserialize, Serialize};
use std::fs;
use std::io;

fn default_color_theme() -> String {
    "default".to_string()
}

fn default_log_enabled() -> bool {
    true
}

fn default_log_path() -> String {
    "grainx_metrics.log".to_string()
}

fn default_log_interval_iterations() -> u32 {
    10
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DashboardConfig {
    pub name: String,
    pub layout: Vec<String>,
    pub refresh_interval_ms: u64,
    pub cpu_warning_threshold: f32,
    pub memory_warning_threshold: f32,
    pub show_predictions: bool,
    pub show_correlations: bool,
    pub max_processes: usize,
    pub graph_history_size: usize,
    #[serde(default = "default_color_theme")]
    pub color_theme: String,
    #[serde(default = "default_log_enabled")]
    pub log_enabled: bool,
    #[serde(default = "default_log_path")]
    pub log_path: String,
    #[serde(default = "default_log_interval_iterations")]
    pub log_interval_iterations: u32,
}

#[derive(Debug, Default, Clone)]
pub struct ConfigOverrides {
    pub refresh_interval_ms: Option<u64>,
    pub cpu_warning_threshold: Option<f32>,
    pub memory_warning_threshold: Option<f32>,
    pub color_theme: Option<String>,
}

impl From<&crate::cli::MonitorArgs> for ConfigOverrides {
    fn from(args: &crate::cli::MonitorArgs) -> Self {
        ConfigOverrides {
            refresh_interval_ms: args.refresh_interval_ms,
            cpu_warning_threshold: args.cpu_warning_threshold,
            memory_warning_threshold: args.memory_warning_threshold,
            color_theme: args.color_theme.clone(),
        }
    }
}

impl DashboardConfig {
    pub fn load_from_file(path: &str) -> io::Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: DashboardConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn load_resolved(path: &str, overrides: &ConfigOverrides) -> io::Result<Self> {
        let mut config = match Self::load_from_file(path) {
            Ok(config) => config,
            Err(_) => {
                let default_config = Self::default_config();
                default_config.save_to_file(path)?;
                default_config
            }
        };
        config.apply_env();
        config.apply_overrides(overrides);
        Ok(config)
    }

    fn apply_env(&mut self) {
        if let Ok(value) = std::env::var("GRAINX_REFRESH_INTERVAL_MS")
            && let Ok(ms) = value.parse()
        {
            self.refresh_interval_ms = ms;
        }
        if let Ok(value) = std::env::var("GRAINX_CPU_WARNING_THRESHOLD")
            && let Ok(threshold) = value.parse()
        {
            self.cpu_warning_threshold = threshold;
        }
        if let Ok(value) = std::env::var("GRAINX_MEMORY_WARNING_THRESHOLD")
            && let Ok(threshold) = value.parse()
        {
            self.memory_warning_threshold = threshold;
        }
        if let Ok(theme) = std::env::var("GRAINX_COLOR_THEME")
            && !theme.is_empty()
        {
            self.color_theme = theme;
        }
    }

    fn apply_overrides(&mut self, overrides: &ConfigOverrides) {
        if let Some(ms) = overrides.refresh_interval_ms {
            self.refresh_interval_ms = ms;
        }
        if let Some(threshold) = overrides.cpu_warning_threshold {
            self.cpu_warning_threshold = threshold;
        }
        if let Some(threshold) = overrides.memory_warning_threshold {
            self.memory_warning_threshold = threshold;
        }
        if let Some(theme) = overrides.color_theme.clone() {
            self.color_theme = theme;
        }
    }

    pub fn save_to_file(&self, path: &str) -> io::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn default_config() -> Self {
        DashboardConfig {
            name: "grainx_advanced".to_string(),
            layout: vec![
                "cpu_graph".to_string(),
                "memory_usage".to_string(),
                "network_stats".to_string(),
                "process_list".to_string(),
                "analytics".to_string(),
            ],
            refresh_interval_ms: 500,
            cpu_warning_threshold: 80.0,
            memory_warning_threshold: 85.0,
            show_predictions: true,
            show_correlations: true,
            max_processes: 10,
            graph_history_size: 100,
            color_theme: default_color_theme(),
            log_enabled: default_log_enabled(),
            log_path: default_log_path(),
            log_interval_iterations: default_log_interval_iterations(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_default_config_creation() {
        let config = DashboardConfig::default_config();
        assert_eq!(config.name, "grainx_advanced");
        assert!(config.refresh_interval_ms > 0);
        assert_eq!(config.color_theme, "default");
    }

    #[test]
    fn test_config_precedence_cli_over_env_and_file() {
        let path = "test_precedence_config.json";
        let mut file_config = DashboardConfig::default_config();
        file_config.refresh_interval_ms = 500;
        file_config.cpu_warning_threshold = 70.0;
        file_config.save_to_file(path).unwrap();

        unsafe {
            std::env::set_var("GRAINX_REFRESH_INTERVAL_MS", "750");
            std::env::set_var("GRAINX_CPU_WARNING_THRESHOLD", "72");
        }

        let overrides = ConfigOverrides {
            refresh_interval_ms: Some(1000),
            cpu_warning_threshold: Some(85.0),
            ..Default::default()
        };

        let resolved = DashboardConfig::load_resolved(path, &overrides).unwrap();
        assert_eq!(resolved.refresh_interval_ms, 1000);
        assert_eq!(resolved.cpu_warning_threshold, 85.0);

        unsafe {
            std::env::remove_var("GRAINX_REFRESH_INTERVAL_MS");
            std::env::remove_var("GRAINX_CPU_WARNING_THRESHOLD");
        }
        fs::remove_file(path).ok();
    }

    #[test]
    fn test_config_file_operations() {
        let config = DashboardConfig::default_config();
        let test_file = "test_config.json";

        config.save_to_file(test_file).unwrap();
        let loaded_config = DashboardConfig::load_from_file(test_file).unwrap();
        assert_eq!(config.name, loaded_config.name);
        assert_eq!(config.color_theme, loaded_config.color_theme);

        fs::remove_file(test_file).ok();
    }
}
