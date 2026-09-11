use grainx::rendering::DashboardLayout;

/// The memory label must be drawn outside the memory graph rectangle, which the
/// clear pass and braille pass rewrite cell by cell, and below the CPU graph
/// clear pass. Otherwise the label is erased before the frame reaches the screen.
#[test]
fn memory_label_row_is_not_cleared_by_graph_passes() {
    for (width, height) in [(60, 20), (80, 24), (100, 32), (110, 50), (200, 80)] {
        let layout = DashboardLayout::from_terminal_size(width, height);
        let label_y = layout.memory_label_y();
        let mem_top = layout.mem_rect.y;
        let mem_bottom = mem_top + layout.mem_rect.height;
        let cpu_bottom = layout.cpu_rect.y + layout.cpu_rect.height;

        assert!(
            label_y < mem_top,
            "memory label row {label_y} overlaps memory graph rows {mem_top}..{mem_bottom} at {width}x{height}"
        );
        assert!(
            label_y >= cpu_bottom,
            "memory label row {label_y} is inside the CPU graph clear pass ending at {cpu_bottom} at {width}x{height}"
        );
    }
}
