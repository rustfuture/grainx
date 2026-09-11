# Memory-label TUI capture — 2026-09-11

New capture pair showing the `Memory: ...` label in the final rendered frame, after the
fix in `42f4602` moved the label above the memory graph's clear/braille passes. It does not
replace the older 2026-09-11 captures; those remain as historical material.

## Build identity

| Field | Value |
| --- | --- |
| Commit | `42f460271a45907d4e571741de4e79642f9f9df8` |
| Working tree at build time | clean (`git status --porcelain` = 0 paths) |
| Build command | `cargo build --locked` |
| Binary | `target/debug/grainx` |
| Binary SHA-256 | `8572c7e189a5736733869c337482a6a348838c9148d44831ff1048f991f7bbcc` |
| Toolchain | `rustc 1.94.1 (e408947bf 2026-03-25)` / `cargo 1.94.1 (29ea6fb6a 2026-03-24)` |
| Renderer | `pyte 0.8.2` (throwaway venv; capture itself uses only the Python standard library) |
| Host | macOS Apple Silicon (darwin arm64), UTC session date 2026-09-11 |

## Captured artifacts

| File | SHA-256 |
| --- | --- |
| `tui_capture_2026-09-11-memory.raw` | `153bc2be869554ac1850c65af18b91658a1fd025e04a3c271fff9b3d0f5a9a1e` |
| `tui_capture_2026-09-11-memory.txt` | `4b83973388f19e59fbebf77e1660d128740d3fb61894f74632116bf5235a8af9` |

## Render command

~~~bash
python3 -m venv /tmp/grainx-c-render-venv
/tmp/grainx-c-render-venv/bin/pip install pyte==0.8.2
/tmp/grainx-c-render-venv/bin/python demos/capture_tui.py \
  target/debug/grainx \
  demos/tui_capture_2026-09-11-memory.raw \
  demos/tui_capture_2026-09-11-memory.txt \
  5 110 50
~~~

Final-frame check (fails if the label was cleared):

~~~bash
python3 -c 'import sys; lines=open("demos/tui_capture_2026-09-11-memory.txt").read().splitlines(); sys.exit(0 if any("Memory:" in l for l in lines) else 1)'
~~~

Observed final frame (pyte, 110x50), row 18 is the label and rows 19-25 are the memory graph:

~~~text
Memory: 52.4% (12.6GB/24.0GB)
Network I/O: RX 14.0 KB  TX 1.0 KB (last interval)
~~~

The network label keeps its per-interval byte-delta semantics; `KB/s` appears nowhere in the
raw stream.

## Regression test

`tests/tui_memory_label.rs` checks, for several terminal sizes, that `DashboardLayout::memory_label_y()`
is outside the memory graph rectangle (whose clear and braille passes rewrite every cell) and below
the CPU graph clear pass. It fails on the pre-fix row:

~~~bash
cargo test --locked --test tui_memory_label
~~~

## Privacy

Host snapshot fields present: OS name, kernel version, uptime, CPU core percentages, memory totals,
disk label and sizes, top process names, PIDs, per-process CPU and memory sizes. A pattern scan of
the new raw/txt captures found no home-directory paths, credentials, tokens, keys, emails, or IP
addresses. Regenerate on a quiet host if a neutral process list is required.

## Limitations

- Values are one host and one instant, not benchmarks.
- The `CPU Usage:` label still sits inside the CPU graph rectangle and is cleared by the same
  class of pass; this pre-existing issue is out of scope for the memory-label fix.
