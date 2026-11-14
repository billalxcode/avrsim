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
- AC small-signal dengan linearisasi DC, kapasitansi diode/MOSFET/BJT/JFET, controlled sources, dan switch.
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
- Karakterisasi rise, fall, delay, input capacitance, dan drive strength.
- Body effect MOSFET, `gmb`, kapasitansi Meyer, diode nonideal, controlled sources lengkap, dan switch.
- Benchmark solver dan optimasi alokasi.
- Prosedur kalibrasi parameter device.
- Model pin elektrik untuk GPIO.
- Loader ELF32, simbol, workflow CI, dan audit lisensi.
- Peta register UART0 dan forwarding TX/RX ke virtual serial.
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
| `max_iterations` | 50 |
| `rel_tol` | 1e-6 |
| `abs_tol` | 1e-9 |

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
- Setiap langkah memakai loop Newton sampai konvergen.
- `transient_with_solver` menerima `NewtonConfig` eksplisit.

Konfigurasi default:

| Parameter | Nilai |
|---|---|
| `dt` | 1e-6 s |
| `steps` | 100 |
| `gmin` | 1e-12 |
| `use_dc_op` | true |
| `gmin_start` | 1e-3 |
| `max_step` | 0.5 V |
| `max_iterations` | 50 |
| `rel_tol` | 1e-6 |
| `abs_tol` | 1e-9 |

Output:

```rust
pub struct TransientSample {
    pub time: Time,
    pub solution: Solution,
    pub residual: f64,
    pub iterations: usize,
}
```

Catatan:

- `use_dc_op = true` memakai DC operating point sebagai initial state.
- `use_dc_op = false` memakai zero state: capacitor 0 V, inductor 0 A.
- Default adalah DC operating point.
- Jika Newton tidak konvergen, langkah mengembalikan `ConvergenceFailure`.
- `residual` adalah perubahan node voltage maksimum saat konvergen.

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

- Convergence failure belum memiliki diagnostik node atau device.
- Golden comparison dengan ngspice belum dijalankan.
- Writer CSV/VCD tersedia di adapter, belum terintegrasi ke CLI.
- Periodic steady state dan harmonic balance belum ada.
- Diode memakai forward voltage clamp sederhana.
- Transient default memakai DC operating point; zero state harus diminta eksplisit.
- Adaptive transient masih memakai Backward Euler.

---

## 8. Pekerjaan berikutnya

- Golden data dan harness ngspice.
- Diagnostik konvergensi yang menyebut node dan device.
- Integrasi waveform writer ke CLI.
- Verifikasi determinisme macOS dan Windows.
- Eksekusi workflow CI.

Daftar lengkap ada di [TODO.md](../TODO.md).

---

## 9. Test

Total test workspace saat ini:

- `sim-core`: 62.
- `sim-device`: 48 (43 unit + 5 integration).
- `sim-analysis`: 108.
- `sim-cells`: 71.
- `sim-avr`: 215.
- `sim-board`: 23.
- `sim-cli`: 26.
- `sim-gui`: 148.
- Total: 702.

Semua fungsi maksimal 20 baris dan semua file baru ASCII-only.
