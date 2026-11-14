# 32 - Sensitivity

Dokumen ini menjelaskan analisis sensitivitas DC numerik.

---

## 1. Status

P10-06 diimplementasikan untuk satu parameter device dan satu node output.

Metode:

- Central difference.
- Setiap sisi dihitung dengan DC operating point penuh.
- Langkah relatif default `1e-4`.

---

## 2. API

```rust
pub struct DcSensitivity {
    pub device: String,
    pub parameter: String,
    pub value: f64,
    pub derivative: f64,
}

pub fn dc_sensitivity(
    circuit: &Circuit,
    device: &str,
    parameter: &str,
    node: NodeId,
    config: &DcConfig,
) -> Result<DcSensitivity, SimError>;
```

Derivative:

```
dV(node) / d(parameter)
```

Langkah:

```
h = max(|value|, 1.0) * 1e-4
```

Validasi:

- Device harus ada.
- Parameter harus ada pada device.
- Node harus ada.

---

## 3. Test

- Voltage divider: `dVout/dR1 = -0.0025` dan `dVout/dR2 = +0.0025`.
- Sensitivitas sumber: `dVout/dV1 = 0.5`.
- Device atau parameter tidak dikenal ditolak.
- Node tidak dikenal ditolak.

Total test `sim-analysis`: 108.
Total workspace: 675.

---

## 4. Batas

- Hanya derivatif lokal pada satu titik operasi.
- Hanya satu parameter setiap pemanggilan.
- Belum ada adjoint sensitivity.
- Akurasi bergantung pada konvergensi Newton dan ukuran langkah.
- Belum ada sensitivitas untuk branch current atau AC.
