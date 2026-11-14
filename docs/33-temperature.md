# 33 - Suhu

Dokumen ini menjelaskan dukungan suhu pada model device.

---

## 1. Status

P10-07 diimplementasikan pada level parameter device:

- Resistor dengan koefisien suhu.
- Diode dengan thermal voltage dari suhu.
- MOSFET dengan koefisien threshold.

Belum ada model self-heating atau jaringan termal.

---

## 2. Parameter Device

### 2.1 Resistor

Parameter opsional:

- `temp`: suhu device dalam kelvin.
- `tc1`: koefisien suhu orde satu per kelvin.
- `tc2`: koefisien suhu orde dua per kelvin kuadrat.

Rumus:

```
R(T) = R0 * (1 + tc1 * (T - Tnom) + tc2 * (T - Tnom)^2)
```

`Tnom = 300.15 K`. Default `tc1 = tc2 = 0`, sehingga perilaku lama tidak
berubah.

### 2.2 Diode

Parameter `temp` menentukan thermal voltage:

```
Vt = k * T / q
```

dengan `k = 1.380649e-23 J/K` dan `q = 1.602176634e-19 C`.

Parameter `vt` eksplisit tetap menang atas `temp`. Default `vt` sekarang
dihitung dari `Tnom`, sehingga nilainya tetap sekitar `0.02585 V`.

### 2.3 MOSFET

Parameter opsional:

- `temp`: suhu device dalam kelvin.
- `tcv`: koefisien threshold dalam volt per kelvin.

Rumus:

```
Vto(T) = Vto0 + tcv * (T - Tnom)
```

Default `tcv = 0`, sehingga perilaku lama tidak berubah.

---

## 3. API

```rust
pub fn with_temperature(
    circuit: &Circuit,
    temperature: f64,
) -> Result<Circuit, SimError>;
```

Fungsi menyalin circuit dan menulis parameter `temp` pada resistor, diode,
NMOS, dan PMOS. Device lain tidak berubah.

Contoh:

```rust
let hot = with_temperature(&circuit, 400.15)?;
let result = dc_operating_point(&hot, &DcConfig::default())?;
```

Suhu harus finite dan lebih besar dari nol.

---

## 4. Test

- Resistor dengan `tc1 = 0.01` pada `400.15 K` mengubah pembagi tegangan
  sesuai rumus.
- Diode clamp pada suhu lebih tinggi menghasilkan tegangan output lebih
  tinggi.
- Suhu nol ditolak.

Total test `sim-analysis`: 115 (108 unit + 7 integration).
Total workspace: 747.

---

## 5. Batas

- Tidak ada self-heating.
- Tidak ada perubahan mobility, saturation current, atau lifetime selain
  parameter di atas.
- Tidak ada gradien suhu spasial.
- Tidak ada junction temperature per device.
- Kalibrasi terhadap datasheet belum dilakukan.
