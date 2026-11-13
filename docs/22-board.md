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

## 3. Intel HEX dan ELF

`HexImage` memuat record Intel HEX.

- Mendukung data record tipe 00.
- Mendukung EOF tipe 01.
- Mendukung extended linear address tipe 04.
- Validasi panjang record.
- Validasi checksum.
- Alamat di luar image diabaikan.

`ElfImage` memuat ELF32 little-endian untuk AVR.

- Program header `PT_LOAD`.
- Segment flash.
- Symbol table dan string table.
- `program_words()` untuk program AVR.
- `to_hex()` untuk konversi flash ke Intel HEX.

---

## 4. CLI dan trace

- `embed-emu run <file.hex> [steps]` memuat Intel HEX dan menjalankan CPU.
- `embed-emu monitor <hex words...>` menjalankan monitor interaktif.
- `embed-emu disasm <hex words...>` menampilkan disassembly.
- `embed-emu bench [iterations] [nodes]` menjalankan benchmark.
- `embed-emu serial [--link path] [--socket path]` membuka virtual serial
  port STK500 melalui `socat`.
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
- ELF32 parse, simbol, program words, dan konversi ke HEX.
- CLI memuat Intel HEX dan menjalankan CPU.
- CSV board, CSV generik, dan VCD digital/analog.

Total test `sim-board`: 23.
Total test `sim-cli`: 26.
Total workspace: 580.

---

## 6. Batas

- Board belum memakai netlist analog penuh.
- LED hanya memakai arus dan forward voltage.
- Belum ada debounce tombol.
- Firmware hasil kompilasi nyata belum dijalankan di CI.
- Belum ada GUI atau waveform viewer.
- Belum ada writer biner atau kompresi.
