# 16 - AVR Instruction Decode

Dokumen ini menjelaskan decoder ISA AVR di `sim-avr`.

---

## 1. Status

P7-01 selesai untuk instruction set AVRe+ yang dipakai ATmega328P.

Yang sudah didecode:

- Arithmetic dan logic: ADD, ADC, ADIW, SUB, SUBI, SBC, SBCI, SBIW, AND, ANDI, OR, ORI, EOR, COM, NEG, INC, DEC, MUL, MULS, MULSU, FMUL, FMULS, FMULSU.
- Compare dan skip: CP, CPC, CPI, CPSE, SBRC, SBRS, SBIC, SBIS.
- Data transfer: MOV, MOVW, LDI, LDS, STS, LD X/Y/Z, LD post-increment, LD pre-decrement, LDD, ST X/Y/Z, ST post-increment, ST pre-decrement, STD, LPM, ELPM, PUSH, POP.
- Control flow: RJMP, JMP, IJMP, EIJMP, RCALL, CALL, ICALL, EICALL, RET, RETI.
- Branch: BRBS, BRBC, termasuk alias BREQ/BRNE dan lainnya melalui flag index.
- Bit dan I/O: SBI, CBI, BST, BLD, BSET, BCLR, IN, OUT.
- Miscellaneous: NOP, SLEEP, WDR, BREAK, SPM.

Instruksi khusus AVRxm seperti DES, LAC, LAS, LAT, XCH, dan SPM Z+ tidak termasuk karena bukan bagian AVRe+ pada ATmega328P.

---

## 2. API

```rust
pub fn decode(first: u16, second: Option<u16>) -> Result<Instruction, SimError>;
pub fn uses_second_word(first: u16) -> bool;
```

`Instruction` memuat `mnemonic`, dua operand, dan jumlah word.

`Operand` mendukung register, immediate, displacement, alamat data, alamat program, offset cabang, bit, I/O, dan flag.

---

## 3. Aturan Decode

- Instruksi 16-bit hanya memakai word pertama.
- Instruksi 32-bit membutuhkan `second`.
- Jika word kedua hilang, hasilnya `SimError::Parse`.
- Opcode tidak dikenal menghasilkan `SimError::Parse`.
- Branch offset disimpan sebagai word offset bertanda.
- Alamat JMP/CALL disimpan sebagai alamat word 22-bit.
- Alamat LDS/STS disimpan sebagai alamat data 16-bit.
- I/O address disimpan tanpa offset data space.
- LDD/STD dengan displacement 0 tetap didecode sebagai LDD/STD.

---

## 4. Test

Test decoder mencakup:

- Arithmetic register dan immediate.
- Single-register operations.
- Branch dan relative jump.
- LDD/STD.
- LDS/STS.
- Indirect load dan store.
- I/O dan bit operations.
- JMP/CALL 32-bit.
- Fixed operations dan flag operations.
- Deteksi word kedua dan penolakan instruksi terpotong.
- Iterasi seluruh 65.536 word untuk memastikan tidak ada panic.
- Validasi rentang register, bit, flag, I/O, displacement, dan jumlah word.

Total test `sim-avr`: 15.

---

## 5. Batas

- Decoder belum mengeksekusi instruksi.
- Belum ada timing per instruksi.
- Belum ada disassembler.
- Belum ada validasi terhadap `avr-objdump`.
- Belum ada differential test terhadap simavr.
