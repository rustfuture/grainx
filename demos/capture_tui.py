#!/usr/bin/env python3
"""Capture and validate one grainx monitor session under a PTY.

Usage:
  python3 capture_tui.py <binary> <raw-out> [text-out] [seconds] [cols] [rows]

The process is allowed to render until ``seconds`` (default: 4.5), then one
``q`` is sent. A text-out value of ``-`` disables pyte rendering; otherwise
the pre-q screen is rendered to text-out, or to stdout when text-out is
omitted. The final stdout line is always a JSON capture summary.
"""

import errno
import fcntl
import json
import os
import pty
import re
import select
import signal
import struct
import sys
import termios
import time


POLL_SECONDS = 0.05
QUIT_GRACE_SECONDS = 1.0
TERM_GRACE_SECONDS = 1.0
KILL_GRACE_SECONDS = 1.0
FRAME_PATTERN = re.compile(rb"Frame:\s*(\d+)")


def main() -> int:
    if len(sys.argv) < 3 or len(sys.argv) > 7:
        print(__doc__, file=sys.stderr)
        return 2

    binary = os.path.abspath(sys.argv[1])
    raw_out = os.path.abspath(sys.argv[2])
    text_out = sys.argv[3] if len(sys.argv) > 3 else None
    try:
        seconds = float(sys.argv[4]) if len(sys.argv) > 4 else 4.5
        cols = int(sys.argv[5]) if len(sys.argv) > 5 else 100
        rows = int(sys.argv[6]) if len(sys.argv) > 6 else 32
    except ValueError as error:
        print(f"invalid capture argument: {error}", file=sys.stderr)
        return 2
    if seconds <= 0 or cols <= 0 or rows <= 0:
        print("seconds, cols, and rows must be positive", file=sys.stderr)
        return 2

    started = time.monotonic()
    pid, fd = pty.fork()
    if pid == 0:
        os.environ["TERM"] = "xterm-256color"
        os.execv(binary, [binary, "monitor"])

    child_status = None
    child_pgid = None
    chunks = []
    pre_q_raw = b""
    pty_eof = False
    q_sent = False
    timeout = False
    term_sent = False
    kill_sent = False
    reasons = []

    try:
        fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", rows, cols, 0, 0))
        capture_deadline = started + seconds
        child_pgid, child_status, pty_eof = establish_child_group(
            pid, fd, capture_deadline, chunks, child_status, pty_eof
        )
        child_status, pty_eof = poll_child(
            pid, fd, capture_deadline, chunks, child_status, pty_eof
        )

        if child_status is None:
            waited_pid, status = os.waitpid(pid, os.WNOHANG)
            if waited_pid == pid:
                child_status = status

        # Freeze both validation and rendering at the instant before q. One
        # nonblocking read captures bytes already queued at the deadline.
        if child_status is not None:
            drain_pty(fd, chunks, pty_eof)
        elif not pty_eof:
            ready, _, _ = select.select([fd], [], [], 0)
            if ready:
                read_pty(fd, chunks)
        pre_q_raw = b"".join(chunks)

        if child_status is not None:
            reasons.append("early_exit_before_q")
        else:
            try:
                q_sent = os.write(fd, b"q") == 1
            except OSError as error:
                reasons.append(f"q_write_failed:{error.errno}")
            q_write_failed = any(
                reason.startswith("q_write_failed:") for reason in reasons
            )
            if not q_sent and not q_write_failed:
                reasons.append("q_write_failed:short_write")

            quit_deadline = time.monotonic() + QUIT_GRACE_SECONDS
            child_status, pty_eof = poll_child(
                pid, fd, quit_deadline, chunks, child_status, pty_eof
            )
            if child_status is None:
                timeout = True
                reasons.append("quit_timeout")
                if signal_owned_group(pid, child_pgid, signal.SIGTERM):
                    term_sent = True
                else:
                    reasons.append("term_group_verification_failed")

                term_deadline = time.monotonic() + TERM_GRACE_SECONDS
                child_status, pty_eof = poll_child(
                    pid, fd, term_deadline, chunks, child_status, pty_eof
                )
                if child_status is None:
                    if signal_owned_group(pid, child_pgid, signal.SIGKILL):
                        kill_sent = True
                    else:
                        reasons.append("kill_group_verification_failed")

                    kill_deadline = time.monotonic() + KILL_GRACE_SECONDS
                    child_status, pty_eof = poll_child(
                        pid, fd, kill_deadline, chunks, child_status, pty_eof
                    )

        if child_status is None:
            reasons.append("child_not_reaped")
        else:
            drain_pty(fd, chunks, pty_eof)
    finally:
        try:
            os.close(fd)
        except OSError:
            pass

    raw = b"".join(chunks)
    frames = [int(value) for value in FRAME_PATTERN.findall(pre_q_raw)]
    frames_increasing = len(frames) > 1 and all(
        right > left for left, right in zip(frames, frames[1:])
    )
    child_exit_code, child_signal = decode_wait_status(child_status)

    if not pre_q_raw:
        reasons.append("pre_q_output_empty")
    if len(frames) < 5:
        reasons.append("insufficient_pre_q_frames")
    if not frames_increasing:
        reasons.append("frames_not_strictly_increasing")
    if not q_sent:
        reasons.append("q_not_sent")
    if child_exit_code not in (None, 0):
        reasons.append(f"child_exit_code:{child_exit_code}")
    if child_signal is not None:
        reasons.append(f"child_signal:{child_signal}")

    try:
        with open(raw_out, "wb") as handle:
            handle.write(raw)
        print(f"raw ANSI: {len(raw)} bytes -> {raw_out}", file=sys.stderr)
    except OSError as error:
        reasons.append(f"raw_write_failed:{error.errno}")

    if text_out != "-":
        try:
            import pyte

            screen = pyte.Screen(cols, rows)
            pyte.ByteStream(screen).feed(pre_q_raw)
            rendered = "\n".join(line.rstrip() for line in screen.display) + "\n"
            if text_out:
                with open(text_out, "w", encoding="utf-8") as handle:
                    handle.write(rendered)
                print(f"text render -> {text_out}", file=sys.stderr)
            else:
                sys.stdout.write(rendered)
        except Exception as error:
            reasons.append(f"text_render_failed:{type(error).__name__}")
            print(f"text render failed: {error}", file=sys.stderr)

    summary = {
        "success": not reasons,
        "reasons": reasons,
        "child_pid": pid,
        "child_pgid": child_pgid,
        "child_exit_code": child_exit_code,
        "child_signal": child_signal,
        "child_reaped": child_status is not None,
        "q_sent": q_sent,
        "timeout": timeout,
        "term_sent": term_sent,
        "kill_sent": kill_sent,
        "monotonic_duration_seconds": round(time.monotonic() - started, 6),
        "total_bytes": len(raw),
        "pre_q_bytes": len(pre_q_raw),
        "frame_count": len(frames),
        "frame_first": frames[0] if frames else None,
        "frame_last": frames[-1] if frames else None,
        "frame_min": min(frames) if frames else None,
        "frame_max": max(frames) if frames else None,
        "frames_increasing": frames_increasing,
    }
    print(json.dumps(summary, sort_keys=True))
    return 0 if summary["success"] else 1


def poll_child(pid, fd, deadline, chunks, child_status, pty_eof):
    while child_status is None:
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            break
        waited_pid, status = os.waitpid(pid, os.WNOHANG)
        if waited_pid == pid:
            child_status = status
            break

        readable = [] if pty_eof else [fd]
        ready, _, _ = select.select(readable, [], [], min(POLL_SECONDS, remaining))
        if ready:
            pty_eof = not read_pty(fd, chunks)

    return child_status, pty_eof


def establish_child_group(pid, fd, deadline, chunks, child_status, pty_eof):
    while child_status is None:
        waited_pid, status = os.waitpid(pid, os.WNOHANG)
        if waited_pid == pid:
            return None, status, pty_eof
        try:
            child_pgid = os.getpgid(pid)
        except ProcessLookupError:
            child_pgid = None
        if child_pgid == pid:
            return child_pgid, child_status, pty_eof

        remaining = deadline - time.monotonic()
        if remaining <= 0:
            return None, child_status, pty_eof
        readable = [] if pty_eof else [fd]
        ready, _, _ = select.select(readable, [], [], min(POLL_SECONDS, remaining))
        if ready:
            pty_eof = not read_pty(fd, chunks)
    return None, child_status, pty_eof


def read_pty(fd, chunks):
    try:
        data = os.read(fd, 65536)
    except OSError as error:
        if error.errno in (errno.EIO, errno.EBADF):
            return False
        raise
    if not data:
        return False
    chunks.append(data)
    return True


def drain_pty(fd, chunks, pty_eof):
    if pty_eof:
        return
    for _ in range(256):
        ready, _, _ = select.select([fd], [], [], 0)
        if not ready or not read_pty(fd, chunks):
            return


def signal_owned_group(pid, recorded_pgid, sig):
    if recorded_pgid is None or recorded_pgid != pid or recorded_pgid <= 1:
        return False
    try:
        current_pgid = os.getpgid(pid)
    except ProcessLookupError:
        return False
    if current_pgid != recorded_pgid or current_pgid == os.getpgrp():
        return False
    try:
        os.killpg(recorded_pgid, sig)
    except ProcessLookupError:
        return False
    return True


def decode_wait_status(status):
    if status is None:
        return None, None
    if os.WIFEXITED(status):
        return os.WEXITSTATUS(status), None
    if os.WIFSIGNALED(status):
        return None, os.WTERMSIG(status)
    return None, None


if __name__ == "__main__":
    raise SystemExit(main())
