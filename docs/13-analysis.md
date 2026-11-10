# 13 - Analysis

Dokumentasi analisis DC dan transient yang sudah diimplementasikan.

---

## 1. Status

P4 diimplementasikan sebagian:

- MNA context.
- Stamping resistor, capacitor, inductor, source, MOSFET, diode.
- DC operating point.
- Newton-style linearization loop.
- Kriteria konvergensi.
- DC sweep.
- Transient Backward Euler.
- State update device.

Belum diimplementasikan:

- Voltage limiting eksplisit.
- Damping.
- gmin stepping.
- Source stepping.
- Diagnostik konvergensi lanjutan.
- Initial condition dari DC op untuk transient.
- CSV writer.
- VCD writer.
- Golden ngspice.
- Benchmark formal.

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
3. Stamp semua device.
4. Tambahkan gmin ke diagonal node.
5. Solve MNA.
6. Bandingkan solusi lama dan baru.
7. Ulangi sampai konvergen.
8. Update state device.

Konfigurasi default:

| Parameter | Nilai |
|---|---|
| `max_iterations` | 100 |
| `rel_tol` | 1e-6 |
| `abs_tol` | 1e-9 |
| `gmin` | 1e-12 |

Error:

- `ConvergenceFailure` jika batas iterasi tercapai.
- `SingularMatrix` jika matriks tidak dapat diselesaikan.

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
- Zero initial state.
- Waktu virtual `Time` dalam pikodetik.
- State capacitor dan inductor diperbarui setiap langkah.

Konfigurasi default:

| Parameter | Nilai |
|---|---|
| `dt` | 1e-6 s |
| `steps` | 100 |
| `gmin` | 1e-12 |

Output:

```rust
pub struct TransientSample {
    pub time: Time,
    pub solution: Solution,
}
```

Catatan:

- Transient belum memakai DC operating point sebagai initial condition.
- Semua capacitor mulai dari 0 V.
- Semua inductor mulai dari 0 A.

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

- Newton loop belum memakai voltage limiting eksplisit.
- gmin hanya nilai tetap, belum stepping.
- Belum ada source stepping.
- Belum ada damping.
- Convergence failure belum punya diagnostik node/device.
- Transient belum memakai initial DC op.
- Belum ada CSV/VCD.
- Belum ada perbandingan ngspice.
- Belum ada benchmark formal.
- Diode memakai forward voltage clamp sederhana.

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

- `sim-core`: 50.
- `sim-device`: 23 (18 unit + 5 integration).
- `sim-analysis`: 20.
- Total: 93.

Semua fungsi maksimal 20 baris dan semua file baru ASCII-only.
