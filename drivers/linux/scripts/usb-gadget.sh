#!/usr/bin/env bash
set -euo pipefail

GADGET_NAME="embed_emu"
CONFIGFS="/sys/kernel/config"
GADGET="$CONFIGFS/usb_gadget/$GADGET_NAME"
UDC_NAME="${UDC_NAME:-dummy_udc.0}"

require_root() {
    if [[ "$(id -u)" -ne 0 ]]; then
        echo "error: run this script with sudo" >&2
        exit 1
    fi
}

load_modules() {
    modprobe dummy_hcd
    modprobe libcomposite
}

mount_configfs() {
    if ! mountpoint -q "$CONFIGFS"; then
        mount -t configfs none "$CONFIGFS"
    fi
}

create_gadget() {
    mkdir -p "$GADGET/strings/0x409"
    echo "0x1209" > "$GADGET/idVendor"
    echo "0x0001" > "$GADGET/idProduct"
    echo "0x0100" > "$GADGET/bcdDevice"
    echo "0x0200" > "$GADGET/bcdUSB"
    echo "embed-emu" > "$GADGET/strings/0x409/serialnumber"
    echo "embed-emulation" > "$GADGET/strings/0x409/manufacturer"
    echo "embed-emu Virtual Serial" > "$GADGET/strings/0x409/product"
}

create_config() {
    mkdir -p "$GADGET/configs/c.1/strings/0x409"
    echo "CDC ACM" > "$GADGET/configs/c.1/strings/0x409/configuration"
    echo "250" > "$GADGET/configs/c.1/MaxPower"
}

create_function() {
    mkdir -p "$GADGET/functions/acm.usb0"
    ln -s "$GADGET/functions/acm.usb0" "$GADGET/configs/c.1/acm.usb0"
}

bind_udc() {
    if [[ ! -d "/sys/class/udc/$UDC_NAME" ]]; then
        echo "error: UDC $UDC_NAME not found" >&2
        echo "check: ls /sys/class/udc" >&2
        exit 1
    fi
    if [[ "$(cat "$GADGET/UDC")" == "$UDC_NAME" ]]; then
        return 0
    fi
    echo "$UDC_NAME" > "$GADGET/UDC"
}

gadget_complete() {
    [[ -e "$GADGET/configs/c.1/acm.usb0" ]]
}

tty_in_use() {
    command -v fuser >/dev/null || return 1
    [[ -e /dev/ttyGS0 ]] && fuser -s /dev/ttyGS0 2>/dev/null
}

remove_gadget_dirs() {
    rm -f "$GADGET/configs/c.1/acm.usb0"
    rmdir "$GADGET/configs/c.1/strings/0x409" 2>/dev/null || true
    rmdir "$GADGET/configs/c.1" 2>/dev/null || true
    rmdir "$GADGET/functions/acm.usb0" 2>/dev/null || true
    rmdir "$GADGET/strings/0x409" 2>/dev/null || true
    rmdir "$GADGET" 2>/dev/null || true
}

wait_for_device() {
    for _ in $(seq 1 50); do
        if [[ -e "$1" ]]; then
            return 0
        fi
        sleep 0.1
    done
    return 1
}

fix_device() {
    if [[ -n "${SUDO_USER:-}" ]]; then
        chown "$SUDO_USER" "$1"
    fi
    chmod 0660 "$1"
}

fix_permissions() {
    wait_for_device /dev/ttyGS0 || echo "warning: /dev/ttyGS0 not found" >&2
    wait_for_device /dev/ttyACM0 || echo "warning: /dev/ttyACM0 not found" >&2
    [[ -e /dev/ttyGS0 ]] && fix_device /dev/ttyGS0
    [[ -e /dev/ttyACM0 ]] && fix_device /dev/ttyACM0
}

start() {
    require_root
    load_modules
    mount_configfs
    if [[ -d "$GADGET" ]]; then
        if gadget_complete; then
            echo "gadget already exists: $GADGET"
            bind_udc
            fix_permissions
            return 0
        fi
        stop
    fi
    create_gadget
    create_config
    create_function
    bind_udc
    fix_permissions
    echo "started $GADGET_NAME"
}

stop() {
    require_root
    if tty_in_use; then
        echo "error: /dev/ttyGS0 is still open; stop the bridge first" >&2
        exit 1
    fi
    if [[ ! -d "$GADGET" ]]; then
        echo "gadget not found: $GADGET"
        return 0
    fi
    if [[ -e "$GADGET/UDC" ]]; then
        echo "" > "$GADGET/UDC"
    fi
    remove_gadget_dirs
    echo "stopped $GADGET_NAME"
}

status() {
    echo "gadget: $GADGET"
    echo "udc: $UDC_NAME"
    ls -l /sys/class/udc 2>/dev/null || true
    ls -l /dev/ttyGS0 /dev/ttyACM0 2>/dev/null || true
}

case "${1:-start}" in
    start) start ;;
    stop) stop ;;
    status) status ;;
    *)
        echo "usage: $0 {start|stop|status}" >&2
        exit 2
        ;;
esac
