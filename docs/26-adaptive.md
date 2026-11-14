# 26 - Adaptive Transient

Dokumen ini menjelaskan adaptive time stepping dan estimasi error.

---

## 1. Algoritma

- Langkah penuh dihitung dengan `dt`.
- Dua langkah setengah dihitung dengan `dt / 2`.
- Error adalah selisih maksimum node voltage antara hasil penuh dan hasil halus.
- Langkah diterima jika error <= tolerance.
- Langkah ditolak jika error terlalu besar dan `dt` masih di atas minimum.
- `dt` berikutnya memakai faktor 0.5 sampai 2.0.
- State device dipulihkan saat langkah ditolak.
- Initial solution memperbarui state device sebelum langkah pertama.
- Setiap langkah penuh dan setiap setengah langkah memakai Newton loop.

---

## 2. API

```rust
pub struct AdaptiveConfig {
    pub initial_dt: f64,
    pub min_dt: f64,
    pub max_dt: f64,
    pub tolerance: f64,
    pub steps: usize,
}

pub fn transient_adaptive(
    circuit: &Circuit,
    base: &TransientConfig,
    config: &AdaptiveConfig,
) -> Result<Vec<TransientSample>, SimError>;

pub fn transient_adaptive_with_solver(
    circuit: &Circuit,
    base: &TransientConfig,
    config: &AdaptiveConfig,
    newton: &NewtonConfig,
) -> Result<Vec<TransientSample>, SimError>;
```

---

## 3. Test

- RC charge dibandingkan dengan solusi analitik pada waktu akhir.
- RC discharge menghormati initial voltage kapasitor.
- Hasil adaptive dibandingkan dengan fixed-step pada waktu yang sama.
- Waktu selalu naik.
- Tolerance tidak valid ditolak.
- Zero steps menghasilkan output kosong.

Total test `sim-analysis`: 108.
Total workspace: 700.

---

## 4. Batas

- Estimasi error memakai step doubling, bukan formula LTE lengkap.
- Adaptive step masih memakai backward Euler.
- Trapezoidal dan Gear tersedia pada transient tetap, bukan pada adaptive.
- Adaptive step belum dipakai oleh CLI atau board.
- Belum ada kontrol error per device atau per arus.
