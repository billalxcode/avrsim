# Linux Driver

Scaffold proyek driver USB virtual untuk Linux.

**Status:** scaffold build dan parser opsi. Belum ada logika USB.

Riset: [`docs/55-usb-virtual-serial.md`](../../docs/55-usb-virtual-serial.md).
Todolist: [`TODO.md`](../../TODO.md), bagian P19.

---

## 1. Prasyarat

- CMake 3.20 atau lebih baru.
- Compiler C++17, misalnya `g++` atau `clang++`.
- Linux.

Cek:

```bash
cmake --version
g++ --version
```

---

## 2. Build

```bash
cmake -S drivers/linux -B build/drivers-linux
cmake --build build/drivers-linux
```

Atau dari root proyek:

```bash
./scripts/build-drivers.sh
```

---

## 3. Test

```bash
ctest --test-dir build/drivers-linux --output-on-failure
```

Test saat ini memeriksa parser opsi:

- Nilai default.
- Path socket dan device.
- `--help`.
- `--version`.
- Opsi tidak dikenal.
- Nilai yang hilang.

---

## 4. Menjalankan

```bash
./build/drivers-linux/embed-emu-usb --help
./build/drivers-linux/embed-emu-usb --socket /tmp/embed-emu-usb.sock --device /dev/ttyGS0
```

Saat ini program hanya menampilkan konfigurasi dan pesan bahwa transport USB
belum diimplementasikan.

---

## 5. Opsi

| Opsi | Default | Arti |
|---|---|---|
| `--socket PATH` | `/tmp/embed-emu-usb.sock` | Unix socket untuk simulator |
| `--device PATH` | `/dev/ttyGS0` | Device serial sisi gadget |
| `--help` | - | Tampilkan bantuan |
| `--version` | - | Tampilkan versi |

---

## 6. Batas scaffold

Belum ada:

- FunctionFS.
- Descriptor USB.
- Endpoint CDC ACM.
- IPC server.
- Setup `dummy_hcd`.
- DTR/RTS.
- Serial bridge.

Langkah berikutnya ada di P19:

1. Spike FunctionFS dengan `dummy_hcd`.
2. Descriptor CDC ACM.
3. Endpoint dan setup request.
4. IPC ke `sim-cli`.
5. Uji `/dev/ttyACM0`.
