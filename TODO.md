# TODO - Full Task List

Daftar tugas lengkap dari hal terkecil sampai rilis. Setiap item harus kecil, dapat diuji, dan tidak menambah dependensi eksternal.

> **Status:** Shift dan rotate selesai: LSR, ASR, ROR, dan SWAP. Lanjut ke compare, multiply, dan integrasi CPU.
> **Aturan:** satu item selesai = kode + test + dokumentasi singkat.

---

## Cara Memakai

- Kerjakan berurutan dalam satu fase.
- Jangan lompat fase tanpa memenuhi exit criteria.
- Centang hanya jika test lulus.
- Jika item terlalu besar, pecah menjadi sub-item baru.
- Jangan menambah crate eksternal.
- Perbarui dokumentasi terkait saat menyelesaikan item.

### Legend

- `[ ]` belum
- `[x]` selesai
- `[~]` sedang dikerjakan
- `[!]` diblokir

---

## P0 - Fondasi Proyek

- [ ] P0-01 Periksa OS, arsitektur, dan prasyarat build.
- [ ] P0-02 Pasang Rust stable via `rustup`.
- [ ] P0-03 Verifikasi `rustc --version` dan `cargo --version`.
- [ ] P0-04 Tentukan MSRV dan tulis `rust-toolchain.toml`.
- [ ] P0-05 Buat `Cargo.toml` workspace dengan resolver dan edition.
- [ ] P0-06 Buat crate `sim-core` (`Cargo.toml`, `src/lib.rs`).
- [ ] P0-07 Buat crate `sim-device`.
- [ ] P0-08 Buat crate `sim-analysis`.
- [ ] P0-09 Buat crate `sim-cells`.
- [x] P0-10 Buat crate `sim-avr`.
- [ ] P0-11 Buat crate `sim-board`.
- [ ] P0-12 Buat crate `sim-cli` dengan binary `embed-emu`.
- [ ] P0-13 Pastikan seluruh `Cargo.toml` tidak punya dependensi eksternal.
- [x] P0-14 Tambahkan `LICENSE` (MIT) dengan copyright Billal Fauzan.
- [x] P0-15 Lisensi akhir: hanya MIT (Apache dihapus).
- [ ] P0-16 Tulis `README.md`.
- [ ] P0-17 Tulis `docs/README.md` dan seluruh dokumen 00-09.
- [ ] P0-18 Tulis `TODO.md` ini.
- [ ] P0-19 Tambahkan `.gitignore`.
- [ ] P0-20 Tambahkan `rustfmt.toml`.
- [ ] P0-21 Tambahkan konfigurasi lint di `Cargo.toml` atau `clippy.toml`.
- [ ] P0-22 Buat `scripts/fmt.sh`.
- [ ] P0-23 Buat `scripts/test.sh`.
- [ ] P0-24 Buat `scripts/lint.sh`.
- [x] P0-25 Inisialisasi git repository.
- [x] P0-26 Buat commit awal.
- [ ] P0-27 Buat workflow CI Linux.
- [ ] P0-28 Tambahkan CI macOS.
- [ ] P0-29 Tambahkan CI Windows.
- [ ] P0-30 Pastikan CI menjalankan fmt, lint, test, build release.
- [ ] P0-31 Buat folder `examples/`.
- [ ] P0-32 Buat folder `firmware/`.
- [ ] P0-33 Buat placeholder `examples/README.md`.
- [ ] P0-34 Buat placeholder `firmware/README.md`.
- [ ] P0-35 Verifikasi `cargo test --workspace` hijau.
- [ ] P0-36 Verifikasi `cargo build --workspace --release` hijau.

**Exit P0:** workspace ter-build, test kosong hijau, CI jalan, zero dependency.

---

## P1 - Matematika dan Waktu

- [x] P1-01 Definisikan `Time` integer pikodetik.
- [x] P1-02 Implementasikan konversi ns/us/ms/s ke `Time` dan sebaliknya.
- [x] P1-03 Definisikan newtype `Voltage`.
- [x] P1-04 Definisikan newtype `Current`.
- [x] P1-05 Definisikan newtype `Resistance`.
- [x] P1-06 Definisikan newtype `Capacitance`.
- [x] P1-07 Definisikan `NodeId`, `BranchId`, `DeviceId`.
- [x] P1-08 Implementasikan `Complex64` untuk AC.
- [x] P1-09 Implementasikan operasi dasar complex: add, sub, mul, div, magnitude, phase.
- [x] P1-10 Implementasikan dense matrix row-major.
- [x] P1-11 Implementasikan get/set/row/column.
- [x] P1-12 Implementasikan LU decomposition dengan partial pivoting.
- [x] P1-13 Implementasikan forward substitution.
- [x] P1-14 Implementasikan back substitution.
- [x] P1-15 Implementasikan deteksi matriks singular.
- [x] P1-16 Implementasikan norm dan tolerance helper.
- [x] P1-17 Definisikan `SimError`.
- [x] P1-18 Pastikan tidak ada `HashMap` untuk iterasi hasil.
- [x] P1-19 Tulis test LU 2x2.
- [x] P1-20 Tulis test LU 3x3.
- [x] P1-21 Tulis test matriks singular.
- [x] P1-22 Tulis test complex.
- [x] P1-23 Tulis test konversi satuan.
- [x] P1-24 Tulis benchmark LU kecil.
- [x] P1-25 Dokumentasikan API matematika.

**Exit P1:** solver linear benar, singular terdeteksi, test hijau.

---

## P2 - Netlist Core

- [x] P2-01 Definisikan node ground (ID 0).
- [x] P2-02 Implementasikan alokasi node.
- [x] P2-03 Definisikan `NodeKind`.
- [x] P2-04 Definisikan trait/API `Device`.
- [x] P2-05 Definisikan `StampContext`.
- [x] P2-06 Definisikan `Solution`.
- [x] P2-07 Implementasikan `CircuitBuilder`.
- [x] P2-08 Implementasikan penambahan node dengan nama.
- [x] P2-09 Implementasikan penambahan device dengan nama.
- [x] P2-10 Implementasikan subcircuit dengan port.
- [x] P2-11 Implementasikan instance subcircuit.
- [x] P2-12 Implementasikan algoritma flatten.
- [x] P2-13 Implementasikan penamaan hierarkis.
- [x] P2-14 Implementasikan deteksi nama duplikat.
- [x] P2-15 Implementasikan validasi node floating.
- [x] P2-16 Implementasikan validasi tidak ada jalur DC.
- [x] P2-17 Implementasikan resolusi ground.
- [x] P2-18 Implementasikan macro deklaratif `circuit!`.
- [x] P2-19 Pastikan macro dan builder menghasilkan netlist identik.
- [x] P2-20 Implementasikan snapshot netlist.
- [x] P2-21 Tulis test build inverter manual.
- [x] P2-22 Tulis test build inverter via macro.
- [x] P2-23 Tulis test flatten.
- [x] P2-24 Tulis test duplikat nama.
- [x] P2-25 Tulis test floating node.
- [x] P2-26 Dokumentasikan builder API.
- [x] P2-27 Dokumentasikan macro DSL.

**Exit P2:** inverter dapat dibangun, di-flatten, dan divalidasi.

---

## P3 - Device Models

- [x] P3-01 Definisikan parameter MOSFET Level 1.
- [ ] P3-02 Implementasikan perhitungan `Vth` dengan body effect.
- [x] P3-03 Implementasikan region cutoff.
- [x] P3-04 Implementasikan region linear.
- [x] P3-05 Implementasikan region saturation.
- [x] P3-06 Implementasikan channel-length modulation.
- [x] P3-07 Implementasikan polaritas NMOS.
- [x] P3-08 Implementasikan polaritas PMOS.
- [x] P3-09 Implementasikan `gm`.
- [x] P3-10 Implementasikan `gds`.
- [ ] P3-11 Implementasikan `gmb`.
- [x] P3-12 Implementasikan Jacobian MOSFET.
- [ ] P3-13 Implementasikan kapasitansi Meyer cutoff.
- [ ] P3-14 Implementasikan kapasitansi Meyer linear.
- [ ] P3-15 Implementasikan kapasitansi Meyer saturation.
- [ ] P3-16 Implementasikan overlap capacitance.
- [x] P3-17 Implementasikan resistor.
- [x] P3-18 Implementasikan kapasitor.
- [x] P3-19 Implementasikan companion model kapasitor Backward Euler.
- [x] P3-20 Implementasikan induktor.
- [x] P3-21 Implementasikan companion model induktor Backward Euler.
- [x] P3-22 Implementasikan voltage source.
- [x] P3-23 Implementasikan current source.
- [x] P3-24 Implementasikan dioda Shockley.
- [x] P3-25 Implementasikan Jacobian dioda.
- [ ] P3-26 Implementasikan series resistance dioda.
- [ ] P3-27 Implementasikan kapasitansi junction dioda.
- [ ] P3-28 Implementasikan BJT Ebers-Moll.
- [ ] P3-29 Implementasikan JFET.
- [ ] P3-30 Implementasikan VCVS.
- [ ] P3-31 Implementasikan VCCS.
- [ ] P3-32 Implementasikan CCVS.
- [ ] P3-33 Implementasikan CCCS.
- [ ] P3-34 Implementasikan voltage-controlled switch.
- [ ] P3-35 Siapkan placeholder transmission line.
- [x] P3-36 Implementasikan default parameter.
- [x] P3-37 Implementasikan validasi parameter.
- [x] P3-38 Tulis test MOSFET cutoff.
- [x] P3-39 Tulis test MOSFET linear.
- [x] P3-40 Tulis test MOSFET saturation.
- [x] P3-41 Tulis test Jacobian vs finite difference.
- [x] P3-42 Tulis test resistor.
- [x] P3-43 Tulis test kapasitor.
- [x] P3-44 Tulis test induktor.
- [x] P3-45 Tulis test dioda IV.
- [ ] P3-46 Tulis test BJT dasar.
- [ ] P3-47 Tulis test JFET dasar.
- [x] P3-48 Dokumentasikan setiap device dan batasannya.

**Exit P3:** device inti akurat secara analitik.

---

## P4 - MNA dan Analisis

- [x] P4-01 Definisikan struktur matriks MNA.
- [x] P4-02 Implementasikan stamping resistor.
- [x] P4-03 Implementasikan stamping kapasitor.
- [x] P4-04 Implementasikan stamping induktor.
- [x] P4-05 Implementasikan stamping voltage source.
- [x] P4-06 Implementasikan stamping current source.
- [x] P4-07 Implementasikan stamping MOSFET.
- [x] P4-08 Implementasikan stamping dioda.
- [x] P4-09 Implementasikan DC operating point.
- [x] P4-10 Implementasikan Newton-Raphson.
- [x] P4-11 Implementasikan kriteria konvergensi.
- [ ] P4-12 Implementasikan voltage limiting `pnjlim`.
- [x] P4-13 Implementasikan damping.
- [x] P4-14 Implementasikan gmin stepping.
- [ ] P4-15 Implementasikan source stepping.
- [ ] P4-16 Implementasikan fallback dan error.
- [ ] P4-17 Implementasikan diagnostik konvergensi.
- [x] P4-18 Implementasikan DC sweep.
- [ ] P4-19 Implementasikan continuation antar titik sweep.
- [x] P4-20 Implementasikan transient Backward Euler.
- [x] P4-21 Implementasikan initial condition dari DC op.
- [x] P4-22 Implementasikan `UIC` (opsional).
- [x] P4-23 Implementasikan update state device.
- [ ] P4-24 Implementasikan CSV writer.
- [ ] P4-25 Implementasikan VCD writer.
- [ ] P4-26 Implementasikan VCD analog `real`.
- [ ] P4-27 Implementasikan VCD digital `wire`.
- [x] P4-28 Tulis test pembagi tegangan.
- [x] P4-29 Tulis test RC charge.
- [ ] P4-30 Tulis test RC discharge.
- [x] P4-31 Tulis test diode clamp.
- [x] P4-32 Tulis test inverter VTC.
- [ ] P4-33 Buat harness perbandingan ngspice.
- [ ] P4-34 Simpan golden data pertama.
- [ ] P4-35 Tulis benchmark node/solve.
- [x] P4-36 Dokumentasikan analisis dan toleransi.

**Exit P4:** VTC inverter dan RC transient lulus toleransi.

---

## P5 - Pustaka CMOS

- [x] P5-01 Bangun netlist inverter CMOS.
- [x] P5-02 Jalankan DC sweep inverter.
- [x] P5-03 Hitung switching threshold.
- [x] P5-04 Hitung noise margin.
- [ ] P5-05 Jalankan transient switching.
- [ ] P5-06 Ukur rise time.
- [ ] P5-07 Ukur fall time.
- [ ] P5-08 Ukur propagation delay.
- [x] P5-09 Bangun NAND2.
- [x] P5-10 Bangun NOR2.
- [x] P5-11 Bangun buffer.
- [x] P5-12 Bangun tri-state inverter.
- [x] P5-13 Bangun transmission gate.
- [x] P5-14 Bangun SRAM cell 6T.
- [x] P5-15 Bangun D latch.
- [x] P5-16 Bangun D flip-flop master-slave.
- [ ] P5-17 Buat characterization harness.
- [ ] P5-18 Ukur input capacitance.
- [ ] P5-19 Ukur drive strength.
- [x] P5-20 Buat registry sel.
- [x] P5-21 Tulis truth table test inverter.
- [x] P5-22 Tulis truth table test NAND.
- [x] P5-23 Tulis truth table test NOR.
- [x] P5-24 Tulis test DFF.
- [x] P5-25 Tulis test SRAM cell.
- [ ] P5-26 Bandingkan sel dengan ngspice.
- [x] P5-27 Bangun ring oscillator.
- [ ] P5-28 Verifikasi osilasi.
- [x] P5-29 Dokumentasikan cell library.

**Exit P5:** cell library terkarakterisasi dan ring oscillator berosilasi.

---

## P6 - Komposisi Digital

- [x] P6-01 Implementasikan komposisi gerbang hierarkis.
- [x] P6-02 Implementasikan helper bus/vector.
- [x] P6-03 Implementasikan clock ideal.
- [x] P6-04 Implementasikan reset ideal.
- [x] P6-05 Bangun counter.
- [x] P6-06 Bangun shift register.
- [x] P6-07 Bangun multiplexer.
- [x] P6-08 Bangun decoder.
- [x] P6-09 Bangun half adder.
- [x] P6-10 Bangun full adder.
- [x] P6-11 Bangun 8-bit ripple-carry adder.
- [x] P6-12 Implementasikan flag C.
- [x] P6-13 Implementasikan flag Z.
- [x] P6-14 Implementasikan flag N.
- [x] P6-15 Implementasikan flag V.
- [x] P6-16 Implementasikan flag S.
- [x] P6-17 Implementasikan flag H.
- [x] P6-18 Bangun ALU bit-slice.
- [x] P6-19 Bangun register file DFF + decoder.
- [x] P6-20 Bangun PC.
- [x] P6-21 Bangun IR.
- [x] P6-22 Bangun stack pointer.
- [x] P6-23 Bangun control PLA.
- [x] P6-24 Bangun control FSM.
- [x] P6-25 Bangun CPU uji kecil.
- [x] P6-26 Tulis program uji CPU.
- [x] P6-27 Buat expected trace.
- [x] P6-28 Verifikasi trace.
- [x] P6-29 Dokumentasikan blok CPU.

**Exit P6:** CPU uji kecil menjalankan program dan trace benar.

---

## P7 - AVR CPU Struktural

- [x] P7-01 Buat tabel decode seluruh ISA AVR.
- [x] P7-02 Implementasikan fetch 16-bit.
- [x] P7-03 Implementasikan fetch 32-bit.
- [x] P7-04 Implementasikan instruction register.
- [x] P7-05 Implementasikan register file read port A.
- [x] P7-06 Implementasikan register file read port B.
- [x] P7-07 Implementasikan register file write port.
- [x] P7-08 Implementasikan ADD.
- [x] P7-09 Implementasikan ADC.
- [x] P7-10 Implementasikan SUB.
- [x] P7-11 Implementasikan SBC.
- [x] P7-12 Implementasikan SUBI.
- [x] P7-13 Implementasikan SBCI.
- [x] P7-14 Implementasikan AND.
- [x] P7-15 Implementasikan ANDI.
- [x] P7-16 Implementasikan OR.
- [x] P7-17 Implementasikan ORI.
- [x] P7-18 Implementasikan EOR.
- [x] P7-19 Implementasikan COM.
- [x] P7-20 Implementasikan NEG.
- [x] P7-21 Implementasikan INC.
- [x] P7-22 Implementasikan DEC.
- [ ] P7-23 Implementasikan CP.
- [ ] P7-24 Implementasikan CPC.
- [ ] P7-25 Implementasikan CPI.
- [ ] P7-26 Implementasikan CPSE.
- [x] P7-27 Implementasikan LSR.
- [x] P7-28 Implementasikan ASR.
- [x] P7-29 Implementasikan ROR.
- [x] P7-30 Implementasikan SWAP.
- [ ] P7-31 Implementasikan MUL.
- [ ] P7-32 Implementasikan MULS.
- [ ] P7-33 Implementasikan MULSU.
- [ ] P7-34 Implementasikan MOV.
- [ ] P7-35 Implementasikan MOVW.
- [ ] P7-36 Implementasikan LDI.
- [ ] P7-37 Implementasikan LD X.
- [ ] P7-38 Implementasikan LD X+.
- [ ] P7-39 Implementasikan LD -X.
- [ ] P7-40 Implementasikan LD Y.
- [ ] P7-41 Implementasikan LD Y+.
- [ ] P7-42 Implementasikan LD -Y.
- [ ] P7-43 Implementasikan LD Z.
- [ ] P7-44 Implementasikan LD Z+.
- [ ] P7-45 Implementasikan LD -Z.
- [ ] P7-46 Implementasikan ST X.
- [ ] P7-47 Implementasikan ST X+.
- [ ] P7-48 Implementasikan ST -X.
- [ ] P7-49 Implementasikan ST Y.
- [ ] P7-50 Implementasikan ST Y+.
- [ ] P7-51 Implementasikan ST -Y.
- [ ] P7-52 Implementasikan ST Z.
- [ ] P7-53 Implementasikan ST Z+.
- [ ] P7-54 Implementasikan ST -Z.
- [ ] P7-55 Implementasikan LDD/STD displacement.
- [ ] P7-56 Implementasikan LDS.
- [ ] P7-57 Implementasikan STS.
- [ ] P7-58 Implementasikan LPM.
- [ ] P7-59 Implementasikan LPM Z+.
- [ ] P7-60 Implementasikan ELPM.
- [ ] P7-61 Implementasikan PUSH.
- [ ] P7-62 Implementasikan POP.
- [ ] P7-63 Implementasikan RJMP.
- [ ] P7-64 Implementasikan JMP.
- [ ] P7-65 Implementasikan IJMP.
- [ ] P7-66 Implementasikan RCALL.
- [ ] P7-67 Implementasikan CALL.
- [ ] P7-68 Implementasikan ICALL.
- [ ] P7-69 Implementasikan RET.
- [ ] P7-70 Implementasikan RETI.
- [ ] P7-71 Implementasikan BRBS.
- [ ] P7-72 Implementasikan BRBC.
- [ ] P7-73 Implementasikan SBRC.
- [ ] P7-74 Implementasikan SBRS.
- [ ] P7-75 Implementasikan SBI.
- [ ] P7-76 Implementasikan CBI.
- [ ] P7-77 Implementasikan SBIC.
- [ ] P7-78 Implementasikan SBIS.
- [ ] P7-79 Implementasikan BST.
- [ ] P7-80 Implementasikan BLD.
- [ ] P7-81 Implementasikan BSET.
- [ ] P7-82 Implementasikan BCLR.
- [ ] P7-83 Implementasikan IN.
- [ ] P7-84 Implementasikan OUT.
- [ ] P7-85 Implementasikan SREG.
- [ ] P7-86 Implementasikan interrupt controller.
- [ ] P7-87 Implementasikan vector table.
- [ ] P7-88 Implementasikan SEI delay.
- [ ] P7-89 Implementasikan SLEEP.
- [ ] P7-90 Implementasikan WDR.
- [ ] P7-91 Implementasikan reset.
- [ ] P7-92 Implementasikan FSM timing.
- [ ] P7-93 Tulis test vector ADD.
- [ ] P7-94 Tulis test vector ADC.
- [ ] P7-95 Tulis test vector SUB/SBC.
- [ ] P7-96 Tulis test vector logic.
- [ ] P7-97 Tulis test vector shift.
- [ ] P7-98 Tulis test vector multiply.
- [ ] P7-99 Tulis test vector load/store.
- [x] P7-100 Tulis test vector branch.
- [x] P7-101 Tulis test vector call/stack.
- [x] P7-102 Tulis test vector interrupt.
- [x] P7-103 Differential test vs simavr.
- [x] P7-104 Tulis blink assembly.
- [x] P7-105 Kompilasi blink dengan `avr-gcc`.
- [x] P7-106 Jalankan blink di CPU struktural.
- [x] P7-107 Implementasikan CLI monitor.
- [x] P7-108 Implementasikan disassembler.
- [x] P7-109 Dokumentasikan ISA support.

**Exit P7:** blink `avr-gcc` berjalan di CPU struktural.

---

## P8 - Peripheral Struktural

- [ ] P8-01 Implementasikan GPIO port B.
- [ ] P8-02 Implementasikan GPIO port C.
- [ ] P8-03 Implementasikan GPIO port D.
- [ ] P8-04 Implementasikan DDR.
- [ ] P8-05 Implementasikan PORT.
- [ ] P8-06 Implementasikan PIN read.
- [ ] P8-07 Implementasikan PIN toggle.
- [ ] P8-08 Implementasikan pull-up.
- [ ] P8-09 Implementasikan pin electrical model.
- [ ] P8-10 Implementasikan pin change interrupt.
- [ ] P8-11 Implementasikan Timer0 counter.
- [ ] P8-12 Implementasikan Timer0 prescaler.
- [ ] P8-13 Implementasikan Timer0 normal mode.
- [ ] P8-14 Implementasikan Timer0 CTC.
- [ ] P8-15 Implementasikan Timer0 PWM.
- [ ] P8-16 Implementasikan Timer0 compare.
- [ ] P8-17 Implementasikan Timer0 overflow interrupt.
- [ ] P8-18 Implementasikan Timer1 16-bit.
- [ ] P8-19 Implementasikan Timer1 input capture.
- [ ] P8-20 Implementasikan Timer1 compare/PWM.
- [ ] P8-21 Implementasikan Timer2.
- [ ] P8-22 Implementasikan Timer2 async (opsional).
- [ ] P8-23 Implementasikan UART baud generator.
- [ ] P8-24 Implementasikan UART TX shift register.
- [ ] P8-25 Implementasikan UART TX flags.
- [ ] P8-26 Implementasikan UART TX interrupt.
- [ ] P8-27 Implementasikan UART RX shift register.
- [ ] P8-28 Implementasikan UART RX flags.
- [ ] P8-29 Implementasikan UART RX interrupt.
- [ ] P8-30 Integrasikan peripheral ke interrupt controller.
- [ ] P8-31 Implementasikan SPI master.
- [ ] P8-32 Implementasikan SPI slave.
- [ ] P8-33 Implementasikan SPI modes.
- [ ] P8-34 Implementasikan I2C/TWI master.
- [ ] P8-35 Implementasikan I2C/TWI slave.
- [ ] P8-36 Implementasikan ADC mux.
- [ ] P8-37 Implementasikan ADC SAR.
- [ ] P8-38 Implementasikan ADC reference.
- [ ] P8-39 Implementasikan ADC interrupt.
- [ ] P8-40 Implementasikan analog comparator.
- [ ] P8-41 Implementasikan EEPROM.
- [ ] P8-42 Implementasikan watchdog.
- [ ] P8-43 Implementasikan sleep modes.
- [ ] P8-44 Implementasikan fuse bytes.
- [ ] P8-45 Implementasikan clock prescaler.
- [ ] P8-46 Implementasikan POR.
- [ ] P8-47 Implementasikan brown-out.
- [ ] P8-48 Tulis test GPIO.
- [ ] P8-49 Tulis test Timer0.
- [ ] P8-50 Tulis test Timer1.
- [ ] P8-51 Tulis test Timer2.
- [ ] P8-52 Tulis test UART TX.
- [ ] P8-53 Tulis test UART RX.
- [ ] P8-54 Tulis test SPI.
- [ ] P8-55 Tulis test I2C.
- [ ] P8-56 Tulis test ADC.
- [ ] P8-57 Tulis test comparator.
- [ ] P8-58 Tulis test EEPROM.
- [ ] P8-59 Tulis test watchdog.
- [ ] P8-60 Tulis test sleep.
- [ ] P8-61 Jalankan blink + UART hello.
- [ ] P8-62 Jalankan timer interrupt.
- [ ] P8-63 Dokumentasikan peripheral.

**Exit P8:** blink + UART hello + timer interrupt berjalan.

---

## P9 - Board dan Firmware

- [ ] P9-01 Implementasikan model LED.
- [ ] P9-02 Implementasikan resistor board.
- [ ] P9-03 Implementasikan tombol.
- [ ] P9-04 Bangun board netlist dasar.
- [ ] P9-05 Hubungkan PB5 ke resistor dan LED.
- [ ] P9-06 Implementasikan UART host endpoint.
- [ ] P9-07 Implementasikan loader Intel HEX.
- [ ] P9-08 Implementasikan validasi checksum HEX.
- [ ] P9-09 Implementasikan loader ELF.
- [ ] P9-10 Implementasikan simbol ELF.
- [ ] P9-11 Definisikan profil fuse Arduino Uno.
- [ ] P9-12 Implementasikan reset vector.
- [ ] P9-13 Implementasikan startup state.
- [ ] P9-14 Pasang `avr-gcc`.
- [ ] P9-15 Kompilasi firmware blink.
- [ ] P9-16 Konversi ELF ke HEX.
- [ ] P9-17 Jalankan blink di board.
- [ ] P9-18 Kompilasi firmware UART hello.
- [ ] P9-19 Jalankan UART hello.
- [ ] P9-20 Implementasikan timer interrupt demo.
- [ ] P9-21 Implementasikan output CSV board.
- [ ] P9-22 Implementasikan output VCD board.
- [ ] P9-23 Implementasikan CLI `run`.
- [ ] P9-24 Implementasikan CLI `monitor`.
- [ ] P9-25 Implementasikan CLI argumen dan validasi.
- [ ] P9-26 Tulis test blink end-to-end.
- [ ] P9-27 Tulis test UART end-to-end.
- [ ] P9-28 Tulis test reset.
- [ ] P9-29 Dokumentasikan board profile.
- [ ] P9-30 Dokumentasikan contoh firmware.

**Exit P9:** demo board end-to-end deterministik dan reproducible.

---

## P10 - Full SPICE dan Fidelity

- [ ] P10-01 Implementasikan AC small-signal.
- [ ] P10-02 Implementasikan sweep frekuensi.
- [ ] P10-03 Implementasikan output magnitude/phase.
- [ ] P10-04 Implementasikan noise analysis.
- [ ] P10-05 Implementasikan distortion analysis.
- [ ] P10-06 Implementasikan sensitivity analysis.
- [ ] P10-07 Implementasikan model suhu.
- [ ] P10-08 Implementasikan process corner.
- [ ] P10-09 Implementasikan full BJT model.
- [ ] P10-10 Implementasikan full JFET model.
- [ ] P10-11 Implementasikan transmission line.
- [ ] P10-12 Implementasikan sparse matrix.
- [ ] P10-13 Implementasikan sparse LU.
- [ ] P10-14 Implementasikan ordering.
- [ ] P10-15 Implementasikan adaptive time step.
- [ ] P10-16 Implementasikan LTE.
- [ ] P10-17 Implementasikan trapezoidal integration.
- [ ] P10-18 Implementasikan Gear integration.
- [ ] P10-19 Kalibrasi pin-level dari datasheet.
- [ ] P10-20 Dokumentasikan parameter kalibrasi.
- [ ] P10-21 Jalankan golden suite vs ngspice.
- [ ] P10-22 Profiling solver.
- [ ] P10-23 Optimasi hot path.
- [ ] P10-24 Uji determinisme Linux.
- [ ] P10-25 Uji determinisme macOS.
- [ ] P10-26 Uji determinisme Windows.
- [ ] P10-27 Finalisasi fidelity matrix.
- [ ] P10-28 Dokumentasikan batasan model.

**Exit P10:** seluruh analisis/device yang dideklarasikan lulus golden suite.

---

## P11 - Release

- [ ] P11-01 Tulis user guide.
- [ ] P11-02 Tulis API documentation.
- [ ] P11-03 Tambahkan contoh inverter.
- [ ] P11-04 Tambahkan contoh ring oscillator.
- [ ] P11-05 Tambahkan contoh CPU uji.
- [ ] P11-06 Tambahkan contoh blink.
- [ ] P11-07 Tambahkan contoh UART hello.
- [ ] P11-08 Jalankan CI Linux.
- [ ] P11-09 Jalankan CI macOS.
- [ ] P11-10 Jalankan CI Windows.
- [ ] P11-11 Siapkan packaging.
- [ ] P11-12 Lengkapi lisensi dan notices.
- [ ] P11-13 Tulis release notes.
- [ ] P11-14 Tag v0.1.
- [ ] P11-15 Publikasikan binary.
- [ ] P11-16 Perbarui roadmap.
- [ ] P11-17 Buka issue untuk milestone berikutnya.

**Exit P11:** rilis v0.1 tersedia dan dapat direproduksi.

---

## Cross-Cutting Tasks

- [ ] X-01 Pastikan setiap crate punya dokumentasi modul.
- [ ] X-02 Pastikan setiap error punya pesan jelas.
- [ ] X-03 Pastikan tidak ada panic pada input pengguna.
- [ ] X-04 Pastikan tidak ada dependensi eksternal.
- [ ] X-05 Jalankan fmt sebelum commit.
- [ ] X-06 Jalankan clippy sebelum commit.
- [ ] X-07 Jalankan test sebelum commit.
- [ ] X-08 Perbarui fidelity matrix setiap milestone.
- [ ] X-09 Perbarui TODO setiap menyelesaikan item.
- [ ] X-10 Simpan golden data dengan metadata.
- [ ] X-11 Dokumentasikan cara regenerasi golden.
- [ ] X-12 Audit lisensi model/data.
- [ ] X-13 Buat benchmark baseline.
- [ ] X-14 Buat test snapshot/replay.
- [ ] X-15 Buat test fuzz parser HEX.
- [ ] X-16 Buat test fuzz decoder AVR.
- [ ] X-17 Buat test error konvergensi.
- [ ] X-18 Buat test node floating.
- [ ] X-19 Buat test matriks singular.
- [ ] X-20 Buat test input tidak valid.

---

## Milestone Mapping

| Milestone | Task IDs |
|---|---|
| M0 | P0-01 ... P0-36 |
| M1 | P1-01 ... P1-25, P2-01 ... P2-27, P3-01 ... P3-48, P4-01 ... P4-36, P5-01 ... P5-08 |
| M2 | P5-09 ... P5-29 |
| M3 | P6-01 ... P6-29 |
| M4 | P7-01 ... P7-109 |
| M5 | P8-01 ... P8-30, P8-48 ... P8-53, P8-61 ... P8-63 |
| M6 | P8-31 ... P8-60 |
| M7 | P9-01 ... P9-30 |
| M8 | P10-01 ... P10-28 |
| M9 | P11-01 ... P11-17 |

---

## Urutan 10 Tugas Pertama

1. P0-01 Periksa prasyarat.
2. P0-02 Pasang Rust.
3. P0-03 Verifikasi toolchain.
4. P0-05 Buat workspace.
5. P0-06 ... P0-12 Buat crate.
6. P0-13 Cek zero dependency.
7. P0-14 ... P0-18 Tambahkan lisensi dan dokumentasi.
8. P0-22 ... P0-24 Buat skrip dev.
9. P0-35 ... P0-36 Build dan test.
10. P1-01 Definisikan `Time`.

---

## Definition of Done

Sebuah task dianggap selesai jika:

- [ ] Kode ditulis.
- [ ] Test ditulis dan lulus.
- [ ] Tidak ada dependensi eksternal.
- [ ] Dokumentasi diperbarui.
- [ ] `cargo fmt` bersih.
- [ ] `cargo clippy` bersih.
- [ ] `cargo test --workspace` hijau.
- [ ] Tidak ada `unwrap()` yang tidak aman pada input pengguna.
- [ ] Error ditangani dengan `Result`.
- [ ] Perubahan di-commit dengan pesan jelas.

---

## Catatan

- Jangan mengejar seluruh visi sekaligus.
- Jangan menambah peripheral sebelum yang lama stabil.
- Jangan menambah analisis sebelum solver stabil.
- Jangan mengklaim fidelity tanpa test.
- Jangan mengorbankan fondasi demi cakupan.
