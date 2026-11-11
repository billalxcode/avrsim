# 23 - SPI, TWI, dan ADC

Dokumen ini menjelaskan model SPI, TWI, dan ADC.

---

## 1. SPI

- Behavioral master/slave 8-bit.
- Enable dan master dari SPCR.
- Clock select dan SPI2X.
- Transfer LSB-first.
- SPIF dan read data.
- MOSI, MISO, dan SCK level.
- `tick` menerima cycle CPU.
- CPU menyediakan `spi()` dan `poll_spi()` ke vektor 17.

Batas:

- CPHA disimpan tetapi belum diinterpretasikan.
- Clock SPR=3 dibatasi 64 cycle, bukan 128.
- WCOL tidak dimodelkan.

---

## 2. TWI

- Behavioral master I2C.
- TWEN, TWSTA, TWSTO, dan TWINT.
- Start, stop, write byte, read byte.
- Status START, MT_SLA_ACK, MT_DATA_ACK, dan MR_DATA_NACK.
- Timer transaksi dari TWBR dan prescaler.
- CPU menyediakan `twi()` dan `poll_twi()` ke vektor 24.

Batas:

- Alamat slave diabaikan.
- ACK/NACK fixed.
- Kontensi bus tidak dimodelkan.

---

## 3. ADC

- Behavioral 10-bit ADC.
- ADMUX reference dan channel.
- ADCSRA enable, start, prescaler, interrupt enable, dan flag.
- Setiap channel dapat diberi input 10-bit.
- Konversi 13 x divider.
- ADCL dan ADCH.
- CPU menyediakan `adc()` dan `poll_adc()` ke vektor 21.

Batas:

- Belum memakai tegangan dan reference scaling.
- Free-running mode belum dimodelkan.
- Interrupt baru berupa polling method.

---

## 4. Integrasi CPU

SPI, TWI, dan ADC maju mengikuti cycle instruksi CPU.

Test CPU membuktikan:

- Transfer SPI selesai dan SPIF terbaca.
- Konversi ADC selesai dan ADCL/ADCH benar.
- Start TWI selesai dan TWINT terbaca.

---

## 5. Test

Total test `sim-avr` mengikuti `docs/13-analysis.md`.

---

## 6. Batas

- Register SPI, TWI, dan ADC belum dipetakan ke instruksi IN/OUT atau LDS/STS.
- Interrupt memakai polling method.
- Belum ada model elektrik pin.
- SPI slave eksternal belum berupa rangkaian.
- TWI slave belum dimodelkan.
- ADC belum memakai referensi tegangan.
