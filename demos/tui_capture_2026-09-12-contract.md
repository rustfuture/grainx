# TUI capture-contract verification - 2026-09-12

This phase-2 record verifies `demos/capture_tui.py` against the combined Task B
source and Task A as merged on `origin/main`. The historical captures are
unchanged.

## Source and build identity

| Field | Value |
| --- | --- |
| Branch source SHA | `360e5a97a1ded040fc1f849af3cb73fe915579d9` |
| Required Task B commit | `022e448` (ancestor of source SHA) |
| Merged `origin/main` | `a5e41c2cf5b87c0a7394e87f7d3954a3bec01168` (ancestor of source SHA) |
| Source dirty count before build | `0` (`git status --porcelain=v1`) |
| Build command | `cargo build --locked` (exit 0) |
| Binary | `target/debug/grainx` |
| Binary size / SHA-256 | `16648888` bytes / `e39c5545207d749ea8eb8105dde8ed49f948a50e56dd4b3282e8db9346e2b14e` |
| Rust toolchain | `rustc 1.94.1 (e408947bf 2026-03-25)` / `cargo 1.94.1 (29ea6fb6a 2026-03-24)` |
| Text renderer | `pyte 0.8.2` in an owned temporary virtual environment |

## Exact capture commands

The commands ran from the repository root. `CAPTURE_PYTHON` was the Python
executable in the owned temporary venv; no global package was installed.

```bash
GRAINX_REFRESH_INTERVAL_MS=100 "$CAPTURE_PYTHON" demos/capture_tui.py \
  target/debug/grainx \
  demos/tui_capture_2026-09-12-contract-80x24.raw \
  demos/tui_capture_2026-09-12-contract-80x24.txt \
  1.75 80 24

GRAINX_REFRESH_INTERVAL_MS=100 "$CAPTURE_PYTHON" demos/capture_tui.py \
  target/debug/grainx \
  demos/tui_capture_2026-09-12-contract-110x50.raw \
  demos/tui_capture_2026-09-12-contract-110x50.txt \
  1.75 110 50
```

## Capture results

Both runs used the actual debug binary, rendered nonempty pre-`q` ANSI through
pyte, observed seven strictly increasing completed frame markers, sent one
`q`, and reaped a normal exit without timeout, TERM, or KILL.

### 80x24 JSON summary

```json
{"child_exit_code":0,"child_reaped":true,"child_signal":null,"frame_count":7,"frame_first":1,"frame_last":7,"frame_max":7,"frame_min":1,"frames_increasing":true,"kill_sent":false,"monotonic_duration_seconds":1.955509,"pre_q_bytes":126045,"q_sent":true,"reasons":[],"success":true,"term_sent":false,"timeout":false,"total_bytes":126063}
```

### 110x50 JSON summary

```json
{"child_exit_code":0,"child_reaped":true,"child_signal":null,"frame_count":7,"frame_first":1,"frame_last":7,"frame_max":7,"frame_min":1,"frames_increasing":true,"kill_sent":false,"monotonic_duration_seconds":1.983788,"pre_q_bytes":282315,"q_sent":true,"reasons":[],"success":true,"term_sent":false,"timeout":false,"total_bytes":282333}
```

| Size | Raw artifact | Raw bytes / SHA-256 | Text artifact | Text bytes / SHA-256 | `CPU Usage:` | `Memory:` |
| --- | --- | --- | --- | --- | --- | --- |
| `80x24` | `tui_capture_2026-09-12-contract-80x24.raw` | `126063` / `828afd4886726c97c4eb49662f8093b3133272cf2220c251c16e911066712421` | `tui_capture_2026-09-12-contract-80x24.txt` | `4566` / `f8626efc5bca9010c7033ec6cb6176b7408b34ec0c3af2a47e2f25175de79ff1` | visible | visible |
| `110x50` | `tui_capture_2026-09-12-contract-110x50.raw` | `282333` / `57acf55143dffe6401c7940a5d8c351a16c87f7e1c3d0e0a6e4c86c4618913b7` | `tui_capture_2026-09-12-contract-110x50.txt` | `10022` / `0585a27b58ac071ff0b78e7fbea7c7b75fbd93802e7745859fa0e9b20306b027` | visible | visible |

A byte scan of all four artifacts found no `/Users/`, `PRIVATE KEY`, `api_key`,
or `token=` strings.

## Negative control

```bash
GRAINX_REFRESH_INTERVAL_MS=100 "$CAPTURE_PYTHON" demos/capture_tui.py \
  /usr/bin/false "$OWNED_TEMP/grainx-b-phase2-false.raw" - 1 80 24
```

The capture command exited 1 as required. Its summary reported
`success=false`, `child_exit_code=1`, `child_reaped=true`, `q_sent=false`, no
signal, no timeout/TERM/KILL, and reasons including `early_exit_before_q` and
`child_exit_code:1`.

## Quality checks

```text
python3 -m py_compile demos/capture_tui.py demos/test_capture_tui.py  exit 0
python3 -m unittest -v demos/test_capture_tui.py                    exit 0; 11 passed
cargo fmt --check                                                   exit 0
cargo check --locked --all-targets                                  exit 0
cargo clippy --locked --all-targets -- -D warnings                  exit 0
cargo test --locked                                                 exit 0; 75 passed
cargo bench --locked --no-run                                       exit 0; 3 executables built
```

The Python suite's first run overlapped the release-profile bench compilation
and had two fixture startup timing failures. The required isolated rerun above
passed all 11 tests without source changes.

These captures are host snapshots, not performance or cross-platform claims.
