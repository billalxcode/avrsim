# 02 - Arsitektur

Dokumen ini menjelaskan struktur software, model waktu, netlist, API, dan aturan determinisme.

---

## 1. Prinsip Arsitektur

1. **Satu kernel waktu** untuk seluruh domain.
2. **Device tidak mengetahui analisis**; device hanya melakukan stamping.
3. **Netlist adalah pusat**, bukan MCU.
4. **Hierarchy untuk desain, flatten untuk solver.**
5. **Zero external dependency untuk crate simulasi**; hanya `sim-gui` yang boleh memakai `egui`, `eframe`, `serde`, `serde_json`, `rfd`, dan `png`.
6. **Determinisme dulu**, optimasi kemudian.
7. **Setiap lapisan dapat diuji sendiri.**

---

## 2. Struktur Workspace

```
embed-emulation/
|-- Cargo.toml                  # workspace; dependensi eksternal hanya di sim-gui
|-- rust-toolchain.toml
|-- LICENSE
|-- README.md
|-- TODO.md
|-- docs/
|-- crates/
|   |-- sim-core/
|   |-- sim-device/
|   |-- sim-analysis/
|   |-- sim-cells/
|   |-- sim-avr/
|   |-- sim-board/
|   |-- sim-cli/
|   `-- sim-gui/
|-- examples/
`-- firmware/
```

### 2.1 Tanggung Jawab Crate

| Crate | Tanggung jawab | Tidak boleh tahu |
|---|---|---|
| `sim-core` | Time, ID, netlist, builder, matrix, LU, error, VCD/CSV dasar | Device spesifik, AVR |
| `sim-device` | MOSFET, R, C, L, dioda, BJT, JFET, sumber | Analisis, CPU |
| `sim-analysis` | DC op, sweep, transient, AC, noise, konvergensi | Firmware, board |
| `sim-cells` | Sel CMOS, gerbang, DFF, SRAM cell, characterization | AVR |
| `sim-avr` | CPU struktural, ISA, register, peripheral | Board |
| `sim-board` | Board netlist, LED, tombol, UART host | Detail CPU |
| `sim-cli` | Argumen, orkestrasi, output, monitor | Implementasi internal device |
| `sim-gui` | UI egui, canvas, persistence, worker simulasi | Implementasi internal device dan solver |

### 2.2 Dependency Graph

```
sim-cli
  |-- sim-board
  |     |-- sim-avr
  |     |     |-- sim-cells
  |     |     |     |-- sim-device
  |     |     |     |     `-- sim-core
  |     |     |     `-- sim-core
  |     |     `-- sim-core
  |     `-- sim-device / sim-core
  `-- sim-analysis
        |-- sim-device
        `-- sim-core

sim-gui
  |-- sim-board
  |-- sim-avr
  |-- sim-analysis
  |-- sim-device
  `-- sim-core
```

Tidak ada crate simulasi yang bergantung pada crate GUI atau ke luar
workspace. Hanya `sim-gui` yang memakai dependensi eksternal yang
disetujui.

---

## 3. Model Waktu

### 3.1 Time Base

- Waktu digital: **integer pikodetik** (`u64` atau `i64`).
- 16 MHz -> periode 62.500 ps.
- Waktu analog: `f64` detik, diturunkan dari waktu global.
- Tidak ada `Instant`, `SystemTime`, atau wall-clock di dalam simulasi.

### 3.2 Alasan Pikodetik

- 16 MHz dapat direpresentasikan tepat.
- Resolusi cukup untuk delay gerbang CMOS.
- Tidak ada drift floating-point pada event scheduling.
- Rentang `u64` pikodetik ~ 213 hari simulasi; cukup untuk offline.

### 3.3 Sinkronisasi Domain

```
Global time (ps)
      |
      |-- Digital events (integer ps)
      |-- CPU cycles (integer ps)
      `-- Analog steps (f64 seconds, quantized to ps)
```

Aturan:

1. Event diproses dalam urutan `(time, sequence)`.
2. Event pada waktu yang sama diproses dalam urutan penjadwalan (deterministik).
3. Analog solver berintegrasi sampai `min(next_event, next_analog_step)`.
4. Perubahan pin digital menjadi breakpoint analog.
5. Perlintasan threshold analog menjadi event digital pada waktu interpolasi.

### 3.4 Delta Cycle

- Event dengan delay nol dapat membentuk loop.
- Batas iterasi delta cycle harus ada.
- Jika batas terlampaui, hasil menjadi `X`/error, bukan hang.
- Delay propagasi nyata harus dimodelkan untuk menghindari loop tak terbatas.

---

## 4. Model Netlist

### 4.1 Node dan Device

```rust
pub struct NodeId(pub u32);
pub struct DeviceId(pub u32);
pub struct BranchId(pub u32);

pub enum NodeKind {
    Ground,
    Signal,
    Supply,
}
```

- Node 0 selalu ground.
- Setiap device memiliki port (node) dan parameter.
- Branch digunakan untuk sumber tegangan dan induktor.

### 4.2 Device API (rencana)

```rust
pub trait Device {
    fn ports(&self) -> &[NodeId];
    fn stamp_dc(&self, ctx: &mut StampContext);
    fn stamp_transient(&self, ctx: &mut StampContext, dt: f64);
    fn stamp_ac(&self, ctx: &mut StampContext, freq: f64);
    fn evaluate(&mut self, solution: &Solution) -> Result<(), SimError>;
}
```

Prinsip:

- `stamp_*` hanya menulis ke matriks MNA dan RHS.
- `evaluate` memperbarui state internal (misalnya tegangan sebelumnya).
- Tidak ada analisis di dalam device.
- Tidak ada alokasi besar per stamp jika bisa dihindari.

### 4.3 Builder API (rencana)

```rust
let mut c = Circuit::builder();

let vdd = c.node("vdd");
let gnd = c.ground();
let input = c.node("in");
let output = c.node("out");

c.vsource("vdd", vdd, gnd, 3.3);
c.mosfet("mn", Mosfet::nmos(1.0, 0.35), input, output, gnd);
c.mosfet("mp", Mosfet::pmos(2.0, 0.35), input, output, vdd);

let circuit = c.build()?;
```

### 4.4 Macro DSL (rencana)

```rust
let circuit = circuit! {
    nodes { vdd, gnd, input, output }
    vdd: vsource(vdd, gnd, 3.3);
    mn: nmos(w=1.0, l=0.35, gate=input, drain=output, source=gnd);
    mp: pmos(w=2.0, l=0.35, gate=input, drain=output, source=vdd);
};
```

Builder dan macro menghasilkan struktur netlist yang sama. Tidak ada logika tersembunyi di macro.

### 4.5 Hierarchy dan Flatten

- `Subcircuit` memiliki nama, port, dan device internal.
- Instance subcircuit memiliki nama dan mapping port.
- Flatten menghasilkan:
  - Node unik dengan nama hierarkis (`u1.mn.gate`).
  - Device unik dengan nama hierarkis.
  - Tidak ada node duplikat.
- Validasi setelah flatten:
  - Tidak ada node floating.
  - Tidak ada port yang tidak terhubung.
  - Tidak ada nama duplikat.
  - Ground terdefinisi.

---

## 5. Alur Simulasi

```
1. Build netlist (builder/macro)
2. Flatten
3. Validasi
4. Pilih analisis
   |-- DC operating point
   |-- DC sweep
   |-- Transient
   |-- AC
   `-- Noise / distortion / sensitivity
5. Untuk setiap titik:
   |-- Stamp device
   |-- Solve linear system (LU)
   |-- Newton iteration sampai konvergen
   `-- Simpan hasil
6. Output CSV/VCD
```

### 5.1 DC Operating Point

- Semua kapasitor open, induktor short.
- Sumber DC aktif.
- Newton-Raphson dengan konvergensi aids.
- Hasil menjadi initial condition untuk transient.

### 5.2 DC Sweep

- Sumber atau parameter di-sweep.
- Setiap titik diselesaikan sebagai DC op.
- Hasil CSV: `sweep_var, node1, node2, ...`.

### 5.3 Transient

- Backward Euler.
- Time step tetap pada milestone awal.
- Setiap step menyelesaikan sistem nonlinear.
- State device diperbarui setelah konvergen.
- Hasil VCD: time + selected nodes.

### 5.4 AC (nanti)

- Linearisasi di sekitar DC op.
- Matriks kompleks.
- Solver dense complex.
- Hasil magnitude/phase.

---

## 6. Representasi Matriks

### 6.1 MNA

Untuk `n` node non-ground dan `m` branch:

```
[ G  B ] [ v ]   [ i ]
[ C  D ] [ j ] = [ e ]
```

- `G`: konduktansi node.
- `B` dan `C`: kopling branch.
- `D`: biasanya nol.
- `v`: tegangan node.
- `j`: arus branch.
- `i`: arus injeksi.
- `e`: tegangan sumber.

### 6.2 Dense LU

- Row-major `Vec<f64>`.
- Partial pivoting.
- Forward/back substitution.
- Deteksi singular dan pivot kecil.
- Kompleks untuk AC: `Vec<Complex64>`.

### 6.3 Sparse (nanti)

- Threshold jumlah node.
- Format CSR/CSC.
- Ordering untuk mengurangi fill-in.
- Tetap deterministik.

---

## 7. Determinisme

Aturan wajib:

1. Tidak memakai `HashMap` untuk iterasi yang memengaruhi hasil.
2. Gunakan `Vec`, `BTreeMap`, atau struktur dengan urutan stabil.
3. Tidak memakai waktu sistem.
4. Tidak memakai RNG tanpa seed tetap.
5. Tidak melakukan iterasi paralel yang memengaruhi urutan penjumlahan floating-point.
6. Hasil dibandingkan dengan toleransi, bukan bit-identik.
7. Snapshot state harus lengkap dan deterministik.

---

## 8. Error Handling

```rust
pub enum SimError {
    InvalidNetlist(String),
    FloatingNode(NodeId),
    SingularMatrix { row: usize },
    ConvergenceFailure { iteration: usize, node: NodeId },
    UnsupportedDevice(String),
    Io(std::io::Error),
    Parse(String),
    Internal(String),
}
```

Prinsip:

- Error tidak boleh berupa panic pada input pengguna.
- `Result` digunakan di seluruh API publik.
- Panic hanya untuk bug internal yang tidak dapat dipulihkan.
- Pesan error menyebut node/device/analisis.

---

## 9. Output dan Observability

### 9.1 CSV

Digunakan untuk:

- DC operating point.
- DC sweep (VTC).
- Tabel hasil analisis.

Format:

```csv
sweep,vin,vout
0.0,0.000,3.300
0.1,0.100,3.300
...
```

### 9.2 VCD

Digunakan untuk:

- Transient.
- Perubahan pin digital.
- Node analog terpilih.

Fitur:

- Header waktu.
- Scope per hierarchy.
- Variabel analog (`real`) dan digital (`wire`).
- Kompatibel dengan GTKWave.

### 9.3 CLI Monitor

Perintah yang direncanakan:

```
embed-emu run --netlist inverter --analysis dc-sweep --output vtc.csv
embed-emu run --board uno --firmware blink.hex --vcd blink.vcd
embed-emu monitor --board uno --firmware blink.hex
```

Monitor:

- `step`
- `continue`
- `break <addr>`
- `regs`
- `mem <addr> <len>`
- `disasm <addr> <count>`
- `pin <port> <bit>`
- `quit`

---

## 10. Performa

### 10.1 Prinsip

- Hindari alokasi di hot path.
- Reuse matriks dan vektor.
- Simpan device dalam `Vec`/arena, bukan `Box<dyn Trait>` per operasi.
- Cache hasil stamping yang tidak berubah.
- Profiling sebelum optimasi.

### 10.2 Skala Awal

- Rangkaian kecil: < 100 node.
- Dense LU cukup.
- Time step tetap.
- Single-threaded.

### 10.3 Skala Akhir

- Rangkaian besar: ribuan node.
- Sparse LU.
- Adaptive step.
- Parallel analog islands (hati-hati determinisme).
- Snapshot/replay.

---

## 11. Batas Antara Struktural dan Behavioral

| Batas | Struktural | Behavioral |
|---|---|---|
| Register file <-> SRAM | Register file | SRAM/flash |
| CPU <-> memory bus | CPU, decoder, control | Array memori |
| MCU <-> host UART | Pin MCU | Host endpoint |
| Digital <-> analog | Pin electrical model | - |
| Board <-> PCB | Komponen | Parasitik PCB (tidak dimodelkan) |

Setiap batas harus memiliki kontrak yang jelas dan test tersendiri.

---

## 12. API Publik (Rencana)

```rust
pub fn run_dc_sweep(circuit: &Circuit, sweep: Sweep) -> Result<Vec<DcPoint>, SimError>;
pub fn run_transient(circuit: &Circuit, cfg: TransientConfig) -> Result<Vec<TransientPoint>, SimError>;
pub fn run_ac(circuit: &Circuit, cfg: AcConfig) -> Result<Vec<AcPoint>, SimError>;
```

CLI adalah konsumen pertama API ini. Library tidak bergantung pada CLI.

---

## 13. Aturan Kontribusi Teknis

1. Setiap crate harus bisa diuji tanpa crate di atasnya.
2. Tidak ada dependensi eksternal.
3. Setiap device memiliki test unit.
4. Setiap analisis memiliki test integrasi.
5. Setiap perubahan solver harus menjalankan golden suite.
6. Setiap pengecualian fidelity harus dicatat.
7. Dokumentasi diperbarui bersama kode.
