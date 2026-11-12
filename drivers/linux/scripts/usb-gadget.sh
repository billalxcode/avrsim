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
    echo "$UDC_NAME" > "$GADGET/UDC"
}

fix_permissions() {
    for _ in $(seq 1 50); do
        if [[ -e /dev/ttyGS0 ]]; then
            if [[ -n "${SUDO_USER:-}" ]]; then
                chown "$SUDO_USER" /dev/ttyGS0
            fi
            chmod 0660 /dev/ttyGS0
            return 0
        fi
        sleep 0.1
    done
    echo "warning: /dev/ttyGS0 not found" >&2
}

start() {
    require_root
    load_modules
    mount_configfs
    if [[ -d "$GADGET" ]]; then
        echo "gadget already exists: $GADGET"
        return 0
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
    if [[ ! -d "$GADGET" ]]; then
        echo "gadget not found: $GADGET"
        return 0
    fi
    if [[ -e "$GADGET/UDC" ]]; then
        echo "" > "$GADGET/UDC"
    fi
    rm -f "$GADGET/configs/c.1/acm.usb0"
    rmdir "$GADGET/configs/c.1/strings/0x409" 2>/dev/null || true
    rmdir "$GADGET/configs/c.1" 2>/dev/null || true
    rmdir "$GADGET/functions/acm.usb0" 2>/dev/null || true
    rmdir "$GADGET/strings/0x409" 2>/dev/null || true
    rmdir "$GADGET" 2>/dev/null || true
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
