# Dokumentasi Proyek

Pintu masuk dokumentasi `embed-emulation` versi 0.1.0.

**Status saat ini:** 617 test lulus, 8 crate, crate simulasi zero external
dependency Rust. Dependensi GUI dibatasi pada daftar yang disetujui di
`AGENTS.md`. Tag `v0.1` tersedia.

---

## 1. Mulai di sini

| Kebutuhan | Mulai dari |
|---|---|
| Mencoba simulator | [Panduan pengguna](43-user-guide.md) |
| Memahami arsitektur | [Visi](00-vision.md), [Keputusan](01-decisions.md), [Arsitektur](02-architecture.md) |
| Berkontribusi | [AGENTS.md](../AGENTS.md), [TODO.md](../TODO.md), [Verifikasi](07-verification.md) |
| Memeriksa fidelity | [Fidelity](24-fidelity.md), [Verifikasi](07-verification.md) |
| Merencanakan GUI | [Desain GUI](57-gui-egui.md), [Backlog GUI](../TODO-GUI.md) |
| Melihat isi rilis | [Release notes](53-release-notes.md) |
| Melihat contoh kode | [Examples](../examples/README.md) |

---

## 2. Jalur baca

### 2.1 Pengguna baru

1. [Panduan pengguna](43-user-guide.md): build, CLI, library, dan contoh.
2. [Examples](../examples/README.md): program runnable per crate.
3. [Fidelity](24-fidelity.md): apa yang sudah akurat dan apa yang belum.
4. [Release notes](53-release-notes.md): isi dan batas rilis.

### 2.2 Kontributor kode

1. [AGENTS.md](../AGENTS.md): aturan wajib.
2. [Visi](00-vision.md): tujuan dan non-goal.
3. [Keputusan](01-decisions.md): keputusan desain.
4. [Arsitektur](02-architecture.md): boundary crate dan aliran data.
5. [Device dan solver](03-device-solver.md): MNA dan model device.
6. [Verifikasi](07-verification.md): cara membuktikan perilaku.
7. [TODO.md](../TODO.md): pekerjaan yang tersisa.

### 2.3 Verifikator dan reviewer

1. [Verifikasi](07-verification.md): strategi dan oracle.
2. [Fidelity](24-fidelity.md): matriks keterbatasan.
3. [Determinisme](27-determinism.md): uji hasil berulang.
4. [Performa](38-performance.md): benchmark.
5. [Characterization](42-characterization.md): timing sel.
6. [CI](51-ci.md): workflow lintas platform.

---

## 3. Peta dokumen

Setiap dokumen muncul satu kali pada peta ini.

### 3.1 Panduan

| No | Dokumen | Fokus |
|---|---|---|
| 43 | [Panduan pengguna](43-user-guide.md) | Build, CLI, library, analisis, firmware, troubleshooting |
| 55 | [Virtual USB serial](55-usb-virtual-serial.md) | Riset CDC ACM, dummy_hcd, USB/IP, Windows/macOS |
| 56 | [Tutorial USB aman](56-usb-safe-tutorial.md) | Urutan start, upload, stop, dan pemulihan device busy |
| 57 | [GUI egui](57-gui-egui.md) | Desain canvas CAD, grid, wire, probe, dan grafik |

### 3.2 Fondasi

| No | Dokumen | Fokus |
|---|---|---|
| 00 | [Visi](00-vision.md) | Tujuan, non-goal, definisi sukses |
| 01 | [Keputusan](01-decisions.md) | Keputusan desain dan alasannya |
| 02 | [Arsitektur](02-architecture.md) | Struktur crate, netlist, waktu, API |
| 08 | [Roadmap](08-roadmap.md) | Milestone dan exit criteria |
| 09 | [Risiko](09-risks.md) | Risiko, realisme, dan guardrail |

### 3.3 Simulator analog

| No | Dokumen | Fokus |
|---|---|---|
| 03 | [Device dan solver](03-device-solver.md) | MNA, Newton, device, kalibrasi |
| 10 | [Math API](10-math-api.md) | Waktu, satuan, complex, matrix, error |
| 11 | [Netlist](11-netlist.md) | Node, device, builder, validasi |
| 12 | [Device models](12-device-models.md) | Katalog device dan parameter |
| 13 | [Analysis](13-analysis.md) | DC, transient, AC, noise, distorsi |
| 25 | [Sparse](25-sparse.md) | CSR, sparse LU, degree ordering |
| 26 | [Adaptive](26-adaptive.md) | Adaptive step dan estimasi error |
| 28 | [Integrasi](28-integration.md) | Backward Euler, trapezoidal, Gear2 |
| 29 | [AC](29-ac.md) | Small-signal, sweep, kapasitansi |
| 30 | [Noise](30-noise.md) | Noise termal resistor |
| 31 | [Distorsi](31-distortion.md) | Distorsi harmonik kuasi-statis |
| 32 | [Sensitivitas](32-sensitivity.md) | Sensitivitas DC numerik |
| 33 | [Suhu](33-temperature.md) | Suhu device dan koefisien suhu |
| 34 | [Process corner](34-process-corner.md) | Scaling parameter device |
| 35 | [BJT](35-bjt.md) | Ebers-Moll NPN/PNP dan kapasitansi |
| 36 | [JFET](36-jfet.md) | Shichman-Hodges N/P dan kapasitansi |
| 37 | [Transmission line](37-transmission-line.md) | Line lossless dengan delay |
| 38 | [Performa](38-performance.md) | Benchmark dan optimasi solver |
| 39 | [Kalibrasi](39-calibration.md) | Prosedur dan inventaris parameter |
| 41 | [Pulse source](41-pulse-source.md) | Sumber pulsa untuk transient |
| 42 | [Characterization](42-characterization.md) | Rise, fall, dan propagation delay |
| 44 | [Body effect](44-body-effect.md) | Threshold bias body dan `gmb` |
| 45 | [MOSFET capacitance](45-mosfet-capacitance.md) | Meyer dan overlap capacitance |
| 46 | [Diode nonideal](46-diode-nonideal.md) | Series resistance dan junction capacitance |
| 47 | [Controlled sources](47-controlled-sources.md) | VCVS dan VCCS |
| 48 | [CCCS dan switch](48-cccs-switch.md) | Current-controlled source dan switch |
| 49 | [CCVS](49-ccvs.md) | Current-controlled voltage source |
| 54 | [Analisis lanjutan](54-analisis-lanjutan.md) | Temuan dan rencana P12 |

### 3.4 AVR, sel, dan board

| No | Dokumen | Fokus |
|---|---|---|
| 04 | [CPU AVR](04-cpu-avr.md) | ISA, ALU, register, kontrol, interupsi |
| 05 | [Peripheral](05-peripherals.md) | GPIO, timer, UART, SPI, I2C, ADC |
| 06 | [Board dan firmware](06-board-firmware.md) | Komponen board, loader, toolchain |
| 14 | [CMOS cells](14-cells.md) | Inverter, NAND, NOR, adder, ALU |
| 15 | [Digital](15-digital.md) | Logika dua-state, clock, counter |
| 16 | [AVR decode](16-avr-decode.md) | Decoder, fetch, register bank, ALU |
| 17 | [GPIO](17-gpio.md) | Port B/C/D, DDR, PORT, PIN |
| 18 | [Timer0](18-timer0.md) | Counter, prescaler, CTC, compare |
| 19 | [Timer1 dan Timer2](19-timer1-timer2.md) | 16-bit, capture, compare/PWM |
| 20 | [UART](20-uart.md) | TX/RX, baud, flag, interrupt |
| 21 | [System](21-system.md) | EEPROM, watchdog, fuse, clock |
| 22 | [Board](22-board.md) | LED, resistor, button, UART host, HEX |
| 23 | [SPI, TWI, ADC](23-spi-twi-adc.md) | Peripheral serial dan ADC |
| 40 | [Pin electrical](40-pin-electrical.md) | Tegangan, arus, high-Z, contention |
| 50 | [ELF](50-elf.md) | Loader ELF32, simbol, ELF ke HEX |

### 3.5 Kualitas, operasi, dan rilis

| No | Dokumen | Fokus |
|---|---|---|
| 07 | [Verifikasi](07-verification.md) | Test pyramid, oracle, toleransi |
| 24 | [Fidelity](24-fidelity.md) | Matriks fidelity dan batasan model |
| 27 | [Determinisme](27-determinism.md) | Uji hasil berulang di Linux |
| 51 | [CI](51-ci.md) | Workflow Linux, macOS, Windows |
| 52 | [Lisensi](52-licenses.md) | Lisensi kode, model, dan data |
| 53 | [Release notes](53-release-notes.md) | Isi rilis 0.1.0 dan packaging |

### 3.6 Referensi proyek

| Dokumen | Isi |
|---|---|
| [AGENTS.md](../AGENTS.md) | Aturan wajib agent dan kontributor |
| [TODO.md](../TODO.md) | Daftar tugas dan status milestone |
| [README.md](../README.md) | Ringkasan proyek dan quick start |
| [Examples](../examples/README.md) | Contoh runnable per crate |
| [Firmware](../firmware/README.md) | Firmware contoh dan batas toolchain |
| [Drivers](../drivers/README.md) | Rencana driver virtual USB C++ |
| [LICENSE](../LICENSE) | Lisensi MIT |

---

## 4. Status v0.1

| Aspek | Status |
|---|---|
| Versi workspace | `0.1.0` |
| Tag | `v0.1` |
| Test | 617 lulus |
| Crate | 7 |
| Dependensi eksternal | 0 |
| `unsafe` | Dilarang di seluruh workspace |
| Build release | Lulus |
| Archive Linux | `dist/embed-emu-x86_64-unknown-linux-gnu.tar.gz` |
| Golden ngspice | Belum dijalankan |
| Firmware `avr-gcc` | Bare-metal dan sketsa Arduino core terverifikasi |
| Determinisme macOS/Windows | Belum dijalankan |
| GitHub Actions | Workflow siap, belum dieksekusi |

### 4.1 Test per crate

| Crate | Test |
|---|---|
| `sim-core` | 62 |
| `sim-device` | 48 |
| `sim-analysis` | 108 |
| `sim-cells` | 71 |
| `sim-avr` | 215 |
| `sim-board` | 23 |
| `sim-cli` | 26 |
| `sim-gui` | 63 |
| **Total** | **617** |

---

## 5. Peta kemampuan

| Area | Sudah tersedia | Batas utama |
|---|---|---|
| Solver analog | MNA, dense dan sparse, DC, transient, AC, noise, distorsi, sensitivitas | Golden eksternal belum |
| Device | R, C, L, sumber, diode, MOSFET, BJT, JFET, transmission line, controlled source, switch | Model Level 1 dan sederhana |
| Sel digital | Inverter, NAND, NOR, buffer, DFF, latch, SRAM, adder, ALU | Delay dari sel |
| AVR | ISA, ALU, register, SRAM, flash, GPIO, timer, UART, SPI, TWI, ADC | Timing mendekati, bukan silikon |
| Board | LED, resistor, button, UART host, pin electrical | Belum netlist analog penuh |
| Format | Intel HEX, ELF32, CSV, VCD digital, VCD analog | Belum ada GUI |
| Virtual serial | Upload STK500/Optiboot via `socat`, jalankan firmware, TX/RX UART, sketsa Arduino core, cleanup | RX timing sederhana; IDE discovery terbatas |
| Virtual USB | Bridge C++, ConfigFS ACM, dan perintah `usb` | Upload avrdude dan Serial Monitor via tty berhasil; GUI Arduino IDE belum diuji |
| CLI | `run`, `monitor`, `disasm`, `bench`, `serial` | Belum ada parser netlist SPICE |

---

## 6. Istilah

| Istilah | Arti |
|---|---|
| MNA | Modified Nodal Analysis, metode menyusun persamaan rangkaian |
| DC operating point | Solusi tegangan dan arus pada kondisi tunak |
| Transient | Simulasi terhadap waktu |
| AC small-signal | Respons linear di sekitar titik operasi |
| VTC | Voltage Transfer Characteristic, kurva input-output inverter |
| Fidelity | Tingkat kesesuaian model dengan referensi |
| Corner | Kombinasi parameter device untuk skenario cepat/lambat |
| Netlist | Representasi node dan device suatu rangkaian |
| Adapter | Kode yang menghubungkan core dengan file, CLI, atau format luar |
| Zero dependency | Tidak ada crate eksternal di luar workspace |

---

## 7. Konvensi dokumentasi

- Semua teks baru atau yang diubah harus ASCII.
- Dokumen dan penjelasan memakai bahasa Indonesia.
- Identifier, nama fungsi, dan nama tipe memakai bahasa Inggris.
- Setiap angka test harus sesuai dengan hasil `cargo test --workspace`.
- Setiap tautan relatif harus menunjuk file yang ada.
- Klaim fidelity harus merujuk test atau dokumen bukti.
- Batasan ditulis eksplisit, bukan disembunyikan.

---

## 8. Cara memperbarui dokumentasi

1. Ubah dokumen yang relevan.
2. Perbarui total test di `docs/13-analysis.md`, `docs/29-ac.md`,
   `docs/35-bjt.md`, `docs/36-jfet.md`, `docs/53-release-notes.md`,
   `docs/README.md`, dan `README.md` bila berubah.
3. Jalankan pemeriksaan ASCII:

   ```bash
   python3 .opencode/skills/senior-writing/scripts/check_ascii.py <file>
   ```

4. Jalankan verifikasi workspace:

   ```bash
   bash .opencode/skills/clean-code/scripts/check.sh
   ```

5. Periksa tautan relatif dan status rilis.
