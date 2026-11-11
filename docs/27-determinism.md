# 27 - Determinisme

Dokumen ini mencatat uji determinisme solver.

---

## 1. Cakupan

- DC operating point dijalankan dua kali pada circuit yang sama.
- Transient dijalankan dua kali dengan konfigurasi yang sama.
- DC sweep dijalankan dua kali.
- Perbandingan voltage memakai representasi bit `f64`.
- Waktu sampel dibandingkan dalam pikodetik.

---

## 2. Hasil Linux

Test determinisme Linux lulus:

- DC operating point.
- Transient 100 langkah.
- DC sweep.

Tidak ada HashMap atau wall-clock yang mempengaruhi urutan solver.

---

## 3. Batas

- macOS dan Windows belum diuji.
- Determinisme lintas arsitektur floating-point belum diuji.
- Parallel solver belum ada.
- Test memakai circuit RC kecil.
- Belum ada pengujian pada matriks besar atau device nonlinier kompleks.
