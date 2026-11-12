# 07 - Verifikasi dan Fidelity

Dokumen ini menjelaskan bagaimana kebenaran diukur, dibuktikan, dan dilaporkan.

---

## 1. Filosofi

Proyek ini tidak mengklaim "100% identik dengan silikon". Yang diklaim adalah:

> **Firmware-observable equivalence untuk board profile yang didukung, dalam toleransi yang dinyatakan.**

Setiap klaim harus punya:

1. **Profile** - board, clock, VDD, firmware.
2. **Fitur** - instruksi/peripheral/analisis yang dicakup.
3. **Toleransi** - angka yang disepakati.
4. **Test** - bukti yang dapat dijalankan ulang.
5. **Batasan** - apa yang tidak dimodelkan.

---

## 2. Fidelity Matrix

| Bagian | Level | Bukti | Batasan |
|---|---|---|---|
| MOSFET | Device model (SH L1) | Analitik dan transient | Tidak akurat short-channel |
| Kapasitansi MOS | Meyer | Analitik dan transient charge | Tidak charge-conserving |
| Resistor | Device model | Analitik | Ideal |
| Kapasitor | Device model | RC transient | Ideal |
| Induktor | Device model | RL transient | Ideal |
| Dioda | Device model | Analitik, clamp, dan transient | Model sederhana |
| BJT | Device model | DC bias dan AC gain | Ebers-Moll |
| JFET | Device model | DC bias dan AC gain | Model sederhana |
| Register file | Struktural | Test logika | DFF ideal |
| ALU/control | Struktural | ISA test vector | Delay dari sel |
| SRAM | Behavioral | Firmware test | Timing kontrak |
| Flash | Behavioral | Firmware test | Timing kontrak |
| GPIO | Struktural | Pin test | Model listrik terbatas |
| Timer | Struktural | Periode/PWM test | Detail silikon tidak |
| UART | Struktural | Loopback/hello | Tanpa noise |
| SPI | Struktural | Loopback | Tanpa noise |
| I2C | Struktural | Master/slave test | Arbitration terbatas |
| ADC | Struktural | Kode vs tegangan | Model analog L1 |
| Comparator | Struktural | Threshold test | Hysteresis terbatas |
| EEPROM | Struktural | Read/write | Timing ideal |
| Watchdog | Struktural | Reset test | Timing ideal |
| LED | Device model | IV + visual state | Tanpa cahaya |
| Host UART | Behavioral | Output byte | Bukan USB |
| Reset/POR | Struktural (akhir) | Startup test | Power ramp disederhanakan |
| Power tree | Struktural (akhir) | Node voltage | Tanpa thermal |
| PCB parasitics | Tidak dimodelkan | - | Di luar scope |

---

Golden ngspice belum dijalankan. Kolom bukti menyebut test yang ada saat ini.

## 3. Test Pyramid

```
Board / firmware    end-to-end demos
Peripheral tests    timer/UART/ADC
ISA / CPU tests     instruction vectors
Cell / block tests  gates, ALU, DFF
Device / solver     unit + analytic
```

Semakin ke bawah, semakin banyak test dan semakin cepat eksekusinya.

---

## 4. Level Verifikasi

### 4.1 Unit

- Matematika: LU, complex, norm.
- Device: Ids, Jacobian, IV.
- Netlist: build, flatten, validasi.
- Parser: HEX, VCD, CSV.

### 4.2 Analitik

- Pembagi tegangan.
- RC charge/discharge.
- RL transient.
- Diode clamp.
- Inverter VTC kualitatif.

### 4.3 Golden SPICE

Direncanakan, belum dijalankan.

- ngspice sebagai referensi dev-only.
- Data akan disimpan sebagai CSV kecil.
- Perbandingan memakai toleransi.
- Regresi akan dicek setiap perubahan solver.

### 4.4 Cell

- Tabel kebenaran.
- VTC.
- Noise margin.
- Propagation delay.
- Rise/fall time.
- Fan-out.
- Ring oscillator.

### 4.5 ISA

- Test vector per instruksi.
- Arithmetic exhaustif 8-bit.
- Flag edge case.
- Load/store.
- Branch/call/stack.
- Interrupt.
- Differential vs simavr.

### 4.6 Peripheral

- Register access.
- Timing.
- Interrupt.
- Firmware driver.

### 4.7 Board

- Blink.
- UART hello.
- Timer interrupt.
- Analog sensor (target lanjut).

---

## 5. Oracle

| Oracle | Penggunaan | Runtime? |
|---|---|---|
| ngspice | Golden analog | Tidak |
| simavr | Differential ISA | Tidak |
| Datasheet | Referensi perilaku/timing | Tidak |
| Analitik | Solusi tertutup | Tidak |
| Hardware asli | Tidak tersedia | Tidak |
| Model behavioral internal | Pembanding struktural | Bisa |

Aturan:

- Oracle eksternal tidak boleh menjadi dependensi runtime.
- Data golden harus disimpan di repo atau dihasilkan oleh skrip dev.
- Perbedaan harus dijelaskan, bukan diabaikan.

---

## 6. Toleransi

Nilai awal yang diusulkan (semua `TBD` sampai divalidasi):

| Analisis | Toleransi usulan |
|---|---|
| DC operating point | 1e-6 V absolut / 1e-3 relatif |
| DC sweep | 1% relatif |
| Transient | 1-5% relatif |
| VTC | 2% dari VDD |
| Propagation delay | 5% |
| UART byte | exact |
| Register/flag | exact |
| Cycle count | exact untuk instruksi yang diuji |
| Node analog vs referensi eksternal | 2% relatif (TBD) |

Toleransi harus cukup ketat untuk menangkap bug, tetapi cukup longgar untuk mengakomodasi Level 1 dan floating-point.

---

## 7. Determinisme

- Waktu simulasi integer pikodetik.
- Tidak ada wall-clock.
- Tidak ada RNG tanpa seed.
- Iterasi memakai urutan stabil.
- Single-threaded pada tahap awal.
- Hasil lintas platform dibandingkan dengan toleransi.
- Snapshot/replay harus menghasilkan hasil yang sama.

Test determinisme:

1. Jalankan simulasi dua kali -> hasil identik.
2. Jalankan di Linux/macOS/Windows -> hasil dalam toleransi.
3. Ubah urutan build crate -> hasil sama.
4. Simpan snapshot, restore, lanjutkan -> hasil sama dengan run penuh.

---

## 8. Golden Data

Aturan:

- Ukuran kecil (< 1 MB per test).
- Format CSV/VCD.
- Disertai metadata: versi ngspice, parameter, suhu, time step.
- Tidak menyimpan data proprietary.
- Regenerasi didokumentasikan.

---

## 9. Fuzzing

Target:

- Parser HEX.
- Parser VCD/CSV.
- Decoder instruksi AVR.
- Parameter device tidak valid.
- Netlist dengan node floating.
- Matriks singular.
- Konvergensi gagal.

Tujuan:

- Tidak panic.
- Error jelas.
- Tidak hang.

---

## 10. CI

Matriks:

| Platform | Target |
|---|---|
| Linux x86_64 | Wajib |
| macOS x86_64/ARM64 | Wajib |
| Windows x86_64 | Wajib |

Job:

1. `cargo fmt --check`.
2. `cargo clippy -- -D warnings` (tanpa dependensi eksternal).
3. `cargo test --workspace`.
4. Golden test (subset).
5. Build release.
6. Dokumentasi (opsional).

Catatan: `clippy` adalah komponen toolchain, bukan crate eksternal.

---

## 11. Test Naming

Format:

```
<area>_<behavior>_<condition>
```

Contoh:

- `mosfet_saturation_matches_analytic`
- `lu_singular_returns_error`
- `avr_add_sets_carry`
- `uart_tx_hello`
- `inverter_vtc_matches_spice`

---

## 12. Acceptance per Milestone

| Milestone | Kriteria |
|---|---|
| M0 | Workspace build, test kosong hijau, zero dependency |
| M1 | VTC inverter + RC transient vs ngspice dalam toleransi |
| M2 | Cell library lulus truth table, ring oscillator berosilasi |
| M3 | CPU uji menjalankan program dan trace benar |
| M4 | Blink `avr-gcc` berjalan di CPU struktural |
| M5 | Blink + UART hello berjalan end-to-end |
| M6 | Peripheral utama lulus test driver |
| M7 | Board demo deterministik dan reproducible |
| M8 | Analisis/device yang dideklarasikan lulus golden suite |
| M9 | Release v0.1 |

---

## 13. Pelaporan Fidelity

Setiap rilis harus menyertakan:

- Fidelity matrix terbaru.
- Daftar fitur yang didukung.
- Daftar batasan.
- Hasil test.
- Perbedaan yang diketahui terhadap hardware/ngspice.
- Cara mereproduksi.

---

## 14. Batasan Verifikasi Tanpa Lab

Yang **bisa** diverifikasi:

- Perilaku fungsional firmware.
- Timing instruksi/peripheral terhadap datasheet.
- Tegangan/arus terhadap model dan ngspice.
- Konsistensi internal.
- Determinisme.

Yang **tidak bisa** diverifikasi secara penuh:

- Kecocokan silikon internal.
- Perilaku pada suhu/corner.
- Noise dan mismatch.
- Karakteristik analog presisi.
- Efek parasitik PCB.

Semua batasan ini harus tertulis, bukan disembunyikan.

---

## 15. Keputusan Terbuka

| ID | Pertanyaan |
|---|---|
| V01 | Toleransi final per analisis |
| V02 | Versi ngspice golden |
| V03 | Ukuran maksimum golden data |
| V04 | Apakah fuzzing memakai crate? (Tidak, zero dependency) |
| V05 | Apakah differential simavr dijalankan di CI? |
| V06 | Format laporan fidelity |
| V07 | Apakah snapshot/replay masuk M7? |
