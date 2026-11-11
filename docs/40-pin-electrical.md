# 40 - Pin Electrical

Dokumen ini menjelaskan model elektrik pin GPIO.

---

## 1. Status

P8-09 diimplementasikan sebagai model aditif:

- Tegangan pin.
- Arus keluar pin.
- High-Z.
- Pull-up.
- Deteksi contention.

Model digital GPIO yang ada tetap dipakai untuk register DDR, PORT, dan PIN.

---

## 2. Konfigurasi

```rust
pub struct PinElectricalConfig {
    pub supply: f64,
    pub output_resistance: f64,
    pub pull_up_resistance: f64,
    pub input_threshold: f64,
}
```

Default:

| Field | Nilai |
|---|---|
| `supply` | `5.0 V` |
| `output_resistance` | `25 Ohm` |
| `pull_up_resistance` | `20 kOhm` |
| `input_threshold` | `2.5 V` |

Nilai default adalah asumsi board, belum dikalibrasi dari datasheet.
P10-19 tetap diperlukan untuk kalibrasi pin-level.

---

## 3. Mode Pin

```rust
pub enum PinMode {
    Input,
    InputPullUp,
    OutputLow,
    OutputHigh,
}
```

Pemetaan dari register:

| DDR | PORT | Mode |
|---|---|---|
| 0 | 0 | `Input` |
| 0 | 1 | `InputPullUp` |
| 1 | 0 | `OutputLow` |
| 1 | 1 | `OutputHigh` |

---

## 4. Koneksi Eksternal

```rust
pub enum ExternalPin {
    HighZ,
    Thevenin { voltage: f64, resistance: f64 },
}
```

`Thevenin` memodelkan sensor, tombol dengan resistor, atau driver lain.

---

## 5. Perhitungan

Sumber internal:

| Mode | Tegangan | Resistansi |
|---|---|---|
| `Input` | tidak ada | tidak ada |
| `InputPullUp` | supply | pull-up |
| `OutputLow` | 0 | output |
| `OutputHigh` | supply | output |

Jika sumber internal dan eksternal sama-sama ada:

```
V = (Vi / Ri + Ve / Re) / (1 / Ri + 1 / Re)
I = (Vi - V) / Ri
```

`I` adalah arus yang keluar dari pin menuju koneksi eksternal.

Jika hanya ada sumber eksternal, pin berperilaku high-Z dan tegangan
mengikuti sumber eksternal. Jika tidak ada koneksi, tegangan pin high-Z
dilaporkan `0.0 V` dan arus `0.0 A`.

Contention didefinisikan sebagai:

- Output high dan tegangan pin di bawah `input_threshold`.
- Output low dan tegangan pin di atas `input_threshold`.

---

## 6. API CPU

```rust
pub fn pin_electrical(
    &self,
    port: u8,
    bit: u8,
    external: ExternalPin,
    config: &PinElectricalConfig,
) -> Result<PinElectrical, SimError>;

pub fn set_pin_voltage(
    &mut self,
    port: u8,
    bit: u8,
    external: ExternalPin,
    config: &PinElectricalConfig,
) -> Result<PinElectrical, SimError>;
```

`set_pin_voltage` juga memperbarui input digital jika pin sedang menjadi
input, sehingga pembacaan PIN mengikuti level hasil model elektrik.

---

## 7. Integrasi Board

`Board::set_pb5_voltage` menghitung arus LED dari tegangan pin:

```
I = max(V - Vf, 0) / R
```

`Board::set_pb5` tetap tersedia dan memakai `5.0 V` atau `0.0 V`.

---

## 8. Test

- Output high dan low menghasilkan tegangan supply atau nol.
- Input high-Z mengikuti sumber eksternal.
- Pull-up menghasilkan idle high.
- Pembagi tegangan dengan beban menghasilkan tegangan dan arus sesuai
  rumus.
- Driver eksternal kuat memicu contention.
- Konfigurasi tidak valid ditolak.
- CPU memperbarui PIN dari tegangan eksternal.
- Kontensi terdeteksi pada pin output.
- LED board mengikuti tegangan pin.

Total test `sim-avr`: 197.
Total test `sim-board`: 14.
Total workspace: 445.

---

## 9. Batas

- Belum ada kapasitansi pin.
- Belum ada clamp diode atau perlindungan overvoltage.
- Belum ada batas arus short-circuit.
- Belum ada integrasi netlist analog penuh.
- Register PIN untuk pin output tetap membaca PORT, bukan hasil contention.
- Nilai default belum dikalibrasi datasheet.
