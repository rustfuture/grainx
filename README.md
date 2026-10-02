# grainx

![grainx project overview](docs/images/social-preview.png)

grainx shows live computer and process activity in a terminal and can share those readings over a local web connection.

[![CI](https://github.com/rustfuture/grainx/actions/workflows/ci.yml/badge.svg)](https://github.com/rustfuture/grainx/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Status:** Experimental CLI tool (v0.1.2, pre-1.0); CI checks Linux and macOS.

- Shows CPU, memory, disk, network, and process activity in an interactive dashboard.
- Serves host readings through a local HTTP service.
- Saves local or remote readings as JSON and CSV files.
- Adjusts dashboard refresh and rendering under high system load.

## Quick start

You need Git and stable Rust 1.88 or newer with Cargo ([rustup](https://rustup.rs/)), plus an interactive terminal for the dashboard. The first build downloads Cargo dependencies. The crate uses Rust edition 2024; the minimum version is checked against the committed dependency lockfile.

~~~bash
git clone https://github.com/rustfuture/grainx.git
cd grainx
cargo run --locked
~~~

The default command opens the dashboard; press `q` to quit. For a first check without an interactive terminal, export local readings instead:

~~~bash
cargo run --locked -- export --json /tmp/grainx-stats.json --csv /tmp/grainx-stats.csv
~~~

The command prints the two output paths. Open the JSON or CSV file to inspect the readings; no HTTP agent or external service is needed. On Windows, choose local output paths instead of `/tmp/...`; Windows is not covered by CI.

See [usage and project details](docs/readme-details.md) for configuration, commands, controls, metric definitions, demos, benchmarks, and compatibility notes.

## Architecture

The CLI starts the interactive dashboard by default, or runs the HTTP service, exports readings, or generates shell completions. The monitor collects system and process readings into a shared snapshot. The dashboard displays that snapshot, while the `--remote <url>` option of the dashboard and export commands fetches the snapshot from a running agent instead. The agent serves health and metrics endpoints over HTTP and only binds to a loopback address, so reaching it from another machine needs a tunnel or authenticated proxy that grainx does not provide. Export writes snapshots as JSON and CSV.

## Tests

The CI workflow runs these commands:

~~~bash
cargo fmt --check
cargo check --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo bench --locked --no-run
~~~

The tests cover command parsing, system sampling, dashboard layout, HTTP responses, and JSON/CSV export behavior.

## Limitations

- The HTTP agent has no TLS, authentication, or rate limiting. It rejects non-loopback bind addresses; loopback does not isolate users or processes on the same machine.
- Network and disk readings are aggregates, without per-socket or per-file detail.
- The formula evaluator accepts whitespace-separated operations from left to right, without operator precedence or parentheses.
- The dashboard requires an interactive terminal. Windows is not covered by the CI matrix.

## License

grainx is released under the MIT License. See [LICENSE](LICENSE).
