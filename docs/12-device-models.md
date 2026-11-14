# 12 - Device Models

Katalog device yang tersedia di `sim-device` dan parameter utamanya.

**Status:** seluruh device inti, device nonlinier, controlled source, switch,
dan transmission line sudah diimplementasikan. Total test `sim-device`: 80.

---

## 1a. Katalog generik 500

`sim-device` juga menyediakan katalog 500 model generik melalui
`catalog()`, `ComponentSpec`, `Family`, dan `ModelClass`. Katalog ini
menjadi registry untuk Python builder dan pemetaan ke solver.

Detail taksonomi, akurasi, dan status ada di
`docs/60-component-catalog.md` dan `TODO-COMPONENTS.md`.

Registry:

```rust
let registry = ComponentRegistry::new();
let request = ComponentRequest::new(
    "passive.resistor.ideal_resistor",
    "R1",
    DeviceId::new(0),
    vec![a, b],
)
.with_parameter("resistance", 1000.0);
registry.add_to(&mut builder, &request)?;
```

Registry memvalidasi pin, parameter default, dan pemetaan ke
`DeviceKind`. Test emulasi membangun voltage divider dan saklar ideal
melalui registry, lalu menjalankan analisis DC.

---

## 1. API model

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

- `dt = None` untuk DC dan AC bias.
- `dt = Some(value)` untuk transient.
- `update` menyimpan state device setelah solusi konvergen.

Factory:

```rust
pub fn model_from_instance(
    instance: &DeviceInstance,
) -> Result<Box<dyn DeviceModel>, SimError>
```

---

## 2. Katalog device

| Device | Pin | Model | Dokumen |
|---|---|---|---|
| Resistor | 2 | Ohm, koefisien suhu | [33](33-temperature.md) |
| Capacitor | 2 | Linear | - |
| Inductor | 2 | Linear, branch current | [28](28-integration.md) |
| Voltage source | 2 | DC dan AC | - |
| Current source | 2 | DC dan AC | - |
| Pulse source | 2 | Rise, fall, width, period | [41](41-pulse-source.md) |
| Diode | 2 | Shockley, series resistance, junction capacitance | [46](46-diode-nonideal.md) |
| MOSFET | 3 atau 4 | Shichman-Hodges Level 1, body effect, Meyer | [44](44-body-effect.md), [45](45-mosfet-capacitance.md) |
| BJT | 3 | Ebers-Moll, junction dan diffusion capacitance | [35](35-bjt.md) |
| JFET | 3 | Shichman-Hodges, kapasitansi konstan | [36](36-jfet.md) |
| Transmission line | 4 | Lossless, constant step | [37](37-transmission-line.md) |
| VCVS | 4 | Linear gain | [47](47-controlled-sources.md) |
| VCCS | 4 | Linear transconductance | [47](47-controlled-sources.md) |
| CCCS | 4 | Linear current gain | [48](48-cccs-switch.md) |
| CCVS | 4 | Linear transresistance | [49](49-ccvs.md) |
| Switch | 4 | Smooth on/off conductance | [48](48-cccs-switch.md) |
| AnalogSwitch | 4 | Voltage-controlled analog conductance | - |
| Relay | 5 | Coil and changeover contacts | - |
| IdealSwitch | 2 | Fixed on/off conductance | - |
| OpAmp | 5 | Linear high-gain VCVS macromodel | - |
| Comparator | 5 | Saturating nonlinear comparator | - |

---

## 3. Parameter passif dan sumber

| Device | Parameter | Default | Wajib |
|---|---|---|---|
| Resistor | `resistance` | - | ya |
| Resistor | `temp` | 300.15 K | tidak |
| Resistor | `tc1` | 0 | tidak |
| Resistor | `tc2` | 0 | tidak |
| Capacitor | `capacitance` | - | ya |
| Inductor | `inductance` | - | ya |
| Voltage source | `voltage` | 0.0 | tidak |
| Voltage source | `ac` | `voltage` | tidak |
| Voltage source | `ac_phase` | 0 deg | tidak |
| Current source | `current` | 0.0 | tidak |
| Current source | `ac` | `current` | tidak |
| Current source | `ac_phase` | 0 deg | tidak |

Sumber pulsa:

| Parameter | Default |
|---|---|
| `v1` | 0.0 V |
| `v2` | 5.0 V |
| `delay` | 0 s |
| `rise` | 1 ns |
| `fall` | 1 ns |
| `width` | 1 us |
| `period` | 2 us |

---

## 4. Diode

Parameter:

| Parameter | Default | Arti |
|---|---|---|
| `is` | 1e-12 A | saturation current |
| `n` | 1.0 | emission coefficient |
| `vt` | k*T/q | thermal voltage |
| `rs` | 0 ohm | series resistance |
| `cj0` | 0 F | zero-bias junction capacitance |
| `vj` | 0.7 V | junction potential |
| `m` | 0.5 | grading coefficient |

Arus:

```text
I = is * (exp(V / (n * vt)) - 1)
```

Junction capacitance:

```text
C = cj0 / (1 - V / vj)^m
```

Dengan `rs > 0`, tegangan junction diselesaikan secara iteratif.

---

## 5. MOSFET

Parameter:

| Parameter | Default | Arti |
|---|---|---|
| `vto` | 0.7 V | threshold |
| `kp` | 1e-3 A/V^2 | transconductance |
| `lambda` | 0.0 | channel-length modulation |
| `w` | 1e-6 m | width |
| `l` | 1e-6 m | length |
| `gamma` | 0.0 | body effect |
| `phi` | 0.7 V | surface potential |
| `cox` | 0 F | oxide capacitance |
| `cgso` | 0 F | gate-source overlap |
| `cgdo` | 0 F | gate-drain overlap |

Region:

- Cutoff: `vgs <= vto`.
- Linear: `vds < vgs - vto`.
- Saturation: `vds >= vgs - vto`.

Kapasitansi memakai pembagian Meyer sederhana berdasarkan region.
MOSFET tiga terminal memakai source sebagai body.

---

## 6. BJT

Parameter:

| Parameter | Default | Arti |
|---|---|---|
| `is` | 1e-15 A | saturation current |
| `bf` | 100 | forward beta |
| `br` | 1 | reverse beta |
| `vt` | k*T/q | thermal voltage |
| `temp` | 300.15 K | suhu |
| `cje` | 0 F | base-emitter capacitance |
| `vje` | 0.75 V | base-emitter potential |
| `mje` | 0.33 | base-emitter grading |
| `cjc` | 0 F | base-collector capacitance |
| `vjc` | 0.75 V | base-collector potential |
| `mjc` | 0.33 | base-collector grading |
| `tf` | 0 s | forward transit time |
| `tr` | 0 s | reverse transit time |

Arus Ebers-Moll:

```text
IF = is * (exp(vbe / vt) - 1)
IR = is * (exp(vbc / vt) - 1)
Ic = alpha_f * IF - IR
Ib = (1 - alpha_f) * IF + (1 - alpha_r) * IR
Ie = -IF + alpha_r * IR
```

Kapasitansi:

```text
Cbe = depletion(cje, vje, mje, vbe) + tf * gbe
Cbc = depletion(cjc, vjc, mjc, vbc) + tr * gbc
```

PNP memakai transformasi tanda pada `vbe`, `vbc`, dan arus.

---

## 7. JFET

Parameter:

| Parameter | Default | Arti |
|---|---|---|
| `vto` | -2 V untuk N, +2 V untuk P | threshold |
| `beta` | 1e-3 A/V^2 | transconductance |
| `lambda` | 0.0 | channel-length modulation |
| `cgs` | 0 F | gate-source capacitance |
| `cgd` | 0 F | gate-drain capacitance |

Arus channel:

```text
overdrive = vgs - vto
overdrive <= 0: Id = 0
vds < overdrive: Id = beta * (2 * overdrive * vds - vds^2) * (1 + lambda * vds)
vds >= overdrive: Id = beta * overdrive^2 * (1 + lambda * vds)
```

P-JFET memakai transformasi tanda. Kapasitansi `cgs` dan `cgd` konstan.

---

## 8. Controlled source dan switch

| Device | Parameter | Default |
|---|---|---|
| VCVS | `gain` | 1.0 |
| VCCS | `gm` | 1e-3 S |
| CCCS | `gain` | 1.0 |
| CCVS | `rm` | 1000 ohm |
| Switch | `ron` | 1 ohm |
| Switch | `roff` | 1e9 ohm |
| Switch | `vth` | 2.5 V |
| Switch | `width` | 0.1 V |
| IdealSwitch | `ron` | 0.01 ohm |
| IdealSwitch | `roff` | 1e12 ohm |
| IdealSwitch | `closed` | 0 |
| AnalogSwitch | `ron` | 1 ohm |
| Relay | `rcoil` | 100 ohm |
| Relay | `ron` | 0.1 ohm |
| OpAmp | `gain` | 1e5 |
| Comparator | `gain` | 1e4 |

CCVS memakai dua branch: sense branch dan output branch.

Switch memakai conductance halus:

```text
g = 1/roff + (1/ron - 1/roff) * sigmoid((vctrl - vth) / width)
```

---

## 9. Transmission line

Parameter:

| Parameter | Default | Arti |
|---|---|---|
| `z0` | 50 ohm | characteristic impedance |
| `delay` | 1 ns | one-way delay |

Transmission line hanya mendukung transient dengan time step konstan.
Device ini belum didukung AC.

---

## 10. Test

Test device mencakup:

- Validasi parameter.
- Stamping resistor, capacitor, dan inductor.
- Sumber tegangan dan arus.
- MOSFET cutoff, linear, saturation, lambda, width/length.
- Body effect dan `gmb`.
- Kapasitansi MOSFET.
- Diode IV, series resistance, dan junction capacitance.
- BJT arus, gain, suhu, dan kapasitansi.
- JFET arus, tanda P, dan kapasitansi.
- Controlled source dan switch.
- Transmission line.

Total test `sim-device`: 48 unit + 5 integration = 48.

---

## 11. Batas

- MOSFET Level 1 tidak akurat untuk short-channel.
- Meyer capacitance bukan charge-conserving.
- BJT dan JFET memakai model kapasitansi sederhana.
- Diode belum memiliki breakdown dan diffusion capacitance.
- Switch belum memiliki hysteresis atau delay.
- Transmission line hanya lossless dan constant step.
- Parameter belum dikalibrasi ke silikon.

Matriks fidelity lengkap ada di [24-fidelity.md](24-fidelity.md).
