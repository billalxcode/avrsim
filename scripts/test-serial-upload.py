#!/usr/bin/env python3
"""Smoke test for the embed-emu virtual serial STK500 upload."""

import os
import select
import shutil
import signal
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BLINK = [0xE200, 0xB904, 0xB905, 0xEF1F, 0x951A, 0xF7F1, 0xE220, 0x2702, 0xCFF9]
UART = [
    0xE008, 0x9300, 0x00C1, 0xE000, 0x9300, 0x00C4,
    0x9300, 0x00C5, 0xE401, 0x9300, 0x00C6, 0xCFFF,
]
ECHO = [
    0xE108, 0x9300, 0x00C1, 0xE000, 0x9300, 0x00C4,
    0x9300, 0x00C5, 0x9110, 0x00C0, 0xFF17, 0xCFFC,
    0x9100, 0x00C6, 0x9300, 0x00C6, 0xCFF7,
]


def frame(command: int, sequence: int, data: bytes = b"") -> bytes:
    complement = (~sequence) & 0xFF
    checksum = command ^ sequence ^ complement
    for byte in data:
        checksum ^= byte
    return bytes([0x1B, command, sequence, complement]) + data + bytes([checksum])


def wait_for(path: Path, timeout: float = 2.0) -> bool:
    deadline = time.time() + timeout
    while time.time() < deadline:
        if path.exists():
            return True
        time.sleep(0.02)
    return False


def read_response(fd: int, timeout: float = 1.0) -> bytes:
    deadline = time.time() + timeout
    data = b""
    while time.time() < deadline:
        ready, _, _ = select.select([fd], [], [], 0.05)
        if not ready:
            continue
        data += os.read(fd, 1024)
        if b"\x10" in data or b"\x00" in data:
            return data
    return data


def send(fd: int, sequence: int, command: int, data: bytes = b"") -> bytes:
    os.write(fd, frame(command, sequence, data))
    return read_response(fd)


def upload(fd: int, program: list[int]) -> None:
    if send(fd, 1, 0x30) != b"\x14\x10":
        raise RuntimeError("sync failed")
    if send(fd, 2, 0x55, bytes([0x00, 0x00])) != b"\x14\x10":
        raise RuntimeError("load address failed")
    payload = bytes([0x00, len(program) * 2, ord("F")])
    payload += b"".join(word.to_bytes(2, "little") for word in program)
    if send(fd, 3, 0x64, payload) != b"\x14\x10":
        raise RuntimeError("program page failed")
    if send(fd, 4, 0x51) != b"\x14\x10":
        raise RuntimeError("leave programming mode failed")


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


def read_uart(fd: int, expected: bytes = b"A", timeout: float = 2.0) -> bool:
    deadline = time.time() + timeout
    data = b""
    while time.time() < deadline:
        ready, _, _ = select.select([fd], [], [], 0.05)
        if not ready:
            continue
        data += os.read(fd, 1024)
        if expected in data:
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
    binary = ROOT / "target/release/embed-emu"
    if not binary.exists():
        binary = ROOT / "target/debug/embed-emu"
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
