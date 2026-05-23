use crate::analytics::{AnomalyDetector, AnomalyDetectorConfig};
use crate::cli::MonitorArgs;
use crate::config::{ConfigOverrides, DashboardConfig};
use crate::error::{GrainxError, Result};
use crate::input::{Action, handle_input};
use crate::logging::MetricLogger;
use crate::metrics::MetricBackend;
use crate::performance::PerformanceMonitor;
use crate::rendering::{AdvancedCanvas, DashboardLayout};
use crate::theme::palette_for;
use crate::ui::{DashboardState, DrawContext, draw_dashboard};
use crossterm::{cursor, execute, style::ResetColor, terminal};
use std::io::{self, IsTerminal};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{thread, time::Duration};

pub async fn run(args: MonitorArgs) -> Result<()> {
    if !io::stdout().is_terminal() {
        return Err(GrainxError::NoTty);
    }

    execute!(io::stdout(), terminal::EnterAlternateScreen, cursor::Hide)?;
    terminal::enable_raw_mode()?;

    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_flag = Arc::clone(&shutdown);
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            shutdown_flag.store(true, Ordering::SeqCst);
        }
    });

    let result = run_loop(args, shutdown).await;

    terminal::disable_raw_mode()?;
    execute!(
        io::stdout(),
        terminal::LeaveAlternateScreen,
        cursor::Show,
        ResetColor
    )?;

    result
}

async fn run_loop(args: MonitorArgs, shutdown: Arc<AtomicBool>) -> Result<()> {
    let (term_w, term_h) = terminal::size()?;
    let mut layout = DashboardLayout::from_terminal_size(term_w, term_h);

    let overrides = ConfigOverrides::from(&args);
    let dashboard_config = DashboardConfig::load_resolved(&args.config, &overrides)?;
    let theme_palette = palette_for(&dashboard_config.color_theme);

    let mut backend = match args.remote {
        Some(url) => {
            eprintln!("grainx monitor: remote mode ({url})");
            MetricBackend::remote(url)
        }
        None => MetricBackend::local(),
    };

    let mut canvas = AdvancedCanvas::new();
    let anomaly_detector = AnomalyDetector::new(AnomalyDetectorConfig {
        threshold_multiplier: 2.0,
    });
    let mut state = DashboardState::default();

    let mut selected_process = 0;
    let mut perf_monitor = PerformanceMonitor::new(60.0);
    let mut metric_logger = MetricLogger::from_config(&dashboard_config);

    while !shutdown.load(Ordering::SeqCst) {
        perf_monitor.start_frame();

        let processes = backend.get_processes()?;
        let current_cpu = backend.last_cpu_usage();

        if perf_monitor.should_skip_frame(current_cpu) {
            thread::sleep(Duration::from_millis(100));
            continue;
        }

        match handle_input(
            &mut selected_process,
            &processes,
            &mut backend,
            &mut canvas,
            &layout,
            &theme_palette,
            Some(&mut perf_monitor),
        )? {
            Action::Exit => break,
            Action::Resize(w, h) => {
                layout = DashboardLayout::from_terminal_size(w, h);
            }
            Action::Continue => {}
        }

        draw_dashboard(
            &mut canvas,
            &mut state,
            DrawContext {
                backend: &mut backend,
                processes: &processes,
                config: &dashboard_config,
                palette: &theme_palette,
                anomaly_detector: &anomaly_detector,
                layout: &layout,
                selected_process,
                perf_monitor: &mut perf_monitor,
            },
        )
        .await?;

        metric_logger.maybe_log(state.iteration_count, &mut backend)?;

        let frame_duration = perf_monitor.end_frame();
        let base_refresh = dashboard_config.refresh_interval_ms;
        let adaptive_refresh = perf_monitor.calculate_adaptive_refresh(current_cpu);
        let target_refresh = adaptive_refresh.max(base_refresh);
        let sleep_duration = target_refresh.saturating_sub(frame_duration.as_millis() as u64);
        thread::sleep(Duration::from_millis(sleep_duration));
    }

    Ok(())
}
