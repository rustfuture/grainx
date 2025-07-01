use serde::{Serialize, Deserialize};
use std::fs;
use std::io;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DashboardConfig {
    pub name: String,
    pub layout: Vec<String>, // Simple representation of layout for now
    pub refresh_interval_ms: u64,
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
            name: "default".to_string(),
            layout: vec!["cpu_graph".to_string(), "memory_usage".to_string()],
            refresh_interval_ms: 1000,
        }
    }
}
