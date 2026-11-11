# 35 - BJT

Dokumen ini menjelaskan model bipolar junction transistor Ebers-Moll.

---

## 1. Status

P10-09 diimplementasikan untuk NPN dan PNP pada analisis DC dan transient:

- Ebers-Moll DC.
- Linearisasi analitik.
- Dukungan suhu melalui thermal voltage.

Belum ada model charge storage, Early effect, atau breakdown.

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

## 5. Test

Device:

- Arus tiga terminal berjumlah nol.
- Rasio `Ic / Ib` sama dengan `bf` pada forward active.
- PNP menghasilkan arus collector negatif.
- Suhu lebih tinggi menurunkan arus pada `vbe` tetap karena `is` belum
  bergantung suhu.

Analisis:

- Common emitter NPN konvergen.
- `bf` lebih tinggi menurunkan tegangan collector.
- Common emitter PNP konvergen.
- AC menolak BJT sebagai device nonlinier.

Total test `sim-analysis`: 64.
Total test `sim-device`: 31.
Total workspace: 427.

---

## 6. Batas

- Tidak ada Early effect.
- Tidak ada kapasitansi junction atau diffusion.
- Tidak ada breakdown.
- Tidak ada temperature-dependent `is`.
- Beta dianggap konstan.
- Belum ada noise BJT.
- Belum ada kalibrasi datasheet.
