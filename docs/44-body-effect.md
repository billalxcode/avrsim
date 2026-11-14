# 44 - MOSFET Body Effect

Dokumen ini menjelaskan body effect dan transconductance body `gmb`.

---

## 1. Status

P3-02 dan P3-11 diimplementasikan:

- Threshold voltage bergantung pada body-source bias.
- Turunan `gmb`.
- MOSFET 4-pin.
- Kompatibilitas MOSFET 3-pin.

Belum ada body diode, leakage body, atau kapasitansi body.

---

## 2. Pin

MOSFET menerima dua bentuk:

- 3 pin: `gate, drain, source`. Body diikat ke source.
- 4 pin: `gate, drain, source, body`.

Rangkaian 3-pin yang ada tidak berubah karena body effect hanya aktif jika
`gamma` bukan nol.

---

## 3. Parameter

| Parameter | Default | Arti |
|---|---|---|
| `gamma` | `0` | body effect coefficient |
| `phi` | `0.7 V` | surface potential |

`gamma = 0` menghasilkan threshold konstan seperti model sebelumnya.

---

## 4. Persamaan

Untuk NMOS:

```
Vbs_eff = Vb - Vs
Vth = Vto + gamma * (sqrt(phi - Vbs_eff) - sqrt(phi))
```

`phi - Vbs_eff` dibatasi minimal nol.

Untuk PMOS, tegangan ditransformasi dengan tanda yang sama seperti arus
channel:

```
Vbs_eff = -(Vb - Vs)
```

Threshold yang lebih tinggi menurunkan overdrive dan arus drain.

---

## 5. gmb

`gmb` dihitung dengan finite difference:

```
gmb = dId / dVb
```

Nilai ini distamp pada kolom body untuk baris drain dan source:

- Drain: `+gmb`.
- Source: `-gmb`.
- Body tidak menerima arus DC.

MOSFET 3-pin memakai body sama dengan source dan `gmb = 0` secara efektif.

---

## 6. Test

Device:

- Body bias reverse menaikkan threshold.
- Body bias reverse menurunkan arus drain.
- `gmb` positif pada reverse body bias.
- Seluruh test MOSFET 3-pin lama tetap lulus.

Analisis:

- Common source NMOS 4-pin dengan body `-2 V` menghasilkan tegangan drain
  lebih tinggi daripada body `0 V`.

Total test `sim-device`: 48.
Total test `sim-analysis`: 110 (108 unit + 2 integration).
Total workspace: 731.

---

## 7. Batas

- Tidak ada body diode.
- Tidak ada body leakage.
- Tidak ada kapasitansi junction body.
- `gamma` dan `phi` belum dikalibrasi datasheet.
- Belum ada model body effect untuk JFET.
- Belum ada Meyer capacitance.
