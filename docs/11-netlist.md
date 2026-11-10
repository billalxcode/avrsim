# 11 - Netlist Core

Dokumentasi netlist core P2 di `sim-core`.

---

## 1. Tujuan

Netlist core menyediakan:

- Node dan ground.
- Device struktural.
- Builder.
- Subcircuit dan flatten.
- Validasi nama, pin, floating node, dan DC path.
- Macro deklaratif `circuit!`.
- Snapshot netlist.

Netlist core belum melakukan stamping MNA atau perhitungan listrik. Itu masuk P3 dan P4.

---

## 2. Peta Modul

| File | Isi |
|---|---|
| `netlist/node.rs` | `Node`, `NodeKind` |
| `netlist/device.rs` | `DeviceKind`, `DeviceInstance`, `Device`, `StampContext`, `Solution` |
| `netlist/builder.rs` | `CircuitBuilder` |
| `netlist/circuit.rs` | `Circuit`, validasi, snapshot |
| `netlist/subcircuit.rs` | `Port`, `Subcircuit` |
| `netlist/macros.rs` | Macro `circuit!` |
| `netlist/tests.rs` | Test integrasi netlist |

Re-export dari `sim_core`:

```rust
use sim_core::{
    Circuit, CircuitBuilder, DeviceKind, Node, NodeId, NodeKind, Subcircuit,
};
```

---

## 3. Node

`Node` memiliki:

- `NodeId`.
- Nama.
- `NodeKind`.

`NodeKind`:

- `Ground`.
- `Signal`.
- `Supply`.

Ground selalu node 0. Gunakan `NodeId::GROUND` atau `builder.ground()`.

---

## 4. Device

`DeviceKind` yang tersedia:

- `Resistor`.
- `Capacitor`.
- `Inductor`.
- `VoltageSource`.
- `CurrentSource`.
- `Nmos`.
- `Pmos`.
- `Diode`.
- `Custom(String)`.

`DeviceInstance` menyimpan:

- `DeviceId`.
- Nama.
- Kind.
- Pin.
- Parameter `BTreeMap<String, f64>`.

Trait `Device` menyediakan akses struktural:

```rust
pub trait Device {
    fn name(&self) -> &str;
    fn kind(&self) -> &DeviceKind;
    fn pins(&self) -> &[NodeId];
    fn parameters(&self) -> &BTreeMap<String, f64>;
}
```

`StampContext` dan `Solution` disiapkan untuk P4, tetapi belum dipakai.

---

## 5. Builder

Contoh inverter:

```rust
use sim_core::{CircuitBuilder, DeviceKind};
use std::collections::BTreeMap;

let mut builder = CircuitBuilder::new();
let vdd = builder.node("vdd")?;
let input = builder.node("input")?;
let output = builder.node("output")?;
let ground = builder.ground();
builder.device(
    "mp",
    DeviceKind::Pmos,
    vec![input, output, vdd],
    BTreeMap::new(),
)?;
builder.device(
    "mn",
    DeviceKind::Nmos,
    vec![input, output, ground],
    BTreeMap::new(),
)?;
let circuit = builder.build()?;
```

Aturan builder:

- `new()` selalu membuat ground node 0.
- `node()` menambah node signal.
- `supply()` menambah node supply.
- `device()` menambah device dengan minimal 2 pin.
- Nama node dan device harus unik.
- Nama tidak boleh kosong atau memiliki spasi di awal/akhir.
- Pin harus merujuk node yang sudah ada.
- `build()` memvalidasi netlist.

---

## 6. Macro `circuit!`

Sintaks:

```rust
let circuit = circuit! {
    nodes { vdd, input, output }
    ground gnd;
    devices {
        mp: DeviceKind::Pmos => [input, output, vdd];
        mn: DeviceKind::Nmos => [input, output, gnd];
    }
}?;
```

Dengan parameter:

```rust
let circuit = circuit! {
    nodes { a, b }
    ground gnd;
    devices {
        r1: DeviceKind::Resistor => [a, b], params { resistance: 1000.0 };
        r2: DeviceKind::Resistor => [b, gnd];
    }
}?;
```

Aturan:

- `ground` wajib dideklarasikan dengan identifier yang akan dipakai di pin.
- Jangan memakai identifier `gnd` atau nama ground lain di daftar `nodes`.
- `ground` adalah satu-satunya node yang tidak dibuat lewat `builder.node`.
- Macro mengembalikan `Result<Circuit, SimError>`.
- Macro dan builder menghasilkan netlist yang sama untuk input yang sama.

---

## 7. Subcircuit dan Flatten

Template inverter:

```rust
let template = build_inverter()?;
let input = template.node_by_name("input").expect("input").id();
let output = template.node_by_name("output").expect("output").id();
let vdd = template.node_by_name("vdd").expect("vdd").id();
let subcircuit = Subcircuit::new(
    "inverter",
    template,
    vec![
        ("input".to_string(), input),
        ("output".to_string(), output),
        ("vdd".to_string(), vdd),
    ],
)?;
```

Instansiasi:

```rust
let mut top = CircuitBuilder::new();
let input = top.node("a")?;
let output = top.node("y")?;
let vdd = top.supply("vdd")?;
top.instantiate(
    "u1",
    &subcircuit,
    &[("input", input), ("output", output), ("vdd", vdd)],
)?;
let circuit = top.build()?;
```

Hasil flatten:

- Device menjadi `u1.mp` dan `u1.mn`.
- Node port memetakan ke node parent.
- Node internal menjadi `u1.<nama>`.
- Ground template memetakan ke ground parent.
- Instansiasi bersifat atomik: jika gagal, builder asli tidak berubah.

Aturan port:

- Port tidak boleh ground.
- Port harus merujuk node template yang ada.
- Nama port unik.
- Node port unik.
- Semua port wajib terhubung saat instansiasi.
- Koneksi duplikat atau port tidak dikenal ditolak.

---

## 8. Validasi

`Circuit::validate()` memeriksa:

1. Ground ada dan bertipe `Ground`.
2. Nama node unik.
3. Nama device unik.
4. Ground adalah node 0.
5. Setiap device pin merujuk node yang ada.
6. Setiap node non-ground dipakai minimal satu device.
7. Setiap node non-ground punya jalur DC ke ground.

Error:

- `InvalidNetlist`.
- `FloatingNode`.
- `NoDcPath`.
- `IndexOutOfBounds` dari matrix, bukan netlist.

### Batasan DC Path

Untuk P2, semua pin device konduktif dianggap terhubung. Capacitor tidak konduktif.

Konsekuensi:

- MOSFET gate dianggap terhubung ke drain/source pada pemeriksaan DC.
- Ini konservatif dan akan diperbaiki saat model device P3 tersedia.
- Node yang hanya terhubung lewat capacitor akan ditolak sebagai `NoDcPath`.
- Rangkaian coupling capacitor murni perlu model yang lebih tepat di tahap berikutnya.

---

## 9. Snapshot

`Circuit::snapshot()` mengembalikan clone independen dari netlist.

```rust
let snapshot = circuit.snapshot();
assert_eq!(circuit, snapshot);
```

Snapshot berguna untuk test, replay, dan perbandingan sebelum/sesudah refactor.

---

## 10. Keputusan Desain

- Netlist core tidak mengetahui model listrik.
- `DeviceKind` adalah label struktural, bukan model device.
- `StampContext` dan `Solution` disiapkan agar API stabil.
- Flatten dilakukan saat instansiasi, bukan saat build.
- Builder memakai `BTreeMap`/`BTreeSet` agar urutan deterministik.
- Instansiasi memakai staged builder agar atomik.
- Validasi DC path sederhana agar bug netlist awal cepat terdeteksi.
- Tidak ada dependensi eksternal.

---

## 11. Test

Test netlist mencakup:

- Builder inverter.
- Macro sama dengan builder.
- Duplicate node name.
- Duplicate device name.
- Floating node.
- DC path capacitor.
- Instansiasi subcircuit.
- Prefix node internal.
- Port hilang.
- Macro dengan parameter.
- Snapshot.
- Ground node 0.

Total test `sim-core`: 45.

---

## 12. Berikutnya

P3 akan menambahkan:

- Model MOSFET Level 1.
- Resistor, capacitor, inductor.
- Sumber tegangan dan arus.
- Dioda.
- Stamping MNA.
- Jacobian.

Netlist core sudah menyediakan struktur yang dibutuhkan.
