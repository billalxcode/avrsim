# 31 - Distortion

Dokumen ini menjelaskan analisis distorsi harmonik kuasi-statis.

---

## 1. Status

P10-05 diimplementasikan untuk rangkaian memoryless:

- Satu sumber independen sebagai input.
- Satu node sebagai output.
- Distorsi harmonik orde dua dan seterusnya.

Analisis belum mencakup memori dinamis, intermodulasi, atau sumber sinus
transient.

---

## 2. Metode

Untuk sumber dengan bias `V0` dan amplitudo `A`:

1. Ambil `M` sampel fase `theta` yang seragam.
2. Hitung input `V0 + A * sin(theta)`.
3. Selesaikan DC operating point untuk setiap sampel.
4. Baca tegangan output.
5. Hitung DFT untuk orde `1..=N`.
6. Hitung THD dari magnitude harmonik.

Amplitudo harmonik:

```
amp_n = 2 / M * |sum(y_k * exp(-j * n * theta_k))|
```

THD:

```
THD = sqrt(sum(amp_n^2 untuk n >= 2)) / amp_1
```

THD bergantung pada amplitudo input. Nilai kecil mendekati perilaku linear,
nilai besar memasuki daerah nonlinier.

---

## 3. API

```rust
pub struct DistortionConfig {
    pub amplitude: f64,
    pub samples: usize,
    pub harmonics: usize,
    pub dc: DcConfig,
}

pub struct DistortionResult {
    pub fundamental: f64,
    pub harmonics: Vec<f64>,
    pub thd: f64,
}

pub fn distortion_analysis(
    circuit: &Circuit,
    device: &str,
    output: NodeId,
    config: &DistortionConfig,
) -> Result<DistortionResult, SimError>;
```

Validasi:

- Amplitudo harus positif dan finite.
- `samples` antara 8 dan 1.000.000.
- `harmonics` minimal 2 dan di bawah batas Nyquist.
- Device harus voltage source atau current source.
- Node output harus ada.

Jika fundamental nol, analisis mengembalikan error.

---

## 4. Test

- Voltage divider linear menghasilkan THD mendekati nol.
- Diode clamp menghasilkan THD terukur.
- Amplitudo nol, device bukan sumber, device tidak dikenal, dan konfigurasi
  di atas Nyquist ditolak.

Total test `sim-analysis`: 116 (108 unit + 8 integration).
Total workspace: 751.

---

## 5. Batas

- Hanya rangkaian memoryless.
- Tidak ada distorsi dinamis dari capacitor atau inductor.
- Tidak ada intermodulasi dua tone.
- Belum ada sumber sinus pada transient.
- Hasil bergantung pada resolusi sampel dan amplitudo.
- Belum ada perbandingan dengan ngspice.
