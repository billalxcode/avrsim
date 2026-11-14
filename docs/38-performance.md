# 38 - Profiling dan Optimasi

Dokumen ini menjelaskan benchmark solver dan optimasi hot path.

---

## 1. Status

P10-22 dan P10-23 diimplementasikan:

- Command benchmark di CLI.
- Pengukuran DC dan transient.
- Tiga optimasi pada jalur solver.

Belum ada profiler eksternal, pengukuran release, atau analisis memori.

---

## 2. Benchmark

Command:

```
embed-emu bench [iterations] [nodes]
```

Default `iterations = 1000` dan `nodes = 10`.

Benchmark memakai rangkaian RC:

- Voltage source `5 V`.
- Resistor `1 kOhm`.
- Capacitor `1 uF`.

Setiap iterasi menjalankan:

- Satu DC operating point.
- Satu transient 100 langkah.

Benchmark kedua memakai resistor ladder dengan jumlah node yang dapat
 dipilih. Setiap node dihubungkan oleh resistor seri `1 kOhm` dan resistor
 shunt `2 kOhm` ke ground. Ladder mengukur biaya per node.

Output:

```
dc: 1000 runs, 0.037 s, 37.1 us/run, ...
transient: 1000 runs, 0.290 s, 290.4 us/run, 2.90 ns/step
ladder-10: 200 runs, 0.091 s, 454.1 us/run, 45.41 us/node
```

Angka bergantung mesin dan build.

---

## 3. Optimasi

### 3.1 Direct DC untuk Rangkaian Linear

Rangkaian tanpa device nonlinear langsung diselesaikan pada `gmin` final.
Gmin stepping hanya dipakai jika ada diode, MOSFET, BJT, JFET, atau device
nonlinear lain.

Alasan: gmin stepping dibutuhkan untuk konvergensi nonlinear, tetapi hanya
menambah biaya pada sistem linear.

### 3.2 Solve Tanpa Clone Matriks

`DenseMatrix::solve_owned` memakai matriks yang sudah dimiliki pemanggil.
`MnaContext::solve` memakai jalur ini sehingga tidak menyalin matriks setiap
solve.

`DenseMatrix::solve` tetap tersedia dan mendelegasikan ke `solve_owned`
setelah clone.

### 3.3 Shared Mapping

`MnaContext` dan `SimulationSystem` berbagi `Arc<Vec<Option<usize>>>` untuk
node dan branch. Sebelumnya kedua mapping di-clone pada setiap solve.

### 3.4 Settle per Komponen Digital

Sirkuit digital CPU dipecah menjadi komponen terhubung saat build.
Perubahan input hanya menandai komponen yang terpengaruh. `settle` kemudian
mengevaluasi gate pada komponen kotor saja, bukan seluruh sirkuit.

DFF juga menandai komponennya saat nilai output berubah. Perubahan pada
register bank tidak lagi memaksa ALU, timer, atau GPIO ikut disettle.

Hasil release build, 300 ribu langkah:

| Jalur | Sebelum | Sesudah |
|---|---|---|
| LDI | 4.93 s | 1.85 s |
| ADD | 7.94 s | 2.19 s |
| OUT | 1.84 s | 0.33 s |
| Sketsa Arduino 1 juta langkah | 22.5 s | 14.7 s |

Angka bergantung mesin. Optimasi ini mempertahankan model register bank
dan unit datapath struktural.

Bottleneck yang tersisa adalah komponen register bank struktural.
Penulisan register masih harus men-settle komponen tersebut, sehingga
simulasi belum mendekati real-time.

---

## 4. Hasil

Pengukuran pada mesin pengembangan Linux, debug build, 1000 iterasi:

| Jalur | Sebelum | Sesudah |
|---|---|---|
| DC | 106.7 us/run | 36.8 - 37.1 us/run |
| Transient | 3.11 us/step | 2.90 - 2.92 us/step |

Penurunan DC sekitar 65 persen. Penurunan transient sekitar 6 persen.

Angka ini bukan jaminan untuk mesin lain.

---

## 5. Test

- `DenseMatrix::solve_owned` menghasilkan solusi yang sama.
- CLI memvalidasi jumlah iterasi.
- Format laporan benchmark diuji.
- Komponen digital independen tetap settle setelah input berubah.
- Sketsa Arduino `Serial.println("Hello World")` diverifikasi lewat
  bridge PTY setelah optimasi.
- Seluruh test workspace lulus.

Total test `sim-core`: 62.
Total test `sim-cli`: 26.
Total test `sim-gui`: 148.
Total workspace: 717.

---

## 6. Batas

- Benchmark memakai debug build.
- Tidak ada CPU pinning atau pengukuran statistik.
- Tidak ada profiling memori.
- Tidak ada perbandingan release build.
- Benchmark hanya memakai rangkaian RC linear.
- Belum ada optimasi sparse untuk transient.
