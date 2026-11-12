# 16 - AVR Instruction Decode

Dokumen ini menjelaskan decoder ISA AVR di `sim-avr`.

---

## 1. Status

P7-01 sampai P7-30 sudah mencakup decoder AVRe+, fetch, register bank, unit
arithmetic, unit logic, dan unit shift. Eksekusi instruksi end-to-end belum
dihubungkan ke control FSM.

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

pub struct ProgramMemory { /* ... */ }

impl ProgramMemory {
    pub fn new(words: Vec<u16>) -> Self;
    pub fn read(&self, address: usize) -> Result<u16, SimError>;
    pub fn fetch(&self, pc: u16) -> Result<Fetched, SimError>;
}
```

`Fetched` memuat instruksi hasil decode, word pertama, word kedua opsional, dan jumlah word.

`Instruction` memuat `mnemonic`, dua operand, dan jumlah word.

`Operand` mendukung register, immediate, displacement, alamat data, alamat program, offset cabang, bit, I/O, dan flag.

---

## 3. Fetch

- PC adalah word address.
- Fetch 16-bit membaca satu word.
- Fetch 32-bit membaca dua word berurutan.
- `uses_second_word` menentukan apakah word kedua diperlukan.
- Word kedua yang hilang menghasilkan error.
- Alamat di luar program memory menghasilkan error.

---

## 4. Instruction Register dan Register Bank

- `InstructionRegister` adalah register 32-bit dari D flip-flop.
- IR memuat instruction word hasil fetch dan menahannya saat load rendah.
- Reset global mengosongkan IR.
- `RegisterBank` membungkus register file 32 x 8.
- Read port A dan B dapat mengakses register berbeda secara bersamaan.
- Write port memakai alamat 5-bit dan write enable.
- Pasangan register dibaca little-endian, misalnya X = r27:r26.

---

## 5. Aturan Decode

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

## 6. Arithmetic Unit

- `arithmetic_unit` adalah ripple adder 8-bit dari gate digital.
- Input `subtract` membalik operand B dan carry input.
- Input `carry_in` memakai flag C untuk ADC dan SBC.
- Output: result, C/borrow, Z, N, V, S, dan H.
- Overflow memakai formula addition atau subtraction sesuai `subtract`.
- Hasil ADD, ADC, SUB, dan SBC dibandingkan dengan reference `sim-cells`.
- Test menjalankan seluruh 256 nilai operand pertama untuk beberapa operand kedua dan carry.
- `select_operand` memilih register atau immediate.
- SUBI dan SBCI memakai subtractor yang sama dengan operand immediate.
- Test SUBI dan SBCI memakai seluruh 256 nilai register, beberapa immediate, dan kedua nilai carry.
- NEG memakai arithmetic unit dengan operand nol dan subtract aktif.
- INC dan DEC memakai arithmetic unit dengan operand satu.
- INC dan DEC mempertahankan C dan H; hanya Z, N, V, dan S yang dipakai.
- Test NEG, INC, dan DEC menjalankan seluruh 256 nilai.

## 7. Logic Unit

- `logic_unit` memakai gate AND, OR, XOR, dan NOT per bit.
- Opcode 2-bit memilih AND, OR, XOR, dan NOT.
- Output: result, Z, N, dan S.
- V selalu nol untuk operasi logic.
- C dan H tidak dihitung dan harus dipertahankan oleh control SREG.
- Test membandingkan hasil dengan reference untuk seluruh 256 operand pertama, beberapa operand kedua, dan empat operasi.
- ANDI dan ORI memakai logic unit dengan operand immediate dari `select_operand`.

## 8. Shift Unit

- `shift_unit` mendukung LSR, ASR, ROR, dan SWAP.
- Opcode 2-bit memilih operasi.
- LSR mengisi bit 7 dengan nol.
- ASR mempertahankan bit 7.
- ROR memasukkan C lama ke bit 7.
- SWAP menukar nibble atas dan bawah.
- Output: result, C, Z, N, V, dan S.
- Test seluruh 256 nilai, kedua nilai carry, dan tiga operasi shift.
- SWAP diuji terpisah karena flag-nya diabaikan.

## 9. Multiply Unit

- `multiply_unit` adalah multiplier unsigned 8 x 8 berbasis shift-and-add.
- `multiply_signed_unit` memakai nilai absolut dan koreksi tanda.
- `multiply_signed_unsigned_unit` menangani operand pertama signed dan kedua unsigned.
- Hasil 16-bit, flag Z, dan C dari bit 15.
- Test MUL, MULS, dan MULSU memakai seluruh 256 nilai operand pertama, beberapa operand kedua, dan reference Rust.

## 10. Compare Unit

- `compare_unit` memakai arithmetic unit dengan subtract aktif.
- Hasil arithmetic tidak diekspos karena compare tidak menulis register.
- CP memakai carry input nol.
- CPC memakai flag C sebagai carry input.
- CPI memakai operand immediate dari `select_operand`.
- Test seluruh 256 nilai operand pertama, beberapa operand kedua atau immediate, dan kedua nilai carry.
- Flag hasil dibandingkan dengan reference subtraction.

## 11. CPU Integration

CPU kecil di `sim-avr/src/cpu.rs` menghubungkan fetch, register bank, dan unit
ALU menjadi eksekusi instruksi end-to-end.

- Control masih behavioral di Rust, bukan PLA gate-level.
- Datapath memakai register bank DFF dan unit arithmetic/logic/shift/multiply struktural.
- SREG disimpan sebagai state Rust.
- PC disimpan sebagai word address Rust.
- Trace mencatat PC, instruksi, 32 register, SREG, dan halt.
- I/O register disimpan sebagai array 64 byte behavioral.
- Data space disimpan sebagai array 4 KB behavioral.
- Stack pointer reset ke 0x08FF dan tumbuh ke bawah.
- LPM/ELPM membaca byte dari program memory behavioral.
- SREG dapat dibaca dan ditulis melalui I/O address 0x3F.
- SLEEP menghentikan core sampai interrupt, yang belum dimodelkan.
- WDR tidak melakukan apa pun tanpa watchdog.
- Reset mengosongkan register, data, I/O, SREG, PC, SP, dan halt.

Instruksi yang sudah berjalan end-to-end:

- LDI, MOV, MOVW.
- ADD, ADC, SUB, SBC, SUBI, SBCI.
- AND, ANDI, OR, ORI, EOR, COM.
- NEG, INC, DEC.
- CP, CPC, CPI.
- LSR, ASR, ROR, SWAP.
- MUL.
- RJMP, JMP, BRBS, BRBC.
- CPSE, SBRC, SBRS, SBIC, SBIS.
- SBI, CBI, IN, OUT.
- BSET, BCLR, BST, BLD.
- LD/ST X, Y, Z, post-increment, pre-decrement.
- LDD/STD, LDS/STS, LPM, ELPM.
- PUSH, POP, RCALL, CALL, ICALL, IJMP, RET, RETI.
- NOP, BREAK.

Test program mencakup arithmetic, carry, logic, compare, branch, jump, MOVW,
shift, rotate, multiply, trace, dan halt.

## 12. Interrupt

- `VECTORS` memuat 26 alamat word vektor ATmega328P.
- `request_interrupt` menandai pending interrupt berdasarkan index vektor.
- Interrupt dilayani sebelum fetch jika flag I aktif dan pending tidak nol.
- Interrupt entry mendorong PC ke stack, menghapus pending, clear I, dan memuat PC vektor.
- RETI mengembalikan PC dan men-set I.
- SEI menunda enable interrupt selama satu instruksi berikutnya.
- SLEEP menghentikan eksekusi sampai interrupt pending dan I aktif.

## 13. Test

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
- Fetch 16-bit dan 32-bit dari program memory.
- Fetch menggunakan word address.
- Penolakan fetch terpotong dan alamat di luar batas.
- IR load, hold, reset, dan validasi lebar data.
- Register bank write/read, dua read port, pasangan little-endian, hold, dan reset.
- Test vector arithmetic ADD, ADC, SUB, SBC, dan immediate.
- Test vector logic, shift, multiply, load/store, branch, call/stack, dan interrupt.
- Blink assembly dijalankan dan PORTB bit 5 terbukti toggle.

Total test `sim-avr`: 202.
Total test `sim-cli`: 20.

---

## 14. Timing

- `AvrTrace` mencatat cycle instruksi dan total cycle.
- Sebagian besar instruksi memakai 1 cycle.
- MUL, LD/ST, LDD/STD, LDS/STS, PUSH/POP, RJMP, IJMP, ADIW, SBIW, dan SBI/CBI memakai 2 cycle.
- JMP, RCALL, ICALL, EICALL, LPM, dan ELPM memakai 3 cycle.
- CALL, RET, dan RETI memakai 4 cycle.
- Branch taken menambah 1 cycle.
- Skip yang melewati instruksi 2 word memakai 3 cycle.
- Interrupt entry menambah 4 cycle.
- Model ini belum memasukkan wait state flash dan timing silikon detail.

## 15. Disassembler

- `disassemble` mengubah `Instruction` menjadi teks assembly.
- Nama mnemonic dibagi per kategori agar fungsi tetap pendek.
- Operand register, immediate, alamat, displacement, bit, I/O, dan flag diformat terpisah.
- LD/ST pointer dan LDD/STD memakai format Y/Z eksplisit.
- Test mencakup arithmetic, logic, memory, control flow, I/O, dan bit operations.
- Belum ada validasi output terhadap `avr-objdump`.

## 16. CLI Monitor

`embed-emu monitor <hex words...>` menjalankan monitor interaktif.

Perintah:

- `step`: eksekusi satu instruksi.
- `run [n]`: eksekusi sampai halt atau batas langkah.
- `regs`: dump 32 register.
- `sreg`: dump flag SREG.
- `pc`: tampilkan PC dan cycle.
- `disasm <pc> <count>`: disassembly dari program memory.
- `mem <addr> <len>`: dump data space.
- `io <addr>`: baca I/O register.
- `quit`: keluar.

`embed-emu disasm <hex words...>` mencetak disassembly tanpa menjalankan program.

## 17. ISA Support

| Kelompok | Status |
|---|---|
| Arithmetic ADD, ADC, SUB, SBC, SUBI, SBCI | Berjalan di CPU |
| NEG, INC, DEC | Berjalan di CPU |
| Logic AND, ANDI, OR, ORI, EOR, COM | Berjalan di CPU |
| Compare CP, CPC, CPI | Berjalan di CPU |
| Shift LSR, ASR, ROR, SWAP | Berjalan di CPU |
| Multiply MUL | Berjalan di CPU |
| Multiply MULS, MULSU, FMUL | Unit tersedia, belum dihubungkan ke CPU |
| Move LDI, MOV, MOVW | Berjalan di CPU |
| Load/store LD, ST, LDD, STD, LDS, STS | Berjalan di CPU |
| LPM, ELPM | Berjalan dengan program memory behavioral |
| PUSH, POP, RCALL, CALL, ICALL, IJMP, RET, RETI | Berjalan di CPU |
| Branch RJMP, JMP, BRBS, BRBC, CPSE, SBRC, SBRS | Berjalan di CPU |
| Bit dan I/O SBI, CBI, SBIC, SBIS, IN, OUT, BST, BLD, BSET, BCLR | Berjalan di CPU |
| Interrupt controller, vector table, SEI delay, SLEEP | Berjalan di CPU |
| SPM, DES, LAC, LAS, LAT, XCH | Belum diimplementasikan |
| Timing | Model cycle-level, bukan timing silikon penuh |

Firmware `firmware/blink.S` sudah ditulis dan encoding manualnya diuji di CPU.
Kompilasi dengan `avr-gcc` belum dilakukan karena toolchain tidak tersedia.

## 18. Batas

- Control CPU masih behavioral, bukan PLA gate-level.
- Belum ada validasi disassembler terhadap `avr-objdump`.
- Belum ada differential test terhadap simavr.
- Blink assembly sudah ada, tetapi belum dikompilasi dengan `avr-gcc`.
- Belum ada differential test terhadap simavr.
- Belum ada timer, UART, GPIO, dan peripheral struktural.
