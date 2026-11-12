#!/usr/bin/env python3
"""Smoke test for the C++ USB bridge and embed-emu usb mode."""

import os
import pty
import select
import signal
import subprocess
import sys
import tempfile
import time
from pathlib import Path

from stk500_client import BLINK, ECHO, UART, read_uart, upload, wait_for

ROOT = Path(__file__).resolve().parent.parent


def wait_for_text(process: subprocess.Popen, needle: str, timeout: float = 3.0) -> bool:
    deadline = time.time() + timeout
    while time.time() < deadline:
        ready, _, _ = select.select([process.stdout], [], [], 0.2)
        if not ready:
            continue
        line = process.stdout.readline()
        if needle in line:
            return True
    return False


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


def wait_for_cleanup(path: Path, timeout: float = 2.0) -> bool:
    deadline = time.time() + timeout
    while time.time() < deadline:
        if not path.exists():
            return True
        time.sleep(0.02)
    return False


def main() -> int:
    driver = ROOT / "build/drivers-linux/embed-emu-usb"
    binary = ROOT / "target/release/embed-emu"
    if not binary.exists():
        binary = ROOT / "target/debug/embed-emu"
    if not driver.exists() or not binary.exists():
        print("error: build the C++ driver and the CLI first")
        return 2
    master, slave = pty.openpty()
    slave_name = os.ttyname(slave)
    with tempfile.TemporaryDirectory() as temp:
        socket = Path(temp) / "usb.sock"
        bridge = subprocess.Popen(
            [str(driver), "--socket", str(socket), "--device", slave_name],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        cli = None
        try:
            if not wait_for(socket):
                print("error: USB bridge socket was not created")
                return 1
            cli = subprocess.Popen(
                [str(binary), "usb", "--socket", str(socket)],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
            )
            if not wait_for_text(bridge, "client connected"):
                print("error: CLI did not connect to the USB bridge")
                return 1
            upload(master, BLINK)
            if not wait_for_led(cli):
                print("error: firmware did not toggle the LED")
                return 1
            upload(master, UART)
            if not read_uart(master):
                print("error: UART output was not forwarded")
                return 1
            upload(master, ECHO)
            os.write(master, b"Z")
            if not read_uart(master, b"Z"):
                print("error: UART input was not echoed")
                return 1
        finally:
            if cli is not None:
                cli.send_signal(signal.SIGINT)
                try:
                    cli.wait(timeout=3)
                except subprocess.TimeoutExpired:
                    cli.kill()
            bridge.send_signal(signal.SIGINT)
            try:
                bridge.wait(timeout=3)
            except subprocess.TimeoutExpired:
                bridge.kill()
            os.close(master)
            os.close(slave)
            if not wait_for_cleanup(socket):
                print("error: USB bridge socket was not cleaned up")
                return 1
    print("USB bridge smoke test passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
