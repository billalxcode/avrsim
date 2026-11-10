# embed-emulation

**Native Transistor-Level Emulator** - simulator elektronika native Rust yang memodelkan transistor, gerbang CMOS, CPU AVR, peripheral, dan board, serta mampu menjalankan firmware AVR/Arduino.

> **Status:** desain / brainstorming. Belum ada kode implementasi.
> Dokumen ini adalah spesifikasi proyek, bukan klaim bahwa seluruh fitur sudah selesai.

---

## 1. Apa yang ingin dibangun?

Satu binary native Rust yang:

- Memodelkan **transistor-level** sebagai fondasi.
- Mampu menjalankan **firmware AVR/Arduino**.
- Memodelkan **ATmega328P secara struktural** (clean-room, bukan salinan die).
- Memodelkan komponen board, termasuk IC aktif, secara struktural.
- Memiliki solver SPICE sendiri: MNA, Newton-Raphson, DC, transient, AC, noise, dan lainnya.
- Berjalan **tanpa VM, browser, atau simulator eksternal** saat runtime.
- **Zero external dependency**: hanya Rust standard library dan crate internal workspace.

Target akhirnya adalah **program riset jangka panjang**, bukan proyek sprint. Milestone pertama sengaja kecil: **kurva VTC inverter CMOS**.

---

## 2. Keputusan inti

| Topik | Keputusan |
|---|---|
| Target chip | ATmega328P (Arduino Uno) |
| Fondasi | Transistor-level |
| Model MOSFET | Shichman-Hodges Level 1 |
| Kapasitansi | Model Meyer |
| Solver | MNA, dense LU, Backward Euler, Newton-Raphson |
| CPU | Seluruh ISA AVR, multi-cycle FSM, hardwired/PLA |
| Memori | Register file struktural; SRAM/flash behavioral |
| Peripheral | Semua struktural (target akhir); GPIO/timer/UART lebih dulu |
| Board | IC aktif struktural; pasif model device; host UART behavioral |
| Firmware | `avr-gcc` eksternal, loader HEX/ELF |
| Bahasa | Rust, workspace, zero external dependency |
| Output | CSV untuk DC, VCD untuk transient |
| Lisensi | MIT |

---

## 3. Struktur dokumentasi

Baca berurutan:

| Dokumen | Isi |
|---|---|
| [`AGENTS.md`](AGENTS.md) | Aturan wajib agent dan kontributor |
| [`docs/00-vision.md`](docs/00-vision.md) | Visi, tujuan, non-goal, definisi keberhasilan |
| [`docs/01-decisions.md`](docs/01-decisions.md) | Decision log lengkap dan resolusi kontradiksi |
| [`docs/02-architecture.md`](docs/02-architecture.md) | Arsitektur workspace, netlist, waktu, API |
| [`docs/03-device-solver.md`](docs/03-device-solver.md) | Device model, MNA, solver, analisis, kalibrasi |
| [`docs/04-cpu-avr.md`](docs/04-cpu-avr.md) | CPU AVR struktural, ISA, memori, interupsi |
| [`docs/05-peripherals.md`](docs/05-peripherals.md) | GPIO, timer, UART, SPI, I2C, ADC, dan lainnya |
| [`docs/06-board-firmware.md`](docs/06-board-firmware.md) | Board, komponen, toolchain firmware, demo |
| [`docs/07-verification.md`](docs/07-verification.md) | Strategi verifikasi, oracle, fidelity matrix |
| [`docs/08-roadmap.md`](docs/08-roadmap.md) | Milestone M0-M9 dan exit criteria |
| [`docs/09-risks.md`](docs/09-risks.md) | Risiko, realisme, dan guardrail scope |
| [`TODO.md`](TODO.md) | **Todolist lengkap dari hal terkecil** |

---

## 4. Mulai dari mana?

Urutan implementasi yang disarankan:

1. **P0** - pasang Rust, buat workspace kosong.
2. **P1** - matematika: waktu, satuan, dense matrix, LU.
3. **P3** - device: MOSFET Level 1, R, C, sumber.
4. **P4** - MNA, DC sweep, transient.
5. **M1** - inverter CMOS dan kurva VTC sebagai hasil nyata pertama.
6. Baru setelah itu pustaka CMOS, ALU, CPU AVR, dan peripheral.

Todolist rinci ada di [`TODO.md`](TODO.md).

---

## 5. Definisi "100% native" dan "100% berfungsi"

**100% native** berarti:

- Emulator dikompilasi menjadi kode mesin host.
- Tidak ada VM, browser, WASM, atau proses simulator eksternal saat runtime.
- Solver, device model, parser, dan CLI ditulis sendiri dengan Rust std.
- Firmware boleh dikompilasi dengan `avr-gcc` di luar runtime.

**100% berfungsi** didefinisikan sebagai **firmware-observable equivalence** untuk sebuah *board profile*:

- Register, SRAM, I/O, flag, dan state interupsi benar pada batas instruksi.
- Output UART benar.
- Level pin dan protokol benar.
- Tegangan node analog berada dalam toleransi yang dinyatakan.
- Timing mendekati hardware, dengan toleransi yang didokumentasikan.

Klaim "100%" selalu terikat pada **profile, fitur, dan toleransi** yang diuji, bukan seluruh elektronika di dunia.

---

## 6. Peringatan realisme

Scope yang dipilih adalah scope maksimal:

- Seluruh ATmega328P transistor-level.
- Seluruh peripheral struktural.
- Pustaka SPICE lengkap.
- Seluruh analisis SPICE.
- Seluruh IC aktif board struktural.
- Solo developer, jangka panjang.

Ini setara **program riset multi-person-decade**. Proyek tetap bisa dimulai dan bermanfaat, tetapi harus dijalankan sebagai rangkaian milestone kecil. Baca [`docs/09-risks.md`](docs/09-risks.md) sebelum menetapkan ekspektasi.

---

## 7. Lisensi

MIT. Lihat `LICENSE`.
