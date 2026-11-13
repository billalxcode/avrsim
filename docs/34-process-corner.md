# 34 - Process Corner

Dokumen ini menjelaskan penerapan process corner secara eksplisit.

---

## 1. Status

P10-08 diimplementasikan sebagai mekanisme scaling parameter:

- Resistor.
- Capacitor.
- Threshold voltage MOSFET.
- Transconductance MOSFET.

Tidak ada nilai corner bawaan. Faktor harus diberikan oleh pemanggil agar
tidak ada data yang dikarang.

---

## 2. API

```rust
pub struct ProcessCorner {
    pub resistor: f64,
    pub capacitor: f64,
    pub vto: f64,
    pub kp: f64,
}

pub fn with_process_corner(
    circuit: &Circuit,
    corner: &ProcessCorner,
) -> Result<Circuit, SimError>;
```

Contoh:

```rust
let corner = ProcessCorner {
    resistor: 1.10,
    capacitor: 0.95,
    vto: 0.90,
    kp: 1.05,
};
let adjusted = with_process_corner(&circuit, &corner)?;
let result = dc_operating_point(&adjusted, &DcConfig::default())?;
```

Parameter yang diskalakan:

| Device | Parameter |
|---|---|
| Resistor | `resistance` |
| Capacitor | `capacitance` |
| NMOS, PMOS | `vto` |
| NMOS, PMOS | `kp` |

`ProcessCorner::typical()` mengembalikan semua faktor `1.0`.

Semua faktor harus finite dan lebih besar dari nol.

---

## 3. Test

- Faktor resistor `2.0` menggandakan tegangan pada sumber arus tetap.
- Faktor capacitor `2.0` menggeser respons RC sesuai cutoff baru.
- Faktor threshold `0.5` menurunkan tegangan diode-connected NMOS.
- Faktor nol ditolak.

Total test `sim-analysis`: 108.
Total workspace: 608.

---

## 4. Batas

- Tidak ada corner global bawaan.
- Tidak ada mismatch antar device.
- Tidak ada Monte Carlo.
- Tidak ada scaling induktor atau sumber.
- Tidak ada model corner dari datasheet.
- Kombinasi suhu dan corner harus dipanggil terpisah.
