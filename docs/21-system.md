# 21 - System Peripherals

Dokumen ini menjelaskan model EEPROM, watchdog, fuse, clock prescaler,
analog comparator, POR, dan brown-out.

---

## 1. EEPROM

- Ukuran 1024 byte.
- Nilai awal 0xFF.
- Alamat dibungkus modulo ukuran.
- Operasi read, write, dan erase.

---

## 2. Watchdog

- Enable dengan timeout dalam cycle.
- Kick mereset counter.
- Tick mengembalikan true saat timeout.
- Disable menghentikan timeout.

---

## 3. Fuse

- Struct `Fuses` memuat low, high, dan extended.
- Profil Arduino Uno: 0xFF, 0xDE, 0xFD.

---

## 4. Clock Prescaler

- CKDIV8 menghasilkan divider 8.
- Divider 1 saat CKDIV8 mati.
- `apply` membagi jumlah cycle.

---

## 5. Analog Comparator

- Enable flag.
- Input positif dan negatif 16-bit.
- Output aktif saat enable dan positif lebih besar dari negatif.

---

## 6. POR dan Brown-out

- `PowerState` menyimpan status POR dan brown-out.
- `reset_required` true jika salah satu aktif.
- `clear` mematikan keduanya.

---

## 7. Test

- EEPROM read, write, erase.
- Watchdog timeout, kick, disable.
- Profil fuse Uno.
- Clock prescaler.
- Comparator.
- Power reset state.

Total test `sim-avr` mengikuti total pada `docs/13-analysis.md`.

---

## 8. Batas

- Model sistem belum dipetakan ke register I/O CPU.
- EEPROM belum memiliki timing write.
- Watchdog belum memicu reset CPU.
- Fuse belum mempengaruhi clock atau boot.
- POR dan brown-out belum menghentikan CPU.
- Comparator belum terhubung ke pin analog.
