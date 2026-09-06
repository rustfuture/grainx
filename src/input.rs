use crate::export::StatsSnapshot;
use crate::help::show_help;
use crate::metrics::MetricBackend;
use crate::performance::PerformanceMonitor;
use crate::rendering::{AdvancedCanvas, DashboardLayout};
use crate::theme::ThemePalette;
use crossterm::event::{self, Event, KeyCode, poll};
use std::io;
use std::time::Duration;

pub enum Action {
    Continue,
    Exit,
    Resize(u16, u16),
}

pub fn handle_input(
    selected_process: &mut usize,
    processes: &[(usize, String, f32, u64)],
    backend: &mut MetricBackend,
    canvas: &mut AdvancedCanvas,
    layout: &DashboardLayout,
    palette: &ThemePalette,
    perf_monitor: Option<&mut PerformanceMonitor>,
) -> io::Result<Action> {
    if poll(Duration::from_millis(50))? {
        match event::read()? {
            Event::Resize(width, height) => {
                return Ok(Action::Resize(width, height));
            }
            Event::Key(key_event) => match key_event.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    return Ok(Action::Exit);
                }
                KeyCode::Up if *selected_process > 0 => {
                    *selected_process -= 1;
                }
                KeyCode::Down if *selected_process < processes.len().saturating_sub(1) => {
                    *selected_process += 1;
                }
                KeyCode::Char('k')
                    if !processes.is_empty() && *selected_process < processes.len() =>
                {
                    let (pid, name, _, _) = &processes[*selected_process];

                    canvas.set_cursor(0, layout.proc_start_y + 10)?;
                    canvas.set_color(palette.critical)?;
                    canvas.draw_str(&format!("Kill process '{name}' (PID: {pid})? (y/N): "))?;

                    if let Event::Key(confirm_key) = event::read()?
                        && let KeyCode::Char('y') | KeyCode::Char('Y') = confirm_key.code
                    {
                        if backend.kill_process(*pid) {
                            canvas.set_cursor(0, layout.proc_start_y + 11)?;
                            canvas.set_color(palette.ok)?;
                            canvas.draw_str(&format!("Process {name} killed successfully!"))?;
                        } else {
                            canvas.set_cursor(0, layout.proc_start_y + 11)?;
                            canvas.set_color(palette.critical)?;
                            canvas.draw_str(&format!("Failed to kill process {name}"))?;
                        }
                    }

                    std::thread::sleep(Duration::from_secs(2));
                    canvas.set_cursor(0, layout.proc_start_y + 10)?;
                    canvas.draw_str(&" ".repeat(layout.term_width as usize))?;
                    canvas.set_cursor(0, layout.proc_start_y + 11)?;
                    canvas.draw_str(&" ".repeat(layout.term_width as usize))?;
                }
                KeyCode::Char('r') => {
                    canvas.set_cursor(0, 0)?;
                    canvas.set_color(palette.header)?;
                    canvas.draw_str("Refreshing...")?;
                }
                KeyCode::Char('h') | KeyCode::Char('?') => {
                    show_help(canvas, palette)?;
                    event::read()?;
                }
                KeyCode::Char('p') => {
                    canvas.set_cursor(0, 0)?;
                    canvas.set_color(palette.warning)?;
                    canvas.draw_str("PAUSED - Press any key to continue...")?;
                    event::read()?;
                }
                KeyCode::Char('s') => {
                    const JSON_PATH: &str = "grainx_stats.json";
                    const CSV_PATH: &str = "grainx_stats.csv";

                    canvas.set_cursor(0, 0)?;
                    match StatsSnapshot::save_both(JSON_PATH, CSV_PATH, backend) {
                        Ok(()) => {
                            canvas.set_color(palette.ok)?;
                            canvas
                                .draw_str(&format!("Stats saved to {JSON_PATH} and {CSV_PATH}"))?;
                        }
                        Err(err) => {
                            canvas.set_color(palette.critical)?;
                            canvas.draw_str(&format!("Failed to save stats: {err}"))?;
                        }
                    }
                }
                KeyCode::Char('a') => {
                    if let Some(perf) = perf_monitor {
                        perf.toggle_adaptive_refresh();
                        canvas.set_cursor(0, 0)?;
                        canvas.set_color(palette.header)?;
                        canvas.draw_str("Adaptive refresh toggled!")?;
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
    Ok(Action::Continue)
}
