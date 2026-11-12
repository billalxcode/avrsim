# 24 - Fidelity Matrix dan Batasan Model

Dokumen ini mencatat tingkat fidelity aktual proyek.

---

## 1. Matriks Fidelity

| Bagian | Tingkat | Status |
|---|---|---|
| MOSFET Level 1, body effect, Meyer, overlap | Device model | Ada |
| Resistor, kapasitor, induktor | Device model | Ada |
| Sumber tegangan, arus, pulse | Device model | Ada |
| Dioda Shockley, series resistance, junction capacitance | Device model | Ada |
| BJT Ebers-Moll NPN/PNP | Device model | Ada |
| JFET Shichman-Hodges N/P | Device model | Ada |
| VCVS, VCCS, CCVS, CCCS, switch | Device model | Ada |
| Transmission line lossless | Device transient | Ada |
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
| GPIO | DFF digital dan model pin elektrik | Ada |
| Timer0 counter | DFF digital | Ada |
| Timer0 prescaler | Behavioral | Belum struktural |
| Timer1 dan Timer2 | Counter struktural, control behavioral | Sebagian |
| UART | Behavioral | Ada |
| SPI, TWI, ADC | Behavioral | Ada |
| EEPROM, watchdog, fuse | Behavioral | Ada |
| Board LED/resistor/button | Behavioral dan pin voltage | Ada |
| Intel HEX | Parser | Ada |
| ELF32 dan simbol | Parser | Ada |
| AC analysis | Complex MNA dengan linearisasi DC, kapasitansi diode/MOSFET, controlled sources, dan switch | Ada, kapasitansi BJT/JFET belum |
| Noise analysis | Thermal resistor | Ada |
| Transient analysis | Newton per langkah | Ada |
| Distortion analysis | Quasi-static memoryless | Ada |
| Sensitivity analysis | Numerik DC | Ada |
| Temperature dan process corner | Parameter device | Ada |
| Sparse matrix/LU | Solver | Ada |
| Adaptive step/LTE | Solver | Ada |
| Trapezoidal dan Gear2 | Integrasi | Ada |
| Golden ngspice | Belum | Tool tidak tersedia |
| Determinisme macOS/Windows | Workflow siap, belum dijalankan | Lingkungan tidak tersedia |

---

## 2. Batasan Analisis

- AC memakai linearisasi DC; kapasitansi diode dan MOSFET distamp dari bias. Kapasitansi BJT dan JFET belum ada.
- Switch AC memakai conductance dari bias kontrol; kontrol AC tidak dimodulasi.
- Noise hanya noise termal resistor.
- Distortion hanya rangkaian memoryless tanpa intermodulasi.
- Sensitivitas memakai finite difference pada satu parameter dan satu node.
- Temperature hanya memengaruhi parameter device; belum ada self-heating.
- Process corner memakai faktor yang diberikan pengguna.
- Belum ada periodic steady state atau harmonic balance.
- Golden comparison dengan ngspice belum dijalankan.

---

## 3. Batasan Device

- MOSFET Level 1 belum mencakup subthreshold, breakdown, gate resistance,
  dan junction capacitance.
- Meyer capacitance bukan model charge-conserving.
- Dioda belum memiliki breakdown dan diffusion capacitance.
- Kapasitansi dioda dengan `rs > 0` memakai pendekatan terminal.
- BJT belum memiliki Early effect, charge storage, breakdown, dan
  temperature-dependent saturation current.
- JFET belum memiliki gate current, kapasitansi, dan temperature dependence.
- Transmission line hanya lossless, constant step, dan transient.
- Controlled sources dan switch sudah didukung AC dengan linearisasi bias.
- Switch belum memiliki hysteresis atau delay.
- Parameter device sebagian besar masih default, belum dikalibrasi silikon.

---

## 4. Batasan Digital dan CPU

- Digital simulator hanya dua-state.
- Tidak ada delay gerbang di inti digital.
- Tidak ada nilai X di inti digital.
- Pin elektrik menyediakan high-Z dan contention di luar inti digital.
- Control CPU masih behavioral.
- SREG, PC, dan SP masih state Rust.
- Belum ada timing silikon penuh.
- Belum ada differential test terhadap simavr.

---

## 5. Batasan Board dan Firmware

- Board belum memakai netlist analog penuh.
- Pin elektrik bersifat aditif, bukan pengganti inti digital.
- LED hanya model arus dan forward voltage.
- Belum ada debounce tombol.
- `avr-gcc` belum tersedia.
- Blink dijalankan dari encoding manual.
- Belum ada firmware UART hello hasil kompilasi.
- ELF loader sudah ada, tetapi firmware ELF nyata belum diuji.

---

## 6. Kalibrasi

Kalibrasi pin-level memakai datasheet ATmega328P:

| Parameter | Nilai model | Dasar |
|---|---|---|
| Output resistance | 45 Ohm | VOH 4.1 V pada 20 mA |
| Pull-up resistance | 35 kOhm | rentang 20-50 kOhm |
| Input threshold | 2.5 V | asumsi 0.5 supply |

Parameter device lain masih memakai nilai awal. Prosedur kalibrasi ada di
`docs/39-calibration.md`, tetapi data pengukuran belum tersedia.

---

## 7. Kesimpulan

Fondasi proyek sudah luas: device inti, solver DC/transient/AC, noise,
distorsi, sensitivitas, sel CMOS, digital, CPU AVR, peripheral, board,
pin elektrik, loader HEX/ELF, dan CLI.

Yang belum adalah fidelity penuh: golden ngspice, firmware hasil
`avr-gcc`, determinisme macOS/Windows, model breakdown, dan kalibrasi
silikon.
