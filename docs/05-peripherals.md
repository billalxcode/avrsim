# 05 — Peripheral

Dokumen ini menjelaskan peripheral ATmega328P yang harus dibangun secara struktural.

---

## 1. Prinsip

- **Target akhir:** seluruh peripheral transistor-level.
- **Milestone pertama:** GPIO, timer, UART.
- **Setiap peripheral** memiliki:
  - Register map.
  - State machine/FSM.
  - Timing.
  - Interrupt.
  - Test vector.
- **Pin** adalah batas antara digital dan analog; model listriknya harus eksplisit.
- **Tidak ada peripheral behavioral** kecuali dinyatakan sebagai pengecualian.

---

## 2. GPIO

### 2.1 Register

| Port | PIN | DDR | PORT |
|---|---|---|---|
| B | 0x23 | 0x24 | 0x25 |
| C | 0x26 | 0x27 | 0x28 |
| D | 0x29 | 0x2A | 0x2B |

Data address = I/O address + 0x20.

### 2.2 Perilaku

- `DDRx = 1` → pin output.
- `DDRx = 0` → pin input.
- Output value dari `PORTx`.
- Input value dari pin eksternal.
- Pull-up aktif jika `DDRx = 0` dan `PORTx = 1`.
- Menulis `PINx` toggle `PORTx` (sesuai AVR).
- Reset: DDR/PORT = 0.

### 2.3 Pin Electrical Model

| Properti | Arti |
|---|---|
| `Voh` | Tegangan output high pada arus tertentu |
| `Vol` | Tegangan output low pada arus tertentu |
| `Rout` | Resistansi output ekivalen |
| `Vin_high` | Threshold input high |
| `Vin_low` | Threshold input low |
| `Hysteresis` | Histeresis Schmitt |
| `Ileak` | Arus bocor input |
| `Cin` | Kapasitansi input |
| `Pull-up` | Resistansi pull-up internal |
| `Clamp` | Dioda clamp ke VDD/GND |

Model digital memakai level logika; model analog memakai Thevenin:

```
Vout = Voh - I * Rout   (high)
Vout = Vol - I * Rout   (low)
```

### 2.4 Pin Change Interrupt

- `PCICR`, `PCMSK0/1/2`.
- Set pending saat pin berubah.
- Interrupt vector PCINT0/1/2.
- Edge/level sesuai register.

### 2.5 Test

- Output high/low.
- Input dengan pull-up.
- Input floating.
- Toggle via PIN.
- Pin change interrupt.
- Beban resistor/LED.

---

## 3. Timer/Counter 0

### 3.1 Register

| Register | Address |
|---|---|
| TCCR0A | 0x44 |
| TCCR0B | 0x45 |
| TCNT0 | 0x46 |
| OCR0A | 0x47 |
| OCR0B | 0x48 |
| TIMSK0 | 0x6E |
| TIFR0 | 0x35 |

### 3.2 Fitur

- 8-bit counter.
- Prescaler: 1, 8, 64, 256, 1024.
- Mode: normal, CTC, fast PWM, phase-correct PWM.
- Compare A/B.
- Overflow interrupt.
- Compare match interrupt.

### 3.3 Perilaku

- Counter increment pada setiap clock prescaler.
- Overflow saat 0xFF → 0x00.
- `TOV0` set saat overflow.
- `OCF0A/OCF0B` set saat compare match.
- Interrupt jika bit enable di `TIMSK0`.
- `TIFR0` write-one-to-clear.

### 3.4 Test

- Prescaler.
- Overflow period.
- CTC frequency.
- PWM duty cycle.
- Compare interrupt.
- `millis()`/`micros()` firmware.

---

## 4. Timer/Counter 1 (16-bit)

Fitur:

- 16-bit counter.
- Prescaler sama.
- Normal, CTC, fast PWM, phase-correct PWM.
- Input capture.
- Compare A/B.
- `ICR1`, `OCR1A`, `OCR1B`.
- Interrupt: CAPT, COMPA, COMPB, OVF.

Test:

- 16-bit overflow.
- Input capture.
- PWM 16-bit.
- Servo/tone firmware.

---

## 5. Timer/Counter 2 (8-bit)

Fitur:

- Mirip Timer0.
- Mendukung asynchronous clock dari crystal 32 kHz (target lanjut).
- Prescaler.
- PWM.
- Compare/overflow interrupt.

Test:

- Sama seperti Timer0.
- Async mode bila diimplementasikan.

---

## 6. USART0

### 6.1 Register

| Register | Address |
|---|---|
| UCSR0A | 0xC0 |
| UCSR0B | 0xC1 |
| UCSR0C | 0xC2 |
| UBRR0L | 0xC4 |
| UBRR0H | 0xC5 |
| UDR0 | 0xC6 |

### 6.2 Fitur

- Baud rate generator.
- TX shift register.
- RX shift register.
- 8-bit data, parity, stop bit.
- Interrupt: RX complete, TX complete, data register empty.
- Flag: RXC0, TXC0, UDRE0, FE0, DOR0, UPE0, U2X0.

### 6.3 Perilaku

- Menulis `UDR0` memulai transmisi jika TX enable.
- `UDRE0` set saat buffer kosong.
- `TXC0` set setelah transmisi selesai.
- RX menerima byte, set `RXC0`.
- Baud rate = `F_CPU / (16 * (UBRR + 1))` (mode normal) atau `F_CPU / (8 * (UBRR + 1))` (U2X).
- Host endpoint menerima byte dari pin TX dan mengirim byte ke pin RX.

### 6.4 Test

- Kirim “hello”.
- Baud rate 9600/115200.
- Interrupt TX/RX.
- Framing error.
- Echo.

---

## 7. SPI

Fitur:

- Master/slave.
- Clock polarity/phase (mode 0–3).
- Bit order.
- Prescaler.
- `SPCR`, `SPSR`, `SPDR`.
- Interrupt `SPI_STC`.
- Flag `SPIF`, `WCOL`.

Test:

- Loopback master-slave.
- Transfer byte.
- Mode 0–3.
- Interrupt.

---

## 8. I2C / TWI

Fitur:

- START/STOP.
- Address match.
- ACK/NACK.
- Clock stretching.
- `TWBR`, `TWSR`, `TWAR`, `TWDR`, `TWCR`.
- Interrupt `TWI`.

Test:

- Master write/read.
- Slave address match.
- ACK/NACK.
- Bus error.

---

## 9. ADC

Fitur:

- 10-bit SAR.
- 8-channel mux.
- Reference: AVCC, internal 1.1 V, AREF.
- Prescaler.
- Free-running mode.
- Interrupt `ADC`.
- `ADMUX`, `ADCSRA`, `ADCSRB`, `ADCL`, `ADCH`, `DIDR0`.

Perilaku:

- Konversi membutuhkan 13 ADC clock (mode normal).
- Hasil 10-bit.
- Input analog berasal dari solver analog atau model behavioral.
- `ADIF` set setelah konversi.

Test:

- Konversi 0 V, VREF, mid-scale.
- Mux channel.
- Reference.
- Free-running.
- Interrupt.

---

## 10. Analog Comparator

Fitur:

- Bandingkan AIN0 dan AIN1.
- Output `ACO`.
- Interrupt `ANALOG_COMP`.
- `ACSR`.
- Input dari pin analog.

Test:

- Threshold.
- Hysteresis bila ada.
- Interrupt.

---

## 11. EEPROM

Fitur:

- 1 KB.
- `EECR`, `EEDR`, `EEARL`, `EEARH`.
- Read/write/erase.
- Ready interrupt.
- Timing write.

Test:

- Write/read byte.
- Address wrap.
- Interrupt.
- Firmware `EEPROM.read/write`.

---

## 12. Watchdog

Fitur:

- `WDTCSR`.
- Prescaler.
- System reset mode.
- Interrupt mode.
- `WDR` reset counter.

Test:

- Timeout reset.
- Interrupt.
- `WDR` mencegah reset.

---

## 13. Sleep Modes

Fitur:

- Idle, ADC noise reduction, power-down, power-save, standby, extended standby.
- `SMCR`.
- Wake source: interrupt, watchdog, pin change.
- Clock gating.

Test:

- Masuk/tidur.
- Wake dari interrupt.
- Konsumsi daya tidak dimodelkan awal.

---

## 14. Fuses dan Clock

Fitur:

- Low/high/extended fuse.
- Clock source: internal RC, external crystal, external clock.
- CKDIV8.
- Boot reset vector.
- Brown-out level.

Test:

- Clock 16 MHz.
- Prescaler.
- Reset vector.
- Fuse default Arduino Uno.

---

## 15. Power-On Reset dan Brown-Out

Target akhir:

- POR circuit.
- Brown-out detector.
- Reset filtering.
- Reset tree.
- Watchdog reset.
- Power ramp.

Tahap awal:

- Reset ideal.
- VDD ideal 5 V.
- POR/brownout ditambahkan setelah peripheral inti stabil.

---

## 16. Integrasi dengan Solver Analog

- Pin digital memiliki model Thevenin.
- Net dengan komponen analog dipromosikan menjadi net analog.
- ADC membaca tegangan node analog.
- Comparator membandingkan node analog.
- Timer/PWM menghasilkan waveform digital; jika terhubung ke RC, menjadi breakpoint analog.
- Konversi waktu harus konsisten dengan global time.

---

## 17. Urutan Implementasi

1. GPIO + pin model.
2. Timer0.
3. Timer1.
4. Timer2.
5. UART TX.
6. UART RX.
7. Interrupt integration.
8. SPI.
9. I2C.
10. ADC.
11. Comparator.
12. EEPROM.
13. Watchdog.
14. Sleep.
15. Fuses.
16. POR/brownout.

---

## 18. Risiko Peripheral

| Risiko | Dampak | Mitigasi |
|---|---|---|
| Terlalu banyak peripheral sekaligus | Tidak ada yang selesai | Urutan bertahap |
| Timing timer salah | `millis()`/PWM salah | Test periodik ketat |
| UART baud mismatch | Output rusak | Test baud populer |
| ADC butuh solver analog | Ketergantungan besar | Model behavioral dulu, analog kemudian |
| Interrupt race | Firmware hang | Test interrupt intensif |
| Structural UART besar | Waktu simulasi | Optimasi sel dan flatten |

---

## 19. Keputusan Terbuka

| ID | Pertanyaan |
|---|---|
| P01 | Apakah UART structural penuh di milestone pertama? |
| P02 | Detail timing flash wait state |
| P03 | Apakah Timer2 async diimplementasikan? |
| P04 | Apakah ADC memakai solver analog di M5? |
| P05 | Model pin capacitance awal |
| P06 | Apakah sleep mode menghentikan clock internal? |
| P07 | Apakah brown-out masuk M5 atau M8? |
| P08 | Dukungan input capture |
