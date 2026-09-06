use crate::analytics::{
    AnomalyDetector, TimeSeriesPoint, calculate_correlation, evaluate_metric_formula,
    predict_next_value,
};
use crate::config::DashboardConfig;
use crate::error::Result;
use crate::metrics::MetricBackend;
use crate::network::throughput_kbps;
use crate::performance::PerformanceMonitor;
use crate::rendering::{AdvancedCanvas, DashboardLayout};
use crate::theme::ThemePalette;
use chrono::Utc;
use std::collections::HashMap;

/// Mutable graph/history state carried across TUI frames.
pub struct DashboardState {
    pub cpu_points: Vec<(f64, f64)>,
    pub mem_points: Vec<(f64, f64)>,
    pub net_rx_points: Vec<(f64, f64)>,
    pub net_tx_points: Vec<(f64, f64)>,
    pub cpu_history: Vec<f64>,
    pub mem_history: Vec<f64>,
    pub last_rx_bytes: u64,
    pub last_tx_bytes: u64,
    pub iteration_count: i32,
    pub current_cpu_y_val: f64,
    pub current_mem_y_val: f64,
    pub current_net_rx_y: f64,
    pub current_net_tx_y: f64,
}

impl Default for DashboardState {
    fn default() -> Self {
        Self {
            cpu_points: Vec::new(),
            mem_points: Vec::new(),
            net_rx_points: Vec::new(),
            net_tx_points: Vec::new(),
            cpu_history: Vec::new(),
            mem_history: Vec::new(),
            last_rx_bytes: 0,
            last_tx_bytes: 0,
            iteration_count: 0,
            current_cpu_y_val: 0.0,
            current_mem_y_val: 0.0,
            current_net_rx_y: 0.0,
            current_net_tx_y: 0.0,
        }
    }
}

pub struct DrawContext<'a> {
    pub backend: &'a mut MetricBackend,
    pub processes: &'a [(usize, String, f32, u64)],
    pub config: &'a DashboardConfig,
    pub palette: &'a ThemePalette,
    pub anomaly_detector: &'a AnomalyDetector,
    pub layout: &'a DashboardLayout,
    pub selected_process: usize,
    pub perf_monitor: &'a mut PerformanceMonitor,
}

pub async fn draw_dashboard(
    canvas: &mut AdvancedCanvas,
    state: &mut DashboardState,
    ctx: DrawContext<'_>,
) -> Result<()> {
    canvas.set_cursor(0, 0)?;

    let metrics = ctx.backend.refresh()?;
    let cpu_usage = metrics.cpu_usage;
    let used_memory = metrics.memory_used;
    let total_memory = metrics.memory_total;
    let memory_percentage = if total_memory > 0 {
        (used_memory as f64 / total_memory as f64) * 100.0
    } else {
        0.0
    };
    let rx_bytes = metrics.network_rx;
    let tx_bytes = metrics.network_tx;
    let cpu_cores = &metrics.cpu_cores;
    let disk_info = &metrics.disks;
    let os_name = &metrics.os_name;
    let kernel_version = &metrics.kernel_version;
    let uptime = metrics.uptime_seconds;
    let alerts = ctx.backend.take_alerts();

    state.iteration_count += 1;

    canvas.set_color(ctx.palette.header)?;
    canvas.set_cursor(0, 0)?;
    canvas.draw_str(&format!(
        "=== grainx System Monitor === Iteration: {} ===",
        state.iteration_count
    ))?;

    canvas.set_cursor(0, 2)?;
    canvas.set_color(ctx.palette.label)?;
    canvas.draw_str("CPU Usage:")?;

    let cpu_color = if cpu_usage > ctx.config.cpu_warning_threshold {
        ctx.palette.critical
    } else if cpu_usage > ctx.config.cpu_warning_threshold * 0.7 {
        ctx.palette.warning
    } else {
        ctx.palette.ok
    };
    canvas.set_color(cpu_color)?;
    canvas.set_cursor(12, 2)?;
    canvas.draw_str(&format!("{cpu_usage:6.2}%"))?;

    state.cpu_history.push(cpu_usage as f64);
    state.mem_history.push(memory_percentage);

    if state.cpu_history.len() > ctx.config.graph_history_size {
        state.cpu_history.remove(0);
        state.mem_history.remove(0);
    }

    ctx.anomaly_detector.add_point(cpu_usage as f64);

    let current_point = TimeSeriesPoint {
        timestamp: Utc::now(),
        value: cpu_usage as f64,
    };

    if let Some(anomaly) = ctx
        .anomaly_detector
        .detect_statistical_anomaly(&current_point)
        .await
    {
        canvas.set_cursor(0, 3)?;
        canvas.set_color(ctx.palette.critical)?;
        canvas.draw_str(&format!("! ANOMALY: {}", anomaly.message))?;
    }

    if !alerts.is_empty() {
        canvas.set_cursor(0, 4)?;
        canvas.set_color(ctx.palette.warning)?;
        canvas.draw_str(&format!("! {}", alerts.last().unwrap_or(&String::new())))?;
    }

    let target_cpu_y = (cpu_usage as f64 / 100.0) * ctx.layout.cpu_rect.height as f64;
    state.current_cpu_y_val = state.current_cpu_y_val * 0.8 + target_cpu_y * 0.2;

    state
        .cpu_points
        .push((state.iteration_count as f64, state.current_cpu_y_val));
    if state.cpu_points.len() > ctx.layout.cpu_rect.width as usize {
        state.cpu_points.remove(0);
        for p in state.cpu_points.iter_mut() {
            p.0 -= 1.0;
        }
    }

    for y in ctx.layout.cpu_rect.y..(ctx.layout.cpu_rect.y + ctx.layout.cpu_rect.height) {
        canvas.set_cursor(ctx.layout.cpu_rect.x, y)?;
        canvas.draw_str(&" ".repeat(ctx.layout.cpu_rect.width as usize))?;
    }

    canvas.set_color(cpu_color)?;
    canvas.draw_braille_line(&state.cpu_points, &ctx.layout.cpu_rect)?;

    canvas.set_cursor(0, ctx.layout.mem_rect.y)?;
    canvas.set_color(ctx.palette.label)?;
    canvas.draw_str(&format!(
        "Memory: {:.1}% ({:.1}GB/{:.1}GB)",
        memory_percentage,
        used_memory as f64 / 1_073_741_824.0,
        total_memory as f64 / 1_073_741_824.0
    ))?;

    let target_mem_y = (memory_percentage / 100.0) * ctx.layout.mem_rect.height as f64;
    state.current_mem_y_val = state.current_mem_y_val * 0.8 + target_mem_y * 0.2;

    state
        .mem_points
        .push((state.iteration_count as f64, state.current_mem_y_val));
    if state.mem_points.len() > ctx.layout.mem_rect.width as usize {
        state.mem_points.remove(0);
        for p in state.mem_points.iter_mut() {
            p.0 -= 1.0;
        }
    }

    for y in ctx.layout.mem_rect.y..(ctx.layout.mem_rect.y + ctx.layout.mem_rect.height) {
        canvas.set_cursor(ctx.layout.mem_rect.x, y)?;
        canvas.draw_str(&" ".repeat(ctx.layout.mem_rect.width as usize))?;
    }

    let mem_color = if memory_percentage > ctx.config.memory_warning_threshold as f64 {
        ctx.palette.critical
    } else if memory_percentage > ctx.config.memory_warning_threshold as f64 * 0.7 {
        ctx.palette.warning
    } else {
        ctx.palette.accent
    };
    canvas.set_color(mem_color)?;
    canvas.draw_braille_line(&state.mem_points, &ctx.layout.mem_rect)?;

    let rx_kbps = throughput_kbps(rx_bytes, state.last_rx_bytes);
    let tx_kbps = throughput_kbps(tx_bytes, state.last_tx_bytes);
    state.last_rx_bytes = rx_bytes;
    state.last_tx_bytes = tx_bytes;

    canvas.set_cursor(0, ctx.layout.net_rect.y.saturating_sub(1))?;
    canvas.set_color(ctx.palette.label)?;
    canvas.draw_str(&format!(
        "Network I/O: RX {rx_kbps:.1} KB/s  TX {tx_kbps:.1} KB/s"
    ))?;

    let max_net_kbps = (rx_kbps + tx_kbps).max(1.0);
    let target_net_rx_y = (rx_kbps / max_net_kbps) * ctx.layout.net_rect.height as f64;
    let target_net_tx_y = (tx_kbps / max_net_kbps) * ctx.layout.net_rect.height as f64;
    state.current_net_rx_y = state.current_net_rx_y * 0.8 + target_net_rx_y * 0.2;
    state.current_net_tx_y = state.current_net_tx_y * 0.8 + target_net_tx_y * 0.2;

    state
        .net_rx_points
        .push((state.iteration_count as f64, state.current_net_rx_y));
    state
        .net_tx_points
        .push((state.iteration_count as f64, state.current_net_tx_y));
    if state.net_rx_points.len() > ctx.layout.net_rect.width as usize {
        state.net_rx_points.remove(0);
        state.net_tx_points.remove(0);
        for p in state.net_rx_points.iter_mut() {
            p.0 -= 1.0;
        }
        for p in state.net_tx_points.iter_mut() {
            p.0 -= 1.0;
        }
    }

    for y in ctx.layout.net_rect.y..(ctx.layout.net_rect.y + ctx.layout.net_rect.height) {
        canvas.set_cursor(ctx.layout.net_rect.x, y)?;
        canvas.draw_str(&" ".repeat(ctx.layout.net_rect.width as usize))?;
    }

    canvas.set_color(ctx.palette.accent)?;
    canvas.draw_braille_line(&state.net_rx_points, &ctx.layout.net_rect)?;
    canvas.set_color(ctx.palette.warning)?;
    canvas.draw_braille_line(&state.net_tx_points, &ctx.layout.net_rect)?;

    canvas.set_cursor(0, ctx.layout.network_start_y)?;
    canvas.set_color(ctx.palette.header)?;
    canvas.draw_str(&format!(
        "System: {} | Kernel: {} | Uptime: {}h",
        os_name,
        kernel_version,
        uptime / 3600
    ))?;

    canvas.set_cursor(0, ctx.layout.network_start_y + 1)?;
    canvas.set_color(ctx.palette.accent)?;
    canvas.draw_str(&format!(
        "Network totals: RX:{:.1}MB TX:{:.1}MB",
        rx_bytes as f64 / 1_048_576.0,
        tx_bytes as f64 / 1_048_576.0
    ))?;

    canvas.set_cursor(0, ctx.layout.network_start_y + 2)?;
    canvas.set_color(ctx.palette.label)?;
    canvas.draw_str("CPU Cores: ")?;
    for (i, core_usage) in cpu_cores.iter().enumerate().take(8) {
        let core_color = if *core_usage > 80.0 {
            ctx.palette.critical
        } else if *core_usage > 50.0 {
            ctx.palette.warning
        } else {
            ctx.palette.ok
        };
        canvas.set_color(core_color)?;
        canvas.draw_str(&format!("C{i}:{core_usage:4.1}% "))?;
    }

    canvas.set_cursor(0, ctx.layout.network_start_y + 3)?;
    canvas.set_color(ctx.palette.label)?;
    canvas.draw_str("Disks: ")?;
    for (name, total, _available, used_pct) in disk_info.iter().take(3) {
        let disk_color = if *used_pct > 90.0 {
            ctx.palette.critical
        } else if *used_pct > 75.0 {
            ctx.palette.warning
        } else {
            ctx.palette.ok
        };
        canvas.set_color(disk_color)?;
        canvas.draw_str(&format!(
            "{}:{:.1}%({:.1}GB) ",
            name.chars().take(3).collect::<String>(),
            used_pct,
            *total as f64 / 1_073_741_824.0
        ))?;
    }

    if ctx.config.show_correlations || ctx.config.show_predictions {
        canvas.set_cursor(0, ctx.layout.network_start_y + 4)?;
        canvas.set_color(ctx.palette.warning)?;

        if ctx.config.show_correlations
            && state.iteration_count % 10 == 0
            && state.cpu_history.len() > 5
            && state.mem_history.len() > 5
            && let Some(correlation) = calculate_correlation(&state.cpu_history, &state.mem_history)
        {
            canvas.set_cursor(0, ctx.layout.network_start_y + 4)?;
            canvas.draw_str(&format!("Corr(CPU↔Mem): {correlation:.3}  "))?;
        }

        if ctx.config.show_predictions
            && let Some(predicted_cpu) = predict_next_value(&state.cpu_history, 5)
        {
            canvas.set_cursor(25, ctx.layout.network_start_y + 4)?;
            canvas.draw_str(&format!("Pred CPU: {predicted_cpu:.1}%  "))?;
        }

        let mut formula_metrics = HashMap::new();
        formula_metrics.insert("cpu_usage", cpu_usage as f64);
        if let Some(custom_value) =
            evaluate_metric_formula("cpu_usage * 1.5 + 5.0", &formula_metrics)
        {
            canvas.set_cursor(45, ctx.layout.network_start_y + 4)?;
            canvas.draw_str(&format!("Custom: {custom_value:.1}"))?;
        }
    }

    canvas.set_cursor(0, ctx.layout.proc_start_y)?;
    canvas.set_color(ctx.palette.label)?;
    canvas.draw_str("Top Processes (UP/DOWN to select, 'k' to kill, 'q' to quit):")?;

    for (i, (pid, name, cpu, memory)) in ctx
        .processes
        .iter()
        .take(ctx.config.max_processes)
        .enumerate()
    {
        let y_pos = ctx.layout.proc_start_y + 1 + i as u16;
        canvas.set_cursor(0, y_pos)?;

        if i == ctx.selected_process {
            canvas.set_color(ctx.palette.selection)?;
            canvas.draw_str(&format!(
                "> {:5} {:20} {:6.1}% {:8}KB",
                pid,
                name,
                cpu,
                memory / 1024
            ))?;
        } else {
            canvas.set_color(ctx.palette.label)?;
            canvas.draw_str(&format!(
                "  {:5} {:20} {:6.1}% {:8}KB",
                pid,
                name,
                cpu,
                memory / 1024
            ))?;
        }
    }

    let (fps, frame_time, adaptive) = ctx.perf_monitor.get_performance_stats();
    canvas.set_cursor(0, ctx.layout.footer_y - 1)?;
    canvas.set_color(ctx.palette.muted)?;
    canvas.draw_str(&format!(
        "Performance: {:.1}FPS | {:.1}ms | Adaptive: {}",
        fps,
        frame_time,
        if adaptive { "ON" } else { "OFF" }
    ))?;

    canvas.set_cursor(0, ctx.layout.footer_y)?;
    canvas.set_color(ctx.palette.muted)?;
    canvas.draw_str(&format!(
        "Config: {} | Controls: q=quit, h=help, p=pause, a=adaptive, s=export",
        ctx.config.name
    ))?;

    Ok(())
}
