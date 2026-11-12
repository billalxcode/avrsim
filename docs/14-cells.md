# 14 - CMOS Cells

Dokumentasi pustaka sel CMOS dasar di `sim-cells`.

---

## 1. Status

Sel yang sudah diimplementasikan:

- Inverter.
- NAND2.
- NOR2.
- Buffer.
- Transmission gate.
- Tri-state inverter.
- D latch.
- D flip-flop.
- SRAM cell 6T.
- Ring oscillator.
- Registry sel.
- AND2, OR2, XOR2, dan MUX2 dari NAND2.
- Flag arithmetic C, Z, N, V, S, H.
- ALU bit-slice 1 bit.

Belum diimplementasikan:

- Rise/fall time.
- Propagation delay.
- Input capacitance.
- Drive strength.
- Verifikasi osilasi ring oscillator.
- Perbandingan ngspice.

---

## 2. Struktur Modul

- `basic/`: inverter, NAND, NOR, buffer, logic gates, transmission gate, tri-state.
- `arithmetic/`: half/full adder, ripple-carry adder, ALU slice, flags.
- `sequential/`: latch, D flip-flop, SRAM6T, ring oscillator.
- `digital/`: simulator dua-state, blok digital, ALU digital, register file, CPU uji.
- Root: `builder`, `cell`, `characterize`, dan `registry`.

## 3. API

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

## 4. Parameter Transistor

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

## 5. Inverter

Struktur:

- PMOS: gate `a`, drain `y`, source `vdd`.
- NMOS: gate `a`, drain `y`, source `ground`.

Test:

- VTC dari 0 V sampai 5 V.
- Output tinggi pada input 0 V.
- Output rendah pada input 5 V.
- Switching threshold sekitar 2.5 V.

---

## 6. NAND2

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

## 7. NOR2

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

## 8. Buffer

Struktur:

- Dua inverter CMOS bertingkat.
- Node internal `mid`.

Test:

- Input 0 V menghasilkan output rendah.
- Input 5 V menghasilkan output tinggi.

---


## 9. Sel Tambahan

### Transmission Gate

- NMOS dan PMOS paralel antara `a` dan `b`.
- Port `en` dan `enb`.
- Cell dibangun dengan validasi topologi, bukan DC path.

### Tri-State Inverter

- Inverter input `a` ke node internal `mid`.
- Transmission gate dari `mid` ke `y`.
- Port `en` dan `enb`.

### D Latch

- Transmission gate dari `d` ke `q`.
- Dua inverter cross-coupled untuk menyimpan state.
- Port `en` dan `enb`.
- Test menangkap input 5 V dan 0 V.

### D Flip-Flop

- Master latch dan slave latch.
- Inverter untuk clock bar.
- Port `d`, `clk`, `q`.
- Test saat ini memeriksa struktur dan port.

### SRAM 6T

- Dua inverter cross-coupled.
- Dua NMOS access.
- Port `q`, `qb`, `wl`, `bl`, `blb`.
- Test saat ini memeriksa struktur dan port.

### Ring Oscillator

- Tiga inverter dalam loop.
- Port observasi `y`.
- Struktur dapat dibangun.
- Osilasi belum terverifikasi karena transient deterministik dapat bertahan di titik ekuilibrium.

### Registry

- `CellKind` mencakup semua sel.
- `CellKind::build()` membangun sel.
- `CellKind::name()` mengembalikan nama.


### Half Adder

- Dibangun dari 5 sel NAND2.
- Output `sum` dan `carry`.
- Truth table test lengkap.

### Full Adder

- Dibangun dari 2 half adder dan 3 NAND2.
- Output `sum` dan `carry`.
- Truth table test lengkap.

### Logic Gates

- `and2` adalah NAND2 ditambah inverter.
- `or2` adalah tiga NAND2.
- `xor2` adalah empat NAND2.
- `mux2` memilih `a` saat `sel` rendah dan `b` saat `sel` tinggi.
- Semua punya truth table test.

### ALU Bit-Slice

- Port `a`, `b`, `cin`, `op0`, `op1`, `y`, `carry`.
- `op1:op0` memilih AND, OR, XOR, dan ADD.
- ADD memakai full adder.
- Output dipilih oleh tiga MUX2.
- Truth table test mencakup empat operasi.

### Arithmetic Flags

- `AluFlags` memuat `carry`, `zero`, `negative`, `overflow`, `sign`, `half_carry`.
- `add_with_carry` dan `sub_with_borrow` mengembalikan nilai dan flag.
- Flag `S` dihitung sebagai `N xor V`.
- Test exhaustive untuk nilai, carry/borrow, dan half carry.

### Noise Margin

- `noise_margins` mengambil sampel VTC `(input, output)`.
- Menghitung `VOH`, `VOL`, `VIL`, `VIH`, `NMH`, `NML`.
- Test inverter memastikan noise margin positif.

### Ripple-Carry Adder

- `ripple_carry_adder(bits)` membangun adder dari `bits` full adder.
- `adder8()` adalah shorthand 8 bit.
- Port `cin`, `a0..aN`, `b0..bN`, `sum0..sumN`, dan `cout`.
- 4-bit truth table test lulus untuk beberapa kombinasi.
- 8-bit test memeriksa port dan struktur.

## 10. Test

Test `sim-cells` mencakup:

- Inverter VTC.
- Inverter truth table.
- Switching threshold.
- NAND truth table.
- NOR truth table.
- Buffer follow input.
- Powered cell memiliki supply.

Total test `sim-cells`: 70.

---

## 11. Keterbatasan

- Belum ada analisis rise/fall time.
- Belum ada propagation delay.
- Belum ada input capacitance.
- Belum ada drive strength.
- Test D flip-flop dan SRAM masih struktural.
- Osilasi ring oscillator belum terverifikasi.
- Belum ada perbandingan ngspice.

---

## 12. Berikutnya

- Rise/fall time dan propagation delay.
- Verifikasi osilasi ring oscillator.
- Test fungsional D flip-flop dan SRAM.
- Golden data ngspice.
- Flag arithmetic dan ALU bit-slice.

Lihat `TODO.md` untuk daftar lengkap.
