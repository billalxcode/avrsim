# 52 - Lisensi dan Notices

Dokumen ini mencatat status lisensi kode, model, dan data.

---

## 1. Kode

- Lisensi proyek: MIT, lihat `LICENSE`.
- Semua crate workspace memakai lisensi yang sama.
- Tidak ada crate Rust eksternal.
- Tidak ada kode sumber pihak ketiga yang disalin.

---

## 2. Model Device

Persamaan device mengikuti model publik yang umum:

- Shockley diode.
- Shichman-Hodges MOSFET.
- Ebers-Moll BJT.
- Meyer capacitance.
- Lossless transmission line.
- Controlled sources.

Implementasi ditulis ulang dari persamaan, bukan salinan kode simulator
lain.

---

## 3. Data Datasheet

Kalibrasi pin memakai DC characteristics ATmega328P:

- VOH 4.1 V pada 20 mA.
- VOL 0.8 V pada 20 mA.
- Pull-up 20-50 kOhm.

Sumber:

<https://ww1.microchip.com/downloads/en/DeviceDoc/Atmel-7810-Automotive-Microcontrollers-ATmega328P_Datasheet.pdf>

Repository hanya menyimpan nilai turunan dan prosedur kalibrasi, bukan
salinan tabel datasheet.

---

## 4. CI

Workflow memakai `actions/checkout@v4` untuk checkout repository. Tidak ada
dependency Rust eksternal di CI.

---

## 5. Batas

- Dokumen ini bukan nasihat hukum.
- Belum ada audit lisensi untuk firmware contoh.
- Belum ada NOTICE biner release.
- Belum ada verifikasi lisensi datasheet untuk redistribusi penuh.
