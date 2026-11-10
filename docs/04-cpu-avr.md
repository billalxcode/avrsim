# 04 — CPU AVR Struktural

Dokumen ini menjelaskan desain CPU ATmega328P yang dibangun dari transistor/gerbang, bukan instruction-set simulator.

---

## 1. Prinsip

- **Struktural, bukan behavioral** untuk CPU, register file, ALU, control, dan peripheral.
- **SRAM dan flash behavioral** (pengecualian fidelity D19).
- **Seluruh ISA AVR** menjadi target.
- **Multi-cycle FSM** dengan control hardwired/PLA.
- **Clock fisik 16 MHz**; delay gerbang menentukan perilaku.
- **Clean-room**, bukan salinan die.

---

## 2. Target Profil

| Properti | Nilai |
|---|---|
| MCU | ATmega328P |
| Arsitektur | AVR 8-bit, Harvard |
| Flash | 32 KB (behavioral) |
| SRAM | 2 KB (behavioral) |
| EEPROM | 1 KB (behavioral) |
| Register | 32 × 8-bit |
| PC | 16-bit (word/byte address sesuai model) |
| SP | 16-bit |
| Clock | 16 MHz |
| VDD | 5 V |
| Interrupt | 26 vector |
| Stack | Di SRAM, tumbuh ke bawah |

---

## 3. Organisasi CPU

### 3.1 Multi-Cycle FSM

Fase yang direncanakan:

1. **FETCH** — ambil instruction word dari flash.
2. **DECODE** — decode opcode, baca register.
3. **EXECUTE** — ALU, address calculation, branch.
4. **MEMORY** — load/store bila ada.
5. **WRITEBACK** — tulis register/SREG.
6. **INTERRUPT CHECK** — sebelum instruksi berikutnya.

Tidak semua instruksi melewati semua fase; FSM memilih jalur sesuai opcode.

### 3.2 Control Unit

- Hardwired/PLA.
- Input: opcode, flags, interrupt pending, reset.
- Output: sinyal kontrol ALU, register write, memory read/write, PC load, SREG update.
- PLA diimplementasikan dari gerbang CMOS, bukan table di Rust.
- Untuk milestone awal, table perilaku boleh menjadi spesifikasi test, tetapi implementasi tetap struktural.

### 3.3 ALU

- 8-bit ripple-carry adder.
- Bit-slice: satu slice per bit.
- Operasi:
  - ADD, ADC
  - SUB, SBC, SUBI, SBCI
  - AND, ANDI
  - OR, ORI
  - EOR
  - COM, NEG
  - INC, DEC
  - CP, CPC, CPI
  - LSR, ASR, ROR, SWAP
- Flag:
  - C (carry)
  - Z (zero)
  - N (negative)
  - V (two's complement overflow)
  - S (N XOR V)
  - H (half carry)
  - T (bit transfer)
  - I (interrupt enable)

### 3.4 Register File

- 32 register × 8-bit.
- DFF statis per bit.
- Address decoder 5-to-32.
- Dua port baca, satu port tulis (atau sesuai kebutuhan ISA).
- Register 26–31 (X, Y, Z) dapat diakses sebagai pasangan 16-bit.
- Reset state: 0.

### 3.5 PC, IR, SP

- **PC**: 16-bit, word/byte address.
- **IR**: instruction register 16/32-bit.
- **SP**: 16-bit, reset ke `RAMEND` (0x08FF untuk ATmega328P).
- Stack push/pop melalui SRAM behavioral.

### 3.6 Memory Interface

- Flash: behavioral array 32 KB.
- SRAM: behavioral array 2 KB.
- EEPROM: behavioral array 1 KB.
- Data space map:
  - 0x0000–0x001F: register.
  - 0x0020–0x005F: I/O.
  - 0x0060–0x00FF: extended I/O.
  - 0x0100–0x08FF: SRAM.
- Akses memori mengikuti timing yang didefinisikan; detail wait state `TBD`.

---

## 4. Instruction Set

### 4.1 Kelompok Instruksi

| Kelompok | Contoh |
|---|---|
| Arithmetic | ADD, ADC, SUB, SBC, SUBI, SBCI, INC, DEC, NEG, COM |
| Logic | AND, ANDI, OR, ORI, EOR |
| Compare | CP, CPC, CPI, CPSE, TST |
| Branch | RJMP, JMP, IJMP, BRBS, BRBC, CPSE, SBRC, SBRS |
| Call/Return | RCALL, CALL, ICALL, RET, RETI |
| Data transfer | MOV, MOVW, LDI, LD, ST, LDD, STD, LDS, STS, PUSH, POP, IN, OUT |
| Bit | SBI, CBI, SBIC, SBIS, BST, BLD, BSET, BCLR |
| Shift | LSR, ASR, ROR, SWAP |
| Multiply | MUL, MULS, MULSU |
| Program memory | LPM, ELPM, SPM |
| Control | NOP, SLEEP, WDR, BREAK |
| Flags | SEC, CLC, SEI, CLI, dan lainnya lewat BSET/BCLR |

### 4.2 Instruksi 32-bit

- LDS, STS
- JMP, CALL
- Instruksi dengan immediate 16-bit

Fetch harus mendeteksi panjang instruksi sebelum PC maju.

### 4.3 Perilaku Flag

Contoh:

**ADD/ADC:**
```
H = (Rd3 & Rr3) | (Rr3 & ~R3) | (~R3 & Rd3)
V = (Rd7 & Rr7 & ~R7) | (~Rd7 & ~Rr7 & R7)
N = R7
Z = (R == 0)
C = (Rd7 & Rr7) | (Rr7 & ~R7) | (~R7 & Rd7)
S = N ^ V
```

**SUB/SBC:**
```
H = (~Rd3 & Rr3) | (Rr3 & R3) | (R3 & ~Rd3)
V = (Rd7 & ~Rr7 & ~R7) | (~Rd7 & Rr7 & R7)
N = R7
Z = (R == 0)
C = (~Rd7 & Rr7) | (Rr7 & R7) | (R7 & ~Rd7)
S = N ^ V
```

**INC:** V = (R == 0x80); C tidak berubah.
**DEC:** V = (R == 0x7F); C tidak berubah.
**LSR:** C = bit0; N = 0; V = N ^ C.
**ASR:** C = bit0; N = bit7; V = N ^ C.
**ROR:** C = bit0 lama; N = bit7 hasil; V = N ^ C.
**ADIW/SBIW:** flag sesuai datasheet, dengan carry/overflow dari 16-bit.

Setiap instruksi harus punya test vector, termasuk edge case.

### 4.4 Timing

- Sebagian besar instruksi 1 siklus.
- Load/store 2 siklus.
- Branch taken 2 siklus.
- CALL/RCALL/RET/RETI 3–4 siklus.
- Interrupt entry 4 siklus.
- Nilai persis mengikuti datasheet dan diverifikasi lewat test.

Clock 16 MHz → 62.5 ns per siklus. Structural delay harus cukup cepat untuk 16 MHz; jika tidak, itu bug desain.

---

## 5. Interrupt

### 5.1 Vektor

ATmega328P memiliki 26 vektor, termasuk:

| Vektor | Alamat word |
|---|---|
| RESET | 0x0000 |
| INT0 | 0x0002 |
| INT1 | 0x0004 |
| PCINT0 | 0x0006 |
| PCINT1 | 0x0008 |
| PCINT2 | 0x000A |
| WDT | 0x000C |
| TIMER2 COMPA | 0x000E |
| TIMER2 COMPB | 0x0010 |
| TIMER2 OVF | 0x0012 |
| TIMER1 CAPT | 0x0014 |
| TIMER1 COMPA | 0x0016 |
| TIMER1 COMPB | 0x0018 |
| TIMER1 OVF | 0x001A |
| TIMER0 COMPA | 0x001C |
| TIMER0 COMPB | 0x001E |
| TIMER0 OVF | 0x0020 |
| SPI STC | 0x0022 |
| USART RX | 0x0024 |
| USART UDRE | 0x0026 |
| USART TX | 0x0028 |
| ADC | 0x002A |
| EE READY | 0x002C |
| ANALOG COMP | 0x002E |
| TWI | 0x0030 |
| SPM READY | 0x0032 |

### 5.2 Perilaku

- Interrupt dicek di antara instruksi.
- Jika `I = 1` dan ada pending:
  - Simpan PC ke stack.
  - Clear `I`.
  - Load PC dengan vektor.
  - Set flag pending clear.
- `RETI` mengembalikan PC dan set `I`.
- `SEI` menunda interrupt satu instruksi (sesuai AVR).
- Prioritas berdasarkan alamat vektor terendah.
- Interrupt latency harus sesuai datasheet.

---

## 6. Reset dan Fuses

- Reset vector 0x0000.
- SP diinisialisasi ke `RAMEND`.
- Register dan SRAM state awal `TBD`; dokumentasikan.
- Fuse clock:
  - Arduino Uno memakai crystal 16 MHz.
  - CKDIV8 dan prescaler harus dimodelkan.
- Bootloader:
  - Awalnya tidak dimodelkan.
  - Firmware dimuat langsung di 0x0000.
  - Jika bootloader diperlukan, flash layout dan reset vector harus disesuaikan.
- Watchdog reset dan brown-out menjadi tahap lanjut.

---

## 7. Debugging

### 7.1 CLI Monitor

Perintah:

```
step
continue
break <addr>
delete <id>
regs
mem <addr> <len>
disasm <addr> <count>
pin <port> <bit>
stack
sreg
quit
```

### 7.2 Breakpoint

- PC breakpoint.
- Data watchpoint (opsional).
- Cycle counter.
- Trace instruksi (opsional, verbose).

### 7.3 Disassembler

- Seluruh ISA AVR.
- Format mirip `avr-objdump`.
- Simbol dari ELF bila tersedia.

---

## 8. Verifikasi

### 8.1 Test Vector ISA

- Setiap instruksi diuji dengan operand normal, edge case, dan flag.
- Arithmetic exhaustif untuk 8-bit add/sub (256 × 256).
- Shift/rotate edge case.
- Load/store semua mode.
- Branch taken/not taken.
- Call/return dan stack.
- Interrupt entry/return.

### 8.2 Differential Testing

- Bandingkan dengan simavr (dev-only).
- Program acak terbatas pada instruksi yang didukung.
- Bandingkan register, SREG, SRAM, dan cycle count.
- Jangan memakai simavr sebagai runtime.

### 8.3 Firmware

- Blink hasil assembly.
- Blink hasil `avr-gcc`.
- UART hello.
- Timer interrupt.
- Semakin banyak sketch seiring peripheral.

### 8.4 Struktural vs Behavioral

- Model struktural harus cocok dengan model behavioral/ISS untuk perilaku yang sama.
- Perbedaan timing dicatat dan dijelaskan.
- Model behavioral dapat menjadi oracle tambahan.

---

## 9. Urutan Implementasi

1. 1-bit full adder.
2. 8-bit ripple-carry adder.
3. Flag logic.
4. ALU bit-slice.
5. Register file.
6. PC/IR/SP.
7. Control PLA/FSM.
8. CPU uji kecil.
9. AVR decode/execute subset.
10. Seluruh ISA.
11. Memory interface.
12. Interrupt.
13. Timing.
14. Firmware blink.
15. UART/peripheral.

---

## 10. Risiko CPU

| Risiko | Dampak | Mitigasi |
|---|---|---|
| Jumlah gerbang sangat besar | Simulasi lambat | Mulai dari subset, optimasi kemudian |
| Control PLA kompleks | Bug decode | Test vector per instruksi |
| Timing 16 MHz tidak tercapai | Firmware timing salah | Ukuran transistor/pipeline, verifikasi delay |
| Interrupt race | Firmware hang | Test interrupt intensif |
| SRAM/flash behavioral mismatch | Bug akses memori | Kontrak timing eksplisit |
| Register file besar | Waktu simulasi | DFF statis, optimasi flatten |

---

## 11. Keputusan Terbuka

| ID | Pertanyaan |
|---|---|
| A01 | Apakah bootloader Arduino dimodelkan? |
| A02 | Detail timing flash wait state |
| A03 | Nilai awal SRAM/register setelah reset |
| A04 | Apakah pipeline fetch/execute ditambahkan nanti? |
| A05 | Format simbol ELF |
| A06 | Apakah watchpoint didukung di monitor? |
| A07 | Target cycle exact per instruksi |
| A08 | Strategi optimasi register file |
