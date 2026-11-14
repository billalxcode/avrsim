# 49 - CCVS

Dokumen ini menjelaskan current-controlled voltage source.

---

## 1. Status

P3-32 diimplementasikan.

CCVS memakai dua branch:

- Sense branch 0 V pada port kontrol.
- Output branch untuk sumber tegangan.

---

## 2. Pin dan Parameter

Pin:

```
output+, output-, control+, control-
```

Parameter:

| Parameter | Default | Arti |
|---|---|---|
| `rm` | `1000 Ohm` | transresistance |

Persamaan:

```
Vout = rm * Icontrol
```

---

## 3. Branch

Sense branch mengukur arus kontrol. Output branch menetapkan tegangan
output. Alokasi branch:

- Satu slot untuk sense.
- Satu slot untuk output.

Pasangan branch diambil dari indeks sense dan `sense + 1`.

---

## 4. Test

Kontrol `1 V` melalui resistor `1 kOhm` menghasilkan arus kontrol `1 mA`.
Dengan `rm = 1000 Ohm`, output menjadi `1 V`.

Total test `sim-analysis`: 115 (108 unit + 7 integration).
Total workspace: 747.

---

## 5. Batas

- CCVS belum didukung AC analysis.
- Belum ada bandwidth atau slew rate.
- Belum ada breakdown.
- Belum ada kalibrasi parameter.
- Alokasi dua branch adalah konvensi internal khusus CCVS.
