# 36 - JFET

Dokumen ini menjelaskan model junction field-effect transistor
Shichman-Hodges.

---

## 1. Status

P10-10 diimplementasikan untuk N-JFET dan P-JFET pada analisis DC,
transient, dan AC.

Model mencakup:

- Cutoff, linear, dan saturation.
- Channel-length modulation.
- Transformasi tanda untuk P-JFET.
- Kapasitansi gate-source dan gate-drain.

Belum ada gate current atau breakdown.

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
| `cgs` | `0 F` | gate-source capacitance |
| `cgd` | `0 F` | gate-drain capacitance |

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

## 5. Kapasitansi

JFET memakai kapasitansi small-signal konstan:

- `cgs` antara gate dan source.
- `cgd` antara gate dan drain.

Pada transient, keduanya distamp sebagai kapasitor linear. Pada AC,
keduanya menjadi admitansi `j * 2 * pi * f * C`.

---

## 6. Test

Device:

- Arus saturasi N-JFET sesuai rumus.
- N-JFET cutoff di bawah threshold.
- P-JFET menghasilkan arus negatif.
- Bias gate negatif menurunkan arus N-JFET.
- `cgs` dan `cgd` dibaca dari parameter dan default nol.

Analisis:

- Common source N-JFET konvergen.
- Bias gate negatif menaikkan tegangan drain.
- Common source P-JFET konvergen.
- AC common source memiliki gain dan rolloff saat `cgd` aktif.
- Kapasitansi JFET memperlambat kenaikan drain pada transient.

Total test `sim-analysis`: 108.
Total test `sim-device`: 48.
Total workspace: 537.

---

## 7. Batas

- Tidak ada arus gate.
- Kapasitansi `cgs` dan `cgd` konstan, belum bergantung bias.
- Tidak ada temperature dependence.
- Tidak ada breakdown.
- Linearisasi numerik, bukan analitik.
- Belum ada noise JFET.
- Belum ada kalibrasi datasheet.
