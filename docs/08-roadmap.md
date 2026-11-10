# 08 — Roadmap

Roadmap ini mengubah visi besar menjadi milestone kecil yang dapat diuji.

> **Prinsip:** setiap milestone harus menghasilkan sesuatu yang bisa dijalankan, diukur, dan didokumentasikan.

---

## 1. Ringkasan Milestone

| Milestone | Nama | Hasil utama |
|---|---|---|
| M0 | Fondasi proyek | Workspace Rust, dokumentasi, CI |
| M1 | Device + solver + VTC | Inverter CMOS dan kurva VTC |
| M2 | Pustaka CMOS | Gerbang, DFF, SRAM cell, ring oscillator |
| M3 | Blok CPU | ALU, register file, control, CPU uji |
| M4 | AVR CPU struktural | Seluruh ISA, blink berjalan |
| M5 | Peripheral inti | GPIO, timer, UART, hello |
| M6 | Peripheral lengkap | SPI, I2C, ADC, EEPROM, WDT |
| M7 | Board struktural | Board + firmware + CLI monitor |
| M8 | Full SPICE & fidelity | AC, noise, corners, sparse, kalibrasi |
| M9 | Release | Dokumentasi, CI, packaging |

---

## 2. M0 — Fondasi Proyek

### Scope

- Workspace Rust.
- Tujuh crate internal.
- Dokumentasi.
- CI.
- Zero dependency.

### Deliverable

- `Cargo.toml` workspace.
- `rust-toolchain.toml`.
- LICENSE, LICENSE-APACHE.
- README dan docs.
- Test kosong hijau.

### Exit Criteria

- `cargo test --workspace` hijau.
- `cargo fmt --check` bersih.
- Tidak ada dependensi eksternal.
- CI Linux/macOS/Windows berjalan.

### Dependencies

- Rust stable terpasang.

### Risiko

- Toolchain belum tersedia.
- CI platform berbeda.

---

## 3. M1 — Device + Solver + VTC Inverter

### Scope

- MOSFET Level 1.
- Resistor, kapasitor, induktor.
- Sumber tegangan/arus.
- MNA.
- Dense LU.
- Newton-Raphson.
- DC operating point.
- DC sweep.
- Transient Backward Euler.
- CSV/VCD.
- Inverter CMOS.

### Deliverable

- CLI `run --circuit inverter --analysis dc-sweep`.
- Kurva VTC CSV.
- Transient switching VCD.
- Test vs ngspice.

### Exit Criteria

- VTC memiliki bentuk yang benar.
- Output rail mendekati VDD/GND.
- Switching threshold berada di sekitar VDD/2.
- Transient rise/fall wajar.
- Perbedaan vs ngspice dalam toleransi.
- Tidak ada NaN/divergence.
- Test regresi berjalan.

### Dependencies

- M0.

### Risiko

- Konvergensi Newton.
- Parameter MOSFET.
- Time step terlalu besar/kecil.

---

## 4. M2 — Pustaka CMOS

### Scope

- Inverter.
- NAND2/NOR2.
- Buffer/tri-state.
- Transmission gate.
- SRAM 6T.
- D latch.
- D flip-flop.
- Characterization harness.
- Ring oscillator.

### Deliverable

- Cell library.
- Truth table test.
- VTC/delay/capacitance report.
- Ring oscillator VCD.

### Exit Criteria

- Semua sel lulus tabel kebenaran.
- DFF menyimpan state dengan benar.
- SRAM cell bistabil.
- Ring oscillator berosilasi.
- Delay masuk akal.
- Test vs ngspice.

### Dependencies

- M1.

### Risiko

- Kapasitansi Meyer tidak akurat.
- SRAM cell sulit konvergen.
- Waktu simulasi meningkat.

---

## 5. M3 — Blok CPU

### Scope

- Full adder.
- 8-bit ripple-carry adder.
- Flag logic.
- ALU bit-slice.
- Register file 32×8.
- PC/IR/SP.
- Address decoder/mux.
- Control PLA/FSM.
- CPU uji kecil.

### Deliverable

- ALU test vector.
- Register file test.
- CPU uji menjalankan program sederhana.
- Trace eksekusi.

### Exit Criteria

- ALU benar untuk semua operasi.
- Flag benar.
- Register file dapat dibaca/ditulis.
- CPU uji menjalankan loop dan branch.
- Trace sesuai harapan.

### Dependencies

- M2.

### Risiko

- Jumlah gerbang besar.
- Control logic kompleks.
- Simulasi melambat.

---

## 6. M4 — AVR CPU Struktural

### Scope

- Seluruh ISA AVR.
- Fetch/decode/execute.
- Memory interface (flash/SRAM behavioral).
- Stack.
- Interrupt controller.
- Timing multi-cycle.
- CLI monitor.
- Firmware blink.

### Deliverable

- CPU AVR struktural.
- Test vector ISA.
- Differential vs simavr.
- Blink `avr-gcc`.
- Disassembler.

### Exit Criteria

- Seluruh instruksi lulus test.
- Blink berjalan.
- Tidak ada illegal instruction.
- Register/SREG/SRAM benar.
- Cycle count sesuai datasheet untuk instruksi yang diuji.
- Monitor dapat step/breakpoint.

### Dependencies

- M3.

### Risiko

- ISA edge case.
- Timing 16 MHz tidak tercapai.
- Memori behavioral mismatch.
- Waktu simulasi sangat lama.

---

## 7. M5 — Peripheral Inti

### Scope

- GPIO lengkap.
- Timer0/1/2.
- UART TX/RX.
- Interrupt integration.
- Pin electrical model.
- Firmware blink + UART hello.
- Timer interrupt.

### Deliverable

- Board netlist MCU + LED + resistor + UART host.
- Firmware demo.
- VCD pin.
- Output UART.

### Exit Criteria

- Blink berjalan.
- UART “hello” benar.
- Timer interrupt berjalan.
- GPIO input/pull-up benar.
- Pin electrical model terdokumentasi.

### Dependencies

- M4.

### Risiko

- UART structural besar.
- Timer timing.
- Interrupt race.
- Simulasi lambat.

---

## 8. M6 — Peripheral Lengkap

### Scope

- SPI.
- I2C/TWI.
- ADC.
- Analog comparator.
- EEPROM.
- Watchdog.
- Sleep.
- Fuses.
- POR/brownout.

### Deliverable

- Driver test per peripheral.
- Firmware suite.
- Fidelity matrix update.

### Exit Criteria

- SPI loopback.
- I2C master/slave.
- ADC membaca tegangan.
- EEPROM read/write.
- Watchdog reset.
- Sleep/wake.
- Fuse clock.

### Dependencies

- M5.

### Risiko

- ADC butuh solver analog.
- I2C arbitration.
- Banyak peripheral sekaligus.

---

## 9. M7 — Board Struktural

### Scope

- Board lengkap.
- LED/resistor/tombol.
- UART host.
- Loader HEX/ELF.
- CLI monitor.
- VCD/CSV.
- Demo end-to-end.

### Deliverable

- `embed-emu run --board uno --firmware blink.hex`.
- `embed-emu monitor`.
- Dokumentasi contoh.

### Exit Criteria

- Demo reproducible.
- Output UART benar.
- VCD dapat dibuka GTKWave.
- Monitor berfungsi.
- Board profile terdokumentasi.

### Dependencies

- M5, M6.

### Risiko

- Integrasi.
- Timing.
- Ukuran simulasi.

---

## 10. M8 — Full SPICE & Fidelity

### Scope

- AC small-signal.
- Noise.
- Distortion.
- Sensitivity.
- Temperature.
- Corners.
- Full device library.
- Sparse solver.
- Adaptive step.
- Calibration.
- Golden suite.

### Deliverable

- Analisis lengkap.
- Device library lengkap.
- Fidelity matrix final.
- Golden suite.

### Exit Criteria

- Seluruh analisis yang dideklarasikan lulus.
- Golden vs ngspice dalam toleransi.
- Cross-platform determinism.
- Performa terdokumentasi.

### Dependencies

- M1–M7.

### Risiko

- Implementasi analisis kompleks.
- Kalibrasi tanpa lab.
- Waktu pengembangan.

---

## 11. M9 — Release

### Scope

- Dokumentasi pengguna.
- API docs.
- CI lintas platform.
- Packaging.
- Lisensi.
- Roadmap update.

### Deliverable

- v0.1.
- Binary Linux/macOS/Windows.
- Contoh.
- Fidelity report.

### Exit Criteria

- Semua test hijau.
- Dokumentasi lengkap.
- Lisensi jelas.
- Rilis dapat direproduksi.

---

## 12. Urutan Eksekusi yang Disarankan

```
M0 → M1 → M2 → M3 → M4 → M5 → M6 → M7 → M8 → M9
```

Namun, beberapa pekerjaan bisa paralel:

- Pustaka device dan solver (M1) sebelum CPU.
- Pustaka sel (M2) setelah inverter.
- CPU (M3/M4) setelah sel stabil.
- Peripheral (M5/M6) setelah CPU.
- Board (M7) setelah peripheral inti.
- Full SPICE (M8) bisa dimulai kapan saja setelah M1, tetapi tidak memblokir M2–M7.

---

## 13. Phase Gates

Sebelum pindah milestone:

1. Exit criteria terpenuhi.
2. Test regresi hijau.
3. Dokumentasi diperbarui.
4. Fidelity matrix diperbarui.
5. Tidak ada dependensi eksternal.
6. Tidak ada `TODO` kritis yang menggantung.

Jika gate tidak terpenuhi, milestone belum selesai.

---

## 14. Estimasi Kualitatif

| Milestone | Effort |
|---|---|
| M0 | Hari–minggu |
| M1 | Minggu–bulan |
| M2 | Bulan |
| M3 | Bulan |
| M4 | Bulan–tahun |
| M5 | Bulan–tahun |
| M6 | Tahun |
| M7 | Tahun |
| M8 | Tahun |
| M9 | Berkelanjutan |

Angka ini bukan janji; hanya indikasi skala.

---

## 15. Keputusan Terbuka

| ID | Pertanyaan |
|---|---|
| R01 | Apakah M8 dijalankan paralel dengan M2–M7? |
| R02 | Apakah ada milestone M4a (subset ISA) sebelum full ISA? |
| R03 | Apakah M5 memakai UART behavioral sementara? |
| R04 | Kapan sparse solver dimulai? |
| R05 | Kapan GUI dipertimbangkan? |
