use crate::monitor::SystemMonitor;
use crate::rendering::{AdvancedCanvas, DashboardLayout};
use crate::help::show_help;
use crate::performance::PerformanceMonitor;
use crossterm::{event::{self, Event, KeyCode, poll}, style::Color};
use std::io;
use std::time::Duration;

pub enum Action {
    Continue,
    Exit,
    Resize(u16, u16),
}

pub fn handle_input(
    selected_process: &mut usize, 
    processes: &Vec<(usize, String, f32, u64)>, 
    monitor: &mut SystemMonitor, 
    canvas: &mut AdvancedCanvas, 
    layout: &DashboardLayout,
    perf_monitor: Option<&mut PerformanceMonitor>
) -> io::Result<Action> {
    // Non-blocking input check
    if poll(Duration::from_millis(50))? {
        match event::read()? {
            Event::Resize(width, height) => {
                return Ok(Action::Resize(width, height));
            }
            Event::Key(key_event) => {
            match key_event.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    return Ok(Action::Exit);
                }
                KeyCode::Up => {
                    if *selected_process > 0 {
                        *selected_process -= 1;
                    }
                }
                KeyCode::Down => {
                    if *selected_process < processes.len().saturating_sub(1) {
                        *selected_process += 1;
                    }
                }
                KeyCode::Char('k') => {
                    // Kill selected process with confirmation
                    if !processes.is_empty() && *selected_process < processes.len() {
                        let (pid, name, _, _) = &processes[*selected_process];
                        
                        // Show confirmation dialog
                        canvas.set_cursor(0, layout.proc_start_y + 10)?;
                        canvas.set_color(Color::Red)?;
                        canvas.draw_str(&format!("Kill process '{}' (PID: {})? (y/N): ", name, pid))?;
                        
                        // Wait for confirmation
                        if let Event::Key(confirm_key) = event::read()? {
                            if let KeyCode::Char('y') | KeyCode::Char('Y') = confirm_key.code {
                                if monitor.kill_process(*pid) {
                                    canvas.set_cursor(0, layout.proc_start_y + 11)?;
                                    canvas.set_color(Color::Green)?;
                                    canvas.draw_str(&format!("Process {} killed successfully!", name))?;
                                } else {
                                    canvas.set_cursor(0, layout.proc_start_y + 11)?;
                                    canvas.set_color(Color::Red)?;
                                    canvas.draw_str(&format!("Failed to kill process {}", name))?;
                                }
                            }
                        }
                        
                        // Clear confirmation area after 2 seconds
                        std::thread::sleep(Duration::from_secs(2));
                        canvas.set_cursor(0, layout.proc_start_y + 10)?;
                        canvas.draw_str(&" ".repeat(layout.term_width as usize))?;
                        canvas.set_cursor(0, layout.proc_start_y + 11)?;
                        canvas.draw_str(&" ".repeat(layout.term_width as usize))?;
                    }
                }
                KeyCode::Char('r') => {
                    // Refresh/reset monitoring
                    canvas.set_cursor(0, 0)?;
                    canvas.set_color(Color::Cyan)?;
                    canvas.draw_str("Refreshing...")?;
                }
                KeyCode::Char('h') | KeyCode::Char('?') => {
                    // Show help menu
                    show_help(canvas)?;
                    // Wait for any key to continue
                    event::read()?;
                }
                KeyCode::Char('p') => {
                    // Pause/Resume functionality
                    canvas.set_cursor(0, 0)?;
                    canvas.set_color(Color::Yellow)?;
                    canvas.draw_str("PAUSED - Press any key to continue...")?;
                    event::read()?; // Wait for any key
                }
                KeyCode::Char('s') => {
                    // Save current stats to file
                    canvas.set_cursor(0, 0)?;
                    canvas.set_color(Color::Green)?;
                    canvas.draw_str("Stats saved to grainx_stats.txt")?;
                    // TODO: Implement actual saving
                }
                KeyCode::Char('a') => {
                    // Toggle adaptive refresh
                    if let Some(perf) = perf_monitor {
                        perf.toggle_adaptive_refresh();
                        canvas.set_cursor(0, 0)?;
                        canvas.set_color(Color::Cyan)?;
                        canvas.draw_str("Adaptive refresh toggled!")?;
                    }
                }
                _ => {}
            }
        }
            _ => {} // Ignore mouse, focus, paste events
        } // match event type
    }
    Ok(Action::Continue)
}
