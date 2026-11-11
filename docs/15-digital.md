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
- `register_file` menyediakan 32 x 8 bit, decoder tulis, dan dua read port.
- `ProgramCounter` adalah register 16-bit dengan load dan increment.
- `StackPointer` adalah register 16-bit dengan load, increment, dan decrement.
- `register` adalah register loadable yang dipakai IR dan register file.

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

## 7. Register File

- 32 register x 8 bit.
- Decoder 5-ke-32 memilih register tulis.
- Write enable menjaga isi saat rendah.
- Dua read port 32:1 berbasis mux tree.
- Reset mengosongkan seluruh register.
- Test menulis dan membaca 32 register, hold, port independen, dan reset.

## 8. Program Counter, IR, dan Stack Pointer

- PC 16-bit: increment setiap tick atau load alamat target.
- IR 16-bit: load instruction word dan menahan nilainya.
- SP 16-bit: load, increment, dan decrement.
- Prioritas SP: load, lalu increment, lalu decrement, lalu hold.
- Reset global mengosongkan PC dan SP.

---

## 9. Digital ALU

- Operand 8 bit, opcode 3 bit.
- Operasi: ADD, SUB, AND, OR, XOR, dan pass B.
- SUB memakai two's complement.
- Output: result, zero, carry/borrow, negative, half carry/borrow.
- Carry dan half carry dibalik saat SUB agar berarti borrow.

---

## 10. CPU Uji

CPU kecil memakai FSM dua fase: FETCH dan EXECUTE.

- FETCH: IR memuat instruction word, PC naik.
- EXECUTE: register write, branch, atau halt.
- Zero flag disimpan di D flip-flop saat operasi arithmetic/logic.
- Branch memakai zero flag tersimpan, bukan output ALU saat instruksi branch.

Instruksi:

| Opcode | Instruksi | Format |
|---|---|---|
| 0 | NOP | - |
| 1 | LDI rd, imm6 | rd 5 bit, imm 6 bit |
| 2 | ADD rd, rs | rd 5 bit, rs 5 bit |
| 3 | SUB rd, rs | rd 5 bit, rs 5 bit |
| 4 | AND rd, rs | rd 5 bit, rs 5 bit |
| 5 | OR rd, rs | rd 5 bit, rs 5 bit |
| 6 | XOR rd, rs | rd 5 bit, rs 5 bit |
| 7 | MOV rd, rs | rd 5 bit, rs 5 bit |
| 8 | JMP addr11 | target 11 bit |
| 9 | JZ addr11 | target 11 bit |
| 10 | JNZ addr11 | target 11 bit |
| 31 | HLT | - |

- Register file: 32 x 8 bit.
- Program memory: behavioral `Vec<u16>`, maksimum 2048 word.
- Control PLA: decoder opcode, write enable, flag enable, ALU op, branch, dan halt.
- `CpuTrace` mencatat PC, IR, halt, dan snapshot 32 register.
- Test mencakup arithmetic, branch, loop, JMP, reset, halt, dan trace.

Total test `sim-cells` saat ini: 66.
