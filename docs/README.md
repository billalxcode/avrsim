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
| Kode | P6 CPU uji selesai |
| Toolchain Rust | Terpasang (1.99.1) |
| `avr-gcc` | Belum terpasang |
