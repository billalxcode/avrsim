# 47 - Controlled Sources

Dokumen ini menjelaskan VCVS dan VCCS linear.

---

## 1. Status

P3-30 dan P3-31 diimplementasikan:

- Voltage-controlled voltage source.
- Voltage-controlled current source.

Belum ada CCVS, CCCS, atau voltage-controlled switch.

---

## 2. Pin

Urutan pin untuk kedua device:

```
output+, output-, control+, control-
```

---

## 3. VCVS

Parameter:

| Parameter | Default | Arti |
|---|---|---|
| `gain` | `1` | rasio tegangan |

Persamaan:

```
Vout = gain * (Vcontrol+ - Vcontrol-)
```

VCVS memakai branch MNA. Baris branch berisi koefisien output dan kontrol
dengan tanda yang sesuai.

---

## 4. VCCS

Parameter:

| Parameter | Default | Arti |
|---|---|---|
| `gm` | `1e-3 S` | transconductance |

Persamaan:

```
Iout = gm * (Vcontrol+ - Vcontrol-)
```

Arus positif mengalir dari `output-` ke `output+` di dalam source, sehingga
menyuntikkan arus ke node `output+`.

VCCS tidak memakai branch. Kontribusinya langsung masuk ke matriks pada
baris output dan kolom kontrol.

---

## 5. Test

- VCVS dengan kontrol `1 V` dan gain `2` menghasilkan output `2 V`.
- VCCS dengan kontrol `1 V`, `gm = 1 mS`, dan load `1 kOhm` menghasilkan
  output `1 V`.

Total test `sim-analysis`: 110 (108 unit + 2 integration).
Total workspace: 728.

---

## 6. Batas

- VCVS dan VCCS belum didukung AC analysis.
- Belum ada CCVS dan CCCS.
- Belum ada switch terkendali tegangan.
- Belum ada parameter bandwidth atau slew rate.
- Belum ada kalibrasi terhadap datasheet.
