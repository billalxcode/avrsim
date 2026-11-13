# 41 - Pulse Source

Dokumen ini menjelaskan sumber tegangan pulsa untuk transient switching.

---

## 1. Status

Sumber pulsa diimplementasikan sebagai `DeviceKind::PulseVoltageSource`.

Fitur:

- Nilai awal dan nilai pulsa.
- Delay.
- Rise dan fall time.
- Width.
- Period.
- Pengulangan periodik.

Belum ada pulse current source atau PWL source.

---

## 2. Parameter

| Parameter | Default | Arti |
|---|---|---|
| `v1` | `0 V` | level awal |
| `v2` | `5 V` | level pulsa |
| `delay` | `0 s` | waktu sebelum pulsa |
| `rise` | `1 ns` | waktu naik |
| `fall` | `1 ns` | waktu turun |
| `width` | `1 us` | durasi level tinggi |
| `period` | `2 us` | periode pengulangan |

Semua parameter waktu harus non-negatif. `period` harus positif.

---

## 3. Bentuk Gelombang

Untuk `t < delay`, keluaran adalah `v1`.

Setelah delay, fase dihitung dengan modulo `period`:

- Fase di bawah `rise`: interpolasi linear dari `v1` ke `v2`.
- Fase di bawah `rise + width`: `v2`.
- Fase di bawah `rise + width + fall`: interpolasi linear dari `v2` ke `v1`.
- Fase lainnya: `v1`.

Jika `rise` atau `fall` nol, transisi menjadi vertikal.

---

## 4. Waktu Simulasi

Model menyimpan waktu internal:

- DC operating point memakai `v1` pada waktu nol.
- Transient pertama memakai `t = dt`.
- `update` menambah waktu sebesar `dt` setelah konvergensi.

Waktu tidak bertambah saat iterasi Newton, sehingga nilai sumber stabil
dalam satu langkah.

---

## 5. Batas

- Tidak ada pulse current source.
- Tidak ada PWL atau ekspresi arbitrer.
- Tidak ada AC analysis untuk sumber pulsa.
- Rise dan fall yang lebih kecil dari `dt` terkuantisasi.
- Model tidak membaca waktu global dari solver.

---

## 6. Test

- Bentuk gelombang pada rise, high, fall, dan periode berikutnya.
- `update` menambah waktu.
- Transient pembagi tegangan menghasilkan `2.5 V` saat high dan `0 V`
  saat low.
- AC menolak sumber pulsa.

Total test `sim-device`: 48.
Total test `sim-analysis`: 108.
Total workspace: 613.
