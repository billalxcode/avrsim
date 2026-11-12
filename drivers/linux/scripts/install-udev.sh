#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
source_rule="$root/drivers/linux/scripts/99-embed-emu-usb.rules"
target_rule="/etc/udev/rules.d/99-embed-emu-usb.rules"

if [[ "$(id -u)" -ne 0 ]]; then
    echo "error: run this script with sudo" >&2
    exit 1
fi

install -m 0644 "$source_rule" "$target_rule"
udevadm control --reload-rules
udevadm trigger --subsystem-match=tty
echo "installed $target_rule"
