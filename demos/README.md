# Demos

## Current, verified captures (2026-09-11)

- `tui_capture_2026-09-11.raw` — raw ANSI capture of the monitor TUI, taken from the built
  `target/debug/grainx` binary under a pty. It shows the current network label
  `Network I/O: RX <n> KB  TX <n> KB (last interval)`; it contains no `KB/s`.
- `tui_capture_2026-09-11.txt` — readable 110x50 text render of the same bytes (pyte 0.8.2).
- `http_capture_2026-09-11.txt` — real `GET /health`, `GET /metrics`, agent log, and local `export`
  output from the built binary on a free loopback port. It also preserves the historical
  remote-export failure at commit `5dde108` (see below).
- `verification_2026-09-11.md` — build identity (commit, toolchain, binary SHA-256), exact capture
  commands, artifact hashes, privacy review, and limitations.
- `verification_2026-09-11-remote-export.md` — CLI-boundary verification of the remote-export fix
  on base `3dfa107`: success against a live `grainx agent` and controlled errors for unreachable,
  malformed, and `http://127.0.0.1:0` endpoints, with no panic text in any capture.

Verified build: commit `5dde1082e5f8989793e79bd5130ec728dd0bde83`, binary SHA-256
`9dfd9cccdd5d4f2352d6aca527b0c3f8f75bda35f95aa1134d0bfaee98fdce00`, `rustc 1.94.1`.

Remote export is fixed on `main` at `3dfa107`; the current CLI evidence is in
`verification_2026-09-11-remote-export.md`.

### Reproduce the TUI capture

~~~bash
cargo build --locked

python3 -m venv /tmp/grainx-render-venv
/tmp/grainx-render-venv/bin/pip install pyte==0.8.2

/tmp/grainx-render-venv/bin/python demos/capture_tui.py \
  target/debug/grainx \
  /tmp/tui_capture.raw \
  /tmp/tui_capture.txt \
  5 110 50

# The raw capture also replays in a compatible terminal:
cat /tmp/tui_capture.raw
~~~

`demos/capture_tui.py` uses only the Python standard library for capture; `pyte` is needed only for the
text render. The pty is sized to 110x50 so the process list does not collide with the footer row; the
raw stream is the ground truth.

### Reproduce the HTTP and export capture

~~~bash
cargo build --locked

port=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')
target/debug/grainx agent --bind 127.0.0.1 --port "$port" &
curl -fsS "http://127.0.0.1:$port/health"
curl -fsS "http://127.0.0.1:$port/metrics"

mkdir -p /tmp/grainx-export && cd /tmp/grainx-export
/path/to/target/debug/grainx export
~~~

`network_rx_bytes` and `network_tx_bytes` in the JSON are per-interval deltas, not rates.

### Remote export status

- **Historical:** at `5dde108` (the 2026-09-11 capture commit), `export --remote <url>` panicked
  with `Cannot drop a runtime in a context where blocking is not allowed`. The failure log is
  preserved in `http_capture_2026-09-11.txt` as a defect record, not a current result.
- **Fixed:** `main` at `3dfa107` runs the blocking request on a dedicated OS thread.
  `verification_2026-09-11-remote-export.md` shows exit 0 against a live `grainx agent`, exit 4
  with controlled errors for unreachable, malformed, and `http://127.0.0.1:0` endpoints, and no
  panic text. Automated CLI coverage is in `tests/cli_remote_export.rs`.
- Local `export` works at both commits.

## Historical captures (2026-09-06)

- `http_capture.txt` — snapshot of `GET /health` and `GET /metrics`.
- `tui_capture.raw` — raw ANSI capture of the monitor TUI.

**These are historical and pre-fix.** They were taken on 2026-09-06 from an unreleased working tree
with no recorded commit, toolchain, or binary hash, and the TUI capture still shows the old `KB/s`
network label. Do not treat them as verified build artifacts; use the 2026-09-11 captures above.

## Privacy note

The verified captures are real host snapshots and contain local process names, PID values, disk labels,
memory sizes, OS/kernel versions, and uptime. The only sanitization applied was replacing a
home-directory path in the remote-export panic log with `/Users/<redacted>/`; this is documented in
`verification_2026-09-11.md`. They contain no credentials. Regenerate from a clean host if a neutral
capture is needed. Do not commit captures that contain secrets, tokens, private keys, or personal
filesystem paths.
