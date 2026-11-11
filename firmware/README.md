# Firmware

Direktori ini memuat firmware contoh untuk board AVR.

---

## blink.S

- Target ATmega328P.
- Mengubah PORTB bit 5.
- Delay software sederhana.
- DDRB dan PORTB memakai alamat I/O 0x04 dan 0x05.

Konfigurasi:

```
ldi r16, (1 << PB5)
out DDRB, r16
```

Loop:

```
out PORTB, r16
ldi r17, 0xFF
delay:
dec r17
brne delay
ldi r18, (1 << PB5)
eor r16, r18
rjmp loop
```

---

## Encoding manual

Encoding manual blink sudah diuji di `crates/sim-avr/tests/blink.rs`.
Test membuktikan PORTB bit 5 toggle dan LED board menyala.

---

## Batas

- `avr-gcc` belum tersedia di lingkungan ini.
- Belum ada konversi ELF ke HEX otomatis.
- Belum ada firmware UART hello.
- Belum ada timer interrupt demo.
- Belum ada contoh firmware yang dikompilasi di CI.
