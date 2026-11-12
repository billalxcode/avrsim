# 43 - Panduan Pengguna

Panduan ini menjelaskan cara membangun, menjalankan, dan memakai
`embed-emulation` dari CLI maupun library Rust.

**Kesimpulan:** simulator sudah dapat dipakai untuk rangkaian analog kecil,
sel CMOS, program AVR, dan board sederhana. Batas utama ada pada verifikasi
eksternal dan model device yang disederhanakan.

---

## 1. Ringkasan cepat

Perintah minimum:

```bash
cargo build --workspace
cargo test --workspace
cargo run -p sim-analysis --example rc_analysis
cargo run -p sim-analysis --example inverter_vtc
cargo run -p sim-avr --example cpu_program
```

Contoh `rc_analysis` menjalankan DC, transient, dan AC pada satu rangkaian RC.

---

## 2. Prasyarat

- Rust stable sesuai `rust-toolchain.toml`.
- Linux, macOS, atau Windows dengan toolchain Rust.
- Tidak ada dependensi Rust eksternal.
- Tidak ada `unsafe` di workspace.
- `avr-gcc` hanya diperlukan untuk mengompilasi firmware AVR baru.

Verifikasi toolchain:

```bash
cargo --version
rustc --version
```

Jika `cargo` tidak ditemukan pada shell non-interaktif:

```bash
source "$HOME/.cargo/env"
```

---

## 3. Build dan verifikasi

### 3.1 Build debug

```bash
cargo build --workspace
```

### 3.2 Build release

```bash
cargo build --workspace --release
```

### 3.3 Test

```bash
cargo test --workspace
```

Untuk satu crate:

```bash
cargo test -p sim-analysis
cargo test -p sim-device
cargo test -p sim-avr
```

### 3.4 Pemeriksaan lengkap

```bash
bash .opencode/skills/clean-code/scripts/check.sh
```

Perintah tersebut menjalankan `fmt`, `clippy`, dan seluruh test workspace.

### 3.5 Packaging

```bash
./scripts/package.sh
```

Hasil berada di:

```text
dist/embed-emu-<target>.tar.gz
```

---

## 4. Struktur workspace

| Crate | Peran |
|---|---|
| `sim-core` | Waktu, satuan, node, device, matrix, error, netlist |
| `sim-device` | Model device dan stamping MNA |
| `sim-analysis` | DC, transient, AC, noise, distorsi, sensitivitas |
| `sim-cells` | Sel CMOS struktural dan pustaka digital |
| `sim-avr` | CPU AVR, decoder, ALU, register, peripheral |
| `sim-board` | Board, LED, button, UART host, Intel HEX, ELF, CSV/VCD |
| `sim-cli` | Binary `embed-emu`, monitor, disassembler, benchmark |

Aturan dependensi:

```text
sim-cli -> sim-board -> sim-avr -> sim-cells -> sim-device -> sim-core
sim-cli -> sim-analysis -> sim-device -> sim-core
```

`sim-core` tidak bergantung pada crate lain. `sim-cli` adalah adapter dan
komposisi, bukan tempat logika simulasi.

---

## 5. CLI

Binary hasil build berada di `target/debug/embed-emu` atau
`target/release/embed-emu`.

### 5.1 Menjalankan Intel HEX

```bash
./target/debug/embed-emu run firmware/blink.hex 100000
```

Argumen kedua adalah batas instruksi. Default 100000.

### 5.2 Monitor interaktif

```bash
./target/debug/embed-emu monitor E005 E013 0F01 9598
```

Perintah di dalam monitor:

| Perintah | Arti |
|---|---|
| `step` | Jalankan satu instruksi |
| `run N` | Jalankan sampai N instruksi |
| `regs` | Tampilkan register r0-r31 |
| `sreg` | Tampilkan flag SREG |
| `pc` | Tampilkan PC dan jumlah cycle |
| `disasm [alamat] [jumlah]` | Disassembly |
| `mem [alamat] [panjang]` | Dump data memory |
| `io [alamat]` | Baca I/O register |
| `quit` atau `q` | Keluar |

Alamat dan angka menerima desimal atau heksadesimal `0x`.

### 5.3 Disassembly

```bash
./target/debug/embed-emu disasm E005 E013 0F01 9598
```

### 5.4 Benchmark

```bash
./target/debug/embed-emu bench 1000 10
```

Argumen pertama adalah jumlah iterasi. Argumen kedua adalah jumlah node
ladder, antara 2 dan 1000.

### 5.5 Virtual serial port dan upload Arduino

Mode ini membuat PTY, membuka virtual serial port, dan menerima upload
STK500/Optiboot. Setelah upload selesai, firmware langsung dijalankan.

Prasyarat:

- Linux atau macOS.
- `socat` terpasang.
- Binary hasil build release.

Jalankan binary tanpa argumen, atau gunakan subcommand `serial`:

```bash
./target/release/embed-emu
./target/release/embed-emu serial
```

Output:

```text
virtual serial port: /tmp/embed-emu-tty
press Ctrl-C to close
```

Path dapat diubah:

```bash
./target/release/embed-emu serial \
  --link /tmp/embed-emu-tty \
  --socket /tmp/embed-emu-sock
```

### Port yang dibuat

CLI membuat dua alamat:

```text
/tmp/embed-emu-tty  -> /dev/pts/N
/tmp/embed-emu-sock -> Unix socket internal
```

`/tmp/embed-emu-tty` adalah symlink ke pseudo-terminal slave. PTY bukan
device USB, sehingga tidak memiliki VID/PID dan tidak muncul sebagai
`/dev/ttyACM*` atau `/dev/ttyUSB*`.

### Deteksi Arduino IDE

Arduino IDE 2.x hanya menampilkan port yang ditemukan oleh
`serial-discovery`. PTY di `/tmp` atau `/dev/pts` sering tidak terdeteksi
walaupun port dapat dibuka oleh program lain.

Yang dapat dicoba:

1. Buat symlink dengan nama device yang dipindai IDE:

   ```bash
   sudo ln -sf /tmp/embed-emu-tty /dev/ttyUSB9
   ```

   Jika muncul, pilih `/dev/ttyUSB9` di Arduino IDE. Symlink biasa tidak
   menjamin terdeteksi karena discovery dapat memakai sysfs atau udev.

2. Gunakan path eksplisit tanpa GUI:

   ```bash
   arduino-cli upload -p /tmp/embed-emu-tty --fqbn arduino:avr:uno <sketch>
   ```

   atau langsung dengan avrdude:

   ```bash
   avrdude -P /tmp/embed-emu-tty ...
   ```

3. Untuk GUI Arduino IDE, diperlukan custom discovery plugin atau virtual
   serial device yang terdaftar di sistem. Fitur itu belum disediakan.

Alur upload setelah port dapat dibuka:

1. Jalankan `embed-emu`.
2. Pilih board Arduino Uno atau ATmega328P.
3. Pilih port virtual.
4. Upload sketch.
5. CLI menulis flash, mereset CPU, dan menjalankan firmware.
6. Tekan Ctrl-C untuk menutup port.

Catatan:

- Protokol yang didukung: STK500/Optiboot.
- Signature: `1E 95 0F` untuk ATmega328P.
- Ukuran flash: 32 KB.
- `socat` dibuat sebagai proses anak dan dihentikan saat CLI keluar.
- Link `/tmp` dan Unix socket dihapus otomatis saat Ctrl-C.
- Symlink `/dev` yang dibuat manual harus dihapus sendiri.
- Output serial monitor dari firmware (TX) diteruskan ke virtual port.
- Input serial monitor (RX) diteruskan ke firmware melalui antrean UART.

Smoke test upload:

```bash
cargo build --release -p sim-cli
python3 scripts/test-serial-upload.py
```

Script tersebut membuat port virtual, mengunggah program blink, memastikan
LED berubah, lalu memeriksa cleanup.

### 5.6 USB gadget mode

Jalur USB asli untuk Linux memakai `dummy_hcd`, ConfigFS ACM, dan driver
C++ di `drivers/linux`.

Build driver:

```bash
./scripts/build-drivers.sh
```

Terminal 1:

```bash
drivers/linux/scripts/run-usb.sh
```

Terminal 2:

```bash
./target/release/embed-emu usb --socket /tmp/embed-emu-usb.sock
```

Host melihat `/dev/ttyACM0`. Arduino IDE dapat memilih port tersebut.

Jalankan hanya satu instance `embed-emu usb`. Jika ada beberapa instance,
hanya client pertama yang terhubung ke bridge; sisanya menunggu dan dapat
membuat upload tampak gagal.

Terverifikasi dengan `avrdude 8.0`: sync, signature, upload, verify, dan
firmware blink berjalan. Firmware hasil `avr-gcc` juga berjalan, termasuk
instruksi `ADIW` dan `SBIW`.

Test bridge tanpa gadget kernel:

```bash
python3 scripts/test-usb-bridge.py
```

Jika Arduino IDE mendapat `Permission denied` pada `/dev/ttyACM0`, jangan
jalankan IDE dengan sudo. Pasang udev rule:

```bash
sudo drivers/linux/scripts/install-udev.sh
```

Atau tambahkan user ke group serial, misalnya `dialout`, lalu logout dan
login kembali.

Setup membutuhkan root. Detail ada di
[`drivers/linux/README.md`](../drivers/linux/README.md).

---

## 6. Library API

### 6.1 DC operating point

```rust
use sim_analysis::{DcConfig, dc_operating_point};
use sim_core::{CircuitBuilder, DeviceKind};
use std::collections::BTreeMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = CircuitBuilder::new();
    let input = builder.node("in")?;
    let output = builder.node("out")?;
    let ground = builder.ground();

    add(&mut builder, "v1", DeviceKind::VoltageSource, input, ground, "voltage", 5.0)?;
    add(&mut builder, "r1", DeviceKind::Resistor, input, output, "resistance", 1000.0)?;
    add(&mut builder, "r2", DeviceKind::Resistor, output, ground, "resistance", 1000.0)?;

    let circuit = builder.build()?;
    let result = dc_operating_point(&circuit, &DcConfig::default())?;
    println!("vout={:.4}", result.solution.voltage(output).unwrap_or(0.0));
    Ok(())
}

fn add(
    builder: &mut CircuitBuilder,
    name: &str,
    kind: DeviceKind,
    first: sim_core::NodeId,
    second: sim_core::NodeId,
    parameter: &str,
    value: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut parameters = BTreeMap::new();
    parameters.insert(parameter.to_string(), value);
    builder.device(name, kind, vec![first, second], parameters)?;
    Ok(())
}
```

Contoh lengkap ada di `crates/sim-analysis/examples/inverter_vtc.rs`.

### 6.2 DC sweep

```rust
use sim_analysis::{DcConfig, dc_sweep};

let points = dc_sweep(
    &circuit,
    "vin",
    "voltage",
    0.0,
    5.0,
    0.25,
    &DcConfig::default(),
)?;
```

Setiap titik berisi `sweep` dan `solution`.

### 6.3 Transient

```rust
use sim_analysis::{TransientConfig, transient};

let config = TransientConfig {
    dt: 1e-6,
    steps: 100,
    gmin: 1e-12,
    use_dc_op: false,
    initial_voltages: Vec::new(),
};
let samples = transient(&circuit, &config)?;
for sample in samples {
    println!(
        "t={} ps vout={:.4}",
        sample.time.as_ps(),
        sample.solution.voltage(output).unwrap_or(0.0)
    );
}
```

Setiap `TransientSample` menyimpan `time`, `solution`, `residual`, dan
`iterations`.

Metode integrasi:

```rust
use sim_analysis::transient_with_method;
use sim_device::IntegrationMethod;

let samples = transient_with_method(
    &circuit,
    &config,
    IntegrationMethod::Trapezoidal,
)?;
```

Adaptive transient:

```rust
use sim_analysis::{AdaptiveConfig, transient_adaptive};

let adaptive = AdaptiveConfig {
    initial_dt: 1e-7,
    min_dt: 1e-12,
    max_dt: 1e-3,
    tolerance: 1e-3,
    steps: 1000,
};
let samples = transient_adaptive(&circuit, &config, &adaptive)?;
```

### 6.4 AC small-signal

```rust
use sim_analysis::ac_analysis;

circuit.set_device_parameter("v1", "ac", 1.0)?;
let result = ac_analysis(&circuit, 1000.0)?;
println!(
    "magnitude={:.4} phase={:.1} deg",
    result.magnitude(output).unwrap_or(0.0),
    result.phase_degrees(output).unwrap_or(0.0)
);
```

AC menghitung DC operating point terlebih dahulu, lalu memakai Jacobian DC
sebagai matriks small-signal. Kapasitansi diode, MOSFET, BJT, dan JFET ikut
distamp dari bias.

Sweep:

```rust
use sim_analysis::ac_sweep;

let points = ac_sweep(&circuit, 10.0, 1e6, 50)?;
```

Contoh lengkap DC, transient, dan AC ada di
`crates/sim-analysis/examples/rc_analysis.rs`:

```bash
cargo run -p sim-analysis --example rc_analysis
```

### 6.5 AVR

```rust
use sim_avr::Cpu;

let program = vec![0xE005, 0xE013, 0x0F01, 0x9598];
let mut cpu = Cpu::new(program)?;
cpu.run(10)?;
println!(
    "r16={} cycles={} halted={}",
    cpu.register(16),
    cpu.cycles(),
    cpu.is_halted()
);
```

Contoh lain:

```bash
cargo run -p sim-avr --example uart_hello
cargo run -p sim-avr --example timer_interrupt
```

### 6.6 Board

```rust
use sim_avr::{Cpu, ExternalPin, PinElectricalConfig};
use sim_board::Board;

let mut cpu = Cpu::new(program)?;
let mut board = Board::new();
let config = PinElectricalConfig::default();
cpu.step()?;
let state = cpu.pin_electrical(0, 5, ExternalPin::HighZ, &config)?;
board.set_pb5_voltage(state.voltage);
println!("led_on={}", board.led().is_on());
```

Contoh lengkap ada di `crates/sim-board/examples/blink.rs`.

---

## 7. Referensi device

Urutan pin mengikuti definisi device. Semua parameter berada di
`BTreeMap<String, f64>` saat device dibuat.

| Device | Pin | Parameter utama | Dokumen |
|---|---|---|---|
| Resistor | 2 | `resistance`, `temp`, `tc1`, `tc2` | [12](12-device-models.md) |
| Capacitor | 2 | `capacitance` | [12](12-device-models.md) |
| Inductor | 2 | `inductance` | [12](12-device-models.md) |
| Voltage source | 2 | `voltage`, `ac`, `ac_phase` | [12](12-device-models.md) |
| Current source | 2 | `current`, `ac`, `ac_phase` | [12](12-device-models.md) |
| Pulse source | 2 | `v1`, `v2`, `delay`, `rise`, `fall`, `width`, `period` | [41](41-pulse-source.md) |
| Diode | 2 | `is`, `n`, `vt`, `rs`, `cj0`, `vj`, `m` | [46](46-diode-nonideal.md) |
| MOSFET | 3 atau 4 | `vto`, `kp`, `lambda`, `w`, `l`, `gamma`, `phi`, `cox`, `cgso`, `cgdo` | [45](45-mosfet-capacitance.md) |
| BJT | 3 | `is`, `bf`, `br`, `vt`, `cje`, `vje`, `mje`, `cjc`, `vjc`, `mjc`, `tf`, `tr` | [35](35-bjt.md) |
| JFET | 3 | `vto`, `beta`, `lambda`, `cgs`, `cgd` | [36](36-jfet.md) |
| Transmission line | 4 | `z0`, `delay` | [37](37-transmission-line.md) |
| VCVS | 4 | `gain` | [47](47-controlled-sources.md) |
| VCCS | 4 | `gm` | [47](47-controlled-sources.md) |
| CCCS | 4 | `gain` | [48](48-cccs-switch.md) |
| CCVS | 4 | `rm` | [49](49-ccvs.md) |
| Switch | 4 | `ron`, `roff`, `vth`, `width` | [48](48-cccs-switch.md) |

Urutan pin:

| Device | Urutan pin |
|---|---|
| Diode | anoda, katoda |
| MOSFET | gate, drain, source, body (opsional) |
| BJT | collector, base, emitter |
| JFET | drain, gate, source |
| Transmission line | input+, input-, output+, output- |
| Controlled source | output+, output-, control+, control- |
| Switch | output+, output-, control+, control- |

Parameter wajib yang hilang menghasilkan `InvalidNetlist`. Nilai negatif
atau NaN untuk parameter yang divalidasi juga menghasilkan error.

---

## 8. Referensi analisis

| Analisis | Fungsi utama | Keluaran |
|---|---|---|
| DC operating point | `dc_operating_point` | `DcResult` |
| DC sweep | `dc_sweep` | `Vec<DcPoint>` |
| Transient | `transient`, `transient_with_method` | `Vec<TransientSample>` |
| Adaptive transient | `transient_adaptive` | `Vec<TransientSample>` |
| AC | `ac_analysis`, `ac_sweep` | `AcSolution` |
| Noise | `noise_analysis` | `NoiseResult` |
| Distorsi | `distortion_analysis` | `DistortionResult` |
| Sensitivitas | `dc_sensitivity` | `DcSensitivity` |
| Switching | `measure_switching` | `SwitchingTiming` |
| Input capacitance | `measure_input_capacitance` | `f64` |
| Drive strength | `measure_drive_strength` | `DriveStrength` |

Semua fungsi menerima `&Circuit` dan konfigurasi eksplisit. Tidak ada state
global.

---

## 9. Firmware AVR

### 9.1 Alur kerja

1. Tulis firmware C atau assembly.
2. Kompilasi dengan `avr-gcc` di luar simulator.
3. Hasilkan Intel HEX dengan `avr-objcopy`.
4. Jalankan dengan `embed-emu run`.
5. Atau muat ELF melalui API `ElfImage` di `sim_board`.

Contoh perintah toolchain eksternal:

```bash
avr-gcc -mmcu=atmega328p -DF_CPU=16000000UL -Os -o blink.elf blink.c
avr-objcopy -O ihex -R .eeprom blink.elf blink.hex
./target/debug/embed-emu run blink.hex 100000
```

Perintah di atas belum diverifikasi di environment ini karena `avr-gcc`
tidak tersedia.

### 9.2 ELF ke HEX melalui library

```rust
use sim_board::ElfImage;

let data = std::fs::read("blink.elf")?;
let image = ElfImage::parse(&data)?;
let hex = image.to_hex()?;
std::fs::write("blink.hex", hex)?;
```

### 9.3 Firmware contoh

- `firmware/blink.S`: toggle PORTB bit 5.
- `firmware/timer_interrupt.S`: Timer0 overflow interrupt.
- `crates/sim-avr/tests/blink.rs`: encoding manual blink.
- `cargo run -p sim-avr --example timer_interrupt`: demo interrupt.

---

## 10. Output dan format

| Format | Lokasi | Kegunaan |
|---|---|---|
| CSV | `sim_board::CsvWriter`, `csv_header`, `csv_line` | Trace board dan sinyal |
| VCD digital | `sim_board::VcdWriter` | PB5 dan LED |
| VCD analog | `sim_board::AnalogVcdWriter` | Node analog `real` |
| Intel HEX | `sim_board::HexImage` | Program AVR |
| ELF32 | `sim_board::ElfImage` | Program dan simbol AVR |

Timebase VCD memakai 1 ns. Format berada di adapter, bukan di `sim-core`.

---

## 11. Test, benchmark, dan determinisme

### 11.1 Test per crate

```bash
cargo test -p sim-core
cargo test -p sim-device
cargo test -p sim-analysis
cargo test -p sim-cells
cargo test -p sim-avr
cargo test -p sim-board
cargo test -p sim-cli
```

### 11.2 Benchmark

```bash
cargo run -p sim-cli -- bench 1000 10
```

Atau setelah build:

```bash
./target/debug/embed-emu bench 1000 10
```

### 11.3 Determinisme

Test determinisme membandingkan hasil dua run dan hasil setelah snapshot.
Jalankan:

```bash
cargo test -p sim-analysis determinism
```

---

## 12. Troubleshooting

| Gejala | Penyebab umum | Tindakan |
|---|---|---|
| `cargo: command not found` | Shell tidak memuat Cargo | `source "$HOME/.cargo/env"` |
| `ConvergenceFailure` | Rangkaian sulit atau parameter ekstrem | Periksa bias, gmin, dan `max_iterations` |
| `InvalidNetlist` | Parameter wajib hilang atau tidak valid | Periksa nama dan nilai parameter |
| `UnsupportedDevice` | Device tidak didukung analisis tersebut | Lihat daftar device di [29](29-ac.md) |
| Hasil AC nol | Sumber tidak memiliki parameter `ac` | Set `ac` pada sumber |
| Transient tidak berubah | `use_dc_op = true` dan rangkaian tunak | Set `use_dc_op = false` |
| Test lambat | Seluruh workspace diuji | Jalankan `cargo test -p <crate>` |
| ASCII check gagal | Karakter non-ASCII | Ganti dengan padanan ASCII |

---

## 13. Batas saat ini

- Belum ada parser netlist SPICE. Rangkaian dibangun melalui API Rust.
- Belum ada GUI atau waveform viewer.
- Belum ada golden comparison dengan ngspice.
- `avr-gcc` belum tersedia di environment ini.
- Determinisme macOS dan Windows belum dijalankan.
- GitHub Actions sudah disiapkan, tetapi belum dieksekusi.
- Model device memakai Level 1 atau model sederhana.
- AC belum mendukung transmission line dan pulse source.
- Switch AC memakai conductance dari bias; kontrol AC tidak dimodulasi.
- Belum ada periodic steady state atau harmonic balance.
- Virtual serial upload mendukung STK500. TX dan RX UART diteruskan.
- Arduino IDE mungkin memerlukan symlink `/dev` agar port `/tmp` terdeteksi.

Batas model yang lebih rinci ada di [24-fidelity.md](24-fidelity.md).

---

## 14. Dokumen berikutnya

- [12-device-models.md](12-device-models.md): katalog device dan parameter.
- [13-analysis.md](13-analysis.md): detail solver dan analisis.
- [24-fidelity.md](24-fidelity.md): akurasi dan batasan.
- [29-ac.md](29-ac.md): AC small-signal.
- [35-bjt.md](35-bjt.md) dan [36-jfet.md](36-jfet.md): model transistor.
- [53-release-notes.md](53-release-notes.md): isi rilis v0.1.
