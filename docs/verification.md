# Verification

The public verification contract is intentionally small and reproducible.

## Local commands

~~~bash
cargo fmt --check
cargo check --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo bench --locked --no-run
~~~

These commands cover formatting, compilation for all declared targets, linting, unit/integration/doc tests, and benchmark compilation without claiming benchmark results.

To run the Criterion suite locally:

~~~bash
cargo bench --locked
~~~

Benchmark numbers are environment-dependent. Any published measurement should include the machine, operating system, Rust toolchain, and command used.

## CI scope

GitHub Actions runs the local validation contract on Ubuntu with the stable Rust toolchain for pushes to main and pull requests targeting main. A separate job compiles every target with Rust 1.88.0 and the committed lockfile so the declared minimum does not silently drift. Windows and macOS are not currently part of the CI matrix.

`Cargo.lock` is committed because grainx is an application. The `--locked` checks fail instead of silently changing the resolved dependency graph, so a clean checkout and CI evaluate the same package versions.
