# Dokumentasi Proyek

Dokumen ini adalah indeks lengkap spesifikasi **embed-emulation**.

## Urutan baca

| No | Dokumen | Fokus |
|---|---|---|
| 00 | [Visi](00-vision.md) | Mengapa proyek ini ada, tujuan, non-goal, definisi sukses |
| 01 | [Keputusan](01-decisions.md) | Seluruh keputusan desain dan alasannya |
| 02 | [Arsitektur](02-architecture.md) | Struktur crate, netlist, waktu, API, determinisme |
| 03 | [Device & Solver](03-device-solver.md) | MOSFET, MNA, Newton, analisis, kalibrasi |
| 04 | [CPU AVR](04-cpu-avr.md) | ISA, ALU, register, kontrol, interupsi, memori |
| 05 | [Peripheral](05-peripherals.md) | GPIO, timer, UART, SPI, I2C, ADC, dan lainnya |
| 06 | [Board & Firmware](06-board-firmware.md) | Komponen board, loader, toolchain, demo |
| 07 | [Verifikasi](07-verification.md) | Test pyramid, oracle, fidelity matrix, toleransi |
| 08 | [Roadmap](08-roadmap.md) | Milestone M0-M9 dan exit criteria |
| 09 | [Risiko](09-risks.md) | Feasibility, risiko, dan guardrail |
| 10 | [Math API](10-math-api.md) | API waktu, satuan, complex, matrix, LU, error |
| 11 | [Netlist Core](11-netlist.md) | Node, device, builder, subcircuit, flatten, validasi |
| 12 | [Device Models](12-device-models.md) | Resistor, capacitor, inductor, source, MOSFET, dioda |
| 13 | [Analysis](13-analysis.md) | MNA, DC, sweep, transient, hasil M1 |
| 14 | [CMOS Cells](14-cells.md) | Inverter, NAND2, NOR2, buffer, API sel, adder, ALU |
| 15 | [Digital](15-digital.md) | Logika dua-state, clock ideal, counter, shift register |
| 16 | [AVR Decode](16-avr-decode.md) | Decoder, fetch, register bank, ALU arithmetic/logic/shift AVR |
| 17 | [GPIO](17-gpio.md) | Port B/C/D, DDR, PORT, PIN, pull-up |
| 18 | [Timer0](18-timer0.md) | Counter, prescaler, normal, CTC, compare |
| 19 | [Timer1 dan Timer2](19-timer1-timer2.md) | Timer1 16-bit, capture, compare/PWM, Timer2 |
| 20 | [UART](20-uart.md) | TX/RX shift, baud, flag, interrupt |
| 21 | [System](21-system.md) | EEPROM, watchdog, fuse, clock, comparator, POR |
| 22 | [Board](22-board.md) | LED, resistor, button, UART host, Intel HEX |
| 23 | [SPI, TWI, ADC](23-spi-twi-adc.md) | SPI, TWI, dan ADC behavioral |
| 24 | [Fidelity](24-fidelity.md) | Matriks fidelity dan batasan model |
| 25 | [Sparse](25-sparse.md) | CSR, sparse LU, degree ordering |
| 26 | [Adaptive](26-adaptive.md) | Adaptive step dan estimasi error |
| 27 | [Determinisme](27-determinism.md) | Uji determinisme solver Linux |
| 28 | [Integrasi](28-integration.md) | Backward Euler, trapezoidal, Gear2 |
| 29 | [AC](29-ac.md) | AC small-signal dan sweep frekuensi |
| 30 | [Noise](30-noise.md) | Noise termal resistor |
| 31 | [Distorsi](31-distortion.md) | Distorsi harmonik kuasi-statis |
| 32 | [Sensitivitas](32-sensitivity.md) | Sensitivitas DC numerik |
| 33 | [Suhu](33-temperature.md) | Suhu device dan koefisien suhu |
| 34 | [Process corner](34-process-corner.md) | Scaling parameter device |
| 35 | [BJT](35-bjt.md) | Model Ebers-Moll NPN dan PNP |
| 36 | [JFET](36-jfet.md) | Model Shichman-Hodges N dan P |
| 37 | [Transmission line](37-transmission-line.md) | Line lossless dengan delay |
| 38 | [Performa](38-performance.md) | Benchmark dan optimasi solver |
| 39 | [Kalibrasi](39-calibration.md) | Prosedur dan inventaris parameter |
| 40 | [Pin electrical](40-pin-electrical.md) | Tegangan, arus, high-Z, contention |
| - | [Todolist](../TODO.md) | Daftar tugas lengkap dari hal terkecil |

## Konvensi dokumen

- **Keputusan** ditulis sebagai fakta desain, bukan opsi.
- **Target akhir** dan **milestone pertama** dipisahkan dengan jelas.
- **Pengecualian fidelity** selalu dicatat eksplisit.
- **Angka toleransi** yang belum final ditandai `TBD` dan harus ditetapkan sebelum milestone terkait.
- **Zero dependency** berarti tidak ada crate eksternal; crate internal workspace diperbolehkan.

## Status

| Area | Status |
|---|---|
| Brainstorming | Selesai |
| Spesifikasi | Dokumen ini |
| Kode | P8 selesai kecuali P8-09; P9 sebagian |
| Toolchain Rust | Terpasang (1.99.1) |
| `avr-gcc` | Belum terpasang |
