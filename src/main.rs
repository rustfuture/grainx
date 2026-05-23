mod rendering;
mod analytics;
mod config;
mod logging;
mod monitor;
mod input;
mod ui;
mod help;
mod performance;

use std::{thread, time::Duration};
use rendering::{AdvancedCanvas, DashboardLayout};
use crossterm::{terminal, execute, cursor, style::ResetColor};

use std::io::{self};
use analytics::{AnomalyDetector, AnomalyDetectorConfig};
use config::DashboardConfig;
use logging::MetricLogger;

use crate::monitor::SystemMonitor;
use crate::input::{handle_input, Action};
use crate::ui::draw_dashboard;
use crate::performance::PerformanceMonitor;


#[tokio::main]
async fn main() -> io::Result<()> {
    execute!(io::stdout(), terminal::EnterAlternateScreen, cursor::Hide)?;
    terminal::enable_raw_mode()?;

    // Detect terminal size for dynamic layout
    let (term_w, term_h) = terminal::size()?;
    let mut layout = DashboardLayout::from_terminal_size(term_w, term_h);

    // Load dashboard configuration
    let config_path = "dashboard_config.json";
    let dashboard_config = match DashboardConfig::load_from_file(config_path) {
        Ok(config) => {
            config
        },
        Err(_) => {
            let default_config = DashboardConfig::default_config();
            default_config.save_to_file(config_path)?;
            default_config
        }
    };

    let mut monitor = SystemMonitor::new();
    let mut canvas = AdvancedCanvas::new();
    let anomaly_detector = AnomalyDetector::new(
        AnomalyDetectorConfig { threshold_multiplier: 2.0 },
    );

    let mut cpu_points: Vec<(f64, f64)> = Vec::new();
    let mut mem_points: Vec<(f64, f64)> = Vec::new();

    let mut current_cpu_y_val = 0.0; // For smooth animation
    let mut current_mem_y_val = 0.0; // For smooth animation

    let mut cpu_history: Vec<f64> = Vec::new();
    let mut mem_history: Vec<f64> = Vec::new();

    let mut iteration_count = 0;
    let mut selected_process = 0;
    let mut perf_monitor = PerformanceMonitor::new(60.0); // Target 60 FPS
    let mut metric_logger = MetricLogger::from_config(&dashboard_config);

    loop {
        perf_monitor.start_frame();
        
        let processes = monitor.get_processes();
        let current_cpu = monitor.last_cpu_usage;

        // Skip frame if system is overloaded
        if perf_monitor.should_skip_frame(current_cpu) {
            thread::sleep(Duration::from_millis(100));
            continue;
        }

        match handle_input(&mut selected_process, &processes, &mut monitor, &mut canvas, &layout, Some(&mut perf_monitor))? {
            Action::Exit => break,
            Action::Resize(w, h) => {
                layout = DashboardLayout::from_terminal_size(w, h);
            }
            Action::Continue => {}
        }

        draw_dashboard(
            &mut canvas,
            &mut monitor,
            &processes,
            &mut cpu_points,
            &mut mem_points,
            &mut cpu_history,
            &mut mem_history,
            &mut iteration_count,
            selected_process,
            &dashboard_config,
            &anomaly_detector,
            &layout,
            &mut current_cpu_y_val,
            &mut current_mem_y_val,
            &mut perf_monitor
        ).await?;

        metric_logger.maybe_log(iteration_count, &mut monitor)?;

        let frame_duration = perf_monitor.end_frame();
        let adaptive_refresh = perf_monitor.calculate_adaptive_refresh(current_cpu);
        
        // Use adaptive refresh rate
        let sleep_duration = adaptive_refresh.saturating_sub(frame_duration.as_millis() as u64);
        thread::sleep(Duration::from_millis(sleep_duration));
    }

    terminal::disable_raw_mode()?;
    execute!(io::stdout(), terminal::LeaveAlternateScreen, cursor::Show, ResetColor)?;

    Ok(())
}
