use crate::monitor::SystemMonitor;
use crate::rendering::{AdvancedCanvas, Rect};
use crate::analytics::{AnomalyDetector, TimeSeriesPoint, calculate_correlation, evaluate_metric_formula, predict_next_value};
use crate::config::DashboardConfig;
use crate::performance::PerformanceMonitor;
use crossterm::style::Color;
use std::io;
use std::collections::HashMap;
use chrono::Utc;

pub async fn draw_dashboard(
    canvas: &mut AdvancedCanvas, 
    monitor: &mut SystemMonitor, 
    processes: &Vec<(usize, String, f32, u64)>, 
    cpu_points: &mut Vec<(f64, f64)>, 
    mem_points: &mut Vec<(f64, f64)>, 
    cpu_history: &mut Vec<f64>,
    dummy_metric_history: &mut Vec<f64>,
    iteration_count: &mut i32,
    selected_process: usize,
    dashboard_config: &DashboardConfig,
    anomaly_detector: &AnomalyDetector,
    cpu_rect: &Rect,
    mem_rect: &Rect,
    network_start_y: u16,
    proc_start_y: u16,
    current_cpu_y_val: &mut f64,
    current_mem_y_val: &mut f64,
    perf_monitor: &mut PerformanceMonitor
) -> io::Result<()> {
    // Clear screen
    canvas.set_cursor(0, 0)?;
    
    // Get system metrics
    let cpu_usage = monitor.get_cpu_usage();
    let (used_memory, total_memory) = monitor.get_memory_usage();
    let memory_percentage = (used_memory as f64 / total_memory as f64) * 100.0;
    let (rx_bytes, tx_bytes) = monitor.get_network_io();
    let cpu_cores = monitor.get_cpu_cores();
    let disk_info = monitor.get_disk_usage();
    let (os_name, kernel_version, uptime) = monitor.get_system_info();
    
    // Update iteration count
    *iteration_count += 1;
    
    // === HEADER ===
    canvas.set_color(Color::Cyan)?;
    canvas.set_cursor(0, 0)?;
    canvas.draw_str(&format!("=== grainx System Monitor === Iteration: {} ===", iteration_count))?;
    
    // === CPU SECTION ===
    canvas.set_cursor(0, 2)?;
    canvas.set_color(Color::White)?;
    canvas.draw_str("CPU Usage:")?;
    
    // CPU percentage with color coding (using config thresholds)
    let cpu_color = if cpu_usage > dashboard_config.cpu_warning_threshold {
        Color::Red
    } else if cpu_usage > dashboard_config.cpu_warning_threshold * 0.7 {
        Color::Yellow
    } else {
        Color::Green
    };
    canvas.set_color(cpu_color)?;
    canvas.set_cursor(12, 2)?;
    canvas.draw_str(&format!("{:6.2}%", cpu_usage))?;
    
    // CPU History and Analytics
    cpu_history.push(cpu_usage as f64);
    dummy_metric_history.push((cpu_usage as f64 * 0.5) + (*iteration_count as f64 * 0.1));
    
    // Keep history size manageable (using config)
    if cpu_history.len() > dashboard_config.graph_history_size {
        cpu_history.remove(0);
        dummy_metric_history.remove(0);
    }
    
    // Anomaly Detection
    let current_point = TimeSeriesPoint {
        timestamp: Utc::now(),
        value: cpu_usage as f64,
    };
    
    if let Some(anomaly) = anomaly_detector.detect_statistical_anomaly(&current_point).await {
        canvas.set_cursor(0, 3)?;
        canvas.set_color(Color::Red)?;
        canvas.draw_str(&format!("! ANOMALY: {}", anomaly.message))?;
    }
    
    // CPU Graph
    let target_cpu_y = (cpu_usage as f64 / 100.0) * cpu_rect.height as f64;
    *current_cpu_y_val = *current_cpu_y_val * 0.8 + target_cpu_y * 0.2;
    
    cpu_points.push((*iteration_count as f64, *current_cpu_y_val));
    if cpu_points.len() > cpu_rect.width as usize {
        cpu_points.remove(0);
        for p in cpu_points.iter_mut() {
            p.0 -= 1.0;
        }
    }
    
    // Clear CPU graph area
    for y in cpu_rect.y..(cpu_rect.y + cpu_rect.height) {
        canvas.set_cursor(cpu_rect.x, y)?;
        canvas.draw_str(&" ".repeat(cpu_rect.width as usize))?;
    }
    
    canvas.set_color(cpu_color)?;
    canvas.draw_braille_line(&cpu_points, &cpu_rect)?;
    
    // === MEMORY SECTION ===
    canvas.set_cursor(0, mem_rect.y)?;
    canvas.set_color(Color::White)?;
    canvas.draw_str(&format!("Memory: {:.1}% ({:.1}GB/{:.1}GB)", 
        memory_percentage, 
        used_memory as f64 / 1_073_741_824.0, 
        total_memory as f64 / 1_073_741_824.0))?;
    
    // Memory Graph
    let target_mem_y = (memory_percentage / 100.0) * mem_rect.height as f64;
    *current_mem_y_val = *current_mem_y_val * 0.8 + target_mem_y * 0.2;
    
    mem_points.push((*iteration_count as f64, *current_mem_y_val));
    if mem_points.len() > mem_rect.width as usize {
        mem_points.remove(0);
        for p in mem_points.iter_mut() {
            p.0 -= 1.0;
        }
    }
    
    // Clear memory graph area
    for y in mem_rect.y..(mem_rect.y + mem_rect.height) {
        canvas.set_cursor(mem_rect.x, y)?;
        canvas.draw_str(&" ".repeat(mem_rect.width as usize))?;
    }
    
    let mem_color = if memory_percentage > dashboard_config.memory_warning_threshold as f64 { 
        Color::Red 
    } else if memory_percentage > dashboard_config.memory_warning_threshold as f64 * 0.7 {
        Color::Yellow
    } else { 
        Color::Blue 
    };
    canvas.set_color(mem_color)?;
    canvas.draw_braille_line(&mem_points, &mem_rect)?;
    
    // === SYSTEM INFO SECTION ===
    canvas.set_cursor(0, network_start_y)?;
    canvas.set_color(Color::Cyan)?;
    canvas.draw_str(&format!("System: {} | Kernel: {} | Uptime: {}h", 
        os_name, 
        kernel_version,
        uptime / 3600))?;

    // === NETWORK SECTION ===
    canvas.set_cursor(0, network_start_y + 1)?;
    canvas.set_color(Color::Magenta)?;
    canvas.draw_str(&format!("Network: RX:{:.1}MB TX:{:.1}MB", 
        rx_bytes as f64 / 1_048_576.0, 
        tx_bytes as f64 / 1_048_576.0))?;

    // === CPU CORES SECTION ===
    canvas.set_cursor(0, network_start_y + 2)?;
    canvas.set_color(Color::Yellow)?;
    canvas.draw_str("CPU Cores: ")?;
    for (i, core_usage) in cpu_cores.iter().enumerate().take(8) { // Show max 8 cores
        let core_color = if *core_usage > 80.0 {
            Color::Red
        } else if *core_usage > 50.0 {
            Color::Yellow
        } else {
            Color::Green
        };
        canvas.set_color(core_color)?;
        canvas.draw_str(&format!("C{}:{:4.1}% ", i, core_usage))?;
    }

    // === DISK USAGE SECTION ===
    canvas.set_cursor(0, network_start_y + 3)?;
    canvas.set_color(Color::Blue)?;
    canvas.draw_str("Disks: ")?;
    for (name, total, _available, used_pct) in disk_info.iter().take(3) { // Show max 3 disks
        let disk_color = if *used_pct > 90.0 {
            Color::Red
        } else if *used_pct > 75.0 {
            Color::Yellow
        } else {
            Color::Green
        };
        canvas.set_color(disk_color)?;
        canvas.draw_str(&format!("{}:{:.1}%({:.1}GB) ", 
            name.chars().take(3).collect::<String>(), 
            used_pct, 
            *total as f64 / 1_073_741_824.0))?;
    }
    
    // === ANALYTICS SECTION ===
    if dashboard_config.show_correlations || dashboard_config.show_predictions {
        canvas.set_cursor(0, network_start_y + 1)?;
        canvas.set_color(Color::Yellow)?;
        
        // Correlation analysis (if enabled)
        if dashboard_config.show_correlations && *iteration_count % 10 == 0 && cpu_history.len() > 5 {
            if let Some(correlation) = calculate_correlation(&cpu_history, &dummy_metric_history) {
                canvas.draw_str(&format!("Correlation: {:.3}", correlation))?;
            }
        }
        
        // Prediction (if enabled)
        if dashboard_config.show_predictions {
            if let Some(predicted_cpu) = predict_next_value(&cpu_history, 5) {
                canvas.set_cursor(25, network_start_y + 1)?;
                canvas.draw_str(&format!("Predicted CPU: {:.1}%", predicted_cpu))?;
            }
        }
    }
    
    // Custom metric
    let mut metrics = HashMap::new();
    metrics.insert("cpu_usage", cpu_usage as f64);
    if let Some(custom_value) = evaluate_metric_formula("cpu_usage * 1.5 + 5.0", &metrics) {
        canvas.set_cursor(50, network_start_y + 1)?;
        canvas.draw_str(&format!("Custom: {:.1}", custom_value))?;
    }
    
    // === PROCESS SECTION ===
    canvas.set_cursor(0, proc_start_y)?;
    canvas.set_color(Color::White)?;
    canvas.draw_str("Top Processes (UP/DOWN to select, 'k' to kill, 'q' to quit):")?;
    
    for (i, (pid, name, cpu, memory)) in processes.iter().take(dashboard_config.max_processes).enumerate() {
        let y_pos = proc_start_y + 1 + i as u16;
        canvas.set_cursor(0, y_pos)?;
        
        // Highlight selected process
        if i == selected_process {
            canvas.set_color(Color::Black)?; // This will be highlighted
            canvas.draw_str(&format!("> {:5} {:20} {:6.1}% {:8}KB", pid, name, cpu, memory / 1024))?;
        } else {
            canvas.set_color(Color::White)?;
            canvas.draw_str(&format!("  {:5} {:20} {:6.1}% {:8}KB", pid, name, cpu, memory / 1024))?;
        }
    }
    
    // === PERFORMANCE SECTION ===
    let (fps, frame_time, adaptive) = perf_monitor.get_performance_stats();
    canvas.set_cursor(0, proc_start_y + 8)?;
    canvas.set_color(Color::DarkGrey)?;
    canvas.draw_str(&format!("Performance: {:.1}FPS | {:.1}ms | Adaptive: {}", 
        fps, frame_time, if adaptive { "ON" } else { "OFF" }))?;

    // === FOOTER ===
    canvas.set_cursor(0, proc_start_y + 9)?;
    canvas.set_color(Color::DarkGrey)?;
    canvas.draw_str(&format!("Config: {} | Controls: q=quit, h=help, p=pause, a=adaptive", 
        dashboard_config.name))?;
    
    Ok(())
}