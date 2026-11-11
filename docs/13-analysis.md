# 13 - Analysis

Dokumentasi analisis DC, transient, AC, noise, distorsi, dan sensitivitas yang sudah diimplementasikan.

---

## 1. Status

P4 diimplementasikan sebagian:

- MNA context.
- Stamping resistor, capacitor, inductor, source, pulse source, MOSFET, diode, BJT, JFET.
- DC operating point.
- Newton-style linearization loop.
- Kriteria konvergensi.
- DC sweep.
- Transient Backward Euler.
- AC small-signal linear.
- Noise termal resistor.
- Distorsi harmonik kuasi-statis.
- Sensitivitas DC numerik.
- Suhu device dan process corner.
- BJT Ebers-Moll NPN dan PNP.
- JFET Shichman-Hodges N dan P.
- Transmission line lossless untuk transient.
- Continuation sweep dan initial state device untuk discharge.
- Voltage limiting junction, source stepping, dan writer CSV/VCD analog.
- Pulse voltage source untuk transient switching.
- Karakterisasi rise, fall, dan propagation delay.
- Body effect MOSFET dan `gmb`.
- Benchmark solver dan optimasi alokasi.
- Prosedur kalibrasi parameter device.
- Model pin elektrik untuk GPIO.
- State update device.

Belum diimplementasikan:

- Diagnostik konvergensi lanjutan.
- Golden comparison dengan ngspice.
- Writer waveform terintegrasi ke CLI.

---

## 2. MNA Context

`MnaContext` menyimpan:

- Dense matrix.
- RHS vector.
- Mapping node ke baris.
- Mapping branch ke baris.
- Jumlah node non-ground.
- Jumlah branch.

Aturan indeks:

- Node non-ground menempati indeks `0..node_count`.
- Branch menempati indeks `node_count..node_count + branch_count`.
- Ground tidak memiliki baris.
- `Solution.node_voltages` diindeks ulang berdasarkan `NodeId`.
- `Solution.branch_currents` memakai indeks matriks absolut.

---

## 3. DC Operating Point

Alur:

1. Bangun `SimulationSystem`.
2. Buat solusi awal nol.
3. Tambahkan gmin stepping dari `gmin_start` ke `gmin`.
4. Jika gagal, coba source stepping dari 0 ke nilai sumber penuh.
5. Stamp semua device.
6. Solve MNA.
7. Batasi perubahan node voltage dengan `max_step`.
8. Bandingkan solusi lama dan baru.
9. Ulangi sampai konvergen.
10. Update state device.

Konfigurasi default:

| Parameter | Nilai |
|---|---|
| `max_iterations` | 100 |
| `rel_tol` | 1e-6 |
| `abs_tol` | 1e-9 |
| `gmin` | 1e-12 |
| `use_dc_op` | true |
| `gmin_start` | 1e-3 |
| `max_step` | 0.5 V |

`DcResult` menyimpan `iterations` dan `residual`, yaitu perubahan node
voltage maksimum saat konvergen.

Error:

- `ConvergenceFailure` jika batas iterasi tercapai.
- `SingularMatrix` jika matriks tidak dapat diselesaikan.
- Jika gmin dan source stepping gagal, error gmin dikembalikan.

---

## 4. DC Sweep

`dc_sweep` mengubah satu parameter device dan menyelesaikan DC op di setiap titik.

Aturan:

- `step > 0`.
- `start` dan `stop` finite.
- `stop >= start`.
- Maksimum 1.000.000 titik.
- Device harus ada.
- Parameter akan ditambahkan jika belum ada.
- Setiap titik memakai solusi titik sebelumnya sebagai initial.

Contoh:

```rust
let points = dc_sweep(
    &circuit,
    "vin",
    "voltage",
    0.0,
    5.0,
    0.5,
    &DcConfig::default(),
)?;
```

---

## 5. Transient

Metode:

- Backward Euler.
- Initial state dari DC operating point atau zero state.
- Waktu virtual `Time` dalam pikodetik.
- State capacitor dan inductor diperbarui setiap langkah.
- `initial_voltages` memperbarui state device sebelum langkah pertama.

Konfigurasi default:

| Parameter | Nilai |
|---|---|
| `dt` | 1e-6 s |
| `steps` | 100 |
| `gmin` | 1e-12 |
| `use_dc_op` | true |
| `gmin_start` | 1e-3 |
| `max_step` | 0.5 V |

Output:

```rust
pub struct TransientSample {
    pub time: Time,
    pub solution: Solution,
}
```

Catatan:

- `use_dc_op = true` memakai DC operating point sebagai initial state.
- `use_dc_op = false` memakai zero state: capacitor 0 V, inductor 0 A.
- Default adalah DC operating point.

---

## 6. Hasil M1

Test yang sudah lulus:

- Pembagi tegangan 10 V dengan dua resistor 1 kOhm menghasilkan 5 V.
- DC sweep pada pembagi tegangan.
- Capacitor open pada DC.
- Inductor short pada DC.
- Inverter CMOS VTC:
  - Input 0 V menghasilkan output tinggi.
  - Input 5 V menghasilkan output rendah.
- RC charge:
  - 5 V, 1 kOhm, 1 uF.
  - Setelah 1 ms, output sekitar 3.16 V.
- Diode clamp:
  - 5 V melalui 1 kOhm ke dioda.
  - Output sekitar 0.4 V sampai 0.9 V.
- Waktu transient meningkat sesuai `dt`.

---

## 7. Keterbatasan

- Belum ada source stepping.
- Voltage limiting eksplisit `pnjlim` belum ada; yang ada adalah damping dan clamp diode.
- Convergence failure belum punya diagnostik node/device.
- Belum ada CSV/VCD.
- Belum ada perbandingan ngspice.
- Belum ada benchmark formal.
- Diode memakai forward voltage clamp sederhana.
- Transient default memakai DC op; zero state harus diminta eksplisit.

---

## 8. Berikutnya

P4 lanjutan:

- Voltage limiting dan damping.
- gmin stepping dan source stepping.
- Diagnostik konvergensi.
- Initial condition dari DC op.
- CSV/VCD writer.
- Golden data ngspice.
- Benchmark.

---

## 9. Test

Total test workspace saat ini:

- `sim-core`: 62.
- `sim-device`: 38 (33 unit + 5 integration).
- `sim-analysis`: 76.
- `sim-cells`: 67.
- `sim-avr`: 198.
- `sim-board`: 17.
- `sim-cli`: 11.
- Total: 469.

Semua fungsi maksimal 20 baris dan semua file baru ASCII-only.
