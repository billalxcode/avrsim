#!/usr/bin/env python3
"""Verify the bridge reopens its tty for each client after device recreation."""

import os
import pty
import select
import signal
import socket
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DRIVER = ROOT / "build/drivers-linux/emucad-usb"


def wait_for_line(process: subprocess.Popen, needle: str, timeout: float = 3.0) -> bool:
    deadline = time.time() + timeout
    while time.time() < deadline:
        ready, _, _ = select.select([process.stdout], [], [], 0.1)
        if not ready:
            continue
        line = process.stdout.readline()
        if needle in line:
            return True
    return False


def connect(path: Path) -> socket.socket:
    client = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    client.connect(str(path))
    return client


def still_connected(client: socket.socket, timeout: float = 0.5) -> bool:
    ready, _, _ = select.select([client], [], [], timeout)
    if not ready:
        return True
    return client.recv(1) != b""


def main() -> int:
    if not DRIVER.exists():
        print("error: build the C++ driver first")
        return 2
    with tempfile.TemporaryDirectory() as temp:
        temp_path = Path(temp)
        socket_path = temp_path / "usb.sock"
        device_link = temp_path / "tty"
        first_master, first_slave = pty.openpty()
        device_link.symlink_to(os.ttyname(first_slave))
        bridge = subprocess.Popen(
            [str(DRIVER), "--socket", str(socket_path), "--device", str(device_link)],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        second_master = None
        second_slave = None
        try:
            deadline = time.time() + 3
            while time.time() < deadline and not socket_path.exists():
                time.sleep(0.02)
            first = connect(socket_path)
            if not wait_for_line(bridge, "client connected"):
                print("error: first client was not accepted")
                return 1
            first.close()
            if not wait_for_line(bridge, "client disconnected"):
                print("error: first client did not disconnect")
                return 1
            os.close(first_master)
            os.close(first_slave)
            second_master, second_slave = pty.openpty()
            device_link.unlink()
            device_link.symlink_to(os.ttyname(second_slave))
            second = connect(socket_path)
            if not wait_for_line(bridge, "client connected"):
                print("error: second client was not accepted")
                return 1
            if not still_connected(second):
                print("error: second client was disconnected by a stale tty")
                return 1
            second.close()
        finally:
            bridge.send_signal(signal.SIGINT)
            try:
                bridge.wait(timeout=3)
            except subprocess.TimeoutExpired:
                bridge.kill()
            if second_master is not None:
                os.close(second_master)
            if second_slave is not None:
                os.close(second_slave)
    print("USB bridge reopen test passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
