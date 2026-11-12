# Linux Driver

Driver user-space C++ untuk menghubungkan virtual USB serial ke simulator.

**Status:** bridge tty ke Unix socket berjalan. Setup ConfigFS ACM tersedia
dan menunggu verifikasi dengan root dan `dummy_hcd`.

Riset: [`docs/55-usb-virtual-serial.md`](../../docs/55-usb-virtual-serial.md).
Todolist: [`TODO.md`](../../TODO.md), bagian P19.

---

## 1. Arsitektur

```text
Arduino IDE
    |
    v
/dev/ttyACM0
    |
    v
cdc_acm (kernel host)
    |
    v
dummy_hcd (kernel)
    |
    v
usb_f_acm (kernel gadget)
    |
    v
/dev/ttyGS0
    |
    v
embed-emu-usb (C++)
    |
    v
Unix socket
    |
    v
embed-emu usb (Rust)
```

C++ hanya melakukan bridge byte antara `/dev/ttyGS0` dan Unix socket.
USB descriptor dan CDC ACM ditangani kernel melalui ConfigFS ACM.

---

## 2. Prasyarat

- CMake 3.20 atau lebih baru.
- Compiler C++17.
- Linux dengan `dummy_hcd`, `libcomposite`, `usb_f_acm`, dan `cdc_acm`.
- Root untuk setup gadget.
- `sudo` untuk script setup.

Cek:

```bash
cmake --version
g++ --version
ls /sys/class/udc
```

---

## 3. Build

```bash
./scripts/build-drivers.sh
```

---

## 4. Test

```bash
ctest --test-dir build/drivers-linux --output-on-failure
python3 scripts/test-usb-bridge.py
```

Test C++ memeriksa parser opsi.

Test Python memakai PTY sebagai pengganti `/dev/ttyGS0` dan menguji:

- Koneksi Rust ke socket driver.
- Upload blink.
- Output UART.
- Echo RX.
- Cleanup socket.

---

## 5. Menjalankan dengan USB gadget

Terminal 1:

```bash
drivers/linux/scripts/run-usb.sh
```

Script akan:

- Menjalankan `usb-gadget.sh start` dengan sudo.
- Membuat gadget ConfigFS ACM.
- Mengikat ke `dummy_udc.0`.
- Menjalankan `embed-emu-usb`.

Terminal 2:

```bash
./target/release/embed-emu usb --socket /tmp/embed-emu-usb.sock
```

Terminal 3:

Buka Arduino IDE, pilih `/dev/ttyACM0`, lalu upload sketch.

Tekan Ctrl-C di terminal 1 untuk menghentikan bridge dan menghapus gadget.

---

## 6. Setup manual

Mulai gadget:

```bash
sudo drivers/linux/scripts/usb-gadget.sh start
```

Status:

```bash
sudo drivers/linux/scripts/usb-gadget.sh status
```

Hentikan gadget:

```bash
sudo drivers/linux/scripts/usb-gadget.sh stop
```

Driver:

```bash
./build/drivers-linux/embed-emu-usb \
  --socket /tmp/embed-emu-usb.sock \
  --device /dev/ttyGS0
```

---

## 7. Opsi driver

| Opsi | Default | Arti |
|---|---|---|
| `--socket PATH` | `/tmp/embed-emu-usb.sock` | Unix socket untuk simulator |
| `--device PATH` | `/dev/ttyGS0` | Device serial sisi gadget |
| `--help` | - | Tampilkan bantuan |
| `--version` | - | Tampilkan versi |

---

## 8. Batas

- ConfigFS ACM memakai driver kernel, bukan FunctionFS C++.
- Setup membutuhkan root.
- `dummy_hcd` adalah driver testing.
- DTR/RTS belum diuji.
- FunctionFS dan descriptor C++ belum diimplementasikan.
- Windows dan macOS belum diimplementasikan.
- Uji `/dev/ttyACM0` dan Arduino IDE menunggu environment dengan root.
