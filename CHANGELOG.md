# Changelog

All notable changes to this project are documented here.

No release was tagged before `v0.1.0`. Version numbers in earlier development were package
versions, not published artifacts, so the release below is the first one a reader can check out.

## [Unreleased]

## [0.1.0] - 2026-09-14

### Added

- Interactive terminal dashboard with Unicode/Braille CPU and memory visualizations.
- Local system collection through `sysinfo`: CPU, memory, disks, network counters, processes, OS,
  kernel, and uptime.
- Statistical analytics: z-score anomaly detection, Pearson correlation, moving-average estimates,
  and the documented `evaluate_metric_formula` evaluator.
- Adaptive refresh, high-load frame skipping, process alerts, themes, logging, and shell completions.
- HTTP metrics agent with `GET /health` and `GET /metrics`, remote monitor mode, and export from a
  running remote agent.
- JSON and CSV snapshot export, with configuration precedence across `dashboard_config.json`,
  environment variables, and CLI flags.
- Unit and integration tests, Criterion benchmarks with recorded results, and CI covering formatting,
  all-target compilation, Clippy, tests, and benchmark compilation.

### Changed

- The HTTP agent refuses any non-loopback bind address. It serves host metrics without
  authentication, TLS, or rate limiting, so the localhost boundary is enforced by the program rather
  than only documented.
- The in-app help menu is English and lists every key the input handler implements.
- CI gained a macOS job alongside the Linux and minimum-supported-version jobs.

### Fixed

- `--bind ::1` never worked. The address was built by formatting `"host:port"`, which produces the
  unparseable `::1:9090`; the host is now parsed as an IP literal and the port attached with
  `SocketAddr::new`, so both address families bind.
- `evaluate_metric_formula` substituted metric names as substrings across the whole expression, so a
  metric whose name contained another metric's name (`cpu` inside `cpu_temp`) could be corrupted, and
  the result depended on `HashMap` iteration order. Substitution is now token-wise and deterministic.
- The README described the remote-export path as failing; that defect had already been fixed and
  covered by `tests/cli_remote_export.rs`.
- The published benchmark log contained an absolute checkout path with a personal directory name.
