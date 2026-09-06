# Contributing

grainx uses stable Rust and a small, conventional validation loop.

## Development setup

~~~bash
git clone https://github.com/rustfuture/grainx.git
cd grainx
cargo run --locked
~~~

## Before opening a pull request

~~~bash
cargo fmt --check
cargo check --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo bench --locked --no-run
~~~

Keep changes focused, add tests for behavior changes, and update the README or docs when a command, configuration field, or operational constraint changes.

The HTTP agent is intentionally a minimal metrics endpoint. Do not expose it to an untrusted network without addressing authentication and transport security.
