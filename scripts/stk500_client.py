"""Minimal STK500 client helpers for the embed-emu smoke tests."""

import os
import select
import time
from pathlib import Path

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


def frame(command: int, data: bytes = b"") -> bytes:
    return bytes([command]) + data + bytes([0x20])


def wait_for(path: Path, timeout: float = 2.0) -> bool:
    deadline = time.time() + timeout
    while time.time() < deadline:
        if path.exists():
            return True
        time.sleep(0.02)
    return False


_BUFFERS: dict[int, bytes] = {}


def _take_response(fd: int) -> bytes:
    data = _BUFFERS.get(fd, b"")
    for index, byte in enumerate(data):
        if byte in (0x10, 0x00):
            _BUFFERS[fd] = data[index + 1:]
            return data[:index + 1]
    return b""


def read_response(fd: int, timeout: float = 2.0) -> bytes:
    deadline = time.time() + timeout
    while time.time() < deadline:
        response = _take_response(fd)
        if response:
            return response
        ready, _, _ = select.select([fd], [], [], 0.05)
        if not ready:
            continue
        _BUFFERS[fd] = _BUFFERS.get(fd, b"") + os.read(fd, 1024)
    return b""


def send(fd: int, command: int, data: bytes = b"") -> bytes:
    os.write(fd, frame(command, data))
    return read_response(fd)


def upload(fd: int, program: list[int]) -> None:
    if send(fd, 0x30) != b"\x14\x10":
        raise RuntimeError("sync failed")
    if send(fd, 0x55, bytes([0x00, 0x00])) != b"\x14\x10":
        raise RuntimeError("load address failed")
    payload = bytes([0x00, len(program) * 2, ord("F")])
    payload += b"".join(word.to_bytes(2, "little") for word in program)
    if send(fd, 0x64, payload) != b"\x14\x10":
        raise RuntimeError("program page failed")
    if send(fd, 0x51) != b"\x14\x10":
        raise RuntimeError("leave programming mode failed")


def read_uart(fd: int, expected: bytes = b"A", timeout: float = 2.0) -> bool:
    deadline = time.time() + timeout
    data = _BUFFERS.pop(fd, b"")
    while time.time() < deadline:
        if expected in data:
            return True
        ready, _, _ = select.select([fd], [], [], 0.05)
        if not ready:
            continue
        data += os.read(fd, 1024)
    return expected in data
