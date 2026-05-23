use crate::monitor::SystemMonitor;
use crate::rendering::{AdvancedCanvas, DashboardLayout};
use crate::analytics::{AnomalyDetector, TimeSeriesPoint, calculate_correlation, evaluate_metric_formula, predict_next_value};
use crate::config::DashboardConfig;
use crate::performance::PerformanceMonitor;
use crate::theme::ThemePalette;
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
    mem_history: &mut Vec<f64>,
    iteration_count: &mut i32,
    selected_process: usize,
    dashboard_config: &DashboardConfig,
    palette: &ThemePalette,
    anomaly_detector: &AnomalyDetector,
    layout: &DashboardLayout,
    current_cpu_y_val: &mut f64,
    current_mem_y_val: &mut f64,
    perf_monitor: &mut PerformanceMonitor
) -> io::Result<()> {
    canvas.set_cursor(0, 0)?;
    
    let cpu_usage = monitor.get_cpu_usage();
    let (used_memory, total_memory) = monitor.get_memory_usage();
    let memory_percentage = (used_memory as f64 / total_memory as f64) * 100.0;
    let (rx_bytes, tx_bytes) = monitor.get_network_io();
    let cpu_cores = monitor.get_cpu_cores();
    let disk_info = monitor.get_disk_usage();
    let (os_name, kernel_version, uptime) = monitor.get_system_info();
    
    *iteration_count += 1;
    
    canvas.set_color(palette.header)?;
    canvas.set_cursor(0, 0)?;
    canvas.draw_str(&format!("=== grainx System Monitor === Iteration: {} ===", iteration_count))?;
    
    canvas.set_cursor(0, 2)?;
    canvas.set_color(palette.label)?;
    canvas.draw_str("CPU Usage:")?;
    
    let cpu_color = if cpu_usage > dashboard_config.cpu_warning_threshold {
        palette.critical
    } else if cpu_usage > dashboard_config.cpu_warning_threshold * 0.7 {
        palette.warning
    } else {
        palette.ok
    };
    canvas.set_color(cpu_color)?;
    canvas.set_cursor(12, 2)?;
    canvas.draw_str(&format!("{:6.2}%", cpu_usage))?;
    
    cpu_history.push(cpu_usage as f64);
    mem_history.push(memory_percentage);
    
    if cpu_history.len() > dashboard_config.graph_history_size {
        cpu_history.remove(0);
        mem_history.remove(0);
    }
    
    anomaly_detector.add_point(cpu_usage as f64);
    
    let current_point = TimeSeriesPoint {
        timestamp: Utc::now(),
        value: cpu_usage as f64,
    };
    
    if let Some(anomaly) = anomaly_detector.detect_statistical_anomaly(&current_point).await {
        canvas.set_cursor(0, 3)?;
        canvas.set_color(palette.critical)?;
        canvas.draw_str(&format!("! ANOMALY: {}", anomaly.message))?;
    }
    
    let target_cpu_y = (cpu_usage as f64 / 100.0) * layout.cpu_rect.height as f64;
    *current_cpu_y_val = *current_cpu_y_val * 0.8 + target_cpu_y * 0.2;
    
    cpu_points.push((*iteration_count as f64, *current_cpu_y_val));
    if cpu_points.len() > layout.cpu_rect.width as usize {
        cpu_points.remove(0);
        for p in cpu_points.iter_mut() {
            p.0 -= 1.0;
        }
    }
    
    for y in layout.cpu_rect.y..(layout.cpu_rect.y + layout.cpu_rect.height) {
        canvas.set_cursor(layout.cpu_rect.x, y)?;
        canvas.draw_str(&" ".repeat(layout.cpu_rect.width as usize))?;
    }
    
    canvas.set_color(cpu_color)?;
    canvas.draw_braille_line(&cpu_points, &layout.cpu_rect)?;
    
    canvas.set_cursor(0, layout.mem_rect.y)?;
    canvas.set_color(palette.label)?;
    canvas.draw_str(&format!("Memory: {:.1}% ({:.1}GB/{:.1}GB)", 
        memory_percentage, 
        used_memory as f64 / 1_073_741_824.0, 
        total_memory as f64 / 1_073_741_824.0))?;
    
    let target_mem_y = (memory_percentage / 100.0) * layout.mem_rect.height as f64;
    *current_mem_y_val = *current_mem_y_val * 0.8 + target_mem_y * 0.2;
    
    mem_points.push((*iteration_count as f64, *current_mem_y_val));
    if mem_points.len() > layout.mem_rect.width as usize {
        mem_points.remove(0);
        for p in mem_points.iter_mut() {
            p.0 -= 1.0;
        }
    }
    
    for y in layout.mem_rect.y..(layout.mem_rect.y + layout.mem_rect.height) {
        canvas.set_cursor(layout.mem_rect.x, y)?;
        canvas.draw_str(&" ".repeat(layout.mem_rect.width as usize))?;
    }
    
    let mem_color = if memory_percentage > dashboard_config.memory_warning_threshold as f64 { 
        palette.critical
    } else if memory_percentage > dashboard_config.memory_warning_threshold as f64 * 0.7 {
        palette.warning
    } else { 
        palette.accent
    };
    canvas.set_color(mem_color)?;
    canvas.draw_braille_line(&mem_points, &layout.mem_rect)?;
    
    canvas.set_cursor(0, layout.network_start_y)?;
    canvas.set_color(palette.header)?;
    canvas.draw_str(&format!("System: {} | Kernel: {} | Uptime: {}h", 
        os_name, 
        kernel_version,
        uptime / 3600))?;

    canvas.set_cursor(0, layout.network_start_y + 1)?;
    canvas.set_color(palette.accent)?;
    canvas.draw_str(&format!("Network: RX:{:.1}MB TX:{:.1}MB", 
        rx_bytes as f64 / 1_048_576.0, 
        tx_bytes as f64 / 1_048_576.0))?;

    canvas.set_cursor(0, layout.network_start_y + 2)?;
    canvas.set_color(palette.label)?;
    canvas.draw_str("CPU Cores: ")?;
    for (i, core_usage) in cpu_cores.iter().enumerate().take(8) {
        let core_color = if *core_usage > 80.0 {
            palette.critical
        } else if *core_usage > 50.0 {
            palette.warning
        } else {
            palette.ok
        };
        canvas.set_color(core_color)?;
        canvas.draw_str(&format!("C{}:{:4.1}% ", i, core_usage))?;
    }

    canvas.set_cursor(0, layout.network_start_y + 3)?;
    canvas.set_color(palette.label)?;
    canvas.draw_str("Disks: ")?;
    for (name, total, _available, used_pct) in disk_info.iter().take(3) {
        let disk_color = if *used_pct > 90.0 {
            palette.critical
        } else if *used_pct > 75.0 {
            palette.warning
        } else {
            palette.ok
        };
        canvas.set_color(disk_color)?;
        canvas.draw_str(&format!("{}:{:.1}%({:.1}GB) ", 
            name.chars().take(3).collect::<String>(), 
            used_pct, 
            *total as f64 / 1_073_741_824.0))?;
    }
    
    if dashboard_config.show_correlations || dashboard_config.show_predictions {
        canvas.set_cursor(0, layout.network_start_y + 4)?;
        canvas.set_color(palette.warning)?;
        
        if dashboard_config.show_correlations && *iteration_count % 10 == 0 && cpu_history.len() > 5 && mem_history.len() > 5 {
            if let Some(correlation) = calculate_correlation(&cpu_history, &mem_history) {
                canvas.set_cursor(0, layout.network_start_y + 4)?;
                canvas.draw_str(&format!("Corr(CPU↔Mem): {:.3}  ", correlation))?;
            }
        }
        
        if dashboard_config.show_predictions {
            if let Some(predicted_cpu) = predict_next_value(&cpu_history, 5) {
                canvas.set_cursor(25, layout.network_start_y + 4)?;
                canvas.draw_str(&format!("Pred CPU: {:.1}%  ", predicted_cpu))?;
            }
        }
        
        let mut metrics = HashMap::new();
        metrics.insert("cpu_usage", cpu_usage as f64);
        if let Some(custom_value) = evaluate_metric_formula("cpu_usage * 1.5 + 5.0", &metrics) {
            canvas.set_cursor(45, layout.network_start_y + 4)?;
            canvas.draw_str(&format!("Custom: {:.1}", custom_value))?;
        }
    }
    
    canvas.set_cursor(0, layout.proc_start_y)?;
    canvas.set_color(palette.label)?;
    canvas.draw_str("Top Processes (UP/DOWN to select, 'k' to kill, 'q' to quit):")?;
    
    for (i, (pid, name, cpu, memory)) in processes.iter().take(dashboard_config.max_processes).enumerate() {
        let y_pos = layout.proc_start_y + 1 + i as u16;
        canvas.set_cursor(0, y_pos)?;
        
        if i == selected_process {
            canvas.set_color(palette.selection)?;
            canvas.draw_str(&format!("> {:5} {:20} {:6.1}% {:8}KB", pid, name, cpu, memory / 1024))?;
        } else {
            canvas.set_color(palette.label)?;
            canvas.draw_str(&format!("  {:5} {:20} {:6.1}% {:8}KB", pid, name, cpu, memory / 1024))?;
        }
    }
    
    let (fps, frame_time, adaptive) = perf_monitor.get_performance_stats();
    canvas.set_cursor(0, layout.footer_y - 1)?;
    canvas.set_color(palette.muted)?;
    canvas.draw_str(&format!("Performance: {:.1}FPS | {:.1}ms | Adaptive: {}", 
        fps, frame_time, if adaptive { "ON" } else { "OFF" }))?;

    canvas.set_cursor(0, layout.footer_y)?;
    canvas.set_color(palette.muted)?;
    canvas.draw_str(&format!("Config: {} | Controls: q=quit, h=help, p=pause, a=adaptive", 
        dashboard_config.name))?;
    
    Ok(())
}
