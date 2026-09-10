# grainx demo verification — 2026-09-11

This record documents how the current `main` checkout was captured as a runnable demo. It is evidence
for the listed build and host only, not a performance or compatibility claim.

## Build identity

| Field | Value |
| --- | --- |
| Commit | `5dde1082e5f8989793e79bd5130ec728dd0bde83` |
| Working tree at build time | clean (`git status --porcelain` = 0 paths) |
| Build command | `cargo build --locked` |
| Binary | `target/debug/grainx` |
| Binary SHA-256 | `9dfd9cccdd5d4f2352d6aca527b0c3f8f75bda35f95aa1134d0bfaee98fdce00` |
| Toolchain | `rustc 1.94.1 (e408947bf 2026-03-25)` / `cargo 1.94.1 (29ea6fb6a 2026-03-24)` |
| Renderer | `pyte 0.8.2` (installed in a throwaway venv; capture itself uses only the Python standard library) |
| Host | macOS Apple Silicon (darwin arm64), UTC session date 2026-09-10/11 |

The build was incremental on a warm `target/` directory, so the wall time is not reported as a build
benchmark.

## Captured artifacts

| File | SHA-256 | Contents |
| --- | --- | --- |
| `tui_capture_2026-09-11.raw` | `3bea9c1bd52b22c257c187b8b74a86976ba8234b041ad44736e4dbf5bec64232` | Raw ANSI bytes from a pty session. |
| `tui_capture_2026-09-11.txt` | `856112d6105524357e81aa8daec955e80f236e27d32d07366d9a97e585a59c6a` | pyte render of the raw bytes at 110x50. |
| `http_capture_2026-09-11.txt` | `7f34b34e8aff900e32fc80a1ddb63bbe61218b90a991086fd3b0d1641149b44d` | Real `/health`, `/metrics`, `agent` log, local `export` output, remote `export` failure log. |
| `capture_tui.py` | `a2194200efed8ea844086f4c543dda9f6c244486e3789825431c6cd4aefc16ab` | Capture helper used for the TUI run. |

## TUI capture commands

The TUI was driven on a fresh pty (no interactive terminal was attached to this session), allowed to
render for 5 seconds, then quit with `q`:

~~~bash
python3 -m venv /tmp/grainx-render-venv
/tmp/grainx-render-venv/bin/pip install pyte==0.8.2
/tmp/grainx-render-venv/bin/python demos/capture_tui.py \
  target/debug/grainx \
  /tmp/tui_capture_2026-09-11.raw \
  /tmp/tui_capture_2026-09-11.txt \
  5 110 50
~~~

The raw stream contains zero occurrences of `KB/s` and one occurrence of the new label
`Network I/O: RX 13.0 KB  TX 4.0 KB (last interval)` (values vary per run). The environment variable
`TERM=xterm-256color` was set for the child. The capture shows `Iteration: 2` because the 5-second
window covers the first frames; the frame counter is not part of the label verification.

## HTTP and export capture commands

The agent was started from the built binary on a free loopback port in an owned temporary working
directory; the export subcommands ran in their own subdirectories:

~~~bash
port=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')
target/debug/grainx agent --bind 127.0.0.1 --port "$port" &
curl -fsS "http://127.0.0.1:$port/health"
curl -fsS "http://127.0.0.1:$port/metrics"
(cd local-export  && /path/to/target/debug/grainx export)
(cd remote-export && /path/to/target/debug/grainx export --remote "http://127.0.0.1:$port")
~~~

Results: `/health` exit 0, `/metrics` exit 0, local `export` exit 0 (JSON + CSV written), remote
`export` exit 101 (panic). The complete transcript is in `http_capture_2026-09-11.txt`.

## Repo checks

Run on the same commit with the committed lockfile:

~~~text
cargo fmt --check                                        PASS
cargo check --locked --all-targets                       PASS
cargo clippy --locked --all-targets -- -D warnings       PASS
cargo test --locked                                      PASS (43 library + 13 binary + 2 integration tests, 0 failed; 0 doctests)
~~~

## Privacy review

Captured fields fall into these categories:

- **Present**: OS name, kernel version, uptime, CPU core percentages, memory totals, disk label
  (`Macintosh HD`) with sizes, top-10 process names, PIDs, per-process CPU percentages, and memory
  sizes. Timestamps are UTC.
- **Sanitized**: the remote-export panic log embedded in `http_capture_2026-09-11.txt` contained the
  absolute Cargo registry path `/Users/<user>/.cargo/...`. That single home-directory path was replaced
  with `/Users/<redacted>/`. No other edits were made to any capture.
- **Absent**: usernames, hostnames, home-directory paths, shell history, environment variables,
  network addresses of the host, credentials, tokens, keys, or secret material.

Process names are real names from the capture host and may reveal which applications were running
(for example `git-remote-https`). Regenerate on a quiet host if a neutral process list is required.

## Known limitations

- **Remote export is broken at this commit.** `export --remote <url>` panics with
  `Cannot drop a runtime in a context where blocking is not allowed` (tokio runtime drop inside an
  async context). The failure log is preserved in the HTTP capture. Local `export` works. This
  pre-existing defect is documented, not fixed, because the demo must bind to the built commit.
- **Memory label overpaint.** In the rendered frame the `Memory: ...%` line is drawn and then the
  memory chart region clears its own first row, so the label is not visible in the final screen. This
  is a pre-existing renderer issue unrelated to the network-label change; the network label and totals
  are unaffected.
- The captures reflect one host and one instant. Values are samples, not benchmarks. Network values
  are per-interval deltas, not rates.
- `pyte` is a rendering aid; `tui_capture_2026-09-11.raw` is the ground truth and can be replayed in
  any compatible terminal with `cat`.

The 2026-09-06 captures (`http_capture.txt`, `tui_capture.raw`) remain in the tree unchanged as
historical material. They record no build identity and still show the old `KB/s` label.
