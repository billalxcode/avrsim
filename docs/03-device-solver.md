# 03 — Device Model dan Solver

Dokumen ini menjelaskan model device, formulasi MNA, solver nonlinear, analisis, dan strategi kalibrasi.

---

## 1. Ruang Lingkup

### Target akhir
Pustaka SPICE lengkap dan seluruh analisis SPICE.

### Milestone 1
- MOSFET Shichman-Hodges Level 1
- Resistor
- Kapasitor
- Induktor
- Sumber tegangan independen
- Sumber arus independen
- DC operating point
- DC sweep
- Transient Backward Euler
- Dense LU
- Newton-Raphson dengan seluruh convergence aids

### Target akhir device
- Dioda
- BJT (Ebers-Moll / Gummel-Poon)
- JFET
- Voltage-controlled switch
- Controlled source: VCVS, VCCS, CCVS, CCCS
- Transmission line
- Model suhu dan corner

---

## 2. MOSFET Shichman-Hodges Level 1

### 2.1 Parameter

| Parameter | Simbol | Arti |
|---|---|---|
| Threshold voltage | `VTO` | Tegangan threshold pada Vbs = 0 |
| Transconductance parameter | `KP` | μ × Cox |
| Width | `W` | Lebar kanal |
| Length | `L` | Panjang kanal |
| Channel-length modulation | `LAMBDA` | Efek Early |
| Body effect | `GAMMA` | Koefisien body effect |
| Surface potential | `PHI` | 2φf |
| Gate-source overlap | `CGSO` | Kapasitansi overlap |
| Gate-drain overlap | `CGDO` | Kapasitansi overlap |
| Gate-bulk overlap | `CGBO` | Kapasitansi overlap |
| Bulk junction cap | `CBD`, `CBS` | Kapasitansi junction |

### 2.2 Threshold dengan Body Effect

Untuk NMOS:

```
Vth = VTO + GAMMA * (sqrt(PHI - Vbs) - sqrt(PHI))
```

Jika `PHI - Vbs < 0`, nilai di dalam akar di-clamp ke nol dan diberi peringatan model.

Untuk PMOS, tanda tegangan dibalik (`Vbs` efektif), atau implementasi memakai polaritas.

### 2.3 Arus Drain-Source (NMOS)

Definisikan:

```
Vov = Vgs - Vth
```

**Cutoff** (`Vov <= 0`):

```
Ids = 0
```

**Linear** (`Vov > 0` dan `Vds < Vov`):

```
Ids = KP * (W/L) * (Vov * Vds - Vds^2 / 2) * (1 + LAMBDA * Vds)
```

**Saturation** (`Vov > 0` dan `Vds >= Vov`):

```
Ids = (KP / 2) * (W/L) * Vov^2 * (1 + LAMBDA * Vds)
```

**PMOS:** gunakan `Vsg`, `Vsd`, `Vth` dengan tanda yang sesuai, atau implementasi polaritas terpisah. Jangan mencampur tanda.

### 2.4 Turunan untuk Newton

Untuk linear:

```
gm  = dIds/dVgs = KP * (W/L) * Vds * (1 + LAMBDA * Vds)
gds = dIds/dVds = KP * (W/L) * (Vov - Vds) * (1 + LAMBDA * Vds)
                  + KP * (W/L) * (Vov * Vds - Vds^2/2) * LAMBDA
```

Untuk saturasi:

```
gm  = KP * (W/L) * Vov * (1 + LAMBDA * Vds)
gds = (KP / 2) * (W/L) * Vov^2 * LAMBDA
```

Body transconductance `gmb` dihitung dari turunan Vth terhadap Vbs bila body effect aktif.

### 2.5 Kapasitansi Meyer

Model Meyer membagi kapasitansi oxide menjadi `Cgs`, `Cgd`, dan `Cgb` menurut region:

| Region | Cgs | Cgd | Cgb |
|---|---|---|---|
| Cutoff | 0 | 0 | Cox |
| Linear | Cox/2 | Cox/2 | 0 |
| Saturation | 2Cox/3 | 0 | 0 |

Dengan `Cox = KP / μ × W × L` (atau parameter `COX` terpisah). Tambahkan overlap:

```
Cgs_total = Cgs + CGSO * W
Cgd_total = Cgd + CGDO * W
Cgb_total = Cgb + CGBO * L
```

**Batasan:** model Meyer tidak konservasi muatan penuh. Untuk timing CMOS awal ini memadai; model berbasis muatan dapat ditambahkan nanti.

### 2.6 Batasan Model

- Tidak ada subthreshold conduction.
- Tidak ada mobility degradation.
- Tidak ada gate leakage.
- Tidak ada impact ionization.
- Tidak ada noise intrinsik.
- Tidak akurat untuk channel pendek.
- Nominal PVT saja pada tahap awal.

---

## 3. Komponen Pasif

### 3.1 Resistor

Stamping konduktansi:

```
g = 1 / R
G[a][a] += g
G[a][b] -= g
G[b][a] -= g
G[b][b] += g
```

### 3.2 Kapasitor — Backward Euler

Persamaan:

```
I = C * (V - Vprev) / dt
```

Companion model:

```
g_eq = C / dt
I_eq = g_eq * Vprev
```

Stamping:

- Tambahkan `g_eq` seperti resistor.
- Tambahkan sumber arus `I_eq` pada node yang sesuai, dengan arah tergantung definisi port.

### 3.3 Induktor — Backward Euler

Persamaan:

```
V = L * (I - Iprev) / dt
```

Companion:

- Resistansi ekivalen `R_eq = L / dt`.
- Sumber tegangan ekivalen `V_eq = -R_eq * Iprev`.

Induktor membutuhkan branch current sebagai unknown tambahan.

### 3.4 Sumber Independen

**Voltage source** antara `a` dan `b`:

```
v_a - v_b = V
```

Tambahkan branch current `j`:

```
B[a][k] = 1
B[b][k] = -1
C[k][a] = 1
C[k][b] = -1
RHS[k] = V
```

**Current source** dari `a` ke `b`:

```
RHS[a] -= I
RHS[b] += I
```

---

## 4. Dioda

### 4.1 Persamaan Shockley

```
I = Is * (exp(V / (n * Vt)) - 1)
```

Dengan `Vt = kT/q ≈ 25.85 mV` pada 300 K.

Tambahkan resistansi seri `Rs` bila diperlukan.

### 4.2 Jacobian

```
g = Is / (n * Vt) * exp(V / (n * Vt))
```

### 4.3 Limiting

Gunakan `pnjlim`:

- Batasi perubahan tegangan junction per iterasi.
- Jika perubahan terlalu besar, potong ke `Vcrit`.
- Tujuannya mencegah overflow `exp`.

### 4.4 Kapasitansi Junction

Model sederhana:

```
Cj = Cj0 / (1 - V/Vj)^m
```

Untuk `V < Vj`; untuk `V >= Vj` gunakan ekstrapolasi linear. Ini dapat ditambahkan setelah model DC stabil.

---

## 5. BJT dan JFET (Target Lanjut)

### 5.1 BJT Ebers-Moll

- Dua junction: base-emitter dan base-collector.
- Parameter: `Is`, `alphaF`, `alphaR`, `Vt`.
- Arus kolektor, basis, emitor diturunkan dari dua dioda junction.
- Kapasitansi junction dan diffusion ditambahkan untuk transient.
- Model Gummel-Poon dapat menjadi target lanjut.

### 5.2 JFET

- Model Shichman-Hodges untuk JFET.
- Parameter: `VTO`, `Beta`, `Lambda`.
- Berguna untuk rangkaian analog tertentu.

### 5.3 Controlled Source

- VCVS: sumber tegangan dikendalikan tegangan.
- VCCS: sumber arus dikendalikan tegangan.
- CCVS: sumber tegangan dikendalikan arus.
- CCCS: sumber arus dikendalikan arus.

Semua membutuhkan branch/unknown tambahan sesuai tipe.

### 5.4 Switch

- Voltage-controlled resistor.
- Resistance berubah antara `Ron` dan `Roff` berdasarkan tegangan kontrol.
- Memerlukan smoothing/limiting agar Newton stabil.

### 5.5 Transmission Line

- Target akhir.
- Model lossless atau lossy.
- Memerlukan delay dan state historis.

---

## 6. Formulasi MNA

Untuk `n` node non-ground dan `m` branch:

```
[ G  B ] [ v ]   [ i ]
[ C  D ] [ j ] = [ e ]
```

### 6.1 Stamping Umum

| Device | Kontribusi |
|---|---|
| Resistor | `G` |
| Kapasitor (BE) | `G` + RHS |
| Induktor (BE) | Branch + `G` |
| Voltage source | Branch + RHS |
| Current source | RHS |
| MOSFET | `G` dari gm/gds/gmb |
| Dioda | `G` dari g |
| BJT | `G` dari gm/gpi/go |

### 6.2 Ground

- Node 0 tidak memiliki persamaan.
- Semua stamping ke node 0 diabaikan.
- Tegangan node 0 = 0 V.

### 6.3 Solver Linear

- Dense row-major.
- Partial pivoting.
- Deteksi pivot `< 1e-18`.
- Jika singular:
  - Coba gmin stepping.
  - Coba source stepping.
  - Jika tetap gagal, kembalikan `SingularMatrix`.

---

## 7. Newton-Raphson

### 7.1 Iterasi

```
J(x_k) * Δx = -F(x_k)
x_{k+1} = x_k + Δx
```

### 7.2 Kriteria Konvergensi

```
|Δv_i| < reltol * |v_i| + vntol
|Δi_j| < reltol * |i_j| + abstol
```

Default awal:

| Parameter | Nilai |
|---|---|
| `reltol` | `1e-3` |
| `vntol` | `1e-6` V |
| `abstol` | `1e-12` A |
| `gmin` | `1e-12` S |
| `max_iter` | `100` |

Nilai ini `TBD` dan harus divalidasi terhadap ngspice.

### 7.3 Convergence Aids

Urutan fallback yang direncanakan:

1. **Voltage limiting** — batasi perubahan tegangan junction.
2. **Damping** — kurangi Δx dengan faktor α.
3. **gmin stepping** — tambahkan konduktansi kecil dari setiap node ke ground, lalu turunkan bertahap.
4. **Source stepping** — naikkan sumber dari 0 ke nilai penuh secara bertahap.
5. **Fallback terakhir** — error dengan diagnostik lengkap.

### 7.4 Diagnostik

Saat gagal:

- Iterasi terakhir.
- Node dengan residual terbesar.
- Device dengan residual terbesar.
- Nilai Δv/Δi terbesar.
- Apakah limiting/gmin/source stepping aktif.

---

## 8. Analisis

### 8.1 DC Operating Point

- Kapasitor open.
- Induktor short.
- Sumber pada nilai DC.
- Newton sampai konvergen.
- Hasil menjadi initial condition transient.

### 8.2 DC Sweep

- Sweep satu atau lebih sumber/parameter.
- Gunakan hasil titik sebelumnya sebagai initial guess (continuation).
- Output CSV.
- Untuk VTC inverter: sweep `Vin`, ukur `Vout`.

### 8.3 Transient

- Backward Euler.
- Fixed step pada milestone awal.
- Initial condition dari DC op.
- Setiap step:
  1. Update companion model kapasitor/induktor.
  2. Newton solve.
  3. Simpan hasil.
  4. Update state.
- Output VCD.

### 8.4 AC Small-Signal (Target Lanjut)

- Linearisasi di sekitar DC op.
- Matriks kompleks.
- Sweep frekuensi.
- Output magnitude/phase.

### 8.5 Noise (Target Lanjut)

- Noise source per device.
- Integrasi PSD pada output.
- Input-referred noise.

### 8.6 Distortion (Target Lanjut)

- Analisis harmonik.
- Multi-tone.
- Perlu model nonlinear yang konsisten.

### 8.7 Sensitivity (Target Lanjut)

- Turunan output terhadap parameter.
- Bisa memakai finite difference atau adjoint.

---

## 9. Kalibrasi

### 9.1 Kondisi

- Tidak ada akses lab.
- Tidak ada data silikon internal.
- Hanya datasheet dan model publik.

### 9.2 Target Kalibrasi Level Pin

| Parameter | Sumber |
|---|---|
| `VIL`, `VIH` | Datasheet ATmega328P |
| `VOL`, `VOH` | Datasheet pada arus tertentu |
| Pull-up resistance | Datasheet |
| Input leakage | Datasheet |
| Pin capacitance | Datasheet |
| Rise/fall time | Datasheet / karakterisasi |
| Drive current | Datasheet |

### 9.3 Strategi

1. Pilih parameter PTM 350 nm sebagai baseline.
2. Sesuaikan `VTO`, `KP`, `W/L`, dan `Rout` agar karakteristik pin cocok.
3. Jangan mengklaim kecocokan internal.
4. Catat semua parameter dan batasan di fidelity matrix.
5. Gunakan toleransi, bukan angka tunggal.

### 9.4 Batasan

- Model tidak akan identik dengan silikon.
- Akurasi analog terbatas pada Level 1.
- Perilaku pada suhu/corner tidak divalidasi.
- Noise dan mismatch tidak dimodelkan.

---

## 10. Test Plan

### 10.1 Unit

- MOSFET `Ids` vs perhitungan tangan untuk cutoff/linear/saturation.
- Jacobian vs finite difference.
- Resistor, kapasitor, induktor.
- Dioda IV.
- LU matriks 2×2, 3×3, singular.

### 10.2 Integrasi

- Pembagi tegangan.
- RC charge/discharge.
- Diode clamp.
- Inverter VTC.
- Ring oscillator.
- Current mirror.
- Differential pair.

### 10.3 Golden

- Bandingkan dengan ngspice.
- Toleransi per analisis.
- Simpan data golden di repo test (ukuran kecil).

### 10.4 Fuzz

- Parameter tidak valid.
- Node floating.
- Matriks singular.
- Konvergensi gagal.
- Input sweep ekstrem.

---

## 11. Performa

- Dense LU: O(n³) per solve.
- Newton: beberapa solve per titik.
- Transient: jumlah step × Newton × LU.
- Rangkaian kecil (≤ 50 node) masih nyaman.
- Rangkaian besar membutuhkan sparse dan adaptive step.
- Benchmark wajib sebelum optimasi.

---

## 12. Keputusan Terbuka

| ID | Pertanyaan |
|---|---|
| S01 | VDD test inverter: 3.3 V atau 5 V? |
| S02 | Time step transient awal |
| S03 | Toleransi VTC/transient |
| S04 | Parameter PTM persisnya |
| S05 | Apakah subthreshold ditambahkan di Level 1? |
| S06 | Model kapasitansi junction dioda kapan? |
| S07 | Threshold sparse solver |
| S08 | Detail noise analysis |
| S09 | Apakah temperature model masuk M8? |
| S10 | Format golden data |
