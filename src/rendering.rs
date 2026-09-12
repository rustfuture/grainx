use crossterm::{
    cursor, execute,
    style::{Color, SetForegroundColor},
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
    #[allow(dead_code)]
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
        let term_width = width.max(60); // Minimum reasonable width
        let term_height = height.max(20); // Minimum reasonable height

        // Proportional layout: CPU graph takes ~35%, Memory ~15%, info ~15%, processes ~35%
        let cpu_h = ((term_height as f32 * 0.35) as u16).max(8);
        let mem_h = ((term_height as f32 * 0.15) as u16).max(4);
        let net_h: u16 = 3;
        let info_section_h: u16 = 4;

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
/// number of braille dots per cell.
///
/// Every finite segment is clipped before its endpoints are quantized, so the
/// amount of work is bounded by the viewport no matter how large the inputs
/// are. Non-finite segments are skipped. A single finite point in the inclusive
/// logical viewport is rounded to the nearest dot, with the right and bottom
/// edges clamped to the final dot cell; points outside are skipped.
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
    let gy = (point.1 * 4.0).round() as usize;
    (gx.min(grid_width - 1), gy.min(grid_height - 1))
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

pub struct AdvancedCanvas {
    stdout: io::Stdout,
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
        }
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

    pub fn flush(&mut self) -> io::Result<()> {
        self.stdout.flush()
    }

    /// Draw a braille-based graph line for high-resolution
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
    fn layout_enforces_minimum_terminal_size() {
        let layout = DashboardLayout::from_terminal_size(20, 10);
        assert!(layout.term_width >= 60);
        assert!(layout.term_height >= 20);
        assert!(layout.cpu_rect.height >= 8);
        assert!(layout.mem_rect.height >= 4);
        assert!(layout.net_rect.height >= 3);
        assert!(layout.footer_y >= 19);
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
