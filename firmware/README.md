# Firmware

Direktori ini memuat firmware contoh untuk board AVR.

---

## blink.S

- Target ATmega328P.
- Mengubah PORTB bit 5.
- Delay software sederhana.
- DDRB dan PORTB memakai alamat I/O 0x04 dan 0x05.

Konfigurasi:

```asm
ldi r16, (1 << PB5)
out DDRB, r16
```

Loop:

```asm
out PORTB, r16
ldi r17, 0xFF
delay:
dec r17
brne delay
ldi r18, (1 << PB5)
eor r16, r18
rjmp loop
```

Encoding manual blink diuji di `crates/sim-avr/tests/blink.rs`.
Test membuktikan PORTB bit 5 toggle dan LED board menyala.

---

## timer_interrupt.S

- Target ATmega328P.
- Timer0 overflow interrupt.
- ISR menulis `r17 = 7`.
- Loop utama menunggu interrupt.

Jalankan demo:

```bash
cargo run -p sim-avr --example timer_interrupt
```

---

## Kompilasi dengan avr-gcc

`avr-gcc` tidak tersedia di environment ini, sehingga file HEX hasil
kompilasi belum disertakan.

Alur yang didukung secara konsep:

```bash
avr-gcc -mmcu=atmega328p -DF_CPU=16000000UL -Os -o blink.elf blink.S
avr-objcopy -O ihex -R .eeprom blink.elf blink.hex
./target/debug/embed-emu run blink.hex 100000
```

ELF juga dapat dimuat melalui API `sim_board::ElfImage`, lalu dikonversi
ke Intel HEX dengan `to_hex()`.

---

## Batas

- `avr-gcc` belum tersedia di environment ini.
- Belum ada firmware yang dikompilasi di CI.
- Contoh yang berjalan saat ini memakai program AVR yang dienkode manual.
- Belum ada firmware UART hello yang dikompilasi.

Panduan lengkap ada di `docs/43-user-guide.md`.
