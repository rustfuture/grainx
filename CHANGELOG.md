# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0] - 2025-07-02

### Added

- **High-Resolution Visualization:** Implemented CPU and memory usage graphs with Unicode braille patterns.
- **Intelligent Analytics:**
    - Statistical anomaly detection for CPU usage.
    - Correlation analysis between CPU and other metrics.
    - Predictive analysis for future CPU usage.
- **Adaptive Monitoring:** The monitoring interval adapts to the system load.
- **Interactive UI:**
    - Process selection using up and down arrow keys.
    - Process killing with the 'k' key (includes a confirmation prompt).
- **Configuration:** Load dashboard configuration from `dashboard_config.json`.

### Changed

- Refactored the main loop into smaller, more manageable functions (`handle_input` and `draw_dashboard`).

### Fixed

- N/A
