# 53 - Release Notes

Catatan rilis untuk versi 0.0.1.

---

## 1. Ringkasan

Versi 0.0.1 adalah fondasi simulator AVR dan SPICE yang dapat dijalankan.

Cakupan utama:

- Device inti dan device lanjutan.
- Solver DC, transient, AC, noise, distorsi, dan sensitivitas.
- Sel CMOS struktural dan simulator digital.
- CPU AVR dan peripheral.
- Board, pin elektrik, Intel HEX, dan ELF32.
- CLI dan contoh runnable.

Belum ada tag, publikasi biner, atau GitHub release.

---

## 2. Device

- Resistor, kapasitor, induktor.
- Voltage source, current source, pulse source.
- Diode dengan series resistance dan junction capacitance.
- MOSFET Level 1 dengan body effect, `gmb`, Meyer, dan overlap capacitance.
- BJT Ebers-Moll NPN/PNP.
- JFET Shichman-Hodges N/P.
- VCVS, VCCS, CCVS, CCCS, dan voltage-controlled switch.
- Transmission line lossless.

---

## 3. Analisis

- DC operating point dan DC sweep dengan continuation.
- Source stepping fallback dan gmin stepping.
- Transient Backward Euler, trapezoidal, dan Gear2.
- Adaptive transient dan LTE.
- AC small-signal linear.
- Noise termal resistor.
- Distorsi harmonik kuasi-statis.
- Sensitivitas DC numerik.
- Karakterisasi switching, input capacitance, dan drive strength.
- Suhu device dan process corner.

---

## 4. AVR dan Board

- CPU AVR subset dengan test ISA.
- GPIO dan model pin elektrik.
- Timer0/1/2, UART, SPI, TWI, ADC.
- EEPROM, watchdog, fuse, clock prescaler, comparator, POR, brown-out.
- Board LED, resistor, button, UART host.
- Intel HEX loader/writer.
- ELF32 loader, simbol, dan konversi flash ke Intel HEX.

---

## 5. CLI dan Contoh

CLI:

```bash
./target/release/embed-emu run firmware/blink.hex 1000
./target/release/embed-emu bench 1000 10
./target/release/embed-emu monitor <hex words...>
./target/release/embed-emu disasm <hex words...>
```

Contoh:

```bash
cargo run -p sim-analysis --example inverter_vtc
cargo run -p sim-cells --example ring_oscillator
cargo run -p sim-avr --example cpu_program
cargo run -p sim-board --example blink
cargo run -p sim-avr --example uart_hello
```

---

## 6. Packaging

Script:

```bash
./scripts/package.sh
```

Menghasilkan:

```
dist/embed-emu-<target>.tar.gz
```

Isi arsip:

- Binary `embed-emu`.
- README dan LICENSE.
- Dokumentasi Markdown.
- Contoh.

`dist/` tidak di-commit.

---

## 7. Verifikasi

Perintah:

```bash
./scripts/fmt.sh
./scripts/lint.sh
./scripts/test.sh
./scripts/build.sh
./scripts/package.sh
```

Total test workspace: 501.

---

## 8. Batas Rilis

- Belum ada tag `v0.0.1`.
- Belum ada publikasi biner.
- Belum ada golden ngspice.
- Firmware hasil `avr-gcc` belum dijalankan.
- Determinisme macOS/Windows belum dijalankan.
- CI GitHub Actions sudah disiapkan tetapi belum dieksekusi.
- X-03 panic audit sebagian.
