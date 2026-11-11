# 24 - Fidelity Matrix dan Batasan Model

Dokumen ini mencatat tingkat fidelity aktual proyek.

---

## 1. Matriks Fidelity

| Bagian | Tingkat | Status |
|---|---|---|
| MOSFET Level 1 | Device model | Ada |
| Resistor, kapasitor, induktor | Device model | Ada |
| Sumber tegangan dan arus | Device model | Ada |
| Dioda | Device model | Ada |
| Inverter, NAND, NOR, gate | Structural analog | Ada |
| D flip-flop, latch, SRAM6T | Structural analog | Ada, sebagian test struktural |
| Digital simulator | Gate-level dua-state | Ada |
| ALU arithmetic/logic/shift | Gate-level struktural | Ada |
| Multiplier | Gate-level struktural | Ada |
| Register bank | DFF digital | Ada |
| CPU control | Behavioral Rust | Belum struktural |
| SREG, PC, SP | State Rust | Belum DFF struktural |
| Flash program | Behavioral | Ada |
| Data space | Behavioral | Ada |
| GPIO | DFF dua-state | Ada |
| Timer0 counter | DFF digital | Ada |
| Timer0 prescaler | Behavioral | Belum struktural |
| Timer1 dan Timer2 | Counter struktural, control behavioral | Sebagian |
| UART | Behavioral | Ada |
| SPI, TWI, ADC | Behavioral | Ada |
| EEPROM, watchdog, fuse | Behavioral | Ada |
| Board LED/resistor/button | Behavioral | Ada |
| Intel HEX | Parser | Ada |
| AC analysis | Belum | Tidak ada |
| Noise, distortion, sensitivity | Belum | Tidak ada |
| Sparse matrix/LU | Belum | Tidak ada |
| Adaptive step/LTE | Belum | Tidak ada |
| Golden ngspice | Belum | Tool tidak tersedia |
| Determinisme macOS/Windows | Belum | Lingkungan tidak tersedia |

---

## 2. Batasan Analisis

- Hanya DC operating point, DC sweep, dan transient Backward Euler.
- Belum ada AC small-signal.
- Belum ada noise analysis.
- Belum ada distortion atau sensitivity.
- Belum ada temperature dan process corner.
- Solver masih dense.
- Belum ada adaptive time step atau LTE.
- Belum ada trapezoidal atau Gear integration.
- Belum ada golden suite terhadap ngspice.

---

## 3. Batasan Device

- MOSFET hanya Level 1 Shichman-Hodges.
- Belum ada model kapasitansi Meyer lengkap.
- Belum ada BJT dan JFET.
- Belum ada transmission line.
- Dioda memakai model sederhana.
- Parameter belum dikalibrasi terhadap silikon.

---

## 4. Batasan Digital dan CPU

- Digital simulator hanya dua-state.
- Tidak ada delay gerbang.
- Tidak ada high-Z, kontensi, atau nilai X.
- Control CPU masih behavioral.
- SREG, PC, dan SP masih state Rust.
- Belum ada timing silikon penuh.
- MULS dan MULSU belum dihubungkan ke CPU.
- Belum ada differential test terhadap simavr.

---

## 5. Batasan Board dan Firmware

- Board belum memakai netlist analog penuh.
- LED hanya model arus dan forward voltage.
- Belum ada debounce tombol.
- Belum ada loader ELF dan simbol.
- `avr-gcc` belum tersedia.
- Blink baru dijalankan dari encoding manual.
- Belum ada firmware UART hello hasil kompilasi.
- Belum ada test end-to-end firmware di board analog.

---

## 6. Kalibrasi

Parameter transistor saat ini:

| Parameter | Nilai |
|---|---|
| `vto` | 0.7 V |
| `kp` | 1e-3 |
| `lambda` | 0.02 |
| `l` | 1e-6 m |
| `w` NMOS | 1e-6 m |
| `w` PMOS | 2e-6 m |

Nilai ini adalah parameter model awal, bukan hasil kalibrasi silikon.
Kalibrasi pin-level dan datasheet-level belum didokumentasikan sebagai hasil
pengukuran.

---

## 7. Kesimpulan

Proyek sudah memiliki fondasi device, solver DC/transient, sel CMOS,
simulator digital, CPU AVR subset, peripheral behavioral, board model, dan CLI.
Yang belum adalah fidelity penuh: AC/noise, device lanjutan, solver sparse,
kalibrasi, golden suite, dan firmware `avr-gcc`.
