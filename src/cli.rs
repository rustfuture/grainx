use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "grainx",
    version,
    about = "Terminal system monitor with optional metrics agent"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run the interactive terminal dashboard (default)
    Monitor(MonitorArgs),
    /// Run HTTP metrics agent (GET /health, GET /metrics)
    Agent {
        /// Bind address
        #[arg(long, default_value = "127.0.0.1")]
        bind: String,
        /// Listen port
        #[arg(short, long, default_value_t = 9090)]
        port: u16,
    },
    /// Print version information
    Version,
}

#[derive(Args, Debug, Clone)]
pub struct MonitorArgs {
    /// Remote agent base URL (e.g. http://host:9090)
    #[arg(long)]
    pub remote: Option<String>,
    /// Path to dashboard configuration file
    #[arg(long, default_value = "dashboard_config.json")]
    pub config: String,
    /// Override refresh interval in milliseconds
    #[arg(long)]
    pub refresh_interval_ms: Option<u64>,
    /// Override CPU warning threshold (percent)
    #[arg(long)]
    pub cpu_warning_threshold: Option<f32>,
    /// Override memory warning threshold (percent)
    #[arg(long)]
    pub memory_warning_threshold: Option<f32>,
    /// Override color theme (default, dark, light, high_contrast)
    #[arg(long)]
    pub color_theme: Option<String>,
}

impl Cli {
    pub fn resolved_command(self) -> Commands {
        self.command.unwrap_or(Commands::Monitor(MonitorArgs {
            remote: None,
            config: "dashboard_config.json".to_string(),
            refresh_interval_ms: None,
            cpu_warning_threshold: None,
            memory_warning_threshold: None,
            color_theme: None,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn defaults_to_monitor() {
        let cli = Cli::parse_from(["grainx"]);
        match cli.resolved_command() {
            Commands::Monitor(args) => {
                assert!(args.remote.is_none());
                assert_eq!(args.config, "dashboard_config.json");
            }
            _ => panic!("expected monitor command"),
        }
    }

    #[test]
    fn parses_agent_flags() {
        let cli = Cli::parse_from(["grainx", "agent", "--bind", "0.0.0.0", "-p", "8080"]);
        match cli.resolved_command() {
            Commands::Agent { bind, port } => {
                assert_eq!(bind, "0.0.0.0");
                assert_eq!(port, 8080);
            }
            _ => panic!("expected agent command"),
        }
    }

    #[test]
    fn parses_monitor_remote_flag() {
        let cli = Cli::parse_from([
            "grainx",
            "monitor",
            "--remote",
            "http://127.0.0.1:9090",
            "--refresh-interval-ms",
            "1000",
        ]);
        match cli.resolved_command() {
            Commands::Monitor(args) => {
                assert_eq!(args.remote.as_deref(), Some("http://127.0.0.1:9090"));
                assert_eq!(args.refresh_interval_ms, Some(1000));
            }
            _ => panic!("expected monitor command"),
        }
    }
}
