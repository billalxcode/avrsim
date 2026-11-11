# 14 - CMOS Cells

Dokumentasi pustaka sel CMOS dasar di `sim-cells`.

---

## 1. Status

Sel yang sudah diimplementasikan:

- Inverter.
- NAND2.
- NOR2.
- Buffer.

Belum diimplementasikan:

- Tri-state inverter.
- Transmission gate.
- SRAM cell 6T.
- D latch.
- D flip-flop master-slave.
- Ring oscillator.
- Cell registry.
- Characterization harness lengkap.
- Noise margin.
- Rise/fall time.
- Propagation delay.
- Input capacitance.
- Drive strength.
- Perbandingan ngspice.

---

## 2. API

```rust
pub struct Cell {
    // ...
}

impl Cell {
    pub fn circuit(&self) -> &Circuit;
    pub fn port(&self, name: &str) -> Option<NodeId>;
    pub fn vdd(&self) -> Option<NodeId>;
    pub fn powered(&self, voltage: f64) -> Result<Circuit, SimError>;
}
```

`powered` menambahkan voltage source `vdd_src` antara port `vdd` dan ground.

Builder:

```rust
pub struct CellBuilder {
    // ...
}

impl CellBuilder {
    pub fn new() -> Self;
    pub fn node(&mut self, name: &str) -> NodeId;
    pub fn vdd(&self) -> NodeId;
    pub fn ground(&self) -> NodeId;
    pub fn nmos(&mut self, name: &str, gate: NodeId, drain: NodeId, source: NodeId, width: f64);
    pub fn pmos(&mut self, name: &str, gate: NodeId, drain: NodeId, source: NodeId, width: f64);
    pub fn build(self, ports: &[(&str, NodeId)]) -> Result<Cell, SimError>;
}
```

---

## 3. Parameter Transistor

| Parameter | Nilai |
|---|---|
| `vto` | 0.7 V |
| `kp` | 1e-3 |
| `lambda` | 0.02 |
| `l` | 1e-6 m |
| `w` NMOS | 1e-6 m |
| `w` PMOS | 2e-6 m |

PMOS dibuat dua kali lebih lebar untuk mendekati simetri inverter.

---

## 4. Inverter

Struktur:

- PMOS: gate `a`, drain `y`, source `vdd`.
- NMOS: gate `a`, drain `y`, source `ground`.

Test:

- VTC dari 0 V sampai 5 V.
- Output tinggi pada input 0 V.
- Output rendah pada input 5 V.
- Switching threshold sekitar 2.5 V.

---

## 5. NAND2

Struktur:

- Dua PMOS paralel antara `vdd` dan `y`.
- Dua NMOS seri antara `y` dan ground melalui node `mid`.

Test truth table:

| a | b | y |
|---|---|---|
| 0 | 0 | 1 |
| 0 | 1 | 1 |
| 1 | 0 | 1 |
| 1 | 1 | 0 |

---

## 6. NOR2

Struktur:

- Dua NMOS paralel antara `y` dan ground.
- Dua PMOS seri antara `vdd` dan `y` melalui node `mid`.

Test truth table:

| a | b | y |
|---|---|---|
| 0 | 0 | 1 |
| 0 | 1 | 0 |
| 1 | 0 | 0 |
| 1 | 1 | 0 |

---

## 7. Buffer

Struktur:

- Dua inverter CMOS bertingkat.
- Node internal `mid`.

Test:

- Input 0 V menghasilkan output rendah.
- Input 5 V menghasilkan output tinggi.

---

## 8. Test

Test `sim-cells` mencakup:

- Inverter VTC.
- Inverter truth table.
- Switching threshold.
- NAND truth table.
- NOR truth table.
- Buffer follow input.
- Powered cell memiliki supply.

Total test `sim-cells`: 7.

---

## 9. Keterbatasan

- Belum ada analisis rise/fall time.
- Belum ada propagation delay.
- Belum ada noise margin.
- Belum ada input capacitance.
- Belum ada drive strength.
- Belum ada sel sekuensial.
- Belum ada ring oscillator.
- Belum ada perbandingan ngspice.

---

## 10. Berikutnya

- Transmission gate dan tri-state.
- D latch dan D flip-flop.
- SRAM cell 6T.
- Ring oscillator.
- Characterization harness.
- Cell registry.
- Golden data ngspice.

Lihat `TODO.md` untuk daftar lengkap.
