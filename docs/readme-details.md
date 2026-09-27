## What it does

- Interactive terminal dashboard with Unicode/Braille CPU and memory charts, process inspection, and process termination.
- Local host resource collection via `sysinfo`: CPU, memory, disk usage, per-interval network I/O, processes, and host metadata.
- Adaptive refresh rate scaling and frame skipping under heavy system CPU load.
- Local HTTP metrics service (`GET /health` and `GET /metrics`) restricted to loopback interfaces.
- Local and remote JSON and CSV metric snapshot export without starting the TUI.

The term *agent* in this repository refers to the HTTP metrics daemon process, not an AI or LLM agent.

## Additional quick-start notes

- Stable Rust 1.88 or newer. The crate uses Rust edition 2024. This minimum is checked against the committed dependency lockfile.
- An interactive terminal is required for the monitor command.
- To build an optimized binary, run `cargo build --locked --release`.
- In a headless environment, use the agent or export command instead.

## Architecture details

- **Monitor (`SystemMonitor`)**: Gathers raw state using `sysinfo`.
- **Analytics Engine (`analytics::*`)**: Performs formula evaluation and anomaly detection on the collected metrics. `evaluate_metric_formula` substitutes whitespace-separated tokens and evaluates strictly left-to-right with no operator precedence; see its doc comment for the full contract.
- **Agent Server (`agent::*`)**: Exposes collected metrics via an HTTP server using `axum`. Binds to `127.0.0.1:9090` by default and refuses any non-loopback address, because it serves without TLS or authentication.
- **TUI Renderer (`ui::*`, `tui::*`)**: Renders the terminal dashboard using `crossterm`. Employs frame-skipping and adaptive refresh intervals to gracefully degrade under heavy load.

## Limitations details

- **Security**: The HTTP agent has no TLS, authentication, or rate limiting. It refuses any non-loopback bind address at runtime, and `src/agent.rs` tests that `0.0.0.0`, `::`, and ordinary interface addresses are rejected. Loopback does not isolate users or processes on the same host; remote metrics access requires placing an authenticated transport in front of the agent.
- **Completeness**: Network and disk I/O are aggregates and do not drill down into per-socket or per-file statistics.
- **Formula Evaluator**: `evaluate_metric_formula` evaluates whitespace-separated tokens strictly left-to-right with no operator precedence or parentheses ([src/analytics.rs](../src/analytics.rs)).
- **Interactive TUI**: The `monitor` command requires an interactive terminal (TTY) and exits with an error in headless environments ([src/tui.rs](../src/tui.rs)).
- **OS Support**: CI tests Linux and macOS on stable Rust. Windows is not verified and is not covered by the CI matrix.

## Commands

~~~bash
# Interactive dashboard
cargo run --locked -- monitor

# Local HTTP metrics service; the bind address must be loopback
cargo run --locked -- agent --bind 127.0.0.1 --port 9090

# Export one snapshot without starting the TUI
cargo run --locked -- export

# Export from a running remote agent
cargo run --locked -- export --remote http://127.0.0.1:9090

# Read metrics in the terminal
curl http://127.0.0.1:9090/health
curl http://127.0.0.1:9090/metrics

# Connect the TUI to a remote agent
cargo run --locked -- monitor --remote http://127.0.0.1:9090

# Generate shell completions
cargo run --locked -- completions bash
~~~

The agent exposes host metrics without authentication, TLS, or rate limiting, so it refuses to start on any address that is not loopback: `--bind 0.0.0.0` and a LAN address exit with an error instead of listening. That boundary is enforced in the program, not only in this document. Loopback is still not a complete protection — it does not separate users or processes on the same machine, and anything that can reach localhost can read the metrics. Remote metrics access is out of scope for this version; if you need it, put an authenticated transport in front of the agent.

## Configuration

The monitor reads dashboard_config.json. If the file is missing, grainx creates a default configuration. The precedence order is:

1. dashboard_config.json
2. Environment variables
3. Monitor CLI flags

Supported environment overrides include GRAINX_REFRESH_INTERVAL_MS, GRAINX_CPU_WARNING_THRESHOLD, GRAINX_MEMORY_WARNING_THRESHOLD, and GRAINX_COLOR_THEME. See [dashboard_config.json](../dashboard_config.json) and [src/config.rs](../src/config.rs) for the current schema.

## Controls

| Key | Action |
| --- | --- |
| q or Esc | Quit |
| Up / Down | Select a process |
| p | Pause or resume |
| k | Request process termination |
| r | Refresh the view |
| a | Toggle adaptive refresh |
| s | Save a snapshot |

Process termination is subject to the operating system permissions of the user running grainx.


## Metrics Semantics

- **CPU Usage**: System-wide CPU utilization is calculated as the average utilization across all logical cores since the last refresh ([src/monitor.rs](../src/monitor.rs)).
- **Memory Usage**: Physical memory utilization reported by the operating system, excluding swap ([src/monitor.rs](../src/monitor.rs)).
- **Network I/O**: `network_rx_bytes` and `network_tx_bytes` are the bytes received and transmitted during the most recent sampling interval (a per-interval delta reported by `sysinfo`, not a cumulative counter or rate). The TUI displays this as `Network I/O: RX <n> KB  TX <n> KB (last interval)` ([src/ui.rs](../src/ui.rs), [src/network.rs](../src/network.rs)).
- **Sampling Interval**: The default refresh interval is 500ms, configured in [dashboard_config.json](../dashboard_config.json) and [src/config.rs](../src/config.rs) (overrideable via `GRAINX_REFRESH_INTERVAL_MS` or `--refresh-interval-ms`).
- **Adaptive Refresh & Frame Skipping**: When enabled, the monitor scales target refresh dynamically based on rolling average CPU load: 250ms (<=50%), 500ms (>50%), 1000ms (>70%), and 2000ms (>90%), taking `adaptive_refresh.max(base_refresh)` ([src/performance.rs](../src/performance.rs), [src/tui.rs](../src/tui.rs)). If CPU load exceeds 95%, the dashboard skips rendering frames ([src/performance.rs](../src/performance.rs)).
- **Warning Thresholds**: CPU warning threshold defaults to 80.0% and memory warning threshold defaults to 85.0% ([src/config.rs](../src/config.rs)), driving alert highlights in the TUI ([src/ui.rs](../src/ui.rs)) and high-load alerts ([src/monitor.rs](../src/monitor.rs)).


## Demos

See [demos/](../demos/) for verified captures with build identity, artifact hashes, a privacy review, and
reproduction commands:

- `tui_capture_2026-09-12-contract-80x24.*` and `-110x50.*` — the enforced bounded-capture contract at
  two terminal sizes, showing the current `Network I/O: ... (last interval)` label.
- `http_capture_2026-09-11.txt` — real `GET /health` and `GET /metrics` output plus a local `export`
  run from the built binary.
- `verification_2026-09-11-remote-export.md` — CLI-boundary verification of the remote-export fix:
  exit 0 against a live `grainx agent`, and controlled non-zero errors for unreachable, malformed, and
  `http://127.0.0.1:0` endpoints with no panic text.
- `verification_2026-09-12-braille-termination.md` — PTY proof that braille rendering and frame
  completion are bounded.

The older 2026-09-06 captures are kept and marked historical.

~~~bash
cargo build --locked
python3 -m venv /tmp/grainx-render-venv && /tmp/grainx-render-venv/bin/pip install pyte==0.8.2
/tmp/grainx-render-venv/bin/python demos/capture_tui.py target/debug/grainx /tmp/tui.raw /tmp/tui.txt 5 110 50
~~~

See [demos/README.md](demos/README.md) and [demos/verification_2026-09-11.md](../demos/verification_2026-09-11.md)
for the full commands, hashes, privacy review, and limitations.

## Performance and Microbenchmarks

Microbenchmarks run via `cargo bench` (Criterion) measure isolated components on the test host; they are not a universal performance claim. A recorded run with its machine, OS, toolchain, command, and base commit is kept in [benches/results/2026-09-10-macos-arm64.txt](../benches/results/2026-09-10-macos-arm64.txt). To reproduce:

~~~bash
cargo bench --locked
~~~

Scope notes:

- `system_monitor_refresh` constructs a fresh `SystemMonitor` and performs a single `refresh()` per iteration. It measures cold construction plus one refresh, and does **not** measure the CPU or memory cost of a long-running monitor loop.
- The formula, prediction, and correlation benchmarks measure the analytical functions only. Those functions do allocate (metric substitution and token splitting build `String`/`Vec` values); they are not zero-allocation.


## Architecture

- **Monitor (`SystemMonitor`)**: Gathers raw state using `sysinfo`.
- **Analytics Engine (`analytics::*`)**: Performs formula evaluation and anomaly detection on the collected metrics. `evaluate_metric_formula` substitutes whitespace-separated tokens and evaluates strictly left-to-right with no operator precedence; see its doc comment for the full contract.
- **Agent Server (`agent::*`)**: Exposes collected metrics via an HTTP server using `axum`. Binds to `127.0.0.1:9090` by default and refuses any non-loopback address, because it serves without TLS or authentication.
- **TUI Renderer (`ui::*`, `tui::*`)**: Renders the terminal dashboard using `crossterm`. Employs frame-skipping and adaptive refresh intervals to gracefully degrade under heavy load.


## Limitations

- **Security**: The HTTP agent has no TLS, authentication, or rate limiting. It refuses any non-loopback bind address at runtime, and `src/agent.rs` tests that `0.0.0.0`, `::`, and ordinary interface addresses are rejected. Loopback does not isolate users or processes on the same host; remote metrics access requires placing an authenticated transport in front of the agent.
- **Completeness**: Network and disk I/O are aggregates and do not drill down into per-socket or per-file statistics.
- **Formula Evaluator**: `evaluate_metric_formula` evaluates whitespace-separated tokens strictly left-to-right with no operator precedence or parentheses ([src/analytics.rs](../src/analytics.rs)).
- **Interactive TUI**: The `monitor` command requires an interactive terminal (TTY) and exits with an error in headless environments ([src/tui.rs](../src/tui.rs)).
- **OS Support**: CI tests Linux and macOS on stable Rust. Windows is not verified and is not covered by the CI matrix.

## Verification

Run the same checks locally that CI runs:

~~~bash
cargo fmt --check
cargo check --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo bench --locked --no-run
~~~

The Criterion benchmarks can be executed locally with cargo bench. Their results depend on the machine, operating system, and toolchain, so this repository does not present a universal performance claim. See [docs/verification.md](verification.md) for the verification contract and [docs/architecture.md](architecture.md) for the module boundaries.

## Compatibility and versioning

grainx follows `0.x` semantics: the version number is a statement about scope, not a compatibility promise. While the major version is 0, a breaking change to the CLI, the configuration schema, or the exported JSON/CSV shape bumps the minor version, and a compatible fix bumps the patch version. Every change is recorded in [CHANGELOG.md](../CHANGELOG.md).

| Platform | Status |
| --- | --- |
| Linux | Verified by CI on stable Rust, plus an all-target compilation job on the minimum supported version. |
| macOS | Verified by CI, and used for the recorded captures under [demos/](../demos/). |
| Windows | Not verified; the CI matrix does not cover it. |

The minimum supported Rust version is 1.88; raising it is a minor-version change. A `1.0` would mean the command surface, the configuration schema, and the exported formats have stopped moving, not that every idea in [TODO.md](../TODO.md) has been implemented.
