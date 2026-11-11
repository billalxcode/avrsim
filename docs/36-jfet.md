# 36 - JFET

Dokumen ini menjelaskan model junction field-effect transistor
Shichman-Hodges.

---

## 1. Status

P10-10 diimplementasikan untuk N-JFET dan P-JFET pada analisis DC dan
transient.

Model mencakup:

- Cutoff, linear, dan saturation.
- Channel-length modulation.
- Transformasi tanda untuk P-JFET.

Belum ada gate current, kapasitansi, atau breakdown.

---

## 2. Pin dan Parameter

Urutan pin:

```
drain, gate, source
```

Parameter:

| Parameter | Default | Arti |
|---|---|---|
| `vto` | `-2 V` untuk N, `+2 V` untuk P | threshold |
| `beta` | `1e-3 A/V^2` | transconductance |
| `lambda` | `0` | channel-length modulation |

Gate dianggap ideal tanpa arus.

---

## 3. Persamaan

Untuk N-JFET:

```
overdrive = vgs - vto
```

Untuk P-JFET, tegangan dan arus ditransformasi dengan tanda `-1`:

```
vgs_eff = -(vg - vs)
vds_eff = -(vd - vs)
Id_actual = -Id_eff
```

Arus channel:

```
overdrive <= 0: Id = 0
vds < overdrive: Id = beta * (2 * overdrive * vds - vds^2) * (1 + lambda * vds)
vds >= overdrive: Id = beta * overdrive^2 * (1 + lambda * vds)
```

---

## 4. Linearisasi

JFET memakai finite difference dengan langkah `1e-6 V` untuk `gm`, `gds`,
dan `gms`. Helper linearisasi tiga terminal dipakai bersama dengan MOSFET.

---

## 5. Test

Device:

- Arus saturasi N-JFET sesuai rumus.
- N-JFET cutoff di bawah threshold.
- P-JFET menghasilkan arus negatif.
- Bias gate negatif menurunkan arus N-JFET.

Analisis:

- Common source N-JFET konvergen.
- Bias gate negatif menaikkan tegangan drain.
- Common source P-JFET konvergen.
- AC menolak JFET sebagai device nonlinier.

Total test `sim-analysis`: 64.
Total test `sim-device`: 31.
Total workspace: 427.

---

## 6. Batas

- Tidak ada arus gate.
- Tidak ada kapasitansi gate-source atau gate-drain.
- Tidak ada temperature dependence.
- Tidak ada breakdown.
- Linearisasi numerik, bukan analitik.
- Belum ada noise JFET.
- Belum ada kalibrasi datasheet.
