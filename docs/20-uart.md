# 20 - UART

Dokumen ini menjelaskan model USART behavioral dan integrasinya ke CPU.

---

## 1. Transmitter

- Frame 10 bit: start 0, 8 data LSB-first, stop 1.
- Baud divider: `(UBRR + 1) * 16` cycle per bit.
- `write_tx` hanya menerima data saat transmitter enable dan idle.
- UDRE turun saat data dimuat dan naik lagi setelah frame selesai.
- TXC di-set setelah frame selesai.
- `tx_line` mengikuti level start, data, dan stop.
- UART maju mengikuti cycle instruksi CPU.

---

## 2. Receiver

- `receive` memasukkan byte langsung dan men-set RXC.
- `receive_bit` menggeser bit serial masuk.
- Start bit rendah memulai frame.
- Delapan bit data dikumpulkan LSB-first.
- Setelah delapan bit, RXC di-set.
- `read_rx` dan `take_rx_ready` membersihkan RXC.

---

## 3. Interrupt

CPU menyediakan `poll_uart(mask)`:

- Bit 0: RX complete, vektor 18.
- Bit 1: TX complete, vektor 20.
- Bit 2: UDRE, vektor 19.

Mask adalah penyederhanaan dari bit enable UCSR0B.

---

## 4. Integrasi CPU

- UART dimiliki oleh CPU dan maju setiap instruksi.
- UDR0, UCSR0A, UCSR0B, UCSR0C, UBRR0L, dan UBRR0H dipetakan ke I/O
  dan data space.
- TX byte diteruskan ke virtual serial port.
- RX dari virtual serial masuk ke antrean dan diteruskan ke UART.
- Interrupt UART dipicu otomatis berdasarkan bit enable UCSR0B.

---

## 5. Test

- Transmitter disabled mengabaikan write.
- UCSR0B bit TX enable mengaktifkan transmitter.
- Frame membutuhkan sepuluh bit time.
- TXC dan UDRE benar.
- Baud divider mengubah durasi frame.
- TX line mengikuti bit frame.
- RX byte dan flag.
- RX shift delapan bit.
- UART transmit dari cycle CPU.
- Frame TX didekode kembali dari TX line memakai cycle CPU.
- RX interrupt masuk ke vektor 18.
- Register UART menggerakkan transmitter melalui STS.
- LDS dari UDR0 membaca byte dan membersihkan RXC.
- Write ke UCSR0A membersihkan TXC.
- RX interrupt otomatis masuk ke vektor 18.
- Smoke test virtual serial menerima byte UART.
- Smoke test echo mengirim dan menerima byte RX.

Total test `sim-avr`: 210.

---

## 6. Batas

- RX timing belum mengikuti baud divider; byte masuk langsung ke antrean.
- Parity, stop bit ganda, dan mode sinkron belum didukung.
- TX dan UDRE interrupt sudah meminta vektor tetapi belum diuji end-to-end.
- Belum ada loopback UART board dua arah.
