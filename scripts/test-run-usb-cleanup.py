#!/usr/bin/env python3
"""Verify run-usb.sh stops the bridge before removing the gadget."""

import os
import signal
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "drivers/linux/scripts/run-usb.sh"


def write_executable(path: Path, text: str) -> None:
    path.write_text(text, encoding="ascii")
    path.chmod(0o755)


def wait_for_text(path: Path, needle: str, timeout: float = 3.0) -> bool:
    deadline = time.time() + timeout
    while time.time() < deadline:
        if path.exists() and needle in path.read_text(encoding="ascii"):
            return True
        time.sleep(0.02)
    return False


def make_fake_gadget(path: Path, log: Path, stopped: Path) -> None:
    text = f"""#!/usr/bin/env bash
log="{log}"
case "$1" in
    start) echo gadget_start >> "$log" ;;
    stop)
        if [[ ! -f "{stopped}" ]]; then
            echo stop_before_driver >> "$log"
        fi
        echo gadget_stop >> "$log"
        ;;
esac
"""
    write_executable(path, text)


def make_fake_driver(path: Path, log: Path, stopped: Path) -> None:
    text = f"""#!/usr/bin/env bash
log="{log}"
echo driver_start >> "$log"
trap 'touch "{stopped}"; echo driver_stop >> "$log"; exit 0' TERM INT
while [[ ! -f "{stopped}" ]]; do
    sleep 0.02
done
"""
    write_executable(path, text)


def run_test(temp: Path) -> int:
    log = temp / "events.log"
    stopped = temp / "driver.stopped"
    fake_sudo = temp / "sudo"
    fake_gadget = temp / "gadget"
    fake_driver = temp / "driver"
    write_executable(fake_sudo, '#!/usr/bin/env bash\nexec "$@"\n')
    make_fake_gadget(fake_gadget, log, stopped)
    make_fake_driver(fake_driver, log, stopped)
    env = os.environ.copy()
    env.update(
        {
            "USB_DRIVER": str(fake_driver),
            "USB_GADGET_SCRIPT": str(fake_gadget),
            "USB_SUDO": str(fake_sudo),
            "USB_SOCKET": str(temp / "usb.sock"),
        }
    )
    process = subprocess.Popen(
        [str(SCRIPT)],
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    try:
        if not wait_for_text(log, "driver_start"):
            print("error: fake driver did not start")
            return 1
        process.send_signal(signal.SIGTERM)
        process.wait(timeout=5)
    finally:
        stopped.touch()
        if process.poll() is None:
            process.kill()
            process.wait()
    events = log.read_text(encoding="ascii").splitlines()
    expected = ["gadget_start", "driver_start", "driver_stop", "gadget_stop"]
    if events != expected:
        print(f"error: unexpected cleanup order: {events}")
        return 1
    return 0


def main() -> int:
    with tempfile.TemporaryDirectory() as temp:
        result = run_test(Path(temp))
    if result == 0:
        print("run-usb cleanup test passed")
    return result


if __name__ == "__main__":
    sys.exit(main())
