# 28 - Metode Integrasi

Dokumen ini menjelaskan metode integrasi transient.

---

## 1. Metode

- `BackwardEuler`: default, stabil orde satu.
- `Trapezoidal`: orde dua, memakai arus atau tegangan sebelumnya.
- `Gear2`: orde dua, memakai dua nilai sebelumnya.

API:

```rust
pub enum IntegrationMethod {
    BackwardEuler,
    Trapezoidal,
    Gear2,
}

pub fn transient_with_method(
    circuit: &Circuit,
    config: &TransientConfig,
    method: IntegrationMethod,
) -> Result<Vec<TransientSample>, SimError>;
```

`transient` tetap memakai Backward Euler.

---

## 2. Capacitor

- Backward Euler: `G = C / dt`.
- Trapezoidal: `G = 2C / dt` dan arus sebelumnya.
- Gear2: `G = 3C / (2 dt)` dan dua tegangan sebelumnya.
- State current dan voltage diperbarui setelah konvergensi.

---

## 3. Inductor

- Backward Euler: `R = L / dt`.
- Trapezoidal: `R = 2L / dt` dan tegangan sebelumnya.
- Gear2: `R = 3L / (2 dt)` dan dua arus sebelumnya.
- Branch equation memakai RHS yang sesuai.

---

## 4. Test

- RC dengan Backward Euler.
- RC dengan Trapezoidal.
- RC dengan Gear2.
- RL dengan Trapezoidal.
- RL dengan Gear2.

Semua hasil mendekati solusi analitik.

Total test `sim-analysis`: 108.
Total workspace: 601.

---

## 5. Batas

- Metode baru hanya diterapkan pada capacitor dan inductor.
- Device nonlinier tetap memakai stamp dasar.
- Adaptive transient masih memakai Backward Euler.
- Belum ada perbandingan error orde formal.
- Belum ada golden SPICE.
