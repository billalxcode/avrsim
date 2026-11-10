# 01 — Decision Log

Dokumen ini mencatat seluruh keputusan desain yang diambil selama brainstorming, beserta alasan dan implikasinya.

> **Aturan:** keputusan yang sudah dicatat di sini tidak dibuka kembali tanpa alasan kuat. Perubahan harus dicatat sebagai revisi dengan tanggal dan alasan.

---

## 1. Ringkasan Keputusan

| ID | Topik | Keputusan |
|---|---|---|
| D01 | Fondasi | Transistor-level |
| D02 | Model MCU | Seluruh MCU transistor-level |
| D03 | Target chip | ATmega328P (Arduino Uno) |
| D04 | Sumber netlist | Clean-room fungsional, bertahap per blok |
| D05 | Blok pertama | Primitif CMOS |
| D06 | Model MOSFET | Shichman-Hodges Level 1 |
| D07 | Integrasi waktu | Backward Euler |
| D08 | Nonlinear solver | Newton-Raphson |
| D09 | Firmware pertama | Bare-metal kecil: blink + UART hello |
| D10 | Target kecepatan | Correctness dulu, offline |
| D11 | Format netlist | Rust builder API |
| D12 | Verifikasi awal | Device & solver vs ngspice |
| D13 | Power/clock awal | VDD/GND ideal, clock ideal |
| D14 | Hierarki | Hierarki + flatten otomatis |
| D15 | Observability | VCD + GTKWave; CSV untuk DC |
| D16 | Toolchain firmware | `avr-gcc` eksternal, HEX/ELF |
| D17 | Cakupan ISA awal | Seluruh ISA AVR + peripheral dasar |
| D18 | Konstruksi CPU | Standard cell + generator |
| D19 | Memori | Register file struktural; SRAM/flash behavioral |
| D20 | Peripheral | Semua struktural (target akhir) |
| D21 | Clock | Waktu fisik 16 MHz |
| D22 | Kalibrasi | Ke ATmega328P asli |
| D23 | Akses lab | Tidak ada; data publik |
| D24 | Strategi kalibrasi | Level pin dari datasheet |
| D25 | Sumber daya | Solo, jangka panjang |
| D26 | Deliverable pertama | Kurva VTC inverter |
| D27 | Analisis | Seluruh analisis SPICE (target akhir) |
| D28 | Device library | Pustaka SPICE lengkap (target akhir) |
| D29 | Prioritas | Visi penuh, milestone kecil |
| D30 | Milestone 1 | MOSFET + R + C + sumber; DC + transient; VTC |
| D31 | Solver linear | Dense LU from scratch |
| D32 | Konvergensi | Semua metode: limiting, gmin, source stepping, damping |
| D33 | Kapasitansi | Model Meyer |
| D34 | PVT | Nominal dulu |
| D35 | Test referensi | Suite lengkap |
| D36 | Bentuk software | Workspace Rust: library + CLI |
| D37 | Lisensi | MIT / Apache-2.0 |
| D38 | Control unit | Hardwired / PLA |
| D39 | Organisasi CPU | Multi-cycle FSM |
| D40 | ALU | Ripple-carry + bit-slice |
| D41 | Register file | DFF + decoder |
| D42 | Reset | Struktural penuh (target akhir) |
| D43 | Board | IC aktif struktural; pasif model device; host UART behavioral |
| D44 | Sumber netlist internal | Clean-room, bukan salinan die |
| D45 | Kriteria sukses | Fungsional + timing + elektrik |
| D46 | Dependensi | Zero external dependency; std-only |
| D47 | Oracle eksternal | Hanya test/golden reference |
| D48 | Bahasa | Kode Inggris, dokumentasi Indonesia |
| D49 | Output | CSV DC, VCD transient |
| D50 | Platform | Linux, macOS, Windows |
| D51 | Determinisme | Toleransi kecil lintas platform |
| D52 | API netlist | Builder + macro deklaratif |
| D53 | Debug CPU | CLI monitor |
| D54 | Model kapasitansi transistor | Meyer |
| D55 | Proses | PTM 350 nm generik, kalibrasi pin-level |

---

## 2. Detail Keputusan

### D01 — Fondasi transistor-level

**Keputusan:** fondasi simulator adalah transistor, bukan gerbang digital atau instruksi.

**Alasan:** visi pengguna adalah emulasi “dari hal terkecil”.

**Implikasi:**
- Device model fisik harus dibangun lebih dulu.
- Gerbang digital adalah komposisi transistor, bukan primitif bawaan.
- Solver analog menjadi inti, bukan tambahan.

### D02 — Seluruh MCU transistor-level

**Keputusan:** ATmega328P dimodelkan secara struktural, bukan ISS.

**Alasan:** konsistensi dengan visi transistor-level.

**Implikasi:**
- CPU, register file, dan peripheral dibangun dari gerbang/sel.
- SRAM dan flash tetap behavioral (pengecualian D19).
- Simulasi akan sangat lambat; target offline.

### D03 — Target ATmega328P

**Keputusan:** chip target pertama adalah ATmega328P.

**Alasan:** target Arduino, ekosistem firmware luas, datasheet lengkap.

**Implikasi:**
- Perlu model 32 KB flash, 2 KB SRAM, 1 KB EEPROM.
- Perlu timer, UART, GPIO, interrupt controller, ADC, dan lainnya.
- Perlu profil fuse/clock Arduino Uno.

### D04 — Clean-room fungsional

**Keputusan:** netlist internal adalah rancangan sendiri yang meniru perilaku teramati, bukan hasil ekstraksi die.

**Alasan:** tidak ada akses lab; menghindari masalah hukum; tetap dapat diverifikasi lewat firmware dan datasheet.

**Implikasi:**
- Tidak ada klaim identik dengan silikon.
- Fidelity diukur di level pin dan perilaku firmware.
- Parameter internal memakai model generik.

### D05 — Primitif CMOS pertama

**Keputusan:** blok pertama adalah inverter, NAND/NOR, transmission gate, SRAM cell, dan flip-flop dari transistor.

**Alasan:** fondasi semua blok digital; mudah diuji dengan tabel kebenaran.

**Implikasi:**
- Perlu pustaka sel standar.
- Perlu characterization harness.
- Perlu model kapasitansi Meyer untuk timing.

### D06 — Shichman-Hodges Level 1

**Keputusan:** MOSFET dimodelkan dengan persamaan kuadratik Level 1.

**Alasan:** cukup untuk CMOS digital, parameter mudah dikalibrasi, jauh lebih cepat daripada BSIM.

**Implikasi:**
- Akurasi analog terbatas.
- Body effect dan channel-length modulation harus ada.
- Model kapasitansi Meyer ditambahkan untuk transient.

### D07–D08 — Backward Euler + Newton-Raphson

**Keputusan:** integrasi waktu Backward Euler dan solver nonlinear Newton-Raphson.

**Alasan:** stabil, sederhana, mudah di-debug untuk milestone awal.

**Implikasi:**
- Akurasi orde satu; time step harus kecil.
- Perlu limiting, gmin, dan source stepping.
- Trapezoidal/Gear dapat ditambahkan nanti.

### D09 — Firmware pertama

**Keputusan:** bare-metal blink + UART hello, bukan Arduino core penuh.

**Alasan:** menguji CPU, clock, GPIO, timer, dan UART tanpa seluruh peripheral.

**Implikasi:**
- Perlu loader HEX.
- Perlu reset vector dan fuse clock.
- Perlu UART host endpoint.

### D10 — Correctness dulu, offline

**Keputusan:** tidak mengejar real-time.

**Alasan:** simulasi struktural penuh terlalu lambat untuk real-time.

**Implikasi:**
- CLI menjalankan simulasi sampai selesai atau batas waktu virtual.
- Snapshot/replay menjadi penting.
- Optimasi performa dilakukan setelah correctness.

### D11 — Rust builder API

**Keputusan:** rangkaian didefinisikan dalam kode Rust, bukan SPICE netlist.

**Alasan:** type-safe, mudah di-refactor, zero dependency.

**Implikasi:**
- Perlu API builder yang ergonomis.
- Perlu macro deklaratif untuk rangkaian besar.
- Impor SPICE tidak menjadi prioritas.

### D12 — Verifikasi device vs ngspice

**Keputusan:** device dan solver divalidasi terhadap ngspice sebagai golden reference.

**Alasan:** ngspice adalah referensi matang; runtime tetap native.

**Implikasi:**
- ngspice hanya dev/test, bukan dependensi runtime.
- Perlu format pertukaran data (CSV).
- Perlu toleransi per analisis.

### D13 — Power/clock ideal awal

**Keputusan:** VDD/GND ideal dan clock ideal pada tahap primitif.

**Alasan:** mengurangi kompleksitas sebelum device dan sel stabil.

**Implikasi:**
- IR drop dan supply noise tidak dimodelkan awal.
- Power tree struktural menjadi tahap lanjut.

### D14 — Hierarki + flatten

**Keputusan:** desain modular, solver melihat netlist datar.

**Alasan:** keterbacaan desain dan efisiensi solver.

**Implikasi:**
- Perlu subcircuit dan port.
- Perlu algoritma flatten yang menjaga nama node.
- Perlu validasi setelah flatten.

### D15 — VCD + CSV

**Keputusan:** CSV untuk DC, VCD untuk transient, GTKWave untuk melihat.

**Alasan:** format standar, hemat waktu, emulator tetap fokus.

**Implikasi:**
- Perlu penulis VCD dan CSV sendiri.
- GTKWave adalah alat eksternal opsional, bukan dependensi.

### D16 — avr-gcc eksternal

**Keputusan:** firmware dikompilasi di luar emulator.

**Alasan:** tidak perlu membangun compiler.

**Implikasi:**
- Perlu loader HEX dan ELF.
- Perlu profil fuse/clock.
- Runtime tetap native.

### D17 — Seluruh ISA + peripheral dasar

**Keputusan:** milestone CPU mencakup seluruh ISA AVR dan peripheral dasar.

**Alasan:** target “firmware berjalan” tanpa illegal instruction.

**Implikasi:**
- Pekerjaan besar; perlu test vector lengkap.
- Peripheral dasar: GPIO, timer, UART, interrupt.
- Peripheral lain menyusul.

### D18 — Standard cell + generator

**Keputusan:** CPU disusun dari pustaka sel CMOS dan generator netlist.

**Alasan:** sistematis, dapat diuji, menghindari desain full-custom yang sangat lama.

**Implikasi:**
- Perlu sel standar dan characterization.
- Perlu generator untuk ALU, register, control.
- Netlist akhir tetap transistor-level setelah flatten.

### D19 — Memori

**Keputusan:** register file struktural; SRAM/flash behavioral.

**Alasan:** SRAM/flash transistor-level akan meledakkan jumlah node.

**Implikasi:**
- Register CPU tetap dapat diverifikasi struktural.
- Akses memori harus mengikuti timing behavioral yang didefinisikan.
- Pengecualian dicatat di fidelity matrix.

### D20 — Semua peripheral struktural

**Keputusan:** target akhir seluruh peripheral struktural.

**Alasan:** konsistensi visi.

**Implikasi:**
- GPIO, timer, UART, SPI, I2C, ADC, EEPROM, WDT semuanya dari transistor.
- Milestone pertama hanya GPIO/timer/UART.
- Beban implementasi sangat besar.

### D21 — Waktu fisik 16 MHz

**Keputusan:** clock 16 MHz dan waktu fisik; delay gerbang menentukan perilaku.

**Alasan:** konsisten dengan simulasi struktural.

**Implikasi:**
- Jika rangkaian tidak stabil pada 16 MHz, itu bug desain.
- Time step harus cukup kecil.
- Timing firmware bergantung pada model delay.

### D22–D24 — Kalibrasi

**Keputusan:** kalibrasi ke ATmega328P, tanpa lab, menggunakan datasheet pada level pin.

**Alasan:** tidak ada akses laboratorium.

**Implikasi:**
- VIH/VIL, VOH/VOL, arus drive, dan timing dicocokkan.
- Parameter transistor internal memakai PTM generik.
- Klaim akurasi harus konservatif.

### D25 — Solo, jangka panjang

**Keputusan:** proyek dikerjakan sendiri tanpa deadline ketat.

**Alasan:** scope besar; kualitas lebih penting daripada kecepatan.

**Implikasi:**
- Milestone harus kecil.
- Dokumentasi dan reproducibility penting.
- Scope guardrail diperlukan.

### D26 — VTC inverter

**Keputusan:** hasil nyata pertama adalah kurva VTC inverter.

**Alasan:** membuktikan device, solver DC, dan alur simulasi.

**Implikasi:**
- Perlu DC sweep.
- Perlu MOSFET + R/C + sumber.
- Perlu perbandingan ngspice.

### D27–D28 — Full SPICE (target akhir)

**Keputusan:** seluruh analisis dan pustaka device adalah target akhir.

**Alasan:** visi simulator elektronika lengkap.

**Implikasi:**
- AC, noise, distortion, sensitivity, temperature, corners.
- BJT, JFET, switch, controlled source, transmission line.
- Tidak harus selesai di milestone pertama.

### D29–D30 — Milestone 1

**Keputusan:** MOSFET + R + C + sumber; DC + transient; VTC inverter.

**Alasan:** vertical slice terkecil yang bermakna.

**Implikasi:**
- Device lain menyusul.
- AC/noise menyusul.
- Fokus pada kebenaran dasar.

### D31–D32 — Solver linear dan konvergensi

**Keputusan:** dense LU from scratch; semua metode konvergensi.

**Alasan:** kesederhanaan awal dan robustness.

**Implikasi:**
- Dense LU cukup untuk rangkaian kecil.
- Sparse menyusul.
- Semua fallback harus diuji.

### D33 — Meyer capacitance

**Keputusan:** kapasitansi MOSFET memakai model Meyer.

**Alasan:** cukup untuk timing CMOS, lebih sederhana daripada model berbasis muatan.

**Implikasi:**
- Perlu implementasi kapasitansi per region.
- Akurasi switching terbatas tetapi memadai.

### D34 — Nominal PVT

**Keputusan:** suhu ruang dan parameter typical dulu.

**Alasan:** mengurangi variabel sebelum model stabil.

**Implikasi:**
- Temperature/corner menyusul.
- Fidelity matrix mencatat batasan.

### D35 — Suite referensi lengkap

**Keputusan:** inverter DC, RC transient, ring oscillator, current mirror, differential pair.

**Alasan:** validasi menyeluruh.

**Implikasi:**
- Perlu banyak golden data.
- Perlu otomasi perbandingan.

### D36–D37 — Workspace dan lisensi

**Keputusan:** workspace library + CLI; MIT/Apache-2.0.

**Alasan:** modular, dapat dipakai ulang, open source.

**Implikasi:**
- Crate internal terpisah.
- Tidak ada crate eksternal.
- Dokumentasi lisensi pada P0.

### D38–D41 — CPU

**Keputusan:** hardwired/PLA, multi-cycle FSM, ripple-carry ALU, DFF register file.

**Alasan:** autentik, dapat diverifikasi, lebih sederhana daripada pipeline.

**Implikasi:**
- Control logic kompleks tetapi eksplisit.
- Timing per instruksi dapat diukur.
- Register file besar tetapi jelas.

### D42–D43 — Reset dan board

**Keputusan:** reset struktural penuh (target akhir); board IC aktif struktural, pasif device, host UART behavioral.

**Alasan:** konsistensi visi dengan batas yang realistis.

**Implikasi:**
- POR/brownout/reset tree menjadi tahap lanjut.
- Host PC tidak disimulasikan.
- PCB parasitics tidak dimodelkan awal.

### D44–D53 — Prinsip lintas area

**Keputusan:** clean-room, kriteria sukses menyeluruh, zero dependency, oracle test-only, bahasa kode Inggris/docs Indonesia, CSV/VCD, cross-platform, determinisme toleran, builder+macro, CLI monitor.

**Implikasi:**
- Semua keputusan ini berlaku sebagai aturan proyek.
- Penyimpangan harus dicatat sebagai revisi.

---

## 3. Kontradiksi dan Resolusi

| ID | Kontradiksi | Resolusi |
|---|---|---|
| C01 | Seluruh MCU transistor-level vs SRAM/flash behavioral | Pengecualian fidelity eksplisit; SRAM/flash tetap behavioral |
| C02 | Semua peripheral struktural vs milestone pertama GPIO/timer/UART | Target akhir vs milestone pertama |
| C03 | Seluruh analisis SPICE vs milestone pertama DC/transient | Target akhir vs milestone pertama |
| C04 | Kalibrasi ke chip asli vs tanpa lab | Kalibrasi pin-level dari datasheet; parameter internal generik |
| C05 | Power/clock ideal vs reset struktural | Tahap primitif ideal; reset/power struktural menyusul |
| C06 | Seluruh board transistor-level vs host UART behavioral | Host berada di luar board; endpoint behavioral |
| C07 | Zero dependency vs workspace | Crate eksternal dilarang; crate internal diperbolehkan |
| C08 | Seluruh ISA + peripheral dasar vs bertahap per blok | Milestone besar dipecah menjadi blok dan test |
| C09 | Real-time vs transistor-level penuh | Offline correctness; real-time bukan target |
| C10 | 100% semua aspek vs keterbatasan verifikasi | Fidelity matrix + toleransi + dokumentasi batasan |

---

## 4. Asumsi

1. Rust stable dapat dipasang di lingkungan pengembangan.
2. `avr-gcc` tersedia saat milestone firmware.
3. Parameter PTM 350 nm tersedia publik.
4. Datasheet ATmega328P tersedia.
5. Tidak ada netlist proprietary yang dipakai.
6. Tidak ada akses lab untuk karakterisasi.
7. Simulasi awal single-threaded.
8. Host memiliki CPU/RAM cukup untuk simulasi offline.
9. GTKWave tersedia secara opsional untuk melihat VCD.
10. Proyek dapat berevolusi menjadi open source dengan kontribusi komunitas.

---

## 5. Keputusan Terbuka (TBD)

| ID | Pertanyaan | Kapan diputuskan |
|---|---|---|
| T01 | VDD untuk test inverter: 3.3 V atau 5 V? | Sebelum M1 |
| T02 | Time step transient awal | Sebelum M1 |
| T03 | Toleransi VTC/transient vs ngspice | Sebelum M1 |
| T04 | Parameter PTM persisnya | Sebelum M1 |
| T05 | Detail model SRAM/flash behavioral | Sebelum M4 |
| T06 | Apakah bootloader Arduino dimodelkan? | Sebelum M5 |
| T07 | Baud rate demo UART | Sebelum M5 |
| T08 | Sumber firmware demo: C atau assembly? | Sebelum M5 |
| T09 | Model timing flash wait state | Sebelum M4 |
| T10 | Apakah ADC input terhubung ke solver analog? | Sebelum M6 |
| T11 | Ambang sparse solver | Sebelum P10 |
| T12 | Detail noise analysis | Sebelum P10 |
| T13 | GUI viewer masa depan | Setelah M9 |
| T14 | CI provider | Sebelum P0 selesai |

---

## 6. Aturan Perubahan

- Setiap perubahan keputusan harus menambah entri revisi.
- Perubahan yang memengaruhi fidelity matrix harus memperbarui `07-verification.md`.
- Perubahan scope harus memperbarui `08-roadmap.md` dan `TODO.md`.
- Keputusan yang belum final ditandai `TBD`, bukan diasumsikan.
