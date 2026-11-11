# 43 - User Guide

Dokumen ini menjelaskan cara membangun, menjalankan, dan memakai simulator.

---

## 1. Prasyarat

- Rust stable sesuai `rust-toolchain.toml`.
- Tidak ada dependensi Rust eksternal.
- Tidak ada `unsafe`.

Verifikasi:

```bash
cargo --version
cargo test --workspace
```

---

## 2. Quick Start CLI

Build:

```bash
cargo build --workspace
```

Menjalankan program AVR dari Intel HEX:

```bash
./target/debug/embed-emu run firmware/blink.hex 1000
```

Monitor interaktif:

```bash
./target/debug/embed-emu monitor <hex words...>
```

Disassembly:

```bash
./target/debug/embed-emu disasm <hex words...>
```

Benchmark:

```bash
./target/debug/embed-emu bench 1000 10
```

---

## 3. Library Workflow

Contoh membangun rangkaian dan menjalankan DC operating point:

```rust
use sim_analysis::{DcConfig, dc_operating_point};
use sim_core::{CircuitBuilder, DeviceKind};
use std::collections::BTreeMap;

let mut builder = CircuitBuilder::new();
let input = builder.node("in")?;
let output = builder.node("out")?;
let ground = builder.ground();

let mut source = BTreeMap::new();
source.insert("voltage".to_string(), 5.0);
builder.device("v1", DeviceKind::VoltageSource, vec![input, ground], source)?;

let mut resistor = BTreeMap::new();
resistor.insert("resistance".to_string(), 1000.0);
builder.device("r1", DeviceKind::Resistor, vec![input, output], resistor)?;
builder.device("r2", DeviceKind::Resistor, vec![output, ground], resistor)?;

let circuit = builder.build()?;
let result = dc_operating_point(&circuit, &DcConfig::default())?;
println!("vout={}", result.solution.voltage(output).unwrap_or(0.0));
```

---

## 4. Analisis yang Tersedia

| Analisis | Fungsi |
|---|---|
| DC operating point | `dc_operating_point` |
| DC sweep | `dc_sweep` |
| Transient | `transient`, `transient_with_method` |
| Adaptive transient | `transient_adaptive` |
| AC small-signal | `ac_analysis`, `ac_sweep` |
| Noise | `noise_analysis` |
| Distortion | `distortion_analysis` |
| Sensitivity | `dc_sensitivity` |
| Switching timing | `measure_switching` |

---

## 5. Device yang Tersedia

- Resistor, capacitor, inductor.
- Voltage source, current source, pulse voltage source.
- Diode.
- MOSFET N dan P.
- BJT NPN dan PNP.
- JFET N dan P.
- Transmission line lossless.

Device nonlinier ditolak oleh AC analysis.

---

## 6. Suhu dan Process Corner

```rust
use sim_analysis::{ProcessCorner, with_process_corner, with_temperature};

let hot = with_temperature(&circuit, 400.15)?;
let corner = ProcessCorner {
    resistor: 1.10,
    capacitor: 0.95,
    vto: 0.90,
    kp: 1.05,
};
let adjusted = with_process_corner(&circuit, &corner)?;
```

Nilai corner diberikan pengguna. Tidak ada nilai corner bawaan.

---

## 7. Pin Elektrik

```rust
use sim_avr::{ExternalPin, PinElectricalConfig};

let config = PinElectricalConfig::default();
let state = cpu.pin_electrical(
    0,
    5,
    ExternalPin::Thevenin {
        voltage: 0.0,
        resistance: 1000.0,
    },
    &config,
)?;
println!("v={} contention={}", state.voltage, state.contention);
```

---

## 8. Output CSV dan VCD

- `sim_board::CsvWriter` untuk CSV generik.
- `sim_board::VcdWriter` untuk VCD digital.
- `sim_board::AnalogVcdWriter` untuk VCD `real`.
- `sim_board::csv_header` dan `csv_line` untuk trace board.

Format berada di adapter, bukan di `sim-core`.

---

## 9. Contoh Runnable

```bash
cargo run -p sim-analysis --example inverter_vtc
cargo run -p sim-cells --example ring_oscillator
cargo run -p sim-avr --example cpu_program
cargo run -p sim-board --example blink
cargo run -p sim-avr --example uart_hello
```

Contoh firmware ada di `firmware/`.

---

## 10. Verifikasi

```bash
bash .opencode/skills/clean-code/scripts/check.sh
```

Perintah tersebut menjalankan fmt, clippy, dan seluruh test workspace.

---

## 11. Batas

- Belum ada parser netlist SPICE.
- Belum ada `avr-gcc`, sehingga firmware penuh belum dikompilasi di CI.
- Belum ada golden comparison dengan ngspice.
- Model device masih memiliki batas yang didokumentasikan per device.
- Belum ada GUI atau waveform viewer.
