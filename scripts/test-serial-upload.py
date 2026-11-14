#!/usr/bin/env python3
"""Smoke test for the emucad virtual serial STK500 upload."""

import os
import select
import shutil
import signal
import subprocess
import sys
import tempfile
import time
from pathlib import Path

from stk500_client import BLINK, ECHO, UART, read_uart, upload, wait_for

ROOT = Path(__file__).resolve().parent.parent


def wait_for_led(process: subprocess.Popen, timeout: float = 5.0) -> bool:
    deadline = time.time() + timeout
    while time.time() < deadline:
        ready, _, _ = select.select([process.stdout], [], [], 0.2)
        if not ready:
            continue
        line = process.stdout.readline()
        if "led_on=true" in line:
            return True
    return False


def wait_for_cleanup(link: Path, socket: Path, timeout: float = 2.0) -> bool:
    deadline = time.time() + timeout
    while time.time() < deadline:
        if not link.exists() and not socket.exists():
            return True
        time.sleep(0.02)
    return False


def main() -> int:
    if shutil.which("socat") is None:
        print("error: socat is not installed")
        return 2
    binary = ROOT / "target/release/emucad"
    if not binary.exists():
        binary = ROOT / "target/debug/emucad"
    if not binary.exists():
        print("error: build the CLI first with cargo build -p sim-cli")
        return 2
    with tempfile.TemporaryDirectory() as temp:
        link = Path(temp) / "tty"
        socket = Path(temp) / "sock"
        process = subprocess.Popen(
            [str(binary), "serial", "--link", str(link), "--socket", str(socket)],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        try:
            if not wait_for(link):
                print("error: virtual port was not created")
                return 1
            fd = os.open(link, os.O_RDWR | os.O_NOCTTY)
            upload(fd, BLINK)
            os.close(fd)
            if not wait_for_led(process):
                print("error: firmware did not toggle the LED")
                return 1
            fd = os.open(link, os.O_RDWR | os.O_NOCTTY)
            upload(fd, UART)
            if not read_uart(fd):
                print("error: UART output was not forwarded")
                return 1
            os.close(fd)
            fd = os.open(link, os.O_RDWR | os.O_NOCTTY)
            upload(fd, ECHO)
            os.write(fd, b"Z")
            if not read_uart(fd, b"Z"):
                print("error: UART input was not echoed")
                return 1
            os.close(fd)
        finally:
            process.send_signal(signal.SIGINT)
            try:
                process.wait(timeout=3)
            except subprocess.TimeoutExpired:
                process.kill()
            if not wait_for_cleanup(link, socket):
                print("error: virtual port was not cleaned up")
                return 1
    print("serial upload smoke test passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
