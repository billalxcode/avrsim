# Linux Driver

Driver user-space C++ untuk menghubungkan virtual USB serial ke simulator.

**Status:** bridge C++, ConfigFS ACM, `/dev/ttyACM0`, STK500v1, upload
avrdude 8.0, dan sketsa Arduino core berhasil. Cleanup gadget memakai
urutan aman dan guard `ttyGS0`; verifikasi GUI Arduino IDE dan cleanup
gadget nyata masih terbatas.

Riset: [`docs/55-usb-virtual-serial.md`](../../docs/55-usb-virtual-serial.md).
Tutorial aman: [`docs/56-usb-safe-tutorial.md`](../../docs/56-usb-safe-tutorial.md).
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
python3 scripts/test-run-usb-cleanup.py
```

Test C++ memeriksa parser opsi.

Test Python bridge memakai PTY sebagai pengganti `/dev/ttyGS0` dan menguji:

- Koneksi Rust ke socket driver.
- Upload blink.
- Output UART.
- Echo RX.
- Cleanup socket.

Test cleanup memakai driver dan gadget palsu untuk memastikan bridge
berhenti sebelum `usb-gadget.sh stop` dijalankan.

---

## 5. Menjalankan dengan USB gadget

Terminal 1:

```bash
drivers/linux/scripts/run-usb.sh
```

Script akan:

- Menjalankan `usb-gadget.sh start` dengan sudo.
- Membuat atau memperbaiki gadget ConfigFS ACM.
- Mengikat ke `dummy_udc.0` hanya jika belum terikat.
- Menjalankan `embed-emu-usb`.
- Menunggu bridge keluar sebelum menjalankan cleanup gadget.

Terminal 2:

```bash
./target/release/embed-emu usb --socket /tmp/embed-emu-usb.sock
```

Terminal 3:

Buka Arduino IDE, tutup Serial Monitor sebelum upload, pilih
`/dev/ttyACM0`, lalu upload sketch.

Tekan Ctrl-C di terminal 2 lebih dulu, lalu Ctrl-C di terminal 1. Script
terminal 1 akan menghentikan bridge, menunggu prosesnya keluar, lalu
menghapus gadget. Urutan lengkap ada di
[`docs/56-usb-safe-tutorial.md`](../../docs/56-usb-safe-tutorial.md).

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

`stop` akan menolak berjalan selama `/dev/ttyGS0` masih dibuka bridge.
Hentikan bridge lebih dulu. Jangan memaksa unbind atau menghapus direktori
ConfigFS secara manual.

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

## 8. Permission `/dev/ttyACM0`

Jika Arduino IDE menampilkan:

```text
cannot open port /dev/ttyACM0: Permission denied
```

jangan menjalankan Arduino IDE dengan sudo. Perbaiki permission device.

Cek:

```bash
ls -l /dev/ttyACM0
id
```

Script `usb-gadget.sh start` sudah melakukan `chown` ke `SUDO_USER` dan
`chmod 0660` untuk `/dev/ttyACM0` dan `/dev/ttyGS0`.

Untuk aturan permanen, pasang udev rule:

```bash
sudo drivers/linux/scripts/install-udev.sh
```

Rule memakai `TAG+="uaccess"` agar user yang sedang login mendapat akses.

Alternatif tanpa udev rule:

```bash
sudo usermod -aG dialout "$USER"
```

Logout dan login kembali setelah menambah group.

Jika Arduino IDE dipasang sebagai Flatpak atau Snap, pastikan izin device
serial juga diberikan oleh sandbox-nya.

---

## 9. Batas

- ConfigFS ACM memakai driver kernel, bukan FunctionFS C++.
- Setup membutuhkan root.
- `dummy_hcd` adalah driver testing.
- DTR/RTS belum diuji.
- FunctionFS dan descriptor C++ belum diimplementasikan.
- Windows dan macOS belum diimplementasikan.
- Cleanup gadget dengan urutan aman sudah diuji memakai driver palsu;
  verifikasi gadget kernel nyata setelah perubahan script belum dijalankan.
- Uji upload dari GUI Arduino IDE menunggu verifikasi ulang.
