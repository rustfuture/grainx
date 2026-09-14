use grainx::rendering::AdvancedCanvas;
use grainx::theme::ThemePalette;
use std::io;

pub fn show_help(canvas: &mut AdvancedCanvas, palette: &ThemePalette) -> io::Result<()> {
    canvas.set_cursor(0, 0)?;
    canvas.set_color(palette.header)?;
    canvas.draw_str("=== grainx Help ===")?;

    canvas.set_cursor(0, 2)?;
    canvas.set_color(palette.label)?;
    canvas.draw_str("Keyboard shortcuts:")?;

    canvas.set_cursor(0, 4)?;
    canvas.set_color(palette.warning)?;
    canvas.draw_str("  q / ESC    - Quit")?;

    canvas.set_cursor(0, 5)?;
    canvas.draw_str("  Up / Down  - Select a process")?;

    canvas.set_cursor(0, 6)?;
    canvas.draw_str("  k          - Request process termination")?;

    canvas.set_cursor(0, 7)?;
    canvas.draw_str("  r          - Refresh the view")?;

    canvas.set_cursor(0, 8)?;
    canvas.draw_str("  h / ?      - This help menu")?;

    canvas.set_cursor(0, 9)?;
    canvas.draw_str("  p          - Pause or resume")?;

    canvas.set_cursor(0, 10)?;
    canvas.draw_str("  a          - Toggle adaptive refresh")?;

    canvas.set_cursor(0, 11)?;
    canvas.draw_str("  s          - Save a JSON and CSV snapshot")?;

    canvas.set_cursor(0, 13)?;
    canvas.set_color(palette.ok)?;
    canvas.draw_str("Features:")?;

    canvas.set_cursor(0, 14)?;
    canvas.set_color(palette.label)?;
    canvas.draw_str("  • Real-time CPU and memory graphs")?;

    canvas.set_cursor(0, 15)?;
    canvas.draw_str("  • Anomaly detection and alerts")?;

    canvas.set_cursor(0, 16)?;
    canvas.draw_str("  • CPU usage prediction")?;

    canvas.set_cursor(0, 17)?;
    canvas.draw_str("  • Correlation analysis")?;

    canvas.set_cursor(0, 18)?;
    canvas.draw_str("  • Adaptive, load-aware monitoring")?;

    canvas.set_cursor(0, 20)?;
    canvas.set_color(palette.header)?;
    canvas.draw_str("Press any key to continue...")?;

    Ok(())
}
