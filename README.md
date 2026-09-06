# grainx

[![CI](https://github.com/rustfuture/grainx/actions/workflows/ci.yml/badge.svg)](https://github.com/rustfuture/grainx/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-1.88%2B-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A Rust terminal system monitor for CPU, memory, disks, network activity, processes, and host metadata.

grainx is a pre-1.0 project. The repository documents the implemented behavior and the checks that run in CI; it does not publish unsupported runtime or benchmark numbers.

## What it does

- Interactive terminal dashboard with Unicode/Braille charts.
- Local system collection through sysinfo: CPU, memory, disks, network counters, processes, OS, kernel, and uptime.
- Optional analytics: z-score anomaly detection, Pearson correlation, moving-average estimates, and a deliberately simple arithmetic formula evaluator.
- Adaptive refresh and frame skipping when CPU load is high.
- Optional HTTP metrics service with GET /health and GET /metrics.
- JSON and CSV snapshots from local or remote metrics.
- JSON configuration with environment-variable and CLI overrides.
- Shell completion generation for Bash, Elvish, Fish, PowerShell, and Zsh.

The term agent in this repository means the HTTP metrics process. It is not an AI or LLM agent.

## Quick start

Prerequisites:

- Stable Rust 1.88 or newer. The crate uses Rust edition 2024. This minimum is checked against the committed dependency lockfile.
- An interactive terminal for the monitor command.

~~~bash
git clone https://github.com/rustfuture/grainx.git
cd grainx

# Run the local dashboard
cargo run --locked

# Build an optimized binary
cargo build --locked --release
~~~

The default command is monitor. In a headless environment, use the agent or export command instead.

## Commands

~~~bash
# Interactive dashboard
cargo run --locked -- monitor

# Local HTTP metrics service; localhost is the default bind address
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

The agent exposes host metrics without authentication or TLS. Keep it on localhost unless you have added appropriate network controls and an authenticated transport around it.

## Configuration

The monitor reads dashboard_config.json. If the file is missing, grainx creates a default configuration. The precedence order is:

1. dashboard_config.json
2. Environment variables
3. Monitor CLI flags

Supported environment overrides include GRAINX_REFRESH_INTERVAL_MS, GRAINX_CPU_WARNING_THRESHOLD, GRAINX_MEMORY_WARNING_THRESHOLD, and GRAINX_COLOR_THEME. See [dashboard_config.json](dashboard_config.json) and [src/config.rs](src/config.rs) for the current schema.

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

## Verification

Run the same checks locally that CI runs:

~~~bash
cargo fmt --check
cargo check --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo bench --locked --no-run
~~~

The Criterion benchmarks can be executed locally with cargo bench. Their results depend on the machine, operating system, and toolchain, so this repository does not present a universal performance claim. See [docs/verification.md](docs/verification.md) for the verification contract and [docs/architecture.md](docs/architecture.md) for the module boundaries.

## Compatibility

grainx is built from cross-platform Rust crates, but the current CI matrix verifies Linux only. Windows and macOS support should be treated as targets to validate on the specific release being used, not as a claim of a tested compatibility matrix.

## License

grainx is released under the MIT License. See [LICENSE](LICENSE).
