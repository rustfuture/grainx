use serde::{Serialize, Deserialize};
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

impl DashboardConfig {
    pub fn load_from_file(path: &str) -> io::Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: DashboardConfig = serde_json::from_str(&content)?;
        Ok(config)
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
                "analytics".to_string()
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
        assert!(config.cpu_warning_threshold > 0.0);
        assert!(config.memory_warning_threshold > 0.0);
        assert!(config.max_processes > 0);
        assert!(config.graph_history_size > 0);
        assert!(config.show_predictions);
        assert!(config.show_correlations);
        assert_eq!(config.color_theme, "default");
        assert!(config.log_enabled);
        assert_eq!(config.log_path, "grainx_metrics.log");
        assert_eq!(config.log_interval_iterations, 10);
    }

    #[test]
    fn test_config_serialization() {
        let config = DashboardConfig::default_config();
        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(json.contains("grainx_advanced"));
        assert!(json.contains("cpu_warning_threshold"));
        assert!(json.contains("color_theme"));
        assert!(json.contains("log_enabled"));
    }

    #[test]
    fn test_config_deserialization() {
        let json = r#"{
            "name": "test_config",
            "layout": ["cpu_graph", "memory_usage"],
            "refresh_interval_ms": 1000,
            "cpu_warning_threshold": 75.0,
            "memory_warning_threshold": 80.0,
            "show_predictions": true,
            "show_correlations": false,
            "max_processes": 5,
            "graph_history_size": 50
        }"#;
        
        let config: DashboardConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.name, "test_config");
        assert_eq!(config.refresh_interval_ms, 1000);
        assert_eq!(config.cpu_warning_threshold, 75.0);
        assert!(!config.show_correlations);
        assert_eq!(config.color_theme, "default");
        assert!(config.log_enabled);
        assert_eq!(config.log_path, "grainx_metrics.log");
        assert_eq!(config.log_interval_iterations, 10);
    }

    #[test]
    fn test_config_deserialization_with_color_theme() {
        let json = r#"{
            "name": "themed",
            "layout": ["cpu_graph"],
            "refresh_interval_ms": 500,
            "cpu_warning_threshold": 80.0,
            "memory_warning_threshold": 85.0,
            "show_predictions": true,
            "show_correlations": true,
            "max_processes": 10,
            "graph_history_size": 100,
            "color_theme": "dark"
        }"#;

        let config: DashboardConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.color_theme, "dark");
    }

    #[test]
    fn test_config_deserialization_without_log_fields() {
        let json = r#"{
            "name": "legacy_config",
            "layout": ["cpu_graph"],
            "refresh_interval_ms": 500,
            "cpu_warning_threshold": 80.0,
            "memory_warning_threshold": 85.0,
            "show_predictions": true,
            "show_correlations": true,
            "max_processes": 10,
            "graph_history_size": 100
        }"#;

        let config: DashboardConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.color_theme, "default");
        assert!(config.log_enabled);
        assert_eq!(config.log_path, "grainx_metrics.log");
        assert_eq!(config.log_interval_iterations, 10);
    }

    #[test]
    fn test_config_file_operations() {
        let config = DashboardConfig::default_config();
        let test_file = "test_config.json";
        
        config.save_to_file(test_file).unwrap();
        
        let loaded_config = DashboardConfig::load_from_file(test_file).unwrap();
        assert_eq!(config.name, loaded_config.name);
        assert_eq!(config.refresh_interval_ms, loaded_config.refresh_interval_ms);
        assert_eq!(config.color_theme, loaded_config.color_theme);
        assert_eq!(config.log_path, loaded_config.log_path);
        
        fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_config_validation() {
        let config = DashboardConfig::default_config();
        
        assert!(config.cpu_warning_threshold >= 0.0 && config.cpu_warning_threshold <= 100.0);
        assert!(config.memory_warning_threshold >= 0.0 && config.memory_warning_threshold <= 100.0);
        assert!(config.refresh_interval_ms >= 100);
        assert!(config.max_processes >= 1 && config.max_processes <= 50);
        assert!(config.graph_history_size >= 10 && config.graph_history_size <= 1000);
    }
}
