use std::io::{self, Write};
use crossterm::{cursor, execute, style::{Color, SetForegroundColor}};

pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

/// Dynamic dashboard layout computed from terminal dimensions
pub struct DashboardLayout {
    pub term_width: u16,
    #[allow(dead_code)]
    pub term_height: u16,
    pub cpu_rect: Rect,
    pub mem_rect: Rect,
    pub network_start_y: u16,
    pub proc_start_y: u16,
    pub footer_y: u16,
}

impl DashboardLayout {
    pub fn from_terminal_size(width: u16, height: u16) -> Self {
        let term_width = width.max(60);  // Minimum reasonable width
        let term_height = height.max(20); // Minimum reasonable height
        
        // Proportional layout: CPU graph takes ~35%, Memory ~15%, info ~15%, processes ~35%
        let cpu_h = ((term_height as f32 * 0.35) as u16).max(8);
        let mem_h = ((term_height as f32 * 0.15) as u16).max(4);
        let info_section_h: u16 = 4; // System info + network + cores + disk
        let _proc_h = term_height.saturating_sub(cpu_h + mem_h + info_section_h + 2).max(3);
        
        let cpu_rect = Rect { x: 0, y: 1, width: term_width, height: cpu_h };
        let mem_rect = Rect { x: 0, y: cpu_h + 2, width: term_width, height: mem_h };
        let network_start_y = cpu_h + mem_h + 3;
        let proc_start_y = network_start_y + info_section_h;
        let footer_y = term_height.saturating_sub(1);
        
        DashboardLayout {
            term_width,
            term_height,
            cpu_rect,
            mem_rect,
            network_start_y,
            proc_start_y,
            footer_y,
        }
    }
}

pub struct AdvancedCanvas {
    stdout: io::Stdout,
}

impl AdvancedCanvas {
    pub fn new() -> Self {
        AdvancedCanvas { stdout: io::stdout() }
    }

    pub fn set_cursor(&mut self, x: u16, y: u16) -> io::Result<()> {
        execute!(self.stdout, cursor::MoveTo(x, y))
    }

    pub fn set_color(&mut self, color: Color) -> io::Result<()> {
        execute!(self.stdout, SetForegroundColor(color))
    }

    pub fn draw_str(&mut self, s: &str) -> io::Result<()> {
        self.stdout.write_all(s.as_bytes())
    }

    /// Draw a braille-based graph line for high-resolution
    pub fn draw_braille_line(&mut self, points: &[(f64, f64)], rect: &Rect) -> io::Result<()> {
        let braille_width = rect.width * 2;
        let braille_height = rect.height * 4;
        
        let mut grid = vec![vec![false; braille_height as usize]; braille_width as usize];
        
        // Map each point to the grid
        for i in 0..points.len() - 1 {
            // Simple line drawing algorithm (Bresenham's or similar could be used)
            let (x1, y1) = points[i];
            let (x2, y2) = points[i+1];

            let dx = (x2 - x1).abs();
            let dy = (y2 - y1).abs();

            let sx = if x1 < x2 { 1.0 } else { -1.0 };
            let sy = if y1 < y2 { 1.0 } else { -1.0 };

            let mut err = dx - dy;

            let mut current_x = x1;
            let mut current_y = y1;

            loop {
                let gx = (current_x / rect.width as f64 * braille_width as f64).round() as usize;
                let gy = (current_y / rect.height as f64 * braille_height as f64).round() as usize;

                if gx < braille_width as usize && gy < braille_height as usize {
                    grid[gx][gy] = true;
                }

                if current_x == x2 && current_y == y2 {
                    break;
                }

                let e2 = 2.0 * err;
                if e2 > -dy {
                    err -= dy;
                    current_x += sx;
                }
                if e2 < dx {
                    err += dx;
                    current_y += sy;
                }
            }
        }
        
        for y in 0..rect.height {
            for x in 0..rect.width {
                let mut braille_value: u32 = 0;
                // Braille dots are 2x4 within a character cell
                // Mapping: 1 4
                //          2 5
                //          3 6
                //          7 8 (bottom-most dot)
                
                // Top-left dot (1)
                if grid[x as usize * 2][y as usize * 4] { braille_value |= 0x01; }
                // Middle-left dot (2)
                if grid[x as usize * 2][y as usize * 4 + 1] { braille_value |= 0x02; }
                // Bottom-left dot (3)
                if grid[x as usize * 2][y as usize * 4 + 2] { braille_value |= 0x04; }
                // Top-right dot (4)
                if grid[x as usize * 2 + 1][y as usize * 4] { braille_value |= 0x08; }
                // Middle-right dot (5)
                if grid[x as usize * 2 + 1][y as usize * 4 + 1] { braille_value |= 0x10; }
                // Bottom-right dot (6)
                if grid[x as usize * 2 + 1][y as usize * 4 + 2] { braille_value |= 0x20; }
                // Bottom-most dot (7) - this is the 7th dot in the braille character
                if grid[x as usize * 2][y as usize * 4 + 3] { braille_value |= 0x40; }
                // 8th dot (8) - this is the 8th dot in the braille character
                if grid[x as usize * 2 + 1][y as usize * 4 + 3] { braille_value |= 0x80; }

                let braille_char = std::char::from_u32(0x2800 + braille_value).unwrap_or('?');
                self.set_cursor(rect.x + x, rect.y + y)?;
                self.draw_str(&braille_char.to_string())?;
            }
        }
        
        Ok(())
    }
}