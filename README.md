# embed-emulation

**Native Transistor-Level Emulator** - simulator elektronika native Rust
yang memodelkan transistor, gerbang CMOS, CPU AVR, peripheral, dan board,
serta mampu menjalankan firmware AVR.

> **Status v0.1:** 601 test lulus, 8 crate, crate simulasi zero external
> dependency; dependensi GUI dibatasi pada daftar yang disetujui di
> `AGENTS.md`.
> Tag `v0.1` tersedia. Archive Linux ada di
> `dist/embed-emu-x86_64-unknown-linux-gnu.tar.gz`.

Dokumentasi lengkap: [`docs/README.md`](docs/README.md).
Panduan pengguna: [`docs/43-user-guide.md`](docs/43-user-guide.md).
Fidelity dan batasan: [`docs/24-fidelity.md`](docs/24-fidelity.md).

---

## 1. Apa ini?

Satu binary native Rust yang:

- Memodelkan rangkaian pada level transistor.
- Memiliki solver SPICE sendiri: MNA, Newton-Raphson, DC, transient, AC,
  noise, distorsi, dan sensitivitas.
- Memodelkan sel CMOS, CPU AVR, peripheral, dan board.
- Memuat Intel HEX dan ELF32.
- Berjalan tanpa VM, browser, atau simulator eksternal saat runtime.
- Hanya memakai Rust standard library dan crate internal workspace.
- Tidak memakai `unsafe`.

Target akhirnya adalah **firmware-observable equivalence** untuk board
profile tertentu, bukan replika silikon identik.

---

## 2. Kemampuan saat ini

| Area | Isi |
|---|---|
| Solver analog | DC operating point, DC sweep, transient tetap dan adaptive, AC, noise, distorsi, sensitivitas |
| Device | Resistor, capacitor, inductor, sumber, pulse source, diode, MOSFET N/P, BJT NPN/PNP, JFET N/P, transmission line, VCVS, VCCS, CCCS, CCVS, switch |
| Sel digital | Inverter, NAND, NOR, buffer, DFF, latch, SRAM, adder, ALU |
| AVR | ISA, decoder, ALU, register, SRAM, flash, GPIO, timer, UART, SPI, TWI, ADC |
| Board | LED, resistor, button, UART host, pin electrical, CSV, VCD |
| Format | Intel HEX, ELF32, CSV, VCD digital, VCD analog |
| Virtual serial | STK500 upload, firmware run, TX/RX UART, sketsa Arduino core, cleanup otomatis |
| Virtual USB | C++ bridge, ConfigFS ACM, `/dev/ttyACM0`, STK500v1, upload avrdude |
| CLI | `run`, `monitor`, `disasm`, `bench`, `serial`, `usb` |

---

## 3. Quick start

### 3.1 Build dan test

```bash
cargo build --workspace
cargo test --workspace
```

### 3.2 Contoh runnable

```bash
cargo run -p sim-analysis --example rc_analysis
cargo run -p sim-analysis --example inverter_vtc
cargo run -p sim-cells --example ring_oscillator
cargo run -p sim-avr --example cpu_program
cargo run -p sim-board --example blink
cargo run -p sim-avr --example uart_hello
cargo run -p sim-avr --example timer_interrupt
```

### 3.3 CLI

```bash
./target/debug/embed-emu monitor E005 E013 0F01 9598
./target/debug/embed-emu disasm E005 E013 0F01 9598
./target/debug/embed-emu bench 1000 10
./target/debug/embed-emu run firmware/blink.hex 100000
./target/release/embed-emu serial
./target/release/embed-emu
./target/release/embed-emu usb --socket /tmp/embed-emu-usb.sock
```

Perintah `run` memerlukan file Intel HEX. Contoh firmware assembly ada di
[`firmware/`](firmware/README.md). Menjalankan binary tanpa argumen, atau
menggunakan subcommand `serial`, membuka virtual serial port STK500 melalui
`socat` dan menutupnya otomatis saat Ctrl-C. Arduino IDE GUI mungkin tidak
mendeteksi PTY; gunakan `arduino-cli` atau custom discovery.

### 3.4 Pemeriksaan lengkap

```bash
bash .opencode/skills/clean-code/scripts/check.sh
./scripts/package.sh
```

---

## 4. Struktur workspace

| Crate | Peran |
|---|---|
| `sim-core` | Waktu, satuan, node, device, matrix, error, netlist |
| `sim-device` | Model device dan stamping MNA |
| `sim-analysis` | DC, transient, AC, noise, distorsi, sensitivitas |
| `sim-cells` | Sel CMOS struktural dan pustaka digital |
| `sim-avr` | CPU AVR, decoder, ALU, register, peripheral |
| `sim-board` | Board, LED, button, UART host, Intel HEX, ELF, CSV/VCD |
| `sim-cli` | Binary `embed-emu`, monitor, disassembler, benchmark |
| `drivers/` | Scaffold driver virtual USB C++ (opsional, di luar crate) |

Aturan utama:

- `sim-core` tidak bergantung pada crate lain.
- `sim-cli` adalah adapter dan komposisi.
- Format CSV, VCD, HEX, dan ELF berada di adapter.
- Logika simulasi tidak berada di `main.rs`.

Detail ada di [`docs/02-architecture.md`](docs/02-architecture.md).

---

## 5. Dokumentasi

Mulai dari [`docs/README.md`](docs/README.md) untuk peta lengkap.

Dokumen yang sering dipakai:

| Dokumen | Isi |
|---|---|
| [`AGENTS.md`](AGENTS.md) | Aturan wajib agent dan kontributor |
| [`docs/43-user-guide.md`](docs/43-user-guide.md) | Build, CLI, library, device, firmware |
| [`docs/12-device-models.md`](docs/12-device-models.md) | Katalog device dan parameter |
| [`docs/13-analysis.md`](docs/13-analysis.md) | Solver dan analisis |
| [`docs/24-fidelity.md`](docs/24-fidelity.md) | Akurasi dan batasan model |
| [`docs/29-ac.md`](docs/29-ac.md) | AC small-signal |
| [`docs/53-release-notes.md`](docs/53-release-notes.md) | Isi rilis v0.1 |
| [`docs/57-gui-egui.md`](docs/57-gui-egui.md) | Desain GUI egui dan kanvas CAD |
| [`TODO.md`](TODO.md) | Pekerjaan yang tersisa |
| [`TODO-GUI.md`](TODO-GUI.md) | Backlog GUI egui |

---

## 6. Batas dan status verifikasi

Sudah:

- 601 test workspace.
- fmt, clippy, test, dan build release lulus.
- Paket Linux dapat dibuat.
- Determinisme Linux diuji.
- Firmware `avr-gcc` bare-metal dan sketsa Arduino core
  `Serial.println("Hello World")` berjalan.

Belum:

- Golden comparison dengan ngspice.
- Upload dan Serial Monitor dari GUI Arduino IDE.
- Determinisme macOS dan Windows.
- Eksekusi GitHub Actions.
- Probe generik, monitor UART di GUI, dan grafik analog.
- Parser netlist SPICE.

Model device memakai Level 1 atau model sederhana. Rincian ada di
[`docs/24-fidelity.md`](docs/24-fidelity.md).

---

## 7. Lisensi

MIT. Lihat [`LICENSE`](LICENSE).
