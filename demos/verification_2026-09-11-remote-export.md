# grainx remote-export CLI verification — 2026-09-11

This record proves the `grainx export --remote <url>` fix at the real CLI boundary on top of
`main` commit `3dfa107` (the commit that moved the blocking `reqwest` client to a dedicated OS
thread). It covers the success path, two controlled-failure paths, and the exact
`http://127.0.0.1:0` case that panicked before the fix. It is evidence for the listed build and
host only, not a compatibility claim.

> Merge note (2026-09-11): this change and its CLI tests were merged to `main` as `2067359879b9`
> (PR #4). The runtime fix under test is unchanged between `3dfa107` and the merge; the additional
> commits only add tests and documentation.

## Build identity

| Field | Value |
| --- | --- |
| Source commit | `3dfa107f78b94812c080eae3b577c7cd6035cfdd` (`fix: build blocking remote metrics client outside the async runtime`) |
| Branch | `test/remote-export-cli` (branched from `origin/main` at the commit above) |
| Worktree at `cargo build` | clean (`git status --porcelain` = 0 paths) |
| Worktree at CLI capture | dirty count 1: untracked `tests/cli_remote_export.rs`; no changes under `src/` |
| Build command | `cargo build --locked` |
| Binary | `target/debug/grainx` (frozen copy used for captures: `grainx.used`) |
| Binary SHA-256 | `5e485df1469c1163db9b253794a30f7124b763969ea8abf39ae908584c7492b6` |
| Toolchain | `rustc 1.94.1 (e408947bf 2026-03-25)` / `cargo 1.94.1 (29ea6fb6a 2026-03-24)` |
| Host | macOS Apple Silicon (darwin arm64), UTC session date 2026-09-11 |

The demo captures below ran against a frozen copy (`grainx.used`) of the `cargo build --locked`
artifact, and both files hash to the value above. `cargo test` builds its own variant of the bin
target, so freezing the binary removes ambiguity about which bytes produced the captures.

## Historical failure (not a current result)

On 2026-09-11, at commit `5dde108`, the same CLI shape exited 101 with:

~~~text
thread 'main' (157332) panicked at .../tokio-1.53.1/src/runtime/blocking/shutdown.rs:51:21:
Cannot drop a runtime in a context where blocking is not allowed. ...
~~~

The full log is preserved in `demos/http_capture_2026-09-11.txt` and
`demos/verification_2026-09-11.md`. That capture is historical evidence of the defect; it is not
evidence of the fix.

## Captured CLI runs

All runs used the frozen binary on loopback only; no internet access is required. The success run
used a real `grainx agent` process as the server; the malformed run used
`python3 -m http.server` serving a syntactically invalid `/metrics` body.

Result summary:

| Case | Exit code | Output files | Panic text |
| --- | --- | --- | --- |
| Success against live `grainx agent` | 0 | `r.json` (2045 bytes), `r.csv` (987 bytes) | none |
| Unreachable closed loopback port | 4 | none | none |
| Malformed `/metrics` JSON (HTTP 200) | 4 | none | none |
| `http://127.0.0.1:0` (historical panic repro) | 4 | none | none |

### Success

~~~bash
port=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')
target/debug/grainx agent --bind 127.0.0.1 --port "$port" &
target/debug/grainx export --remote "http://127.0.0.1:$port" --json r.json --csv r.csv
~~~

Observed stdout: empty. Observed stderr:

~~~text
Exported stats to r.json and r.csv
~~~

Both files parse. The JSON keeps byte counts (per-interval deltas) and does not rename them to
rates:

~~~text
json network_rx_bytes=3072 network_tx_bytes=2048
json keys containing 'rate': []
system,network_rx_bytes,3072
system,network_tx_bytes,2048
~~~

(Values are per-run deltas from the agent and vary between runs; the field names and integer-byte
semantics do not.)

### Unreachable endpoint

The port number comes from a bind/drop probe and is not listening:

~~~bash
target/debug/grainx export --remote "http://127.0.0.1:$closed_port" --json r.json --csv r.csv
~~~

Observed exit code: `4`. stderr:

~~~text
Error: remote metrics: remote metrics: failed to fetch remote metrics from http://127.0.0.1:50320/metrics: error sending request for url (http://127.0.0.1:50320/metrics)
~~~

No `r.json` was written.

### Malformed response

The server returned HTTP 200 with body `{"cpu_usage_percent": "not a snapshot"}` at `/metrics`:

~~~bash
printf '%s' '{"cpu_usage_percent": "not a snapshot"}' > badserver/metrics
python3 -m http.server --bind 127.0.0.1 "$bad_port" --directory badserver &
target/debug/grainx export --remote "http://127.0.0.1:$bad_port" --json r.json --csv r.csv
~~~

Observed exit code: `4`. stderr:

~~~text
Error: remote metrics: remote metrics: invalid metrics JSON: error decoding response body
~~~

No `r.json` or `r.csv` was written.

### Port zero (exact historical repro)

~~~bash
target/debug/grainx export --remote http://127.0.0.1:0 --json r.json --csv r.csv
~~~

Observed exit code: `4`. stderr:

~~~text
Error: remote metrics: remote metrics: failed to fetch remote metrics from http://127.0.0.1:0/metrics: error sending request for url (http://127.0.0.1:0/metrics)
~~~

No `r.json` was written, and no `panicked` or `Cannot drop a runtime` text appears in any capture.

## Automated CLI coverage

`tests/cli_remote_export.rs` spawns the compiled binary via `env!("CARGO_BIN_EXE_grainx")` against
a minimal `std::net::TcpListener` server (no extra dependencies, no internet). Four tests:

1. `remote_export_succeeds_against_a_live_http_server` — exit 0; JSON deserializes as
   `StatsSnapshot`; `network_rx_bytes`/`network_tx_bytes` equal the served byte counts; no
   `network_*_rate` keys; CSV contains the same byte-count rows and process section.
2. `remote_export_reports_controlled_error_when_endpoint_unreachable` — exit 4, controlled
   message, no panic text, no output files.
3. `remote_export_reports_controlled_error_on_malformed_response` — exit 4, `invalid metrics
   JSON`, no panic text, no output files.
4. `remote_export_to_port_zero_reports_controlled_error` — the exact historical repro.

## Repo checks

Run on the same commit with the committed lockfile:

~~~text
cargo fmt --check                                        PASS
cargo check --locked --all-targets                       PASS
cargo clippy --locked --all-targets -- -D warnings       PASS
cargo test --locked                                      PASS
~~~

`cargo test --locked` result: 45 library tests, 2 `src/main.rs` tests, 4 `cli_remote_export`
tests, 13 `integration_tests` tests — 64 passed, 0 failed, 0 ignored; 0 doctests.

## Remaining limits

- The automated tests bind `127.0.0.1` only. No TLS, proxy, redirect, auth, non-200 status, slow
  endpoint, or large-body coverage is claimed here.
- The test server answers one request with `Connection: close`; it is a fixture, not a
  conformance suite.
- The malformed case covers schema-invalid JSON in an HTTP 200 body. Non-200 handling
  (`remote metrics request failed with status ...`) is implemented but not asserted at the CLI
  boundary in this record.
- The success capture is a real host snapshot at one instant; values are samples, not benchmarks.
  Process names, disk labels, OS/kernel versions, and uptime appear in the generated files and
  were not committed.
- `cargo test` builds a slightly different bin artifact than `cargo build`, so the frozen binary
  hash above identifies the captured artifact, not necessarily every local `cargo test` run.
- Windows behavior is not tested; CI runs the test suite on ubuntu-latest and an MSRV
  `cargo check` on Rust 1.88.
