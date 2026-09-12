//! Renderer-core regressions run in killable subprocesses so a non-terminating
//! rasterizer fails under a hard timeout without leaving a spinning thread.

use grainx::rendering::{AdvancedCanvas, DashboardLayout, Rect, braille_grid};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const CASE_ENV: &str = "GRAINX_BRAILLE_TEST_CASE";
const RENDER_TIMEOUT: Duration = Duration::from_secs(2);

fn count_dots(grid: &[Vec<bool>]) -> usize {
    grid.iter()
        .map(|column| column.iter().filter(|dot| **dot).count())
        .sum()
}

fn draw(points: &[(f64, f64)], width: u16, height: u16) {
    let mut canvas = AdvancedCanvas::new();
    let rect = Rect {
        x: 0,
        y: 0,
        width,
        height,
    };
    canvas
        .draw_braille_line(points, &rect)
        .expect("draw_braille_line returned an error");
}

fn run_case(case: &str) {
    let mut child = Command::new(std::env::current_exe().expect("test executable path"))
        .args(["--exact", "rasterizer_worker", "--nocapture"])
        .env(CASE_ENV, case)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn rasterizer worker");
    let deadline = Instant::now() + RENDER_TIMEOUT;

    loop {
        if let Some(status) = child.try_wait().expect("poll rasterizer worker") {
            if status.success() {
                return;
            }
            let output = child.wait_with_output().expect("read worker output");
            panic!(
                "{case}: rasterizer worker failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        if Instant::now() >= deadline {
            child.kill().expect("kill timed-out rasterizer worker");
            child.wait().expect("reap timed-out rasterizer worker");
            panic!("{case}: rasterizer did not finish within {RENDER_TIMEOUT:?}");
        }
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn rasterizer_worker() {
    let Ok(case) = std::env::var(CASE_ENV) else {
        return;
    };

    match case.as_str() {
        "empty" => {
            let grid = braille_grid(&[], 4, 2);
            assert_eq!((grid.len(), grid[0].len(), count_dots(&grid)), (8, 8, 0));
            draw(&[], 4, 2);
        }
        "single" => {
            let grid = braille_grid(&[(1.0, 1.0)], 4, 2);
            assert_eq!(count_dots(&grid), 1);
            assert!(grid[2][4]);

            let edge = braille_grid(&[(3.9, 1.9)], 4, 2);
            assert_eq!(count_dots(&edge), 1);
            assert!(edge[7][7], "edge-adjacent rounding must clamp in bounds");

            let boundary = braille_grid(&[(4.0, 2.0)], 4, 2);
            assert_eq!(count_dots(&boundary), 1);
            assert!(
                boundary[7][7],
                "inclusive right/bottom edge maps to last dot"
            );

            for point in [(-1.0, 0.0), (4.1, 1.0), (1.0, 2.1)] {
                assert_eq!(count_dots(&braille_grid(&[point], 4, 2)), 0);
            }
            draw(&[(3.9, 1.9)], 4, 2);
        }
        "lines" => {
            let horizontal = braille_grid(&[(0.0, 1.0), (4.0, 1.0)], 4, 2);
            assert_eq!(count_dots(&horizontal), 8);
            assert!(horizontal.iter().all(|column| column[4]));

            let vertical = braille_grid(&[(0.0, 0.0), (0.0, 2.0)], 4, 2);
            assert_eq!(count_dots(&vertical), 8);
            assert!(vertical[0].iter().all(|dot| *dot));

            let fractional = braille_grid(&[(1.0, 0.10), (2.0, 0.15)], 4, 2);
            assert!(fractional[2][0]);
            assert!(fractional[4][1]);

            let repeated = braille_grid(&[(1.0, 1.0), (1.0, 1.0)], 4, 2);
            assert_eq!(count_dots(&repeated), 1);
            assert!(repeated[2][4]);

            draw(&[(0.0, 1.0), (4.0, 1.0)], 4, 2);
            draw(&[(0.0, 0.0), (0.0, 2.0)], 4, 2);
            draw(&[(1.0, 0.10), (2.0, 0.15)], 4, 2);
            draw(&[(1.0, 1.0), (1.0, 1.0)], 4, 2);
        }
        "clipping" => {
            assert_eq!(
                count_dots(&braille_grid(&[(100.0, 100.0), (200.0, 200.0)], 4, 2)),
                0
            );
            assert_eq!(
                count_dots(&braille_grid(&[(-10.0, -10.0), (-5.0, -5.0)], 4, 2)),
                0
            );
            let crossing = braille_grid(&[(-4.0, -4.0), (8.0, 8.0)], 4, 2);
            assert!(crossing[0][0]);
            assert!(count_dots(&crossing) > 1);
            draw(&[(-4.0, -4.0), (8.0, 8.0)], 4, 2);
        }
        "non-finite" => {
            let cases = [
                ((f64::NAN, 1.0), (2.0, 1.0)),
                ((1.0, f64::INFINITY), (2.0, 1.0)),
                ((1.0, 1.0), (f64::NEG_INFINITY, 1.0)),
                ((f64::INFINITY, f64::INFINITY), (2.0, 2.0)),
            ];
            for (start, end) in cases {
                assert_eq!(count_dots(&braille_grid(&[start, end], 4, 2)), 0);
                draw(&[start, end], 4, 2);
            }
            assert_eq!(count_dots(&braille_grid(&[(f64::NAN, f64::NAN)], 4, 2)), 0);
        }
        "zero-size" => {
            assert!(braille_grid(&[(1.0, 0.1), (2.0, 0.15)], 0, 0).is_empty());
            draw(&[(1.0, 0.1), (2.0, 0.15)], 0, 2);
            draw(&[(1.0, 0.1), (2.0, 0.15)], 4, 0);
            draw(&[(1.0, 0.1), (2.0, 0.15)], 0, 0);
        }
        "huge" => {
            let large = braille_grid(&[(0.0, 0.0), (1.0e18, 1.0e18)], 80, 20);
            assert!(large[0][0]);
            assert!(count_dots(&large) > 1);

            let horizontal = braille_grid(&[(-f64::MAX, 1.0), (f64::MAX, 1.0)], 4, 2);
            assert_eq!(count_dots(&horizontal), 8);
            assert!(horizontal.iter().all(|column| column[4]));

            let diagonal = braille_grid(&[(-f64::MAX, -f64::MAX), (f64::MAX, f64::MAX)], 4, 2);
            assert!(diagonal[0][0]);
            assert!(diagonal[4][7]);
            assert!(count_dots(&diagonal) > 1);

            draw(&[(-f64::MAX, 1.0), (f64::MAX, 1.0)], 4, 2);
            draw(&[(-f64::MAX, -f64::MAX), (f64::MAX, f64::MAX)], 4, 2);
        }
        other => panic!("unknown rasterizer worker case: {other}"),
    }
}

#[test]
fn empty_points_are_a_bounded_noop() {
    run_case("empty");
}

#[test]
fn single_point_policy_and_edge_rounding_are_bounded() {
    run_case("single");
}

#[test]
fn integer_fractional_and_repeated_lines_are_bounded() {
    run_case("lines");
}

#[test]
fn outside_and_crossing_segments_are_bounded() {
    run_case("clipping");
}

#[test]
fn non_finite_segments_are_bounded_noops() {
    run_case("non-finite");
}

#[test]
fn zero_size_rectangles_are_bounded_noops() {
    run_case("zero-size");
}

#[test]
fn maximum_finite_crossings_are_bounded_and_complete() {
    run_case("huge");
}

#[test]
fn cpu_and_memory_labels_sit_outside_graph_rectangles() {
    for (width, height) in [(60, 20), (80, 24), (110, 50), (200, 80)] {
        let layout = DashboardLayout::from_terminal_size(width, height);
        let cpu_label_y = layout.cpu_label_y();
        let memory_label_y = layout.memory_label_y();
        assert!(cpu_label_y >= 1 && cpu_label_y < layout.cpu_rect.y);
        assert!(
            memory_label_y >= layout.cpu_rect.y + layout.cpu_rect.height
                && memory_label_y < layout.mem_rect.y
        );
        assert_ne!(cpu_label_y, memory_label_y);
    }
}
