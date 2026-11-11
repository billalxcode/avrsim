# 22 - Board dan Firmware

Dokumen ini menjelaskan model board dasar dan loader Intel HEX.

---

## 1. Komponen

- `Resistor` menghitung arus dengan hukum Ohm.
- `Led` memiliki forward voltage dan status on saat arus positif.
- `Button` menyimpan status pressed.
- `Board` menghubungkan PB5 ke resistor 220 ohm dan LED 2.0 V.

---

## 2. UART Host

`UartHost` menyediakan dua antrean:

- Host ke device.
- Device ke host.

API:

```rust
pub fn host_send(&mut self, value: u8);
pub fn device_receive(&mut self) -> Option<u8>;
pub fn device_send(&mut self, value: u8);
pub fn host_receive(&mut self) -> Option<u8>;
```

---

## 3. Intel HEX

`HexImage` memuat record Intel HEX.

- Mendukung data record tipe 00.
- Mendukung EOF tipe 01.
- Mendukung extended linear address tipe 04.
- Validasi panjang record.
- Validasi checksum.
- Alamat di luar image diabaikan.

---

## 4. CLI dan Trace

- `embed-emu run <file.hex> [steps]` memuat Intel HEX dan menjalankan CPU.
- `csv_header` dan `csv_line` menghasilkan trace CSV board.
- `CsvWriter` menulis CSV generik dengan kolom tetap.
- `VcdWriter` menghasilkan trace VCD digital untuk PB5 dan LED.
- `AnalogVcdWriter` menghasilkan trace VCD `real` untuk sinyal analog.
- Format timebase VCD memakai 1 ns.

---

## 5. Test

- Resistor, LED, dan button.
- PB5 menyalakan LED.
- UART host dua arah.
- Blink program menggerakkan LED board.
- Intel HEX valid, checksum salah, dan extended address.
- CLI memuat Intel HEX.
- CSV board, CSV generik, dan VCD digital/analog.
- CLI memuat Intel HEX dan menjalankan CPU.

Total test `sim-board`: 17.
Total test `sim-cli`: 8.

---

## 6. Batas

- Board belum memakai netlist analog penuh.
- LED model hanya arus dan forward voltage.
- Belum ada tombol debounce.
- Belum ada loader ELF dan simbol.
- Belum ada `avr-gcc`, konversi ELF ke HEX, atau firmware UART hello.
- Belum ada debounce tombol.
- Belum ada writer biner atau kompresi.
