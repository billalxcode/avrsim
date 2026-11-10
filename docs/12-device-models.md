# 12 - Device Models

Dokumentasi model device yang sudah diimplementasikan di `sim-device`.

---

## 1. Status

P3 diimplementasikan sebagian:

- Resistor.
- Capacitor.
- Inductor.
- Voltage source.
- Current source.
- MOSFET Level 1 NMOS/PMOS.
- Diode Shockley.

Belum diimplementasikan:

- Body effect.
- Meyer capacitance.
- BJT.
- JFET.
- Controlled source.
- Switch.
- Transmission line.
- Diode series resistance dan junction capacitance.

---

## 2. API

```rust
pub trait DeviceModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError>;

    fn update(&mut self, solution: &Solution) -> Result<(), SimError>;
}
```

`dt` adalah `None` untuk DC dan `Some(dt)` untuk transient.

Factory:

```rust
pub fn model_from_instance(
    instance: &DeviceInstance,
) -> Result<Box<dyn DeviceModel>, SimError>
```

---

## 3. Parameter

| Device | Parameter | Default |
|---|---|---|
| Resistor | `resistance` | wajib |
| Capacitor | `capacitance` | wajib |
| Inductor | `inductance` | wajib |
| Voltage source | `voltage` | 0.0 |
| Current source | `current` | 0.0 |
| MOSFET | `vto` | 0.7 |
| MOSFET | `kp` | 1e-3 |
| MOSFET | `lambda` | 0.0 |
| MOSFET | `w` | 1e-6 |
| MOSFET | `l` | 1e-6 |
| Diode | `is` | 1e-12 |
| Diode | `n` | 1.0 |
| Diode | `vt` | 0.02585 |

Parameter positif divalidasi. Nilai nol, negatif, NaN, atau hilang untuk parameter wajib menghasilkan error.

---

## 4. Resistor

Stamp konduktansi:

```text
g = 1 / R
```

Konduktansi ditambahkan ke empat entri matriks antara dua node.

---

## 5. Capacitor

DC:

- Capacitor open.
- Tidak ada stamp.

Transient Backward Euler:

```text
I = C * (V - Vprev) / dt
g = C / dt
Ieq = g * Vprev
```

State `voltage_prev` diperbarui setelah solusi konvergen.

---

## 6. Inductor

DC:

- Inductor short.
- Branch dengan tegangan 0.

Transient Backward Euler:

```text
V = L * (I - Iprev) / dt
R = L / dt
Veq = -R * Iprev
```

State `current_prev` diperbarui setelah solusi konvergen.

---

## 7. Voltage Source

Branch equation:

```text
Va - Vb = V
```

Branch current menjadi unknown tambahan.

---

## 8. Current Source

Current source mengalir dari pin pertama ke pin kedua.

```text
RHS[a] -= I
RHS[b] += I
```

---

## 9. MOSFET Level 1

Parameter:

- `vto`: threshold voltage magnitude.
- `kp`: transconductance parameter.
- `lambda`: channel-length modulation.
- `w`, `l`: width dan length.

Region NMOS:

```text
Vov = Vgs - Vto
cutoff:     Vov <= 0
linear:     Vds < Vov
saturation: Vds >= Vov
```

Linear:

```text
Ids = kp * (W/L) * (Vov * Vds - 0.5 * Vds^2) * (1 + lambda * Vds)
```

Saturation:

```text
Ids = 0.5 * kp * (W/L) * Vov^2 * (1 + lambda * Vds)
```

PMOS memakai polaritas terbalik:

```text
Ids = -nmos_current(Vsg, Vsd)
```

### Jacobian

Jacobian dihitung secara numerik dengan beda hingga kecil.

- `gm = dIds/dVg`
- `gds = dIds/dVd`
- `gms = dIds/dVs`

Keuntungan:

- Satu implementasi untuk NMOS dan PMOS.
- Mudah diperluas.

Batasan:

- Sedikit lebih lambat daripada Jacobian analitik.
- Perlu step yang sesuai.

### Yang Belum Ada

- Body effect.
- `gmb`.
- Meyer capacitance.
- Subthreshold.
- Mobility degradation.

---

## 10. Diode

Persamaan Shockley:

```text
I = Is * (exp(V / (n * Vt)) - 1)
```

Implementasi:

- Forward voltage dibatasi 0.9 V untuk stabilitas Newton.
- Eksponen dibatasi 40.
- Tidak ada series resistance.
- Tidak ada junction capacitance.
- Tidak ada breakdown.

Test:

- Arus nol pada 0 V.
- Arus maju naik.
- Arus reverse kecil.
- Konduktansi positif.
- Forward voltage terbatas.

---

## 11. Test

Test device mencakup:

- Parameter helper.
- MOSFET cutoff.
- MOSFET linear.
- MOSFET saturation.
- Lambda.
- Width/length scaling.
- Numeric derivative.
- Resistor stamping.
- Voltage source branch.
- Current source injection.
- Custom device unsupported.
- MOSFET factory.
- Diode current.

Total test `sim-device`: 18 unit + 5 integration = 23.

---

## 12. Berikutnya

P3 lanjutan:

- Body effect dan `gmb`.
- Meyer capacitance.
- Diode series resistance dan capacitance.
- BJT Ebers-Moll.
- JFET.
- Controlled source.
- Switch.
- Transmission line.

Lihat `TODO.md` untuk daftar lengkap.
