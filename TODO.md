# Roadmap

grainx is a pre-1.0 terminal monitoring project. This file records what is implemented, what evidence
backs it, and which follow-ups are deliberate future work rather than missing pieces.

## Implemented

- [x] Local TUI for CPU, memory, disk, network, process, and host metrics
- [x] Unicode/Braille chart rendering
- [x] Adaptive refresh and high-load frame skipping
- [x] Statistical anomaly detection, correlation, and moving-average estimates
- [x] Local HTTP metrics service and remote monitor mode
- [x] JSON and CSV snapshot export, locally and from a running remote agent
- [x] Configuration file, environment overrides, and CLI overrides
- [x] Unit and integration tests
- [x] Linux and macOS CI with formatting, all-target checks, Clippy, tests, and benchmark compilation
- [x] Minimum-supported-Rust-version compilation job (1.88.0, committed lockfile)
- [x] Reproducible benchmark results with machine and toolchain recorded
      ([benches/results/2026-09-10-macos-arm64.txt](benches/results/2026-09-10-macos-arm64.txt))
- [x] Recorded capture and verification evidence under [demos/](demos/) and [docs/validation/](docs/validation/)

## Open follow-ups

These are product decisions and future scope. None of them block the current pre-1.0 use on localhost.

- [ ] Decide whether the HTTP agent needs authentication, TLS, or rate limiting before any non-local
      deployment. Today the agent is documented as localhost-only and ships none of those controls.
- [ ] Add a Windows CI job if Windows becomes a supported target. CI currently covers Linux and macOS.
- [ ] Replace the deliberately simple left-to-right formula evaluator if expression support grows
      beyond its current scope.
- [ ] Define the first stable release scope and versioning policy.
