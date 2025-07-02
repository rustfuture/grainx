# grainx

A cross-platform, terminal-native system and process monitoring tool.

## Features

*   **High-Resolution Visualization:** View CPU and memory usage graphs rendered with Unicode braille patterns for 8x higher resolution.
*   **Intelligent Analytics:**
    *   **Anomaly Detection:** Automatically detects statistical anomalies in CPU usage. The sensitivity of the anomaly detection can be configured.
    *   **Correlation Analysis:** Calculates the correlation between CPU usage and other metrics, helping to identify relationships between different system components.
    *   **Predictive Analysis:** Predicts future CPU usage based on historical data, allowing for proactive resource management.
*   **Adaptive Monitoring:** The monitoring interval adapts to the system load for efficient resource usage.
*   **Interactive UI:**
    *   Select processes using the up and down arrow keys.
    *   Kill a selected process by pressing the 'k' key (with a confirmation prompt).

## How to Use

1.  Clone the repository.
2.  Run `cargo run` in the `grainx` directory.
3.  Press 'q' to quit.
4.  Use the up and down arrow keys to select a process.
5.  Press 'k' to kill the selected process.
