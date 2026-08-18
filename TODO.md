# Roadmap

grainx is a pre-1.0 terminal monitoring project. This file tracks the work that is implemented and the follow-ups that still need evidence or product decisions.

## Implemented

- [x] Local TUI for CPU, memory, disk, network, process, and host metrics
- [x] Unicode/Braille chart rendering
- [x] Adaptive refresh and high-load frame skipping
- [x] Statistical anomaly detection, correlation, and moving-average estimates
- [x] Local HTTP metrics service and remote monitor mode
- [x] JSON and CSV snapshot export
- [x] Configuration file, environment overrides, and CLI overrides
- [x] Unit and integration tests
- [x] Linux CI with formatting, all-target checks, Clippy, tests, and benchmark compilation

## Next

- [ ] Add Windows and macOS CI jobs or a documented release validation matrix
- [ ] Decide whether the HTTP agent needs authentication, TLS, or rate limiting before non-local deployment
- [ ] Replace the prototype formula evaluator if expression support grows beyond simple left-to-right arithmetic
- [ ] Publish reproducible benchmark results with the machine and toolchain recorded
- [ ] Define the first stable release scope and versioning policy
