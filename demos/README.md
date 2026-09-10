# Demos

- `http_capture.txt` — a snapshot of `GET /health` and `GET /metrics` from a running `grainx agent`.
- `tui_capture.raw` — a raw ANSI capture of the monitor TUI (not a video; render it in a compatible
  terminal, for example with `cat demos/tui_capture.raw`).

## Build identity

Both captures were taken on 2026-09-06 from an unreleased working tree. They record no commit,
toolchain, or binary hash, and they predate the 2026-09-10 changes (the TUI capture still shows the old
`KB/s` network label). Treat them as illustrative, not as a verified build artifact.

## Reproduce the HTTP snapshot

~~~bash
cargo run --locked -- agent --bind 127.0.0.1 --port 9090
# in another shell:
curl -s http://127.0.0.1:9090/health
curl -s http://127.0.0.1:9090/metrics
~~~

`network_rx_bytes` and `network_tx_bytes` in the JSON are per-interval deltas, not rates.

## Privacy note

`http_capture.txt` is a real host snapshot and therefore contains local process names, PID values, disk
labels, and memory sizes from the machine that produced it. It contains no credentials. Regenerate from
a clean host if a neutral capture is needed. Do not commit captures that contain secrets, tokens,
private keys, or personal filesystem paths.
