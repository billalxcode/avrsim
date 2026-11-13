# 46 - Diode Nonideal

Dokumen ini menjelaskan series resistance dan junction capacitance diode.

---

## 1. Status

P3-26 dan P3-27 diimplementasikan:

- Series resistance `rs`.
- Junction capacitance `cj0`.
- Built-in potential `vj`.
- Grading coefficient `m`.

Belum ada breakdown, diffusion capacitance, atau temperature dependence
kapasitansi.

---

## 2. Parameter

| Parameter | Default | Arti |
|---|---|---|
| `rs` | `0 Ohm` | series resistance |
| `cj0` | `0 F` | capacitance pada bias nol |
| `vj` | `0.7 V` | built-in potential |
| `m` | `0.5` | grading coefficient |

Default nol menjaga perilaku model sebelumnya.

---

## 3. Series Resistance

Dengan tegangan terminal `V` dan tegangan junction `Vj`:

```
V = Vj + rs * I(Vj)
```

`Vj` dicari dengan bisection pada rentang yang dibatasi
`MAX_FORWARD_VOLTAGE`. Konduktansi terminal:

```
g = gj / (1 + rs * gj)
```

Arus terminal sama dengan arus junction. Linearisasi memakai `g` dan arus
tersebut.

Jika `rs = 0`, jalur ini dilewati dan model berperilaku seperti sebelumnya.

---

## 4. Junction Capacitance

```
Cj = cj0 / (1 - Vj / vj)^m
```

Rasio dibatasi minimal `0.1`. Kapasitansi distamp dengan backward Euler
antara anoda dan katode.

State `cap_voltage_prev` diperbarui setelah konvergensi. Pada DC,
kapasitansi bersifat open.

Jika `rs > 0`, kapasitansi tetap distamp pada terminal, bukan pada node
junction internal. Ini adalah pendekatan yang didokumentasikan.

---

## 5. Test

Device:

- Series resistance menurunkan tegangan junction untuk tegangan terminal
  yang sama.
- Kapasitansi junction sesuai rumus pada bias nol, forward, dan reverse.

Analisis:

- Diode clamp dengan `rs = 100 Ohm` menghasilkan tegangan output lebih
  tinggi.
- Transient reverse bias dengan `cj0 = 5 nF` memperlambat perubahan
  tegangan output.

Total test `sim-device`: 48.
Total test `sim-analysis`: 108.
Total workspace: 644.

---

## 6. Batas

- Tidak ada breakdown voltage.
- Tidak ada diffusion capacitance.
- Tidak ada temperature dependence `cj0`.
- Tidak ada node junction internal.
- Kapasitansi pada `rs > 0` memakai pendekatan terminal.
- Bisection memakai 50 iterasi per stamp.
