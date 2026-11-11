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
- Akses konfigurasi melalui `Cpu::uart()`.
- Belum ada pemetaan UDR0 dan UCSR0 ke instruksi IN/OUT atau LDS/STS.

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
- RX interrupt masuk ke vektor 18.

Total test `sim-avr`: 140.

---

## 6. Batas

- UART belum dipetakan ke register I/O CPU.
- RX timing belum mengikuti baud divider.
- Parity, stop bit ganda, dan mode sinkron belum didukung.
- TX dan UDRE interrupt sudah meminta vektor tetapi belum diuji end-to-end.
- Belum ada loopback UART board.
