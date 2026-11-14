# 37 - Transmission Line

Dokumen ini menjelaskan model lossless transmission line untuk transient.

---

## 1. Status

P10-11 diimplementasikan untuk transient dengan langkah waktu konstan:

- Empat pin: `p1+`, `p1-`, `p2+`, `p2-`.
- Delay diskrit.
- Terminasi dan refleksi.
- AC dan DC belum didukung.

---

## 2. Parameter

| Parameter | Default | Arti |
|---|---|---|
| `z0` | `50 Ohm` | characteristic impedance |
| `delay` | `1e-9 s` | one-way delay |

Delay dibulatkan ke atas menjadi jumlah langkah:

```
steps = ceil(delay / dt)
```

Delay efektif menjadi `steps * dt`.

---

## 3. Companion Model

Dengan `Z0` dan state port pada `t - delay`, model memakai:

```
i1(t) = (v1(t) - v2(t-delay) - Z0 * i2(t-delay)) / Z0
i2(t) = (v2(t) - v1(t-delay) - Z0 * i1(t-delay)) / Z0
```

Setiap port distamp sebagai:

- Conductance `1 / Z0` antara pin positif dan negatif.
- Current source yang menyuntikkan nilai ekuivalen.

History menyimpan `v1`, `i1`, `v2`, dan `i2` setiap langkah setelah
konvergensi. History awal bernilai nol.

---

## 4. API

Model dibuat melalui `DeviceKind::TransmissionLine` dengan empat pin dan
parameter `z0` serta `delay`.

Syarat:

- Analisis harus transient dengan `dt` konstan.
- `dt` yang berubah menghasilkan `SimError::UnsupportedDevice`.
- `delay` harus positif.
- Jumlah langkah dibatasi satu juta.

---

## 5. Test

- Source `5 V` melalui resistor `50 Ohm` ke line `50 Ohm` dengan
  terminasi `50 Ohm`.
- Output tetap nol sebelum delay.
- Setelah delay, output menjadi `2.5 V`.
- AC menolak transmission line.

Total test `sim-analysis`: 108.
Total workspace: 717.

---

## 6. Batas

- Tidak ada loss.
- Tidak ada dispersi atau skin effect.
- Delay terkuantisasi ke langkah waktu.
- Tidak ada AC atau DC operating point.
- Tidak ada adaptive time step.
- Tidak ada crosstalk antar line.
