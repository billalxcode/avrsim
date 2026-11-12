# 30 - Noise

Dokumen ini menjelaskan analisis noise termal resistor.

---

## 1. Status

P10-04 diimplementasikan untuk noise termal resistor:

- Density output pada satu node.
- Sweep frekuensi logaritmik.
- Integrasi root-mean-square.

Model lain belum ada:

- Shot noise.
- Flicker noise.
- Noise correlated.
- Noise input-referred.

---

## 2. Model

Resistor menghasilkan noise termal dengan density daya:

```
S = 4 * k * T / R
```

dengan:

- `k = 1.380649e-23 J/K`.
- `T = 300.15 K` secara default.
- `R` adalah resistansi.

Untuk setiap resistor, simulator:

1. Menonaktifkan semua sumber independen dengan `ac = 0`.
2. Menambahkan probe current source `1 A` paralel dengan resistor.
3. Menjalankan AC analysis untuk setiap frekuensi.
4. Menghitung kontribusi `S * |Z|^2`.

Kontribusi semua resistor dijumlahkan karena noise dianggap tidak
berkorelasi.

---

## 3. API

```rust
pub struct NoiseConfig {
    pub start: f64,
    pub stop: f64,
    pub points: usize,
    pub temperature: f64,
}

pub struct NoisePoint {
    pub frequency: f64,
    pub density: f64,
}

pub struct NoiseResult {
    pub points: Vec<NoisePoint>,
    pub total_rms: f64,
}

pub fn noise_analysis(
    circuit: &Circuit,
    output: NodeId,
    config: &NoiseConfig,
) -> Result<NoiseResult, SimError>;
```

Default:

- `start = 1 Hz`.
- `stop = 1 MHz`.
- `points = 100`.
- `temperature = 300.15 K`.

Integrasi memakai trapezoid pada sumbu frekuensi linear.

---

## 4. Test

- Resistor `1 kOhm` menghasilkan density `sqrt(4kTR)`.
- Voltage divider memakai resistansi Thevenin `500 Ohm`.
- RC low-pass menghasilkan total noise `sqrt(kT/C)` dengan error di bawah
  satu persen.
- Konfigurasi tidak valid dan node output tidak dikenal ditolak.

Total test `sim-analysis`: 108.
Total workspace: 553.

---

## 5. Batas

- Hanya noise termal resistor.
- Belum ada noise diode, MOSFET, atau sumber arus.
- Belum ada analisis noise input-referred.
- Belum ada korelasi antar device.
- Solver AC masih dense.
