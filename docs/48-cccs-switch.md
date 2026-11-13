# 48 - CCCS dan Switch

Dokumen ini menjelaskan current-controlled current source dan switch
terkendali tegangan.

---

## 1. Status

P3-33 dan P3-34 diimplementasikan:

- CCCS dengan sense branch.
- Voltage-controlled switch dengan transisi halus.

Belum ada CCVS.

---

## 2. CCCS

Pin:

```
output+, output-, control+, control-
```

Parameter:

| Parameter | Default | Arti |
|---|---|---|
| `gain` | `1` | rasio arus |

CCCS memakai branch 0 V pada port kontrol untuk mengukur arus kontrol.
Arus output:

```
Iout = gain * Icontrol
```

Branch kontrol distamp sebagai voltage source 0 V. Kontribusi output masuk
ke kolom branch pada baris output. Gain positif menyuntikkan arus ke
`output+`.

---

## 3. Switch

Pin:

```
output+, output-, control+, control-
```

Parameter:

| Parameter | Default | Arti |
|---|---|---|
| `ron` | `1 Ohm` | resistansi on |
| `roff` | `1e9 Ohm` | resistansi off |
| `vth` | `2.5 V` | threshold kontrol |
| `width` | `0.1 V` | lebar transisi |

Konduktansi:

```
x = clamp((Vcontrol - vth) / width, -40, 40)
f = 1 / (1 + exp(-x))
g = 1/roff + (1/ron - 1/roff) * f
```

Transisi halus menghindari diskontinuitas Newton. Switch tidak memiliki
state histeresis.

---

## 4. Test

- CCCS dengan arus kontrol `1 mA` dan gain `2` menghasilkan `2 mA` ke
  load `1 kOhm`, sehingga output `2 V`.
- Switch on dengan `ron = 1 Ohm` menarik output di bawah `0.1 V`.
- Switch off dengan `roff = 1e9 Ohm` mempertahankan output di atas `4.9 V`.

Total test `sim-analysis`: 108.
Total workspace: 557.

---

## 5. Batas

- CCVS belum ada.
- Switch belum memiliki hysteresis.
- Switch belum memiliki kapasitansi atau delay.
- CCCS dan switch belum didukung AC analysis.
- Belum ada kalibrasi parameter.
