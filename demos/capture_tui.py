#!/usr/bin/env python3
"""Drive the grainx TUI under a pty, save raw ANSI, and render it with pyte.

Usage:
  python3 capture_tui.py <binary> <raw-out> [text-out] [seconds] [cols] [rows]

The TUI is spawned on a fresh pty, allowed to render for the given duration,
then 'q' is sent so the process exits through its normal input loop. The raw
byte stream is always written; if pyte is importable, a plain-text screen
render is written to <text-out> (default: stdout).
"""
import fcntl
import os
import pty
import select
import struct
import sys
import termios
import time


def main() -> int:
    if len(sys.argv) < 3:
        print(__doc__, file=sys.stderr)
        return 2
    binary = os.path.abspath(sys.argv[1])
    raw_out = os.path.abspath(sys.argv[2])
    text_out = sys.argv[3] if len(sys.argv) > 3 else None
    seconds = float(sys.argv[4]) if len(sys.argv) > 4 else 4.5
    cols = int(sys.argv[5]) if len(sys.argv) > 5 else 100
    rows = int(sys.argv[6]) if len(sys.argv) > 6 else 32

    pid, fd = pty.fork()
    if pid == 0:
        os.environ["TERM"] = "xterm-256color"
        os.execv(binary, [binary, "monitor"])
    fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", rows, cols, 0, 0))

    chunks = []
    start = time.time()
    quit_sent = False
    while time.time() - start < seconds + 2.0:
        timeout = max(0.0, seconds - (time.time() - start)) if not quit_sent else 0.5
        ready, _, _ = select.select([fd], [], [], min(0.2, timeout) or 0.05)
        if ready:
            try:
                data = os.read(fd, 65536)
            except OSError:
                break
            if not data:
                break
            chunks.append(data)
        if not quit_sent and time.time() - start >= seconds:
            os.write(fd, b"q")
            quit_sent = True
        if quit_sent and not ready:
            break

    try:
        os.close(fd)
    except OSError:
        pass
    try:
        os.waitpid(pid, 0)
    except ChildProcessError:
        pass

    raw = b"".join(chunks)
    with open(raw_out, "wb") as handle:
        handle.write(raw)
    print(f"raw ANSI: {len(raw)} bytes -> {raw_out}")

    try:
        import pyte
    except ImportError:
        print("pyte not importable; skipping text render", file=sys.stderr)
        return 0
    screen = pyte.Screen(cols, rows)
    pyte.ByteStream(screen).feed(raw)
    rendered = "\n".join(line.rstrip() for line in screen.display) + "\n"
    if text_out:
        with open(text_out, "w", encoding="utf-8") as handle:
            handle.write(rendered)
        print(f"text render -> {text_out}")
    else:
        sys.stdout.write(rendered)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
