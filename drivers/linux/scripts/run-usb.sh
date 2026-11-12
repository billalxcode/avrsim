#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
driver="${USB_DRIVER:-$root/build/drivers-linux/embed-emu-usb}"
gadget="${USB_GADGET_SCRIPT:-$root/drivers/linux/scripts/usb-gadget.sh}"
sudo_cmd="${USB_SUDO:-sudo}"
socket="${USB_SOCKET:-/tmp/embed-emu-usb.sock}"

if [[ ! -x "$driver" ]]; then
    echo "error: driver not built" >&2
    echo "run: $root/scripts/build-drivers.sh" >&2
    exit 1
fi

driver_pid=""
cleaned=0

cleanup() {
    if [[ "$cleaned" -eq 1 ]]; then
        return
    fi
    cleaned=1
    if [[ -n "$driver_pid" ]]; then
        kill -TERM "$driver_pid" 2>/dev/null || true
        wait "$driver_pid" 2>/dev/null || true
    fi
    "$sudo_cmd" "$gadget" stop >/dev/null 2>&1 || true
}

"$sudo_cmd" "$gadget" start
trap cleanup EXIT INT TERM

echo "USB gadget ready: /dev/ttyACM0"
echo "In another terminal run:"
echo "  $root/target/release/embed-emu usb --socket $socket"
echo "Press Ctrl-C to stop the bridge and remove the gadget."

"$driver" --socket "$socket" --device /dev/ttyGS0 &
driver_pid=$!
wait "$driver_pid"
