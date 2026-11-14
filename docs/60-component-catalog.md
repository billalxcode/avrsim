# 60 - Katalog Komponen EmuCAD

Dokumen ini mendefinisikan taksonomi, kebijakan akurasi, schema katalog,
dan contoh cara kerja komponen. Implementasi dan status per batch ada di
`TODO-COMPONENTS.md`.

## 1. Tujuan

Menyediakan katalog komponen yang:

- Cukup luas untuk membangun rangkaian analog, digital, dan campuran.
- Memiliki model Rust yang deterministik.
- Memiliki referensi yang dapat diaudit.
- Memiliki unit test dan reference vector yang ketat.
- Dapat diekspor ke Python builder.
- Tidak menambah dependensi eksternal Rust.

Target angka: 500 entri katalog. Satu entri adalah satu definisi model
generik, bukan sekadar satu simbol GUI. Status solver ditentukan oleh field
`device`: entry solver-backed dapat memakai model MNA yang ada, entry
behavioral baru memiliki metadata dan smoke response.

## 2. Keputusan Akurasi

Klaim "100 persen akurat" tidak dapat diuji tanpa definisi model,
referensi, dan toleransi. Yang dapat dijamin adalah:

- Persamaan model sesuai referensi yang dikutip.
- Parameter memiliki batas valid.
- Hasil lulus conservation test dan convergence test.
- Hasil dibandingkan dengan reference vector pada toleransi yang
  terdokumentasi.
- Perilaku di luar rentang valid ditolak atau diberi peringatan.

Setiap entri wajib mencantumkan:

- Tingkat model: ideal, linear, nonlinear statis, dinamik, behavioral,
  atau part-specific.
- Rentang parameter dan rentang operasi.
- Referensi: datasheet, buku, manual simulator, atau paper.
- Test yang membuktikan perilaku.
- Batasan yang diketahui.

Toleransi awal yang disarankan:

| Kelas model | Toleransi referensi |
|---|---|
| Ideal linear | 1e-9 relatif |
| Linear dengan parasit | 1e-6 relatif |
| Nonlinear statis | 1 sampai 3 persen |
| Nonlinear dinamik | 2 sampai 5 persen |
| Behavioral digital | truth table dan timing |
| Part-specific | titik datasheet sesuai toleransi datasheet |

Toleransi akhir harus disetujui sebelum batch pertama dimulai.

## 3. Tingkat Model

- T0 ideal: resistor, kapasitor, induktor, source ideal.
- T1 linear parasit: ESR, ESL, tempco, leakage.
- T2 nonlinear statis: diode, BJT DC, JFET DC, MOSFET DC.
- T3 nonlinear dinamik: kapasitansi junction, charge storage, switching.
- T4 behavioral: op-amp macromodel, logic gate, timer, PLL, ADC.
- T5 part-specific: model untuk part number dengan parameter datasheet.

Satu komponen dapat memiliki beberapa tingkat model. Contoh: MOSFET
memiliki T2 untuk DC dan T3 untuk kapasitansi.

## 4. Pembagian 500 Entri

| Kategori | Subkategori | Jumlah |
|---|---|---:|
| 1. Passives RCL | Resistor | 12 |
| | Kapasitor | 13 |
| | Induktor | 10 |
| | Transformer dan coupled inductor | 6 |
| | Crystal dan resonator | 5 |
| | Transmission line dan RF passive | 9 |
| **Subtotal** | | **55** |
| 2. Sources | DC dan AC independent | 6 |
| | Pulse, PWL, sine, noise | 6 |
| | Controlled source E/F/G/H | 4 |
| | Behavioral B source | 4 |
| | Digital clock dan pattern | 3 |
| | Battery, fuel cell, solar cell | 2 |
| **Subtotal** | | **25** |
| 3. Diodes dan opto | Rectifier, Schottky, Zener | 10 |
| | LED, laser, IR | 8 |
| | Photodiode, phototransistor, optocoupler | 8 |
| | Varactor, PIN, TVS, ESD | 6 |
| | Bridge dan array | 4 |
| | Tunnel, avalanche, lainnya | 4 |
| **Subtotal** | | **40** |
| 4. Transistors | BJT NPN dan PNP | 12 |
| | Darlington, array, digital transistor | 8 |
| | JFET N dan P | 8 |
| | MOSFET N dan P | 15 |
| | Power MOSFET, IGBT, GaN, SiC | 8 |
| | UJT, PUT, MESFET, HEMT | 4 |
| **Subtotal** | | **55** |
| 5. Thyristor dan proteksi | SCR, TRIAC, DIAC, GTO | 8 |
| | Fuse, PTC, TVS, varistor | 6 |
| | Spark gap, crowbar, breaker | 4 |
| **Subtotal** | | **18** |
| 6. Switch dan relay | Mechanical switch | 8 |
| | Relay dan solid-state relay | 7 |
| | Analog switch dan multiplexer | 5 |
| | Connector, jumper, test point | 5 |
| **Subtotal** | | **25** |
| 7. Op-amp dan analog IC | Op-amp | 15 |
| | Comparator | 6 |
| | Instrumentation dan isolation | 5 |
| | OTA, Norton, audio, video | 5 |
| | Timer, oscillator, PLL | 5 |
| | Voltage reference | 4 |
| **Subtotal** | | **40** |
| 8. Logic dan digital | Gate | 12 |
| | Flip-flop dan latch | 10 |
| | Counter dan shift register | 10 |
| | Mux, decoder, encoder | 8 |
| | Adder, comparator, buffer | 8 |
| | Level shifter dan interface | 6 |
| | Programmable logic dan state machine | 6 |
| **Subtotal** | | **60** |
| 9. Memory | SRAM, DRAM, ROM | 5 |
| | EEPROM, Flash, FRAM, MRAM | 5 |
| | Module dan controller | 5 |
| **Subtotal** | | **15** |
| 10. MCU dan peripheral | AVR, PIC, 8051 | 5 |
| | ARM, RISC-V, ESP, STM32 | 6 |
| | ADC, DAC, PWM | 5 |
| | UART, SPI, I2C, CAN, USB, Ethernet | 6 |
| | Timer, RTC, watchdog | 3 |
| **Subtotal** | | **25** |
| 11. Power management | Linear regulator dan LDO | 6 |
| | Buck, boost, buck-boost | 8 |
| | Charge pump dan charger | 5 |
| | Reference dan supervisor | 5 |
| | Ideal diode, ORing, load switch | 3 |
| | Motor driver | 3 |
| **Subtotal** | | **30** |
| 12. Sensor | Temperature | 6 |
| | Light dan optical | 5 |
| | Magnetic, Hall, reed | 5 |
| | Motion, accelerometer, gyro, IMU | 5 |
| | Pressure, force, strain | 4 |
| | Humidity, gas, chemical | 4 |
| | Current, voltage, power | 3 |
| | Proximity, touch, ultrasonic | 3 |
| **Subtotal** | | **35** |
| 13. Actuator dan display | LED dan lamp | 4 |
| | Buzzer, speaker, piezo | 4 |
| | Motor, servo, stepper, solenoid | 5 |
| | LCD, OLED, e-ink | 4 |
| | Heater dan Peltier | 3 |
| **Subtotal** | | **20** |
| 14. RF dan microwave | Amplifier | 4 |
| | Mixer dan modulator | 4 |
| | Filter | 4 |
| | Antenna | 3 |
| | Coupler, splitter, isolator | 3 |
| **Subtotal** | | **18** |
| 15. Behavioral block | Integrator, differentiator, filter | 5 |
| | Multiplier, divider, log, RMS | 5 |
| | Comparator, Schmitt, peak detector | 5 |
| | VCO, PLL, modulator | 5 |
| | Sample-hold, ADC, DAC behavioral | 5 |
| **Subtotal** | | **25** |
| 16. Board dan module | Arduino dan board | 3 |
| | Breadboard dan jumper | 3 |
| | Power module dan USB | 3 |
| | Display dan sensor module | 3 |
| | Enclosure dan mechanical | 2 |
| **Subtotal** | | **14** |
| **Total** | | **500** |

## 5. Contoh Cara Kerja

Contoh berikut menjadi pola penulisan untuk setiap entri katalog.

| Komponen | Prinsip | Fokus test |
|---|---|---|
| Resistor ideal | V = I x R; daya P = I^2 x R | linearitas, daya, batas nilai |
| Kapasitor ideal | I = C x dV/dt; energi 0.5 x C x V^2 | integrasi, charge, energi |
| Induktor ideal | V = L x dI/dt; energi 0.5 x L x I^2 | integrasi, saturasi, energi |
| Transformer | Kopling mutual, rasio lilitan, koefisien k | rasio, polaritas, leakage |
| Crystal | Resonansi piezoelektrik dengan model BVD | frekuensi seri dan paralel |
| Diode | Persamaan Shockley dan breakdown reverse | kurva I-V, suhu, breakdown |
| Schottky | Junction metal-semiconductor, Vf rendah | Vf, reverse leakage, switching |
| Zener | Breakdown reverse pada tegangan tertentu | knee, impedans, suhu |
| LED | Rekombinasi radiatif, Vf dan arus | Vf, fluks relatif, batas arus |
| BJT | Ebers-Moll atau Gummel-Poon | beta, Early effect, saturasi |
| JFET | Kontrol junction gate dan pinch-off | Idss, Vp, transconductance |
| MOSFET | Shichman-Hodges atau model lanjutan | Vth, mobility, channel length |
| IGBT | Input MOS dengan output bipolar | Vce(sat), tail current, switching |
| SCR | Struktur empat lapis yang latch | trigger, holding, turn-off |
| TRIAC | SCR bidirectional untuk AC | quadrant, holding, commutation |
| Op-amp | Input diferensial gain tinggi, feedback | offset, slew rate, bandwidth |
| Comparator | Gain loop terbuka, output saturasi | hysteresis, propagation delay |
| Gate logika | Fungsi boolean dan delay propagasi | truth table, fan-out, timing |
| Buck converter | PWM switch dan filter LC | duty cycle, ripple, efisiensi |
| LDO | Pass transistor dengan feedback | dropout, line, load regulation |
| Thermistor NTC | Resistansi turun terhadap suhu | kurva beta, self-heating |
| Hall sensor | Gaya Lorentz menghasilkan tegangan Hall | sensitivitas, linearitas |
| Piezo | Kopling elektromekanik | resonansi, impedans |
| Transmission line | Persamaan telegrapher dan impedans karakteristik | delay, refleksi, loss |
| Memristor | Resistansi bergantung state historis | histeresis, saturasi state |

Setiap entri katalog harus memiliki satu baris seperti tabel di atas,
ditambah referensi dan test konkret.

## 6. Schema Katalog

Contoh entri:

```json
{
  "id": "passive.resistor.ideal",
  "name": "Ideal resistor",
  "family": "passive.resistor",
  "tier": 0,
  "pins": ["A", "B"],
  "params": {
    "resistance": {
      "unit": "ohm",
      "default": 1000.0,
      "min": 1e-9,
      "max": 1e12
    }
  },
  "model": "resistor_ideal",
  "valid_range": "DC dan transient, tanpa parasit",
  "references": [
    "ngspice manual, resistor device"
  ],
  "tests": [
    "ohm_law",
    "power_limit",
    "parameter_bounds"
  ],
  "status": "draft"
}
```

Aturan:

- ID stabil dan lowercase.
- Urutan field deterministik.
- Unit SI.
- Tidak ada nilai default yang tidak terdokumentasi.
- Status: draft, implemented, tested, verified, deprecated.
- Entri verified wajib punya referensi dan test.

## 7. Sumber dan Lisensi

Sumber taksonomi dan model:

- KiCad symbol libraries: kategori, nama generik, dan metadata simbol.
  https://kicad.github.io/symbols
- KiCad-SPICE-Library: indeks model SPICE. Klaim 50.093 model, tetapi
  lisensi tiap file berbeda dan tidak boleh disalin tanpa audit.
  https://github.com/kicad-spice-library/KiCad-Spice-Library
- ngspice manual: tipe device, persamaan, dan parameter model.
  https://ngspice.sourceforge.io/docs/ngspice-html-manual/manual.xhtml
- ngspice model parameters dan koleksi model.
  https://ngspice.sourceforge.io/modelparams.html
- Qucs-S: komponen dasar, library, dan model behavioral.
  https://qucs-s-help.readthedocs.io/en/latest/overview/interface-overview.html
- Qucs-S SPICE components.
  https://qucs-help.readthedocs.io/en/spice4qucs/SPICEComp.html

Kebijakan:

- Prioritaskan datasheet produsen untuk model part-specific.
- Jangan menyalin model LTspice, PSpice, atau vendor yang melarang
  redistribusi.
- Simpan URL, lisensi, tanggal akses, dan versi model.
- Model GPL tidak disalin ke crate MIT. Implementasi generik dibuat ulang
  dari persamaan publik dan datasheet, lalu dicatat sumbernya.
- Symbol dan model elektrik adalah dua artefak terpisah.

## 8. Hubungan dengan Python Builder

Python builder tidak menghitung model. Python hanya:

- Memilih entri katalog.
- Mengisi parameter.
- Membangun topologi.
- Mengirim netlist ke Rust.
- Menampilkan hasil.

Rust mengekspor katalog:

```bash
emucad catalog --json
```

Python memuat katalog itu agar daftar komponen, parameter, dan batas
valid selalu sinkron.

## 9. Status Implementasi

Katalog 500 entri sudah ada di `crates/sim-device/src/catalog.rs` dan
`crates/sim-device/src/catalog_data.rs`. File data dihasilkan oleh
`scripts/gen_component_catalog.py`.

Jumlah saat ini:

- 500 entri katalog.
- 60 model class.
- 206 entri solver-backed dengan `DeviceKind`.
- 294 entri behavioral dengan metadata dan smoke response.

Registry `ComponentRegistry` memetakan entry ke `DeviceInstance` dan
`DeviceModel`. Registry memvalidasi ID, jumlah pin, parameter default, dan
model solver. `ComponentRegistry::add_to` dapat langsung menambah device
ke `CircuitBuilder`.

Test katalog dan emulasi mencakup:

- Jumlah tepat 500 entri.
- ID unik dan slug ASCII.
- Family count sesuai desain.
- Validasi setiap entri.
- Prinsip dan sumber non-empty.
- Pemetaan entry analog ke `DeviceKind`.
- Instansiasi dan pembentukan model untuk semua entry solver-backed.
- Voltage divider melalui registry menghasilkan 2.5 V.
- Saklar ideal melalui registry mengubah tegangan node.
- Op-amp follower melalui registry mengikuti tegangan input.
- Comparator melalui registry saturasi ke rel atas dan bawah.
- Analog switch melalui registry mengikuti tegangan kontrol.
- Relay melalui registry memindahkan kontak NO/NC.
- Regulator melalui registry menetapkan tegangan output.
- AND gate melalui registry menggerakkan output ke rel logika.

Entry behavioral masih berupa metadata dan smoke response; model solver
penuh untuk keluarga tersebut ada di backlog `TODO-COMPONENTS.md`.

## 10. Batasan

- 500 entri adalah target jangka panjang, bukan satu sprint.
- Tidak semua entri memerlukan model fisik penuh.
- Model part-specific memerlukan datasheet dan verifikasi manual.
- Model digital tidak dapat dibuktikan dengan kurva analog.
- Model RF memerlukan data S-parameter jika akurasi tinggi dibutuhkan.
- Beberapa komponen elektromekanik memerlukan model multi-domain.

## 11. Referensi Internal

- `TODO-COMPONENTS.md`: backlog dan status batch.
- `docs/11-netlist.md`: format netlist.
- `docs/12-device-models.md`: model device yang sudah ada.
- `docs/24-fidelity.md`: kebijakan fidelity.
- `docs/60-component-catalog.md`: dokumen ini.
