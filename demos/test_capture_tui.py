#!/usr/bin/env python3
"""Black-box tests and hidden child fixtures for capture_tui.py."""

import json
import os
import shutil
import signal
import subprocess
import sys
import tempfile
import termios
import time
import unittest
from pathlib import Path


CAPTURE = Path(__file__).with_name("capture_tui.py")
FIXTURE_ENV = "GRAINX_CAPTURE_TEST_FIXTURE"


def run_fixture(mode):
    if mode == "sentinel":
        signal.signal(signal.SIGHUP, signal.SIG_IGN)
        signal.signal(signal.SIGTERM, signal.SIG_IGN)
        wait_forever()

    attrs = termios.tcgetattr(0)
    attrs[3] &= ~(termios.ECHO | termios.ECHONL | termios.ICANON)
    attrs[6][termios.VMIN] = 1
    attrs[6][termios.VTIME] = 0
    termios.tcsetattr(0, termios.TCSANOW, attrs)

    frames = b"".join(f"Frame: {number}\r\n".encode() for number in range(1, 6))
    if mode == "empty":
        wait_for_q()
        return 0
    if mode == "repeated":
        os.write(1, b"Frame: 1\r\n" * 5)
        wait_for_q()
        return 0
    if mode == "early":
        os.write(1, frames)
        return 0

    if mode == "term":
        signal.signal(signal.SIGTERM, lambda _signum, _frame: sys.exit(0))
    if mode == "kill":
        signal.signal(signal.SIGHUP, signal.SIG_IGN)
        signal.signal(signal.SIGTERM, signal.SIG_IGN)

    os.write(1, frames)
    wait_for_q()
    if mode == "positive":
        return 0
    if mode == "exit23":
        return 23
    if mode == "post_q":
        os.write(1, b"AFTER_Q_ERASE\r\n")
        return 0
    if mode == "term":
        wait_forever()
    if mode == "kill":
        wait_forever()
    raise RuntimeError(f"unknown fixture: {mode}")


def wait_for_q():
    while os.read(0, 1) != b"q":
        pass


def wait_forever():
    while True:
        try:
            os.read(0, 1)
        except InterruptedError:
            pass


if __name__ == "__main__" and os.environ.get(FIXTURE_ENV):
    raise SystemExit(run_fixture(os.environ[FIXTURE_ENV]))


class CaptureTuiTests(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.TemporaryDirectory(prefix="capture-tui-test-")
        self.addCleanup(self.temp_dir.cleanup)
        self.root = Path(self.temp_dir.name)
        self.fixture = self.root / "capture-tui-fixture-test.py"
        shutil.copy2(__file__, self.fixture)
        self.fixture.chmod(0o755)

    def capture(
        self, mode, *, text_out="-", python_args=("-S",), binary=None, env=None
    ):
        raw_out = self.root / f"{mode}.raw"
        command = [
            sys.executable,
            *python_args,
            str(CAPTURE),
            str(binary or self.fixture),
            str(raw_out),
            str(text_out),
            "0.5",
            "80",
            "24",
        ]
        fixture_env = os.environ.copy()
        fixture_env[FIXTURE_ENV] = mode
        if env:
            fixture_env.update(env)
        completed = subprocess.run(
            command,
            text=True,
            capture_output=True,
            env=fixture_env,
            timeout=8,
            check=False,
        )
        self.assertTrue(completed.stdout.strip(), completed.stderr)
        summary = json.loads(completed.stdout.strip().splitlines()[-1])
        self.assertTrue(raw_out.exists())
        self.assertTrue(summary["child_reaped"], summary)
        return completed, summary, raw_out

    def assert_failed_for(self, completed, summary, reason):
        self.assertNotEqual(completed.returncode, 0)
        self.assertFalse(summary["success"])
        self.assertIn(reason, summary["reasons"])

    def test_positive_fixture_satisfies_monitor_contract(self):
        completed, summary, _ = self.capture("positive")
        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertTrue(summary["success"])
        self.assertEqual(summary["reasons"], [])
        self.assertEqual(summary["child_exit_code"], 0)
        self.assertIsNone(summary["child_signal"])
        self.assertTrue(summary["q_sent"])
        self.assertFalse(summary["timeout"])
        self.assertFalse(summary["term_sent"])
        self.assertFalse(summary["kill_sent"])
        self.assertGreater(summary["pre_q_bytes"], 0)
        self.assertEqual(summary["frame_count"], 5)
        self.assertEqual((summary["frame_first"], summary["frame_last"]), (1, 5))
        self.assertTrue(summary["frames_increasing"])

    def test_usr_bin_false_is_negative_and_reaped(self):
        completed, summary, _ = self.capture("false", binary="/usr/bin/false")
        self.assert_failed_for(completed, summary, "early_exit_before_q")
        self.assertEqual(summary["child_exit_code"], 1)
        self.assertFalse(summary["q_sent"])

    def test_controlled_exit_23_is_reported(self):
        completed, summary, _ = self.capture("exit23")
        self.assert_failed_for(completed, summary, "child_exit_code:23")
        self.assertEqual(summary["child_exit_code"], 23)
        self.assertTrue(summary["q_sent"])

    def test_empty_output_exit_zero_is_rejected(self):
        completed, summary, _ = self.capture("empty")
        self.assert_failed_for(completed, summary, "pre_q_output_empty")
        self.assertEqual(summary["pre_q_bytes"], 0)
        self.assertEqual(summary["child_exit_code"], 0)

    def test_repeated_frame_number_is_rejected(self):
        completed, summary, _ = self.capture("repeated")
        self.assert_failed_for(completed, summary, "frames_not_strictly_increasing")
        self.assertEqual(summary["frame_count"], 5)
        self.assertFalse(summary["frames_increasing"])

    def test_early_exit_before_q_is_rejected(self):
        completed, summary, _ = self.capture("early")
        self.assert_failed_for(completed, summary, "early_exit_before_q")
        self.assertEqual(summary["child_exit_code"], 0)
        self.assertFalse(summary["q_sent"])

    def test_q_timeout_then_term_exit_zero_is_still_failure(self):
        completed, summary, _ = self.capture("term")
        self.assert_failed_for(completed, summary, "quit_timeout")
        self.assertEqual(summary["child_exit_code"], 0)
        self.assertTrue(summary["q_sent"])
        self.assertTrue(summary["timeout"])
        self.assertTrue(summary["term_sent"])
        self.assertFalse(summary["kill_sent"])

    def test_q_and_term_ignored_requires_kill(self):
        completed, summary, _ = self.capture("kill")
        self.assert_failed_for(completed, summary, "quit_timeout")
        self.assertEqual(summary["child_signal"], signal.SIGKILL)
        self.assertTrue(summary["timeout"])
        self.assertTrue(summary["term_sent"])
        self.assertTrue(summary["kill_sent"])

    def test_unrelated_sentinel_survives_owned_group_cleanup(self):
        sentinel_env = os.environ.copy()
        sentinel_env[FIXTURE_ENV] = "sentinel"
        sentinel = subprocess.Popen(
            [str(self.fixture), "monitor"],
            stdin=subprocess.PIPE,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            env=sentinel_env,
            start_new_session=True,
        )
        self.addCleanup(self.reap_sentinel, sentinel)
        time.sleep(0.05)
        completed, summary, _ = self.capture("kill")
        self.assert_failed_for(completed, summary, "quit_timeout")
        self.assertIsNone(sentinel.poll())

    def test_requested_text_requires_pyte(self):
        text_out = self.root / "screen.txt"
        completed, summary, _ = self.capture("positive", text_out=text_out)
        self.assertNotEqual(completed.returncode, 0)
        self.assertTrue(
            any(
                reason.startswith("text_render_failed:")
                for reason in summary["reasons"]
            )
        )
        self.assertFalse(text_out.exists())

    def test_text_render_uses_pre_q_snapshot(self):
        module_dir = self.root / "fake-pyte"
        module_dir.mkdir()
        (module_dir / "pyte.py").write_text(
            "class Screen:\n"
            "    def __init__(self, cols, rows): self.display = ['']\n"
            "class ByteStream:\n"
            "    def __init__(self, screen): self.screen = screen\n"
            "    def feed(self, data): self.screen.display = [data.decode('utf-8')]\n",
            encoding="ascii",
        )
        text_out = self.root / "screen.txt"
        completed, summary, _ = self.capture(
            "post_q", text_out=text_out, env={"PYTHONPATH": str(module_dir)}
        )
        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertTrue(summary["success"])
        rendered = text_out.read_text(encoding="utf-8")
        self.assertIn("Frame: 5", rendered)
        self.assertNotIn("AFTER_Q_ERASE", rendered)

    @staticmethod
    def reap_sentinel(sentinel):
        if sentinel.poll() is None:
            os.killpg(sentinel.pid, signal.SIGKILL)
        if sentinel.stdin:
            sentinel.stdin.close()
        sentinel.wait(timeout=2)


if __name__ == "__main__":
    unittest.main()
