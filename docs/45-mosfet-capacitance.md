# 45 - MOSFET Capacitance

Dokumen ini menjelaskan Meyer capacitance dan overlap capacitance.

---

## 1. Status

P3-13 sampai P3-16 diimplementasikan:

- Kapasitansi Meyer cutoff.
- Kapasitansi Meyer linear.
- Kapasitansi Meyer saturation.
- Overlap capacitance gate-source dan gate-drain.

Kapasitansi hanya aktif pada transient. DC memperlakukan kapasitansi
sebagai open.

---

## 2. Parameter

| Parameter | Default | Arti |
|---|---|---|
| `cox` | `0 F` | total oxide capacitance gate-channel |
| `cgso` | `0 F` | overlap gate-source |
| `cgdo` | `0 F` | overlap gate-drain |

Default nol menjaga perilaku model sebelumnya.

---

## 3. Partisi Meyer

Dengan `overdrive = vgs - Vth`:

| Region | Cgs | Cgd | Cgb |
|---|---|---|---|
| Cutoff | `cgso` | `cgdo` | `cox` |
| Linear | `cgso + cox/2` | `cgdo + cox/2` | 0 |
| Saturation | `cgso + 2cox/3` | `cgdo` | 0 |

`Vth` memakai body effect jika `gamma` bukan nol.

Untuk PMOS, region ditentukan dari tegangan efektif dengan transformasi
tanda yang sama seperti arus channel.

---

## 4. Stamping

Setiap kapasitansi distamp dengan backward Euler:

```
G = C / dt
I = G * Vprev
```

State yang disimpan:

- `vgs_prev`.
- `vgd_prev`.
- `vgb_prev`.

State diperbarui setelah konvergensi. Sebelum langkah pertama, state diisi
dari initial solution, sehingga transient yang dimulai dari DC operating
point tetap konsisten.

---

## 5. Test

Device:

- Partisi cutoff menghasilkan `Cgb = cox`.
- Partisi linear menghasilkan `Cgs = Cgd = cox/2`.
- Partisi saturation menghasilkan `Cgs = 2cox/3`.
- Overlap ditambahkan pada semua region.

Analisis:

- MOS capacitor dengan `cox = 1 pF` dan resistor gate `1 MOhm` mengisi
  gate mendekati `3.1 V` setelah `1 us`.

Total test `sim-device`: 48.
Total test `sim-analysis`: 108.
Total workspace: 630.

---

## 6. Batas

- Meyer capacitance bukan model charge-conserving.
- Belum ada gate resistance.
- Belum ada junction capacitance drain-body dan source-body.
- Belum ada temperature dependence kapasitansi.
- Belum ada kalibrasi `cox`, `cgso`, dan `cgdo` dari datasheet.
- Kapasitansi tidak memengaruhi AC analysis karena MOSFET masih ditolak AC.
