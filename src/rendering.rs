use crossterm::{
    cursor, queue,
    style::{Color, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use std::io::{self, Write};

pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

/// Dynamic dashboard layout computed from terminal dimensions
pub struct DashboardLayout {
    pub term_width: u16,
    pub term_height: u16,
    pub cpu_rect: Rect,
    pub mem_rect: Rect,
    pub net_rect: Rect,
    pub network_start_y: u16,
    pub proc_start_y: u16,
    pub footer_y: u16,
}

impl DashboardLayout {
    pub fn from_terminal_size(width: u16, height: u16) -> Self {
        let term_width = width;
        let term_height = height;

        // Reserve labels, five information rows, process rows and two footer rows.
        // Small terminals lose graph/process rows instead of drawing off-screen.
        let desired_cpu = ((height as u32 * 35 / 100) as u16).max(8);
        let desired_mem = ((height as u32 * 15 / 100) as u16).max(4);
        let graph_rows = (desired_cpu + desired_mem + 3).min(height.saturating_sub(18));
        let net_h = (graph_rows / 4).min(3);
        let remaining = graph_rows - net_h;
        let cpu_h = (remaining as u32 * desired_cpu as u32
            / (desired_cpu as u32 + desired_mem as u32)) as u16;
        let mem_h = remaining - cpu_h;
        let info_section_h: u16 = 5;

        let cpu_rect = Rect {
            x: 0,
            // Row 0 holds the header, row 1 the CPU label; the graph's clear
            // and braille passes start below both.
            y: 2,
            width: term_width,
            height: cpu_h,
        };
        let mem_rect = Rect {
            x: 0,
            y: cpu_rect.y + cpu_h + 1,
            width: term_width,
            height: mem_h,
        };
        let net_rect = Rect {
            x: 0,
            y: mem_rect.y + mem_rect.height + 1,
            width: term_width,
            height: net_h,
        };
        let network_start_y = net_rect.y + net_rect.height + 1;
        let proc_start_y = network_start_y + info_section_h;
        let footer_y = term_height.saturating_sub(1);

        DashboardLayout {
            term_width,
            term_height,
            cpu_rect,
            mem_rect,
            net_rect,
            network_start_y,
            proc_start_y,
            footer_y,
        }
    }

    /// Number of process entries that fit before the performance/footer rows.
    pub fn process_rows(&self) -> usize {
        self.footer_y
            .saturating_sub(self.proc_start_y.saturating_add(2)) as usize
    }

    /// Row for the CPU label, kept just above `cpu_rect` so the CPU graph's
    /// clear and braille passes can never overwrite it.
    pub fn cpu_label_y(&self) -> u16 {
        self.cpu_rect.y.saturating_sub(1)
    }

    /// Row for the memory label, kept just above `mem_rect` so the memory
    /// graph's clear and braille passes can never overwrite it.
    pub fn memory_label_y(&self) -> u16 {
        self.mem_rect.y.saturating_sub(1)
    }
}

/// Rasterize a polyline into a braille dot grid.
///
/// `points` use the same coordinates as [`AdvancedCanvas::draw_braille_line`]:
/// x and y are scaled to `cell_width`/`cell_height` terminal cells. The result
/// is indexed `[gx][gy]` and sized `cell_width * 2` by `cell_height * 4`, the
/// number of braille dots per cell. Logical y = 0 is the bottom row and
/// y = `cell_height` is the top row.
///
/// Every finite segment is clipped before its endpoints are quantized, so the
/// amount of work is bounded by the viewport no matter how large the inputs
/// are. Non-finite segments are skipped. A single finite point in the inclusive
/// logical viewport is rounded to the nearest dot and clamped to the edge
/// dot cells; points outside are skipped.
pub fn braille_grid(points: &[(f64, f64)], cell_width: u16, cell_height: u16) -> Vec<Vec<bool>> {
    let grid_width = cell_width as usize * 2;
    let grid_height = cell_height as usize * 4;
    let mut grid = vec![vec![false; grid_height]; grid_width];
    if grid_width == 0 || grid_height == 0 || points.is_empty() {
        return grid;
    }

    if points.len() == 1 {
        if let Some((gx, gy)) =
            point_cell(points[0], cell_width, cell_height, grid_width, grid_height)
        {
            grid[gx][gy] = true;
        }
        return grid;
    }

    for pair in points.windows(2) {
        let (x1, y1) = pair[0];
        let (x2, y2) = pair[1];
        if let Some(((gx1, gy1), (gx2, gy2))) = clip_to_grid(
            x1,
            y1,
            x2,
            y2,
            cell_width,
            cell_height,
            grid_width,
            grid_height,
        ) {
            plot_segment(&mut grid, gx1, gy1, gx2, gy2);
        }
    }
    grid
}

fn point_cell(
    point: (f64, f64),
    cell_width: u16,
    cell_height: u16,
    grid_width: usize,
    grid_height: usize,
) -> Option<(usize, usize)> {
    if !point.0.is_finite() || !point.1.is_finite() {
        return None;
    }
    if point.0 < 0.0 || point.1 < 0.0 || point.0 > cell_width as f64 || point.1 > cell_height as f64
    {
        return None;
    }
    Some(quantize_point(point, grid_width, grid_height))
}

fn quantize_point(point: (f64, f64), grid_width: usize, grid_height: usize) -> (usize, usize) {
    let gx = (point.0 * 2.0).round() as usize;
    // Logical values rise from zero; terminal grid rows run from top to bottom.
    let gy = grid_height - 1 - ((point.1 * 4.0).round() as usize).min(grid_height - 1);
    (gx.min(grid_width - 1), gy)
}

/// Clip a segment to the logical canvas and return quantized endpoint cells.
///
/// Boundary intersections are computed from half-differences rather than
/// `end - start`, which remains finite even for `-f64::MAX..f64::MAX`.
#[allow(clippy::too_many_arguments)]
fn clip_to_grid(
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    cell_width: u16,
    cell_height: u16,
    grid_width: usize,
    grid_height: usize,
) -> Option<((usize, usize), (usize, usize))> {
    if !x1.is_finite() || !y1.is_finite() || !x2.is_finite() || !y2.is_finite() {
        return None;
    }

    let max_x = cell_width as f64;
    let max_y = cell_height as f64;
    let mut cells = Vec::with_capacity(6);

    if inside(x1, y1, max_x, max_y) {
        push_unique_cell(
            &mut cells,
            quantize_point((x1, y1), grid_width, grid_height),
        );
    }
    if inside(x2, y2, max_x, max_y) {
        push_unique_cell(
            &mut cells,
            quantize_point((x2, y2), grid_width, grid_height),
        );
    }

    if x1 != x2 {
        for boundary_x in [0.0, max_x] {
            if between(boundary_x, x1, x2) {
                let y = interpolate_at(x1, x2, boundary_x, y1, y2);
                if y.is_finite() && (0.0..=max_y).contains(&y) {
                    push_unique_cell(
                        &mut cells,
                        quantize_point((boundary_x, y), grid_width, grid_height),
                    );
                }
            }
        }
    }
    if y1 != y2 {
        for boundary_y in [0.0, max_y] {
            if between(boundary_y, y1, y2) {
                let x = interpolate_at(y1, y2, boundary_y, x1, x2);
                if x.is_finite() && (0.0..=max_x).contains(&x) {
                    push_unique_cell(
                        &mut cells,
                        quantize_point((x, boundary_y), grid_width, grid_height),
                    );
                }
            }
        }
    }

    match cells.as_slice() {
        [] => None,
        [cell] => Some((*cell, *cell)),
        _ => {
            let mut endpoints = (cells[0], cells[1]);
            let mut greatest_distance = cell_distance_squared(cells[0], cells[1]);
            for (index, &first) in cells.iter().enumerate() {
                for &second in &cells[index + 1..] {
                    let distance = cell_distance_squared(first, second);
                    if distance > greatest_distance {
                        endpoints = (first, second);
                        greatest_distance = distance;
                    }
                }
            }
            Some(endpoints)
        }
    }
}

fn inside(x: f64, y: f64, max_x: f64, max_y: f64) -> bool {
    (0.0..=max_x).contains(&x) && (0.0..=max_y).contains(&y)
}

fn between(value: f64, first: f64, second: f64) -> bool {
    value >= first.min(second) && value <= first.max(second)
}

fn interpolate_at(
    axis_start: f64,
    axis_end: f64,
    axis_value: f64,
    other_start: f64,
    other_end: f64,
) -> f64 {
    let axis_midpoint = axis_start / 2.0 + axis_end / 2.0;
    let axis_half_delta = axis_end / 2.0 - axis_start / 2.0;
    let other_midpoint = other_start / 2.0 + other_end / 2.0;
    let other_half_delta = other_end / 2.0 - other_start / 2.0;

    if axis_half_delta == 0.0 {
        let ratio = (axis_value - axis_start) / (axis_end - axis_start);
        other_start + ratio * (other_end - other_start)
    } else {
        let centered_parameter = (axis_value - axis_midpoint) / axis_half_delta;
        other_midpoint + centered_parameter * other_half_delta
    }
}

fn push_unique_cell(cells: &mut Vec<(usize, usize)>, cell: (usize, usize)) {
    if !cells.contains(&cell) {
        cells.push(cell);
    }
}

fn cell_distance_squared(first: (usize, usize), second: (usize, usize)) -> usize {
    let dx = first.0.abs_diff(second.0);
    let dy = first.1.abs_diff(second.1);
    dx * dx + dy * dy
}

/// Integer Bresenham walk between two cells that are already inside the grid.
fn plot_segment(grid: &mut [Vec<bool>], x1: usize, y1: usize, x2: usize, y2: usize) {
    let end_x = x2 as isize;
    let end_y = y2 as isize;
    let mut x = x1 as isize;
    let mut y = y1 as isize;
    let dx = (end_x - x).abs();
    let dy = (end_y - y).abs();
    let sx = if x < end_x { 1 } else { -1 };
    let sy = if y < end_y { 1 } else { -1 };
    let mut err = dx - dy;

    loop {
        grid[x as usize][y as usize] = true;
        if x == end_x && y == end_y {
            return;
        }
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}

/// Clip printable text to a row; control characters cannot move the cursor.
fn clipped_text(text: &str, columns: u16) -> String {
    text.chars()
        .filter(|c| !c.is_control())
        .take(columns as usize)
        .collect()
}

pub struct AdvancedCanvas {
    stdout: io::Stdout,
    output: Vec<u8>,
    buffering: bool,
    bounds: Option<(u16, u16)>,
    position: (u16, u16),
}

impl Default for AdvancedCanvas {
    fn default() -> Self {
        Self::new()
    }
}

impl AdvancedCanvas {
    pub fn new() -> Self {
        AdvancedCanvas {
            stdout: io::stdout(),
            output: Vec::new(),
            buffering: false,
            bounds: None,
            position: (0, 0),
        }
    }

    /// Build a complete replacement frame before writing it to the terminal.
    pub fn begin_frame(&mut self, width: u16, height: u16) -> io::Result<()> {
        self.output.clear();
        self.buffering = true;
        self.bounds = Some((width, height));
        self.position = (0, 0);
        queue!(self.output, ResetColor, Clear(ClearType::All))
    }

    pub fn set_draw_height(&mut self, height: u16) {
        if let Some((_, bound_height)) = &mut self.bounds {
            *bound_height = height;
        }
    }

    pub fn set_cursor(&mut self, x: u16, y: u16) -> io::Result<()> {
        self.position = (x, y);
        if self.bounds.is_none_or(|(w, h)| x < w && y < h) {
            queue!(self.output, cursor::MoveTo(x, y))?;
        }
        self.flush_if_unbuffered()
    }

    pub fn set_color(&mut self, color: Color) -> io::Result<()> {
        queue!(self.output, SetForegroundColor(color))?;
        self.flush_if_unbuffered()
    }

    pub fn draw_str(&mut self, s: &str) -> io::Result<()> {
        if let Some((width, height)) = self.bounds {
            let (x, y) = self.position;
            if x >= width || y >= height {
                return Ok(());
            }
            let text = clipped_text(s, width - x);
            self.position.0 += text.chars().count() as u16;
            self.output.write_all(text.as_bytes())?;
        } else {
            self.output.write_all(s.as_bytes())?;
        }
        self.flush_if_unbuffered()
    }

    fn flush_if_unbuffered(&mut self) -> io::Result<()> {
        if !self.buffering {
            self.flush()?;
        }
        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.stdout.write_all(&self.output)?;
        self.stdout.flush()?;
        self.output.clear();
        self.buffering = false;
        Ok(())
    }

    /// Draw occupied braille cells, preserving other lines in the same graph.
    /// The caller clears the frame before drawing its graph layers.
    pub fn draw_braille_line(&mut self, points: &[(f64, f64)], rect: &Rect) -> io::Result<()> {
        let grid = braille_grid(points, rect.width, rect.height);

        for y in 0..rect.height {
            for x in 0..rect.width {
                let mut braille_value: u32 = 0;
                // Braille dots are 2x4 within a character cell
                // Mapping: 1 4
                //          2 5
                //          3 6
                //          7 8 (bottom-most dot)

                // Top-left dot (1)
                if grid[x as usize * 2][y as usize * 4] {
                    braille_value |= 0x01;
                }
                // Middle-left dot (2)
                if grid[x as usize * 2][y as usize * 4 + 1] {
                    braille_value |= 0x02;
                }
                // Bottom-left dot (3)
                if grid[x as usize * 2][y as usize * 4 + 2] {
                    braille_value |= 0x04;
                }
                // Top-right dot (4)
                if grid[x as usize * 2 + 1][y as usize * 4] {
                    braille_value |= 0x08;
                }
                // Middle-right dot (5)
                if grid[x as usize * 2 + 1][y as usize * 4 + 1] {
                    braille_value |= 0x10;
                }
                // Bottom-right dot (6)
                if grid[x as usize * 2 + 1][y as usize * 4 + 2] {
                    braille_value |= 0x20;
                }
                // Bottom-most dot (7) - this is the 7th dot in the braille character
                if grid[x as usize * 2][y as usize * 4 + 3] {
                    braille_value |= 0x40;
                }
                // 8th dot (8) - this is the 8th dot in the braille character
                if grid[x as usize * 2 + 1][y as usize * 4 + 3] {
                    braille_value |= 0x80;
                }

                if braille_value == 0 {
                    continue;
                }
                let braille_char = std::char::from_u32(0x2800 + braille_value).unwrap_or('?');
                self.set_cursor(rect.x + x, rect.y + y)?;
                self.draw_str(&braille_char.to_string())?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graph_values_rise_from_bottom_to_top() {
        let low = braille_grid(&[(1.0, 0.08)], 4, 2);
        let high = braille_grid(&[(1.0, 1.92)], 4, 2);
        assert!(low[2][7], "low values belong in the bottom grid row");
        assert!(high[2][0], "high values belong in the top grid row");
        assert_eq!(quantize_point((1.0, 0.0), 8, 8), (2, 7));
        assert_eq!(quantize_point((1.0, 2.0), 8, 8), (2, 0));
    }

    #[test]
    fn layout_uses_actual_terminal_size() {
        let layout = DashboardLayout::from_terminal_size(20, 10);
        assert_eq!(layout.term_width, 20);
        assert_eq!(layout.term_height, 10);
        assert_eq!(layout.footer_y, 9);
    }

    #[test]
    fn rows_are_clipped_and_control_characters_cannot_scroll() {
        assert_eq!(clipped_text("182880KB%        0KB", 8), "182880KB");
        assert_eq!(clipped_text("abc\n\r\tdef", 4), "abcd");
        assert_eq!(clipped_text("⠿↔x", 2), "⠿↔");
        assert_eq!(clipped_text("text", 0), "");
    }

    #[test]
    fn frame_starts_by_erasing_previous_rows() {
        let mut canvas = AdvancedCanvas::new();
        canvas.begin_frame(20, 10).unwrap();
        let clear = canvas.output.clone();
        canvas.set_cursor(0, 2).unwrap();
        canvas.draw_str("long process row").unwrap();
        canvas.begin_frame(20, 10).unwrap();
        assert_eq!(canvas.output, clear);
        assert!(String::from_utf8(clear).unwrap().contains("\x1b[2J"));
        let before = canvas.output.clone();
        canvas.set_cursor(0, 10).unwrap();
        canvas.draw_str("off-screen").unwrap();
        assert_eq!(canvas.output, before);
    }

    #[test]
    fn empty_graph_cells_do_not_erase_other_graph_layers() {
        let mut canvas = AdvancedCanvas::new();
        canvas.begin_frame(4, 2).unwrap();
        let rect = Rect {
            x: 0,
            y: 0,
            width: 4,
            height: 2,
        };
        canvas
            .draw_braille_line(&[(0.0, 0.0), (4.0, 0.0)], &rect)
            .unwrap();
        let first_layer = canvas.output.clone();
        canvas.draw_braille_line(&[], &rect).unwrap();
        assert_eq!(canvas.output, first_layer);
        assert!(!String::from_utf8(first_layer).unwrap().contains('⠀'));
    }

    #[test]
    fn processes_and_information_do_not_overlap_the_footer() {
        for height in [18, 20, 24, 32, 40, 60, 100] {
            let layout = DashboardLayout::from_terminal_size(80, height);
            assert!(layout.network_start_y + 4 < layout.proc_start_y);
            assert!(layout.proc_start_y < layout.footer_y - 1);
            for index in 0..layout.process_rows() {
                assert!(layout.proc_start_y + 1 + (index as u16) < layout.footer_y - 1);
            }
        }
        assert_eq!(DashboardLayout::from_terminal_size(20, 5).process_rows(), 0);
    }

    #[test]
    fn layout_scales_with_large_terminal() {
        let layout = DashboardLayout::from_terminal_size(120, 40);
        assert_eq!(layout.term_width, 120);
        assert_eq!(layout.cpu_rect.width, 120);
        assert!(layout.proc_start_y > layout.network_start_y);
        assert!(layout.footer_y <= 39);
    }

    #[test]
    fn layout_network_section_below_graphs() {
        let layout = DashboardLayout::from_terminal_size(80, 24);
        assert!(layout.network_start_y > layout.net_rect.y);
        assert!(layout.proc_start_y > layout.network_start_y);
    }

    #[test]
    fn cpu_label_row_is_between_header_and_graph() {
        for (width, height) in [(60, 20), (80, 24), (110, 50), (200, 80)] {
            let layout = DashboardLayout::from_terminal_size(width, height);
            let label_y = layout.cpu_label_y();
            assert!(
                label_y >= 1,
                "CPU label row {label_y} collides with the header at {width}x{height}"
            );
            assert!(
                label_y < layout.cpu_rect.y,
                "CPU label row {label_y} overlaps the CPU graph at {width}x{height}"
            );
            assert!(
                label_y < layout.memory_label_y(),
                "CPU and memory label rows collide at {width}x{height}"
            );
        }
    }
}
