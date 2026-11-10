# 00 — Visi dan Ruang Lingkup

## 1. Visi

Membangun simulator elektronika **native** yang mampu berpindah dari level transistor sampai level sistem:

> Satu binary native yang memuat firmware AVR, menjalankan board virtual digital + analog secara deterministik, dan memodelkan CPU serta peripheral sampai level transistor.

Proyek ini bukan sekadar emulator instruksi. Ini adalah **simulator struktural** dengan target jangka panjang:

- Transistor MOSFET sebagai fondasi.
- Gerbang dan sel CMOS dibangun dari transistor.
- CPU dan peripheral dibangun dari sel dan gerbang.
- Board dibangun dari MCU, IC aktif, dan komponen pasif.
- Firmware AVR berjalan di atas model struktural tersebut.

## 2. Tujuan

1. **Transistor-level foundation** — device model fisik, bukan switch ideal.
2. **Firmware-capable** — menjalankan firmware AVR hasil `avr-gcc`.
3. **Structural MCU** — ATmega328P clean-room yang dibangun bertahap.
4. **Full SPICE engine** — MNA, DC, transient, AC, noise, dan analisis lainnya.
5. **Native and self-contained** — Rust std-only, tanpa runtime eksternal.
6. **Deterministic** — hasil dapat direproduksi dan dibandingkan.
7. **Observable** — CSV, VCD, CLI monitor, dan fidelity matrix.
8. **Open source** — MIT.

## 3. Non-goal (untuk saat ini)

Hal-hal berikut **bukan** target awal, meskipun bisa muncul kemudian:

- GUI schematic editor.
- GUI waveform viewer bawaan.
- Layout PCB atau ekstraksi parasitik PCB.
- Simulasi termal detail.
- Simulasi mekanik atau elektromagnetik.
- Kompilasi firmware di dalam emulator (compiler C/Arduino).
- Emulasi chip selain ATmega328P.
- Real-time 16 MHz untuk seluruh MCU transistor-level.
- Kecocokan bit-identik dengan silikon asli.
- Emulasi ROM/BIOS/firmware proprietary milik pihak lain.

## 4. Target Profile

**Board profile pertama:** Arduino Uno-like.

| Elemen | Nilai |
|---|---|
| MCU | ATmega328P |
| Clock | 16 MHz |
| Flash | 32 KB (behavioral) |
| SRAM | 2 KB (behavioral) |
| EEPROM | 1 KB (behavioral) |
| VDD | 5 V ideal (tahap awal) |
| GPIO | Port B, C, D |
| Timer | Timer0, Timer1, Timer2 |
| UART | USART0 |
| Firmware demo | Blink + UART “hello” |
| LED | PB5 (D13) melalui resistor |
| Host UART | Endpoint behavioral |

## 5. Definisi “100% Native”

- Emulator adalah binary hasil kompilasi Rust untuk host.
- Tidak ada VM, interpreter bytecode untuk emulator, browser, atau WASM.
- Tidak ada proses simulator eksternal saat runtime.
- Tidak ada crate eksternal; hanya `std` dan crate internal.
- Solver numerik, parser, device model, dan CLI ditulis sendiri.
- Firmware boleh dikompilasi dengan `avr-gcc` di luar runtime; itu toolchain, bukan bagian emulator.
- Viewer VCD (GTKWave) boleh eksternal; itu alat analisis, bukan bagian runtime.

## 6. Definisi “100% Berfungsi”

Kebenaran diukur sebagai **firmware-observable equivalence** pada board profile:

### 6.1 Level CPU
- Register, SRAM, I/O, dan flag benar pada batas instruksi.
- Stack, call/return, dan interrupt state benar.
- Instruksi yang dipakai firmware tidak menghasilkan perilaku menyimpang.

### 6.2 Level timing
- Jumlah siklus per instruksi sesuai datasheet.
- Timer, UART, dan interupsi memiliki timing yang konsisten.
- Toleransi timing dinyatakan per komponen.

### 6.3 Level pin
- Level logika dan transisi benar.
- Tegangan output/input berada dalam toleransi listrik yang dinyatakan.
- Pull-up, drive strength, dan threshold dipahami.

### 6.4 Level analog
- Tegangan node berada dalam toleransi terhadap solusi analitik atau golden SPICE.
- Konvergensi tidak menghasilkan NaN atau osilasi numerik.
- Model memiliki batas validitas yang terdokumentasi.

### 6.5 Level sistem
- Firmware berjalan sampai selesai tanpa illegal instruction atau hang tak terduga.
- Output UART cocok.
- LED/GPIO berperilaku sesuai program.
- Hasil deterministik dan dapat direproduksi.

## 7. Filosofi Fidelity

Proyek ini memakai **multi-level fidelity dengan pengecualian eksplisit**:

| Bagian | Level | Alasan |
|---|---|---|
| MOSFET, dioda, BJT, JFET | Device model | Fondasi fisik |
| R, C, L | Device model | Komponen fisik |
| Register file | Struktural | Bagian inti CPU |
| ALU/control/PC/IR/SP | Struktural | Bagian inti CPU |
| SRAM/flash | Behavioral | Menjaga jumlah node realistis |
| GPIO/timer/UART | Struktural | Target akhir |
| SPI/I2C/ADC/EEPROM/WDT | Struktural | Target akhir |
| LED/tombol/pasif board | Device model | Komponen fisik |
| Host UART | Behavioral | Di luar board |
| Reset/POR/brownout | Struktural | Tahap lanjut |
| Power tree/regulator | Struktural | Tahap lanjut; ideal dulu |

**Aturan:** setiap pengecualian harus muncul di fidelity matrix dan tidak boleh diklaim sebagai transistor-level.

## 8. Prinsip Desain

1. **Vertical slice lebih penting daripada bottom-up murni.**
   Setiap fase harus menghasilkan sesuatu yang bisa diuji.
2. **Satu sumber kebenaran untuk waktu.**
   Waktu digital integer pikodetik; waktu analog f64 yang diturunkan dari waktu global.
3. **Device tidak tahu analisis.**
   Device hanya melakukan stamping; solver dan analisis terpisah.
4. **Hierarchy untuk desain, flatten untuk solver.**
   Netlist modular, tetapi solver bekerja pada netlist datar yang dioptimasi.
5. **Determinisme dulu, optimasi kemudian.**
   Single-threaded, urutan iterasi stabil, tanpa wall-clock.
6. **Fidelity harus diukur, bukan diklaim.**
   Setiap komponen punya test dan toleransi.
7. **Zero dependency bukan berarti zero modularity.**
   Workspace internal dengan crate terpisah tetap dipakai.

## 9. Feasibility Summary

| Target | Realisme |
|---|---|
| Inverter VTC + solver | Beberapa minggu/bulan, sangat feasible |
| Pustaka CMOS + DFF/SRAM | Bulan, feasible |
| ALU + register file + CPU uji | Bulan, feasible |
| Seluruh ISA AVR struktural | Bertahun-tahun, ambisius |
| GPIO/timer/UART struktural | Bertahun-tahun, ambisius |
| Seluruh peripheral struktural | Multi-tahun |
| Seluruh board IC struktural | Multi-tahun |
| Pustaka + seluruh analisis SPICE | Multi-tahun |
| Seluruh visi, solo | Multi-person-decade |

**Kesimpulan:** visi layak sebagai program riset jangka panjang, tetapi milestone harus kecil dan terukur. Baca [`09-risks.md`](09-risks.md).
