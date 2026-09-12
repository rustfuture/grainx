# Braille termination verification - 2026-09-12

This record covers the source and binary built from implementation commit
`62690ca94ea1113eee538c5270123a3b25c5c7b9` on branch
`fix/braille-termination`.

## Build identity

| Field | Value |
| --- | --- |
| UTC capture time | `2026-09-12T05:59:22Z` |
| Working-tree dirty count at build | `0` (`git status --porcelain=v1`) |
| Build command | `cargo build --locked` (exit 0) |
| Binary | `target/debug/grainx` |
| Binary SHA-256 | `a51a95c018cf5b41cb6892a79f9de8672fc2756fa3478a8ea00f2c7bd6a8bcce` |
| Toolchain | `rustc 1.94.1 (e408947bf 2026-03-25)` / `cargo 1.94.1 (29ea6fb6a 2026-03-24)` |

## PTY proof

An owned, uncommitted Python standard-library driver created a fresh PTY,
set its window size, ran the actual binary with
`GRAINX_REFRESH_INTERVAL_MS=100`, parsed the flushed `| Frame: N` suffixes,
sent `q` after five completed frames, and allowed four seconds for graceful
exit. It did not use `demos/capture_tui.py`, TERM, or KILL.

| Size | Duration | Completed frames | Increasing | `q` exit | Signals |
| --- | ---: | --- | --- | ---: | --- |
| `80x24` | 2345 ms | `1, 2, 3, 4, 5` | yes | 0 | none |
| `110x50` | 2344 ms | `1, 2, 3, 4, 5` | yes | 0 | none |

The proof establishes bounded completion and graceful input handling for these
two sizes on this host. It is not a performance or cross-platform claim.

## Command outcomes

All commands used the committed lockfile and passed:

```text
cargo fmt --check                                  exit 0
cargo check --locked --all-targets                 exit 0
cargo clippy --locked --all-targets -- -D warnings exit 0
cargo test --locked                                exit 0; 75 passed, 0 failed
cargo bench --locked --no-run                      exit 0; 3 executables built
cargo +1.88.0 check --locked --all-targets         exit 0
```

The rasterizer integration tests execute `braille_grid` and
`AdvancedCanvas::draw_braille_line` inside killable subprocess workers with a
two-second deadline. They cover empty and zero-sized inputs, the explicit
single-point edge policy, integer/fractional/repeated lines, clipping,
non-finite inputs, edge-adjacent rounding, and finite `f64::MAX` crossings.
