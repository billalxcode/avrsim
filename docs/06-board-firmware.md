# 06 — Board dan Firmware

Dokumen ini menjelaskan model board, komponen, toolchain firmware, loader, dan demo end-to-end.

---

## 1. Ruang Lingkup Board

**Board profile pertama:** Arduino Uno-like.

| Elemen | Model |
|---|---|
| MCU ATmega328P | Struktural |
| LED D13 (PB5) | Dioda + resistor |
| Resistor seri LED | Device model |
| Tombol reset | Device model (opsional) |
| UART host | Endpoint behavioral |
| Power 5 V | Ideal (tahap awal) |
| Crystal 16 MHz | Ideal clock (tahap awal) |
| Regulator | Struktural (tahap lanjut) |
| USB-UART IC | Struktural (tahap lanjut) |
| PCB parasitics | Tidak dimodelkan awal |

### Prinsip

- **IC aktif struktural**: MCU, regulator, USB-UART (target akhir).
- **Komponen pasif model device**: R, C, L.
- **Host PC behavioral**: bukan bagian board.
- **Board tidak menyembunyikan detail MCU**: pin dan net eksplisit.

---

## 2. Komponen Board

### 2.1 LED

Model dioda:

```
I = Is * (exp(V / (n * Vt)) - 1)
```

Parameter:

- `Is`
- `n`
- `Vf` pada arus nominal
- `Rs` seri
- Warna (hanya metadata)

Perilaku:

- Menyala jika arus > threshold.
- Tegangan maju mendekati `Vf`.
- Tidak memodelkan cahaya secara fisik.

### 2.2 Resistor

- Ideal atau dengan toleransi.
- Daya maksimum tidak dimodelkan awal.
- Stamping konduktansi.

### 2.3 Tombol

- Model switch dengan `Ron`/`Roff`.
- Debounce dilakukan firmware atau model mekanik sederhana.
- Pull-up internal MCU dapat digunakan.

### 2.4 UART Host

- Endpoint behavioral.
- Menerima byte dari pin TX MCU.
- Mengirim byte ke pin RX MCU.
- Menampilkan output ke stdout.
- Tidak memodelkan USB.

### 2.5 Power

Tahap awal:

- VDD = 5 V ideal.
- GND = 0 V ideal.
- Tidak ada IR drop.
- Tidak ada noise.

Tahap akhir:

- Regulator linear/switching struktural.
- Decoupling capacitor.
- Power tree.
- Brown-out.

### 2.6 Clock

Tahap awal:

- Clock 16 MHz ideal.
- Tidak ada jitter.
- Tidak ada startup time.

Tahap akhir:

- Crystal/oscillator model.
- Startup time.
- Jitter (opsional).

---

## 3. Board Netlist API (Rencana)

```rust
let mut board = Board::new();

let mcu = board.add_atmega328p("u1", Atmega328pConfig {
    firmware: "blink.hex",
    clock_hz: 16_000_000,
    vdd: 5.0,
});

let led = board.add_led("d13", LedConfig::red());
let r = board.add_resistor("r1", 220.0);

board.connect(mcu.pin("PB5"), r.node(0));
board.connect(r.node(1), led.anode);
board.connect(led.cathode, board.ground());

board.connect_uart(mcu.uart0(), UartHost::stdout());
```

Tujuan:

- Board description eksplisit.
- Tidak ada “magic” yang menghubungkan pin.
- Mudah diganti dengan board lain.

---

## 4. Firmware Toolchain

### 4.1 Kompilasi

- `avr-gcc` eksternal.
- Bahasa: C atau assembly.
- Output: `.elf`, `.hex`.
- Emulator tidak mengompilasi firmware.
- Runtime tetap native.

Contoh (rencana):

```
avr-gcc -mmcu=atmega328p -DF_CPU=16000000UL -Os -o blink.elf blink.c
avr-objcopy -O ihex blink.elf blink.hex
```

### 4.2 Fuses

Profil Arduino Uno:

| Fuse | Nilai umum | Catatan |
|---|---|---|
| Low | 0xFF | External crystal, slow rising |
| High | 0xDE | Bootloader enabled |
| Extended | 0xFD | Brown-out 2.7 V |

Nilai ini `TBD` dan harus diverifikasi. Jika bootloader tidak dimodelkan, reset vector disesuaikan.

### 4.3 Bootloader

- Awalnya tidak dimodelkan.
- Firmware dimuat di 0x0000.
- Jika bootloader diperlukan:
  - Flash layout 32 KB.
  - Boot section.
  - Reset vector.
  - Delay boot.

### 4.4 Reset Vector

- PC = 0x0000.
- SP = RAMEND.
- Register/SRAM initial state `TBD`.
- Clock aktif.
- Interrupt disabled (`I = 0`).

---

## 5. Loader

### 5.1 Intel HEX

- Parse record `:llaaaatt[dd...]cc`.
- Tipe:
  - 00 data.
  - 01 EOF.
  - 02 extended segment address.
  - 04 extended linear address.
- Validasi checksum.
- Deteksi alamat di luar flash.
- Error yang jelas.

### 5.2 ELF

- Target lanjut.
- Parse header, program header, section.
- Simbol untuk debugging.
- Tidak wajib untuk milestone pertama.

### 5.3 Firmware Metadata

- Entry point.
- Fuse.
- Clock.
- Board profile.
- Simbol (opsional).

---

## 6. Demo Firmware

### 6.1 Blink

Contoh pseudo-C:

```c
#include <avr/io.h>
#include <util/delay.h>

int main(void) {
    DDRB |= (1 << PB5);
    while (1) {
        PORTB ^= (1 << PB5);
        _delay_ms(500);
    }
}
```

Yang diuji:

- Reset vector.
- GPIO output.
- Delay loop / timer.
- LED model.
- Clock 16 MHz.

### 6.2 UART Hello

```c
#include <avr/io.h>
#include <util/delay.h>

static void uart_init(void) {
    UBRR0 = 103; // 9600 @ 16 MHz
    UCSR0B = (1 << TXEN0);
    UCSR0C = (3 << UCSZ00);
}

static void uart_tx(char c) {
    while (!(UCSR0A & (1 << UDRE0)));
    UDR0 = c;
}

int main(void) {
    uart_init();
    const char *msg = "hello\r\n";
    while (1) {
        for (const char *p = msg; *p; p++) uart_tx(*p);
        _delay_ms(1000);
    }
}
```

Yang diuji:

- UART baud generator.
- TX shift register.
- Flag UDRE/TXC.
- Host endpoint.
- Timer/delay.

### 6.3 Timer Interrupt

Target lanjut:

- Timer0 overflow.
- `ISR(TIMER0_OVF_vect)`.
- Toggle LED.
- Menguji interrupt controller.

---

## 7. CLI (Rencana)

```bash
# DC sweep inverter → CSV
emucad run --circuit inverter --analysis dc-sweep \
  --start 0 --stop 5 --step 0.01 --output vtc.csv

# Transient inverter → VCD
emucad run --circuit inverter --analysis transient \
  --stop 100n --step 0.1n --output inverter.vcd

# Board + firmware
emucad run --board uno --firmware blink.hex --time 2s --vcd blink.vcd

# Monitor
emucad monitor --board uno --firmware blink.hex
```

---

## 8. Output

### 8.1 CSV

- DC sweep.
- Operating point.
- Tabel parameter.

### 8.2 VCD

- Transient.
- Pin digital.
- Node analog.
- Scope hierarkis.

### 8.3 Log

- Firmware UART ke stdout.
- Error solver ke stderr.
- Trace opsional.

---

## 9. Test Board

### 9.1 Blink

- LED berubah state.
- Periode sesuai delay.
- Tidak ada illegal instruction.
- VCD menunjukkan pin toggling.

### 9.2 UART

- Output “hello”.
- Baud rate benar.
- Tidak ada framing error.
- Flag UART benar.

### 9.3 Reset

- Firmware mulai dari 0x0000.
- SP benar.
- GPIO initial state benar.

### 9.4 Beban

- LED + resistor.
- Tombol + pull-up.
- RC filter (target lanjut).

---

## 10. Non-Goal Board

- PCB layout.
- Parasitic extraction.
- Thermal.
- EMC/EMI.
- USB protocol.
- Mechanical model.
- Power consumption real-time.

---

## 11. Keputusan Terbuka

| ID | Pertanyaan |
|---|---|
| B01 | Nilai resistor LED demo |
| B02 | Baud rate demo |
| B03 | Apakah bootloader dimodelkan? |
| B04 | Fuse default final |
| B05 | Apakah UART host membaca stdin? |
| B06 | Format waktu CLI (`2s`, cycle, ns) |
| B07 | Apakah regulator struktural masuk M7? |
| B08 | Apakah crystal model masuk M7? |
