# Product decisions and roadmap

This file records what grainx implements, which decisions have been made deliberately, and what would
have to change for those decisions to be revisited. It is not a list of missing work.

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

## Decisions

### The HTTP agent is loopback-only

**Decision.** The agent refuses to start on any address that is not loopback. `--bind 0.0.0.0` and
interface addresses exit with a clear error instead of listening.

**Rationale.** The agent serves host metrics with no authentication, TLS, or rate limiting. Documenting
"keep it on localhost" would be advice; refusing a routable bind is a boundary the program enforces.
Loopback is still not a full protection — it does not separate users or processes on the same host —
and that residual limit is stated in the README rather than glossed over.

**Verification.** `src/agent.rs` tests that `127.0.0.1`, `::1`, and `::ffff:127.0.0.1` are accepted
and that `0.0.0.0`, `::`, and ordinary interface addresses are rejected, plus an end-to-end test that
`run()` returns `NonLoopbackBind` before it binds anything.

**Revisit when.** Remote metrics access becomes a requirement. Then the work is an authenticated
transport (TLS plus a credential), not merely a token over plain HTTP, and it re-opens rate limiting
and what the snapshot is allowed to expose.

### The formula evaluator stays deliberately simple

**Decision.** `evaluate_metric_formula` substitutes whitespace-separated tokens and evaluates strictly
left-to-right with no operator precedence, so `1 + 2 * 3` is `9`. Operators must be surrounded by
whitespace.

**Rationale.** The evaluator exists to build a dashboard metric from named values, not to be a
calculator. The surprise was never the simplicity but the possibility of assuming it behaves like
standard arithmetic, so the contract is documented on the function and covered by tests rather than
left implicit.

**Verification.** Unit tests cover precedence, unknown metric names, missing operands, division by
zero, and the token-boundary rule that stops one metric name from corrupting another.

**Revisit when.** Expression support beyond simple left-to-right arithmetic becomes a real
requirement. Then the evaluator is replaced behind the same function.

### Windows is not a supported target

**Decision.** CI covers Linux and macOS. Windows is listed as unverified rather than "experimental".

**Rationale.** Adding a Windows job would widen the support claim without anyone having validated TUI
behaviour in a real Windows terminal, and a passing `cargo test` on Windows would not establish that.
An honest matrix is worth more than a third badge.

**Revisit when.** Windows becomes a target worth supporting. Then the work starts with a CI job plus a
real terminal smoke test, not with the CI job alone.

### 0.x versioning, and what 1.0 would mean

**Decision.** The version is a statement about scope, not a compatibility promise. A breaking change to
the CLI, the configuration schema, or the exported JSON/CSV shape bumps the minor version; a compatible
fix bumps the patch version. Everything is recorded in [CHANGELOG.md](CHANGELOG.md).

**Rationale.** A well-defined `0.1.0` release is a stronger statement than an underspecified `1.0`.

**Verification.** The rule is stated in the README's compatibility section and applied in the
changelog.

**Revisit when.** The command surface, configuration schema, and exported formats have stopped moving.
That is the `1.0` criterion — not the completion of every idea below.

## Not planned for this version

- A Criterion run on CI hardware. Benchmarks are published with their machine and toolchain recorded;
  a CI number would not be comparable to anything.
- Per-socket or per-file I/O breakdowns. Network and disk metrics are aggregates by design.
