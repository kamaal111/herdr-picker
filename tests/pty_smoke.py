"""Exercise the release binary in a real PTY with a fake Herdr CLI (no server)."""

import errno
import fcntl
import json
import os
import pty
import selectors
import struct
import subprocess
import tempfile
import termios
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BINARY = ROOT / "target/release/herdr-picker"
FIXTURE = ROOT / "tests/fixtures/snapshot.json"
FAKE = """#!/usr/bin/env python3
import json, os, sys, time
with open(os.environ["PICKER_TEST_LOG"], "a") as log:
    log.write(json.dumps(sys.argv[1:]) + "\\n")
args = sys.argv[1:]
if args == ["api", "snapshot"]:
    if os.environ.get("PICKER_TEST_WAIT"):
        time.sleep(30)
    print(open(os.environ["PICKER_TEST_FIXTURE"]).read())
elif args[:2] in (["tab", "focus"], ["workspace", "focus"]) and os.environ.get("PICKER_TEST_FAIL") == args[2]:
    print(json.dumps({"error":{"code":"not_found","message":"destination closed"}}))
    sys.exit(1)
elif args[:2] == ["workspace", "focus"]:
    print(json.dumps({"result":{"type":"workspace_info", "workspace":{"workspace_id":args[2]}}}))
elif args[:2] == ["tab", "focus"]:
    print(json.dumps({"result":{"type":"tab_info", "tab":{"tab_id":args[2]}}}))
else:
    print(json.dumps({"result":{"type":"ok"}}))
"""


def read_until(fd, needle):
    deadline = time.monotonic() + 8
    received = b""
    with selectors.DefaultSelector() as selector:
        selector.register(fd, selectors.EVENT_READ)
        while needle not in received:
            assert time.monotonic() < deadline, received[-1000:]
            if selector.select(0.1):
                received += os.read(fd, 65536)
    return received


def case(
    keys, mode="hierarchical", failure="", expected=(), exit_code=0, loading=False
):
    with tempfile.TemporaryDirectory(prefix="herdr-picker-pty-") as directory:
        directory = Path(directory)
        fake = directory / "herdr"
        fake.write_text(FAKE)
        fake.chmod(0o700)
        (directory / "config.toml").write_text(f'search_mode = "{mode}"\n')
        log = directory / "calls.jsonl"
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 20, 80, 0, 0))
        before = termios.tcgetattr(slave)
        environment = dict(
            os.environ,
            HERDR_BIN_PATH=str(fake),
            HERDR_PLUGIN_CONFIG_DIR=str(directory),
            PICKER_TEST_LOG=str(log),
            PICKER_TEST_FIXTURE=str(FIXTURE),
            PICKER_TEST_FAIL=failure,
            TERM="xterm-256color",
            PICKER_TEST_WAIT="1" if loading else "",
        )
        process = subprocess.Popen(
            [str(BINARY), "pick"],
            stdin=slave,
            stdout=slave,
            stderr=slave,
            env=environment,
            start_new_session=True,
        )
        try:
            read_until(master, b"Loading" if loading else b"results")
            log_deadline = time.monotonic() + 3
            while not log.exists():
                assert time.monotonic() < log_deadline, "CLI did not start"
                time.sleep(0.01)
            fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 12, 60, 0, 0))
            os.write(master, keys)
            deadline = time.monotonic() + (2 if loading else 8)
            while process.poll() is None:
                assert time.monotonic() < deadline, "picker did not exit"
                with selectors.DefaultSelector() as selector:
                    selector.register(master, selectors.EVENT_READ)
                    if selector.select(0.1):
                        try:
                            os.read(master, 65536)
                        except OSError as error:
                            assert error.errno == errno.EIO
            assert process.returncode == exit_code, process.returncode
            assert termios.tcgetattr(slave) == before, "terminal mode not restored"
            calls = [json.loads(line) for line in log.read_text().splitlines()]
            assert calls == [["api", "snapshot"], *expected], calls
        finally:
            if process.poll() is None:
                process.kill()
                process.wait()
            os.close(master)
            os.close(slave)


case(b"\x1b")
case(b"\x1b", loading=True)
case(b"col\x1b")
case(b"\r", expected=[["workspace", "focus", "w1"]])
case(b"codex\x0e\r", expected=[["tab", "focus", "w2:t7"]])
case(b"server\r", mode="flat", expected=[["tab", "focus", "w2:t2"]])
case(b"exact terminal\r", expected=[["workspace", "focus", "w3"]])
case(b"exact terminal\r", mode="flat", expected=[["workspace", "focus", "w3"]])
case(
    b"\r",
    failure="w1",
    exit_code=1,
    expected=[
        ["workspace", "focus", "w1"],
        ["tab", "focus", "w2:t7"],
        [
            "notification",
            "show",
            "Herdr Picker",
            "--body",
            "Herdr not_found: destination closed",
        ],
    ],
)
print(
    "9 PTY smoke scenarios passed (Escape including slow discovery, filtering, workspace/tab focus, flat mode, failure recovery; resize and terminal cleanup)"
)
