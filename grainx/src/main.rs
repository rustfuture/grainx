mod rendering;
mod analytics;
mod config;

use sysinfo::{System, Networks};
use std::{thread, time::Duration, collections::VecDeque};
use rendering::{AdvancedCanvas, Rect};
use crossterm::{terminal, execute, cursor, event::{self, Event, KeyCode}, style::{Color, ResetColor}};
use std::io::{self, Write};
use analytics::{AnomalyDetector, AnomalyDetectorConfig, AnomalyStrategy, TimeSeriesPoint, calculate_correlation, evaluate_metric_formula, predict_next_value};
use chrono::Utc;

use config::DashboardConfig;
use std::collections::HashMap;

struct SystemMonitor {
    sys: System,
    networks: Networks,
    last_cpu_usage: f32,
    high_cpu_duration: u32, // Counter for consecutive high CPU readings
    cpu_state_history: VecDeque<bool>, // true for high CPU, false for low
    user_cpu_threshold: f32, // User-learned CPU threshold
    high_cpu_count: u32, // Counter for how many times CPU exceeds user_cpu_threshold
}

impl SystemMonitor {
    fn new() -> Self {
        SystemMonitor {
            sys: System::new_all(),
            
            networks: Networks::new_with_refreshed_list(),
            last_cpu_usage: 0.0,
            high_cpu_duration: 0,
            cpu_state_history: VecDeque::with_capacity(5),
            user_cpu_threshold: 75.0, // Initial threshold
            high_cpu_count: 0,
        }
    }

    fn get_cpu_usage(&mut self) -> f32 {
        self.sys.refresh_cpu();
        thread::sleep(Duration::from_millis(200)); // Initial wait for CPU usage to be calculated
        self.sys.refresh_cpu();
        let current_cpu_usage = self.sys.global_cpu_info().cpu_usage();

        // Adaptive sampling logic
        let sampling_interval_ms = if current_cpu_usage > 80.0 {
            100 // High CPU, sample more frequently
        } else if current_cpu_usage > 50.0 {
            200 // Medium CPU
        } else {
            500 // Low CPU, sample less frequently
        };
        thread::sleep(Duration::from_millis(sampling_interval_ms));

        // Bottleneck identification logic
        if current_cpu_usage > 90.0 {
            self.high_cpu_duration += 1;
            if self.high_cpu_duration >= 5 { // If high CPU for 5 consecutive readings
                println!("\nWARNING: Sustained high CPU usage detected! Potential CPU bottleneck.\n");
            }
        } else {
            self.high_cpu_duration = 0;
        }

        // Behavioral pattern recognition
        let is_high_cpu = current_cpu_usage > 70.0; // Define a threshold for "high"
        self.cpu_state_history.push_back(is_high_cpu);
        if self.cpu_state_history.len() > 5 { // Keep history size limited
            self.cpu_state_history.pop_front();
        }

        // Check for a simple fluctuation pattern: High -> Low -> High
        if self.cpu_state_history.len() == 3 && 
           self.cpu_state_history[0] == true && 
           self.cpu_state_history[1] == false && 
           self.cpu_state_history[2] == true {
            println!("\nPATTERN DETECTED: CPU usage fluctuating (High-Low-High)!\n");
            // Clear history to avoid repeated detection of the same pattern
            self.cpu_state_history.clear();
        }

        // User behavior learning for personalized monitoring
        if current_cpu_usage > self.user_cpu_threshold {
            self.high_cpu_count += 1;
            if self.high_cpu_count >= 10 { // If CPU exceeds user threshold 10 times
                self.user_cpu_threshold += 5.0; // Increase threshold
                println!("\nUSER BEHAVIOR LEARNED: Increased user CPU threshold to {:.1}%\n", self.user_cpu_threshold);
                self.high_cpu_count = 0; // Reset counter
            }
        } else {
            self.high_cpu_count = 0; // Reset if CPU drops below threshold
        }

        self.last_cpu_usage = current_cpu_usage;
        current_cpu_usage
    }

    fn get_memory_usage(&mut self) -> (u64, u64) {
        self.sys.refresh_memory();
        (self.sys.used_memory(), self.sys.total_memory())
    }

    fn get_processes(&mut self) -> Vec<(String, f32, u64)> {
        self.sys.refresh_processes();
        let mut processes: Vec<(String, f32, u64)> = self.sys.processes().values()
            .map(|p| (p.name().to_string(), p.cpu_usage(), p.memory()))
            .collect();
        processes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        processes.truncate(5); // Top 5 processes
        processes
    }

    fn get_network_io(&mut self) -> (u64, u64) {
        self.networks.refresh();
        let mut received_bytes = 0;
        let mut transmitted_bytes = 0;
        for (_interface_name, data) in self.networks.list() {
            received_bytes += data.received();
            transmitted_bytes += data.transmitted();
        }
        (received_bytes, transmitted_bytes)
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    execute!(io::stdout(), terminal::EnterAlternateScreen, cursor::Hide)?;
    terminal::enable_raw_mode()?;

    // Load dashboard configuration
    let config_path = "dashboard_config.json";
    let dashboard_config = match DashboardConfig::load_from_file(config_path) {
        Ok(config) => {
            println!("Loaded dashboard config: {}", config.name);
            config
        },
        Err(_) => {
            println!("No dashboard config found, creating default.");
            let default_config = DashboardConfig::default_config();
            default_config.save_to_file(config_path)?;
            default_config
        }
    };

    

    let mut monitor = SystemMonitor::new();
    let mut canvas = AdvancedCanvas::new();
    let anomaly_detector = AnomalyDetector::new(
        AnomalyDetectorConfig { threshold_multiplier: 2.0 },
        AnomalyStrategy::Statistical,
    );

    let mut cpu_points: Vec<(f64, f64)> = Vec::new();
    let mut mem_points: Vec<(f64, f64)> = Vec::new();

    let cpu_rect = Rect { x: 0, y: 0, width: 80, height: 20 };
    let mem_rect = Rect { x: 0, y: 21, width: 80, height: 10 };
    let network_start_y = 32;
    let proc_start_y = 35;

    let mut current_cpu_y_val = 0.0; // For smooth animation
    let mut current_mem_y_val = 0.0; // For smooth animation

    let mut cpu_history: Vec<f64> = Vec::new();
    let mut dummy_metric_history: Vec<f64> = Vec::new();

    let mut iteration_count = 0;

    loop {
        // Handle events (e.g., keyboard input)
        if event::poll(Duration::from_millis(dashboard_config.refresh_interval_ms / 2))? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.code == KeyCode::Char('q') {
                    break; // Exit on 'q'
                }
            }
        }

        // Clear the entire screen for a fresh redraw
        execute!(io::stdout(), terminal::Clear(terminal::ClearType::All), cursor::MoveTo(0,0))?;

        let cpu_usage = monitor.get_cpu_usage();
        let (used_mem, total_mem) = monitor.get_memory_usage();
        let processes = monitor.get_processes();
        let (received_bytes, transmitted_bytes) = monitor.get_network_io();

        // CPU Panel
        canvas.set_color(Color::Rgb { r: 0, g: 255, b: 255 })?;
        canvas.draw_text_in_rect(&format!("CPU Usage: {:.2}%", cpu_usage), &cpu_rect, 0)?;

        let current_point = TimeSeriesPoint {
            timestamp: Utc::now(),
            value: cpu_usage as f64,
        };

        if let Some(anomaly) = anomaly_detector.detect_statistical_anomaly(&current_point).await {
            canvas.set_color(Color::Rgb { r: 255, g: 0, b: 0 })?;
            canvas.draw_text_in_rect(&format!("ANOMALY DETECTED: {}", anomaly.message), &cpu_rect, 1)?;
        }

        // Store historical data for correlation and prediction
        cpu_history.push(cpu_usage as f64);
        dummy_metric_history.push((cpu_usage as f64 * 0.5) + (iteration_count as f64 * 0.1));

        // Calculate correlation every 10 iterations
        if iteration_count % 10 == 0 && cpu_history.len() > 1 {
            if let Some(correlation) = calculate_correlation(&cpu_history, &dummy_metric_history) {
                canvas.set_color(Color::Rgb { r: 255, g: 0, b: 255 })?;
                canvas.draw_text_in_rect(&format!("Correlation (CPU vs Dummy): {:.2}", correlation), &cpu_rect, 2)?;
            }
        }

        // Predict next CPU usage
        if let Some(predicted_cpu) = predict_next_value(&cpu_history, 5) { // Predict using last 5 values
            canvas.set_color(Color::Rgb { r: 0, g: 0, b: 255 })?;
            canvas.draw_text_in_rect(&format!("Predicted Next CPU Usage: {:.2}%", predicted_cpu), &cpu_rect, 3)?;
        }

        // Evaluate custom metric formula
        let mut metrics = HashMap::new();
        metrics.insert("cpu_usage", cpu_usage as f64);
        let custom_formula = "cpu_usage * 1.5 + 5.0";
        if let Some(custom_metric_value) = evaluate_metric_formula(custom_formula, &metrics) {
            canvas.set_color(Color::Rgb { r: 255, g: 255, b: 0 })?;
            canvas.draw_text_in_rect(&format!("Custom Metric ({}) : {:.2}", custom_formula, custom_metric_value), &cpu_rect, 4)?;
        }

        // Normalize CPU usage to fit within the rect height
        let target_cpu_y_val = (cpu_usage as f64 / 100.0) * cpu_rect.height as f64;
        
        // Smoothly interpolate towards the target_cpu_y_val
        current_cpu_y_val = current_cpu_y_val * 0.8 + target_cpu_y_val * 0.2; // Simple linear interpolation

        cpu_points.push((iteration_count as f64, current_cpu_y_val));

        // Keep only the last `cpu_rect.width` points for scrolling effect
        if cpu_points.len() > cpu_rect.width as usize {
            cpu_points.remove(0);
            // Adjust x coordinates for scrolling
            for p in cpu_points.iter_mut() {
                p.0 -= 1.0;
            }
        }

        // Set color based on CPU usage for the graph
        let graph_color = if cpu_usage > 80.0 {
            Color::Rgb { r: 255, g: 0, b: 0 }
        } else if cpu_usage > 50.0 {
            Color::Rgb { r: 255, g: 255, b: 0 }
        } else {
            Color::Rgb { r: 0, g: 255, b: 0 }
        };
        canvas.set_color(graph_color)?;

        canvas.set_cursor(cpu_rect.x, cpu_rect.y + 5)?;
        canvas.draw_braille_line(&cpu_points, &cpu_rect)?;

        // Memory Panel
        let mem_percentage = (used_mem as f64 / total_mem as f64) * 100.0;
        canvas.set_color(Color::Rgb { r: 0, g: 255, b: 255 })?;
        canvas.draw_text_in_rect(&format!("Memory: {}MB / {}MB ({:.2}%)", used_mem / 1024 / 1024, total_mem / 1024 / 1024, mem_percentage), &mem_rect, 0)?;

        let target_mem_y_val = (mem_percentage / 100.0) * mem_rect.height as f64;
        current_mem_y_val = current_mem_y_val * 0.8 + target_mem_y_val * 0.2;

        mem_points.push((iteration_count as f64, current_mem_y_val));
        if mem_points.len() > mem_rect.width as usize {
            mem_points.remove(0);
            for p in mem_points.iter_mut() {
                p.0 -= 1.0;
            }
        }

        let mem_graph_color = if mem_percentage > 80.0 {
            Color::Rgb { r: 255, g: 0, b: 0 }
        } else if mem_percentage > 50.0 {
            Color::Rgb { r: 255, g: 255, b: 0 }
        } else {
            Color::Rgb { r: 0, g: 255, b: 0 }
        };
        canvas.set_color(mem_graph_color)?;
        canvas.set_cursor(mem_rect.x, mem_rect.y + 1)?;
        canvas.draw_braille_line(&mem_points, &mem_rect)?;

        // Processes Panel
        canvas.set_color(Color::Rgb { r: 0, g: 255, b: 255 })?;
        canvas.draw_text_in_rect("Top 5 Processes (CPU % | Mem MB):", &Rect { x: 0, y: proc_start_y, width: 80, height: 1 }, 0)?;
        for (idx, (name, cpu, mem)) in processes.iter().enumerate() {
            canvas.draw_text_in_rect(&format!("{:<20} {:>5.1}% {:>8}MB", name, cpu, mem / 1024 / 1024), &Rect { x: 0, y: proc_start_y, width: 80, height: 1 }, 1 + idx as u16)?;
        }

        

        // Network I/O Panel
        canvas.set_color(Color::Rgb { r: 0, g: 255, b: 255 })?;
        canvas.draw_text_in_rect(&format!("Network I/O: Received: {} MB, Transmitted: {} MB", received_bytes / 1024 / 1024, transmitted_bytes / 1024 / 1024), &Rect { x: 0, y: network_start_y, width: 80, height: 1 }, 0)?;

        io::stdout().flush()?;

        iteration_count += 1;
    }

    terminal::disable_raw_mode()?;
    execute!(io::stdout(), terminal::LeaveAlternateScreen, cursor::Show, ResetColor)?;

    Ok(())
}