#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
driver="$root/build/drivers-linux/embed-emu-usb"
gadget="$root/drivers/linux/scripts/usb-gadget.sh"
socket="${USB_SOCKET:-/tmp/embed-emu-usb.sock}"

if [[ ! -x "$driver" ]]; then
    echo "error: driver not built" >&2
    echo "run: $root/scripts/build-drivers.sh" >&2
    exit 1
fi

cleanup() {
    sudo "$gadget" stop >/dev/null 2>&1 || true
}

sudo "$gadget" start
trap cleanup EXIT INT TERM

echo "USB gadget ready: /dev/ttyACM0"
echo "In another terminal run:"
echo "  $root/target/release/embed-emu usb --socket $socket"
echo "Press Ctrl-C to stop the bridge and remove the gadget."

"$driver" --socket "$socket" --device /dev/ttyGS0
