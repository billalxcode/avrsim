# 35 - BJT

Dokumen ini menjelaskan model bipolar junction transistor Ebers-Moll.

---

## 1. Status

P10-09 diimplementasikan untuk NPN dan PNP pada analisis DC, transient, dan AC:

- Ebers-Moll DC.
- Linearisasi analitik.
- Dukungan suhu melalui thermal voltage.
- Kapasitansi junction dan diffusion.

Belum ada Early effect atau breakdown.

---

## 2. Pin dan Parameter

Urutan pin:

```
collector, base, emitter
```

Parameter:

| Parameter | Default | Arti |
|---|---|---|
| `is` | `1e-15 A` | saturation current |
| `bf` | `100` | forward beta |
| `br` | `1` | reverse beta |
| `vt` | `k*T/q` | thermal voltage |
| `temp` | `300.15 K` | suhu device |
| `cje` | `0 F` | base-emitter zero-bias capacitance |
| `vje` | `0.75 V` | base-emitter potential |
| `mje` | `0.33` | base-emitter grading |
| `cjc` | `0 F` | base-collector zero-bias capacitance |
| `vjc` | `0.75 V` | base-collector potential |
| `mjc` | `0.33` | base-collector grading |
| `tf` | `0 s` | forward transit time |
| `tr` | `0 s` | reverse transit time |

`vt` eksplisit menang atas `temp`.

---

## 3. Persamaan

Dengan:

```
alpha_f = bf / (bf + 1)
alpha_r = br / (br + 1)
IF = is * (exp(vbe / vt) - 1)
IR = is * (exp(vbc / vt) - 1)
```

Arus NPN:

```
Ic = alpha_f * IF - IR
Ib = (1 - alpha_f) * IF + (1 - alpha_r) * IR
Ie = -IF + alpha_r * IR
```

PNP memakai transformasi tanda:

```
vbe_eff = ve - vb
vbc_eff = vc - vb
I_actual = -I_eff
```

Junction dibatasi pada `0.9 V` dan eksponen maksimum `40` untuk menjaga
stabilitas numerik.

---

## 4. Linearisasi

Turunan analitik terhadap `vbe` dan `vbc` dipetakan ke turunan terhadap
tegangan node base, collector, dan emitter. Setiap terminal distamp pada
baris KCL-nya.

Dengan cara ini Newton tidak memakai finite difference untuk BJT.

---

## 5. Kapasitansi

Kapasitansi small-signal:

```
Cbe = CJE / (1 - vbe / VJE)^MJE + TF * gbe
Cbc = CJC / (1 - vbc / VJC)^MJC + TR * gbc
```

`gbe` dan `gbc` adalah conductance junction forward dan reverse.
Untuk PNP, `vbe` dan `vbc` memakai transformasi tanda.

Pada transient, `Cbe` distamp antara base-emitter dan `Cbc` antara
base-collector. Pada AC, keduanya menjadi admitansi `j * 2 * pi * f * C`.

---

## 6. Test

Device:

- Arus tiga terminal berjumlah nol.
- Rasio `Ic / Ib` sama dengan `bf` pada forward active.
- PNP menghasilkan arus collector negatif.
- Suhu lebih tinggi menurunkan arus pada `vbe` tetap karena `is` belum
  bergantung suhu.
- Kapasitansi depletion cocok dengan bias dan transformasi tanda PNP.
- `tf` menambah diffusion capacitance pada forward bias.

Analisis:

- Common emitter NPN konvergen.
- `bf` lebih tinggi menurunkan tegangan collector.
- Common emitter PNP konvergen.
- AC common emitter memiliki gain dan rolloff saat `cje`/`cjc` aktif.
- Kapasitansi BJT memperlambat charge base pada transient.

Total test `sim-analysis`: 108.
Total test `sim-device`: 48.
Total workspace: 573.

---

## 7. Batas

- Tidak ada Early effect.
- Kapasitansi memakai model sederhana, bukan charge-conserving penuh.
- Tidak ada breakdown.
- Tidak ada temperature-dependent `is`.
- Beta dianggap konstan.
- Belum ada noise BJT.
- Belum ada kalibrasi datasheet.
