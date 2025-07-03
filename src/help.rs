use crate::rendering::AdvancedCanvas;
use crossterm::style::Color;
use std::io;

pub fn show_help(canvas: &mut AdvancedCanvas) -> io::Result<()> {
    canvas.set_cursor(0, 0)?;
    canvas.set_color(Color::Cyan)?;
    canvas.draw_str("=== grainx Yardım Menüsü ===")?;
    
    canvas.set_cursor(0, 2)?;
    canvas.set_color(Color::White)?;
    canvas.draw_str("Klavye Kısayolları:")?;
    
    canvas.set_cursor(0, 4)?;
    canvas.set_color(Color::Yellow)?;
    canvas.draw_str("  q / ESC    - Programdan çık")?;
    
    canvas.set_cursor(0, 5)?;
    canvas.draw_str("  ↑ / ↓      - Process seçimi")?;
    
    canvas.set_cursor(0, 6)?;
    canvas.draw_str("  k          - Seçili process'i öldür")?;
    
    canvas.set_cursor(0, 7)?;
    canvas.draw_str("  r          - Yenile")?;
    
    canvas.set_cursor(0, 8)?;
    canvas.draw_str("  h / ?      - Bu yardım menüsü")?;
    
    canvas.set_cursor(0, 9)?;
    canvas.draw_str("  p          - Pause/Resume monitoring")?;
    
    canvas.set_cursor(0, 11)?;
    canvas.set_color(Color::Green)?;
    canvas.draw_str("Özellikler:")?;
    
    canvas.set_cursor(0, 12)?;
    canvas.set_color(Color::White)?;
    canvas.draw_str("  • Gerçek zamanlı CPU ve Memory grafikleri")?;
    
    canvas.set_cursor(0, 13)?;
    canvas.draw_str("  • Anomali tespiti ve uyarılar")?;
    
    canvas.set_cursor(0, 14)?;
    canvas.draw_str("  • CPU kullanım tahmini")?;
    
    canvas.set_cursor(0, 15)?;
    canvas.draw_str("  • Korelasyon analizi")?;
    
    canvas.set_cursor(0, 16)?;
    canvas.draw_str("  • Adaptif monitoring (yük bazlı)")?;
    
    canvas.set_cursor(0, 18)?;
    canvas.set_color(Color::Cyan)?;
    canvas.draw_str("Herhangi bir tuşa basarak devam edin...")?;
    
    Ok(())
}