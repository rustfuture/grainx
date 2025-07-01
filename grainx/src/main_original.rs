mod rendering;
mod analytics;
mod network;
mod config;

use sysinfo::{System};
use std::{thread, time::Duration, collections::VecDeque};
use rendering::{AdvancedCanvas, Rect};
use crossterm::{terminal, execute, cursor, style::{self, Color, SetForegroundColor, ResetColor}};
use std::io::{self, Write};
use analytics::{AnomalyDetector, AnomalyDetectorConfig, AnomalyStrategy, TimeSeriesPoint, calculate_correlation, evaluate_metric_formula, predict_next_value};
use chrono::Utc;
use network::{start_server, start_agent};
use config::DashboardConfig;
use std::collections::HashMap;

struct SystemMonitor {
    sys: System,
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

    // Network features temporarily disabled due to "No such device or address" error.
    // let server_addr = "127.0.0.1:8080";
    // tokio::spawn(async move {
    //     if let Err(e) = start_server(server_addr).await {
    //         eprintln!("Server error: {}", e);
    //     }
    // });

    // // Give the server a moment to start
    // thread::sleep(Duration::from_secs(2));

    // // Start an agent to send a message
    // if let Err(e) = start_agent(server_addr, "Hello from agent!").await {
    //     eprintln!("Agent error: {}", e);
    // }

    let mut monitor = SystemMonitor::new();
    let mut canvas = AdvancedCanvas::new();
    let anomaly_detector = AnomalyDetector::new(
        AnomalyDetectorConfig { threshold_multiplier: 2.0 },
        AnomalyStrategy::Statistical,
    );

    let mut points: Vec<(f64, f64)> = Vec::new();
    let rect = Rect { x: 0, y: 0, width: 80, height: 20 };

    let mut current_y_val = 0.0; // For smooth animation

    let mut cpu_history: Vec<f64> = Vec::new();
    let mut dummy_metric_history: Vec<f64> = Vec::new();

    for i in 0..100 {
        let cpu_usage = monitor.get_cpu_usage();
        println!("CPU Usage: {:.2}%", cpu_usage);

        let current_point = TimeSeriesPoint {
            timestamp: Utc::now(),
            value: cpu_usage as f64,
        };

        if let Some(anomaly) = anomaly_detector.detect_statistical_anomaly(&current_point).await {
            println!("ANOMALY DETECTED: {}", anomaly.message);
        }

        // Store historical data for correlation and prediction
        cpu_history.push(cpu_usage as f64);
        // Generate a dummy metric that somewhat correlates with CPU usage
        dummy_metric_history.push((cpu_usage as f64 * 0.5) + (i as f64 * 0.1));

        // Calculate correlation every 10 iterations
        if i % 10 == 0 && cpu_history.len() > 1 {
            if let Some(correlation) = calculate_correlation(&cpu_history, &dummy_metric_history) {
                println!("Correlation (CPU vs Dummy): {:.2}", correlation);
            }
        }

        // Predict next CPU usage
        if let Some(predicted_cpu) = predict_next_value(&cpu_history, 5) { // Predict using last 5 values
            println!("Predicted Next CPU Usage: {:.2}%", predicted_cpu);
        }

        // Evaluate custom metric formula
        let mut metrics = HashMap::new();
        metrics.insert("cpu_usage", cpu_usage as f64);
        let custom_formula = "cpu_usage * 1.5 + 5.0";
        if let Some(custom_metric_value) = evaluate_metric_formula(custom_formula, &metrics) {
            println!("Custom Metric ({}) : {:.2}", custom_formula, custom_metric_value);
        }

        // Normalize CPU usage to fit within the rect height
        let target_y_val = (cpu_usage as f64 / 100.0) * rect.height as f64;
        
        // Smoothly interpolate towards the target_y_val
        current_y_val = current_y_val * 0.8 + target_y_val * 0.2; // Simple linear interpolation

        points.push((i as f64, current_y_val));

        // Keep only the last `rect.width` points for scrolling effect
        if points.len() > rect.width as usize {
            points.remove(0);
            // Adjust x coordinates for scrolling
            for p in points.iter_mut() {
                p.0 -= 1.0;
            }
        }

        // Clear the area before redrawing
        for y in rect.y..(rect.y + rect.height) {
            canvas.set_cursor(rect.x, y)?;
            canvas.draw_str(&" ".repeat(rect.width as usize))?;
        }

        // Set color based on CPU usage
        let color = if cpu_usage > 80.0 {
            Color::Red
        } else if cpu_usage > 50.0 {
            Color::Yellow
        } else {
            Color::Green
        };
        canvas.set_color(color)?;

        canvas.draw_braille_line(&points, &rect)?;
        io::stdout().flush()?;

        thread::sleep(Duration::from_millis(dashboard_config.refresh_interval_ms));
    }

    terminal::disable_raw_mode()?;
    execute!(io::stdout(), terminal::LeaveAlternateScreen, cursor::Show, ResetColor)?;

    Ok(())
}