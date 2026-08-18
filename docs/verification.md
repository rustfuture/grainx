# Verification

The public verification contract is intentionally small and reproducible.

## Local commands

~~~bash
cargo fmt --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test
cargo bench --no-run
~~~

These commands cover formatting, compilation for all declared targets, linting, unit/integration/doc tests, and benchmark compilation without claiming benchmark results.

To run the Criterion suite locally:

~~~bash
cargo bench
~~~

Benchmark numbers are environment-dependent. Any published measurement should include the machine, operating system, Rust toolchain, and command used.

## CI scope

GitHub Actions runs the local validation contract on Ubuntu with the stable Rust toolchain for pushes to main and pull requests targeting main. Windows and macOS are not currently part of the CI matrix.