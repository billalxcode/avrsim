#!/usr/bin/env python3
"""Fail if text contains non-ASCII characters.

Usage:
    python3 check_ascii.py FILE [FILE ...]
    printf '%s' "text" | python3 check_ascii.py
"""

import sys
from pathlib import Path


def check_text(name: str, text: str) -> bool:
    bad = False
    for lineno, line in enumerate(text.splitlines(), 1):
        for colno, ch in enumerate(line, 1):
            if ord(ch) > 127:
                print(
                    f"{name}:{lineno}:{colno}: "
                    f"U+{ord(ch):04X} {ch!r}"
                )
                bad = True
    return bad


def main() -> int:
    args = sys.argv[1:]

    if not args:
        text = sys.stdin.read()
        return 1 if check_text("<stdin>", text) else 0

    bad = False
    for arg in args:
        path = Path(arg)
        if not path.exists():
            print(f"{arg}: not found", file=sys.stderr)
            bad = True
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError as exc:
            print(f"{arg}: invalid UTF-8: {exc}", file=sys.stderr)
            bad = True
            continue
        if check_text(str(path), text):
            bad = True

    return 1 if bad else 0


if __name__ == "__main__":
    raise SystemExit(main())
