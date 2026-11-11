# 15 - Digital Abstraction

Dokumen ini menjelaskan lapisan logika digital dua-state di `sim-cells`.

---

## 1. Tujuan

Lapisan ini menjalankan blok sekuensial besar seperti register file, counter,
dan CPU uji tanpa harus menyelesaikan seluruh rangkaian pada level transistor
untuk setiap clock.

Lapisan ini bukan pengganti model transistor. Sel CMOS di `sim-cells` tetap
menjadi model elektrik. Lapisan digital dipakai untuk komposisi gate-level
setelah perilaku sel diverifikasi.

---

## 2. Model

- Nilai net hanya `false` (0) dan `true` (1).
- Gate kombinasional: NOT, AND, OR, XOR, NAND, NOR, MUX.
- Elemen state: D flip-flop rising-edge.
- Clock ideal: satu panggilan `tick()` adalah satu tepi clock.
- Reset ideal: `reset()` mengosongkan seluruh flip-flop.
- Settle dilakukan sampai stabil.
- Loop kombinasional yang tidak stabil menghasilkan
  `SimError::ConvergenceFailure`.
- Batas settle adalah 10.000 iterasi.

Semua operasi deterministik dan tidak memakai wall-clock.

---

## 3. API

```rust
pub struct DigitalBuilder {
    // ...
}

impl DigitalBuilder {
    pub fn input(&mut self, name: &str) -> NetId;
    pub fn output(&mut self, name: &str, net: NetId);
    pub fn constant(&mut self, value: bool) -> NetId;
    pub fn not(&mut self, input: NetId) -> NetId;
    pub fn and(&mut self, first: NetId, second: NetId) -> NetId;
    pub fn or(&mut self, first: NetId, second: NetId) -> NetId;
    pub fn xor(&mut self, first: NetId, second: NetId) -> NetId;
    pub fn nand(&mut self, first: NetId, second: NetId) -> NetId;
    pub fn nor(&mut self, first: NetId, second: NetId) -> NetId;
    pub fn mux(&mut self, first: NetId, second: NetId, select: NetId) -> NetId;
    pub fn dff(&mut self, data: NetId) -> NetId;
    pub fn build(self) -> DigitalCircuit;
}
```

`DigitalCircuit` menyediakan `set_input`, `output`, `settle`, `tick`, dan
`reset`.

---

## 4. Blok Dasar

- `and_all` dan `or_all` mereduksi vektor net.
- `mux_tree` memilih satu dari `2^n` input.
- `decoder` menghasilkan one-hot output.
- `counter` adalah ripple counter dengan enable.
- `shift_register` menggeser saat enable tinggi.
- `read_bits` membaca vektor bit little-endian.

Semua blok diuji secara fungsional dengan clock ideal.

---

## 5. Batas Fidelity

- Belum ada delay gerbang.
- Belum ada nilai X atau high-Z.
- Belum ada timing setup/hold.
- Belum ada konsumsi daya.
- Clock belum terdistribusi sebagai net fisik.
- Lapisan ini tidak membuktikan perilaku elektrik.

Blok yang memakai lapisan ini harus dicatat sebagai gate-level, bukan
transistor-level, sampai diverifikasi ulang dengan model elektrik.

---

## 6. Test

Test lapisan digital mencakup:

- Truth table gate kombinasional.
- MUX.
- D flip-flop capture dan hold.
- Reset.
- Deteksi loop kombinasional.
- Decoder one-hot.
- MUX tree.
- Counter dan hold.
- Shift register dan hold.

Total test `sim-cells` saat ini: 44.
