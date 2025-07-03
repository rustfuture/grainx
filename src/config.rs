use serde::{Serialize, Deserialize};
use std::fs;
use std::io;

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
    }

    #[test]
    fn test_config_serialization() {
        let config = DashboardConfig::default_config();
        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(json.contains("grainx_advanced"));
        assert!(json.contains("cpu_warning_threshold"));
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
    }

    #[test]
    fn test_config_file_operations() {
        let config = DashboardConfig::default_config();
        let test_file = "test_config.json";
        
        // Test saving
        config.save_to_file(test_file).unwrap();
        
        // Test loading
        let loaded_config = DashboardConfig::load_from_file(test_file).unwrap();
        assert_eq!(config.name, loaded_config.name);
        assert_eq!(config.refresh_interval_ms, loaded_config.refresh_interval_ms);
        
        // Cleanup
        fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_config_validation() {
        let config = DashboardConfig::default_config();
        
        // Test reasonable values
        assert!(config.cpu_warning_threshold >= 0.0 && config.cpu_warning_threshold <= 100.0);
        assert!(config.memory_warning_threshold >= 0.0 && config.memory_warning_threshold <= 100.0);
        assert!(config.refresh_interval_ms >= 100); // At least 100ms
        assert!(config.max_processes >= 1 && config.max_processes <= 50);
        assert!(config.graph_history_size >= 10 && config.graph_history_size <= 1000);
    }
}
