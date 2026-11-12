# Drivers

Direktori ini disiapkan untuk driver virtual USB di luar workspace Rust.

**Status:** bridge C++ dan setup ConfigFS ACM selesai. Verifikasi `/dev/ttyACM0` menunggu root dan `dummy_hcd`.

Dokumen riset: [`docs/55-usb-virtual-serial.md`](../docs/55-usb-virtual-serial.md).
Todolist: [`TODO.md`](../TODO.md), bagian P18 dan P19.

---

## 1. Tujuan

Menyediakan virtual USB serial port yang terdeteksi Arduino IDE sebagai
port serial nyata, tanpa menulis kernel module C++.

Target awal:

- Linux: USB CDC ACM melalui ConfigFS ACM dan `dummy_hcd`.
- Host melihat `/dev/ttyACM0`.
- Driver C++ melakukan bridge `/dev/ttyGS0` ke Unix socket.
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
    README.md
    include/embed_emu_usb/
    src/
    tests/
  windows/
    README.md
  macos/
    README.md
```

Scaffold Linux ada di [`linux/README.md`](linux/README.md).
Rencana Windows ada di [`windows/README.md`](windows/README.md).
Rencana macOS ada di [`macos/README.md`](macos/README.md).

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

## 5. Build dan jalankan

```bash
./scripts/build-drivers.sh
python3 scripts/test-usb-bridge.py
drivers/linux/scripts/run-usb.sh
```

Output:

```text
build/drivers-linux/embed-emu-usb
```

Terminal lain:

```bash
./target/release/embed-emu usb --socket /tmp/embed-emu-usb.sock
```

---

## 6. Pembagian kerja

| Komponen | Bahasa | Tanggung jawab |
|---|---|---|
| `sim-cli` | Rust | CLI, CPU, flash, STK500 |
| `embed-emu-usb` | C++ | Bridge tty gadget ke Unix socket |
| `dummy_hcd` | C kernel | Host controller dan UDC virtual |
| `usb_f_acm` | C kernel | Gadget function ACM |
| `cdc_acm` | C kernel | Driver host Linux |

---

## 7. Keputusan

Sudah dipilih:

- Linux: ConfigFS ACM dan `dummy_hcd`.
- Build: CMake.
- IPC: Unix socket.
- Bridge C++ di user space.

Belum diputuskan:

- Apakah FunctionFS diperlukan untuk kontrol descriptor penuh.
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
