# Drivers

Direktori ini disiapkan untuk driver virtual USB di luar workspace Rust.

**Status:** rencana. Belum ada kode driver.

Dokumen riset: [`docs/55-usb-virtual-serial.md`](../docs/55-usb-virtual-serial.md).
Todolist: [`TODO.md`](../TODO.md), bagian P18 dan P19.

---

## 1. Tujuan

Menyediakan virtual USB serial port yang terdeteksi Arduino IDE sebagai
port serial nyata, tanpa menulis kernel module C++.

Target awal:

- Linux: USB CDC ACM melalui FunctionFS.
- Host melihat `/dev/ttyACM0`.
- Driver C++ berjalan di user space.
- Protokol STK500 tetap di Rust.
- Komunikasi Rust dan driver melalui Unix domain socket.

Target berikutnya:

- Windows: server USB/IP C++ dengan client `usbip-win2`.
- macOS: PTY dan discovery plugin, atau DriverKit jika diperlukan.

---

## 2. Struktur

```text
drivers/
  README.md
  linux/
    CMakeLists.txt
    src/
    include/
  windows/
  macos/
```

Folder `windows/` dan `macos/` belum diisi sampai keputusan platform
diambil.

---

## 3. Batas komponen

Driver C++ hanya menangani transport USB:

- Descriptor USB.
- Control request.
- Endpoint bulk dan interrupt.
- Line coding.
- DTR dan RTS.
- Koneksi IPC ke simulator.

Driver C++ tidak menangani:

- CPU AVR.
- Flash AVR.
- Protokol STK500.
- Parser Intel HEX.
- Logika board.

Semua logika simulasi tetap di crate Rust.

---

## 4. Aturan

- C++17.
- Build dengan CMake.
- Tanpa kernel module pada fase pertama.
- Tanpa `unsafe` di Rust.
- Root hanya untuk setup `dummy_hcd` dan ConfigFS.
- IPC default: Unix domain socket.
- Driver harus menutup endpoint dan gadget saat proses berhenti.
- Semua teks dan kode baru harus ASCII.
- Lisensi mengikuti proyek, yaitu MIT.

---

## 5. Rencana build

```bash
cmake -S drivers/linux -B build/drivers-linux
cmake --build build/drivers-linux
```

Output awal:

```text
build/drivers-linux/embed-emu-usb
```

Binary driver dijalankan oleh script setup atau oleh `embed-emu` sebagai
proses anak.

---

## 6. Pembagian kerja

| Komponen | Bahasa | Tanggung jawab |
|---|---|---|
| `sim-cli` | Rust | CLI, CPU, flash, STK500 |
| `embed-emu-usb` | C++ | USB CDC ACM, endpoint, IPC |
| `dummy_hcd` | C kernel | Host controller dan UDC virtual |
| `usb_f_acm` | C kernel | Gadget function ACM |
| `cdc_acm` | C kernel | Driver host Linux |

---

## 7. Keputusan yang belum diambil

- FunctionFS, USB/IP, atau PTY untuk Linux.
- Build system: CMake atau Make.
- IPC: Unix socket, pipe, atau shared memory.
- Apakah driver C++ menjadi komponen resmi rilis.
- Cara distribusi untuk Windows dan macOS.

---

## 8. Verifikasi awal

- `lsusb` melihat device dengan VID/PID yang ditentukan.
- `/dev/ttyACM0` muncul.
- Arduino IDE melihat port.
- Upload STK500 berhasil.
- Serial Monitor TX dan RX berfungsi.
- Ctrl-C menutup endpoint dan menghapus gadget.
