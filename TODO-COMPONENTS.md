# TODO - Komponen 500

Status: katalog 500 entri sudah diimplementasikan di `sim-device`.
Model solver penuh per keluarga behavioral masih bertahap.

Tujuan: membangun katalog 500 model komponen yang terverifikasi, dapat
diekspor ke Python builder, dan diuji dengan unit test ketat.

Dokumen desain: `docs/60-component-catalog.md`.

## 1. Rekomendasi

Mulai dari 80 komponen inti analog dan digital, bukan langsung 500.
Setelah pipeline katalog, test, dan referensi stabil, tambah batch 20
sampai 30 komponen sampai mencapai 500.

Urutan yang disarankan:

1. Passives dan sources.
2. Diode, BJT, JFET, MOSFET.
3. Proteksi, switch, relay.
4. Op-amp, comparator, timer.
5. Logic, memory, MCU peripheral.
6. Power, sensor, actuator.
7. RF, behavioral, board module.
8. Part-specific models.
9. Python builder.

## 2. Keputusan yang Harus Disetujui

- [x] C0-01 Setujui bahwa 500 entri berarti 500 definisi model generik,
  bukan 500 part number.
- [x] C0-02 Setujui tingkat model T0 sampai T5 di `docs/60`.
- [x] C0-03 Setujui toleransi referensi per tingkat model.
- [x] C0-04 Setujui schema katalog dan status draft sampai verified.
- [x] C0-05 Setujui kebijakan sumber dan lisensi clean-room tanpa salin GPL.
- [ ] C0-06 Pilih analisis pertama: DC operating point, transient, atau AC.
- [ ] C0-07 Tentukan rasio model generik dan part-specific.
- [x] C0-08 Tentukan lingkup Python builder: batch atau interaktif.
- [x] C0-09 Setujui lokasi katalog di `sim-device` tanpa crate baru.
- [x] C0-10 Setujui nama, unit SI, dan aturan versioning.

## 3. Aturan Batch

Setiap batch dinyatakan selesai jika:

- Entri katalog memiliki prinsip kerja satu paragraf.
- Parameter memiliki unit, default, min, dan max.
- Model Rust deterministic dan tidak memakai wall-clock.
- Minimal tiga unit test: nominal, batas, dan error.
- Minimal satu conservation atau energy test.
- Minimal satu reference vector dari datasheet atau simulator.
- Referensi dan lisensi dicatat.
- Status berubah menjadi `verified`.
- Dokumentasi dan contoh diperbarui.

Batch yang gagal salah satu syarat tetap berstatus `tested`, bukan
`verified`.

## 4. Infrastruktur

- [ ] C1-01 Buat modul katalog di `crates/sim-device`.
- [x] C1-02 Definisikan struct `ComponentSpec` dan parameter typed.
- [x] C1-03 Buat validator ID, unit, pin, dan rentang parameter.
- [x] C1-04 Buat loader katalog deterministik.
- [ ] C1-05 Tambah perintah `embed-emu catalog --json`.
- [ ] C1-06 Buat format reference vector dan parser test.
- [ ] C1-07 Buat harness unit test model.
- [ ] C1-08 Buat harness conservation test KCL, KVL, dan energi.
- [ ] C1-09 Buat harness convergence dan parameter fuzz.
- [ ] C1-10 Buat cross-check opsional terhadap ngspice.
- [ ] C1-11 Tambah CI Linux untuk katalog dan test model.
- [x] C1-12 Tambah laporan coverage per kategori dan status.

## 4a. Status Katalog

- [x] C1-13 Hasilkan 500 entri katalog deterministik.
- [x] C1-14 Uji jumlah 500, ID unik, slug ASCII, dan family count.
- [x] C1-15 Uji validasi setiap entri dan smoke response finite.
- [x] C1-16 Petakan entry analog ke `DeviceKind` yang sudah didukung.
- [x] C1-19 Buat `ComponentRegistry` dan `ComponentRequest`.
- [x] C1-20 Uji emulasi voltage divider dan saklar ideal melalui registry.
- [ ] C1-17 Implementasikan solver penuh untuk keluarga behavioral.
- [ ] C1-18 Tambah reference vector dan cross-check ngspice per model.

Katalog adalah lapisan registry. Entry dengan `device` sudah dapat
memakai model solver yang ada. Entry behavioral baru memiliki metadata,
prinsip, dan smoke response; solver penuhnya ada di backlog C2 sampai C9.

## 5. Passives dan Sources

Target: 80 entri.

- [ ] C2-01 Resistor 12 entri: ideal, tempco, variable, digital pot,
  NTC, PTC, photoresistor, varistor, strain gauge, memristor, fuse,
  ferrite bead.
- [ ] C2-02 Kapasitor 13 entri: ideal, C0G, X7R, electrolytic, tantalum,
  film, variable, trimmer, supercap, feedthrough, ESR, ESL, voltage
  dependent.
- [ ] C2-03 Induktor 10 entri: ideal, lossy, saturable, coupled, air core,
  ferrite core, toroid, current transformer, flyback, common mode choke.
- [ ] C2-04 Transformer 6 entri: ideal, center tap, autotransformer,
  isolation, pulse, current sense.
- [ ] C2-05 Crystal dan resonator 5 entri: quartz, ceramic, tuning fork,
  TCXO, VCXO.
- [ ] C2-06 Transmission line dan RF passive 9 entri: ideal line, lossy
  line, coax, microstrip, stripline, stub, attenuator, splitter,
  directional coupler.
- [ ] C2-07 Independent source 12 entri: DC V, DC I, AC V, AC I, sine,
  pulse, PWL, exponential, noise V, noise I, clock, pattern.
- [ ] C2-08 Controlled dan behavioral source 8 entri: E, F, G, H, B
  voltage, B current, polynomial, table.
- [ ] C2-09 Battery dan energy source 5 entri: battery, fuel cell, solar
  cell, current-limited source, ideal energy port.

## 6. Semikonduktor

Target: 95 entri.

- [ ] C3-01 Rectifier, Schottky, Zener 10 entri.
- [ ] C3-02 LED, laser, IR 8 entri.
- [ ] C3-03 Photodiode, phototransistor, optocoupler 8 entri.
- [ ] C3-04 Varactor, PIN, TVS, ESD, bridge, array 10 entri.
- [ ] C3-05 Tunnel, avalanche, diac, lainnya 4 entri.
- [ ] C3-06 BJT NPN dan PNP 12 entri.
- [ ] C3-07 Darlington, array, digital transistor 8 entri.
- [ ] C3-08 JFET N dan P 8 entri.
- [ ] C3-09 MOSFET N dan P 15 entri: enhancement, depletion, dual gate,
  four terminal, body diode.
- [ ] C3-10 Power MOSFET, IGBT, GaN, SiC 8 entri.
- [ ] C3-11 UJT, PUT, MESFET, HEMT 4 entri.

## 7. Proteksi, Switch, Relay

Target: 43 entri.

- [ ] C4-01 SCR, TRIAC, DIAC, GTO 8 entri.
- [ ] C4-02 Fuse, PTC, TVS, varistor 6 entri.
- [ ] C4-03 Spark gap, crowbar, breaker 4 entri.
- [ ] C4-04 Mechanical switch 8 entri: SPST, SPDT, DPDT, push, toggle,
  DIP, reed, limit.
- [ ] C4-05 Relay dan solid-state relay 7 entri.
- [ ] C4-06 Analog switch dan multiplexer 5 entri.
- [ ] C4-07 Connector, jumper, test point 5 entri.

## 8. Op-Amp dan Analog IC

Target: 40 entri.

- [ ] C5-01 Op-amp 15 entri: ideal, single, dual, rail-to-rail, precision,
  low-noise, high-speed, power, fully differential.
- [ ] C5-02 Comparator 6 entri.
- [ ] C5-03 Instrumentation dan isolation amplifier 5 entri.
- [ ] C5-04 OTA, Norton, audio, video amplifier 5 entri.
- [ ] C5-05 Timer, oscillator, PLL 5 entri.
- [ ] C5-06 Voltage reference 4 entri.

## 9. Logic, Memory, MCU

Target: 100 entri.

- [ ] C6-01 Gate 12 entri: AND, OR, NOT, NAND, NOR, XOR, XNOR, buffer,
  Schmitt, open drain, tri-state, transmission gate.
- [ ] C6-02 Flip-flop dan latch 10 entri.
- [ ] C6-03 Counter dan shift register 10 entri.
- [ ] C6-04 Mux, decoder, encoder 8 entri.
- [ ] C6-05 Adder, comparator, buffer 8 entri.
- [ ] C6-06 Level shifter dan interface 6 entri.
- [ ] C6-07 Programmable logic dan state machine 6 entri.
- [ ] C6-08 Memory 15 entri: SRAM, DRAM, ROM, EEPROM, Flash, FRAM, MRAM,
  module, controller.
- [ ] C6-09 MCU core 11 entri: AVR, PIC, 8051, ARM, RISC-V, ESP, STM32.
- [ ] C6-10 ADC, DAC, PWM peripheral 5 entri.
- [ ] C6-11 UART, SPI, I2C, CAN, USB, Ethernet 6 entri.
- [ ] C6-12 Timer, RTC, watchdog 3 entri.

## 10. Power, Sensor, Actuator

Target: 85 entri.

- [ ] C7-01 Linear regulator dan LDO 6 entri.
- [ ] C7-02 Buck, boost, buck-boost 8 entri.
- [ ] C7-03 Charge pump dan charger 5 entri.
- [ ] C7-04 Reference dan supervisor 5 entri.
- [ ] C7-05 Ideal diode, ORing, load switch 3 entri.
- [ ] C7-06 Motor driver 3 entri.
- [ ] C7-07 Temperature sensor 6 entri.
- [ ] C7-08 Light dan optical sensor 5 entri.
- [ ] C7-09 Magnetic, Hall, reed 5 entri.
- [ ] C7-10 Motion, accelerometer, gyro, IMU 5 entri.
- [ ] C7-11 Pressure, force, strain 4 entri.
- [ ] C7-12 Humidity, gas, chemical 4 entri.
- [ ] C7-13 Current, voltage, power sensor 3 entri.
- [ ] C7-14 Proximity, touch, ultrasonic 3 entri.
- [ ] C7-15 LED dan lamp 4 entri.
- [ ] C7-16 Buzzer, speaker, piezo 4 entri.
- [ ] C7-17 Motor, servo, stepper, solenoid 5 entri.
- [ ] C7-18 LCD, OLED, e-ink 4 entri.
- [ ] C7-19 Heater dan Peltier 3 entri.

## 11. RF dan Behavioral

Target: 43 entri.

- [ ] C8-01 RF amplifier 4 entri.
- [ ] C8-02 RF mixer dan modulator 4 entri.
- [ ] C8-03 RF filter 4 entri.
- [ ] C8-04 Antenna 3 entri.
- [ ] C8-05 Coupler, splitter, isolator 3 entri.
- [ ] C8-06 Integrator, differentiator, filter 5 entri.
- [ ] C8-07 Multiplier, divider, log, RMS 5 entri.
- [ ] C8-08 Comparator, Schmitt, peak detector 5 entri.
- [ ] C8-09 VCO, PLL, modulator 5 entri.
- [ ] C8-10 Sample-hold, ADC, DAC behavioral 5 entri.

## 12. Board dan Module

Target: 14 entri.

- [ ] C9-01 Arduino dan board 3 entri.
- [ ] C9-02 Breadboard dan jumper 3 entri.
- [ ] C9-03 Power module dan USB 3 entri.
- [ ] C9-04 Display dan sensor module 3 entri.
- [ ] C9-05 Enclosure dan mechanical 2 entri.

## 13. Part-Specific dan Validasi

- [ ] C10-01 Pilih 50 part number prioritas dari kategori populer.
- [ ] C10-02 Ekstrak parameter dari datasheet resmi.
- [ ] C10-03 Bandingkan model dengan ngspice atau referensi eksternal.
- [ ] C10-04 Uji suhu, corner, dan parameter toleransi.
- [ ] C10-05 Buat regression golden untuk model kritis.
- [ ] C10-06 Audit lisensi semua model dan datasheet.

## 14. Python Builder

- [ ] C11-01 Kunci schema request dan response.
- [ ] C11-02 Buat loader katalog dari Rust.
- [ ] C11-03 Buat DSL Python tipis untuk topologi dan parameter.
- [ ] C11-04 Buat exporter netlist JSON.
- [ ] C11-05 Buat runner subprocess ke Rust.
- [ ] C11-06 Buat mapping error Rust ke exception Python.
- [ ] C11-07 Buat contoh divider, RC, diode, dan logic.
- [ ] C11-08 Buat contract test Python ke Rust.

## 15. Release dan Dokumentasi

- [ ] C12-01 Perbarui `docs/12-device-models.md`.
- [ ] C12-02 Perbarui `docs/11-netlist.md`.
- [ ] C12-03 Perbarui `README.md` dan `docs/README.md`.
- [ ] C12-04 Tambah laporan coverage 500 komponen.
- [ ] C12-05 Jalankan fmt, clippy, test, ASCII, dan package.
- [ ] C12-06 Buat tag katalog komponen.

## 16. Definition of Done

- [ ] 500 entri katalog ada dan dapat dimuat.
- [ ] Semua entri punya prinsip kerja, parameter, dan referensi.
- [ ] Semua entri punya minimal tiga unit test.
- [ ] Semua entri verified lulus reference vector.
- [ ] Tidak ada dependensi eksternal baru di crate inti.
- [ ] Tidak ada `unsafe`.
- [ ] Tidak ada wall-clock di simulation.
- [ ] Semua fungsi maksimal 20 baris.
- [ ] ASCII-only.
- [ ] Python builder dapat memuat katalog dan menjalankan netlist.
- [ ] fmt, clippy, test, dan package lulus.

## 17. Pertanyaan Terbuka

- Apakah 500 entri harus 500 model generik, atau 500 part number?
- Berapa rasio part-specific yang diinginkan?
- Toleransi 1 sampai 5 persen dapat diterima untuk model nonlinear?
- Apakah model GPL boleh dipakai, atau hanya datasheet dan model MIT/BSD?
- Apakah Python builder wajib interaktif sejak fase pertama?
- Siapa yang melakukan review elektrik untuk model part-specific?
- Apakah RF memerlukan S-parameter sejak awal atau model lumped cukup?
