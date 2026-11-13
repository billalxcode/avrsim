# 42 - Switching Characterization

Dokumen ini menjelaskan pengukuran rise time, fall time, dan propagation
delay dari transient.

---

## 1. Status

P5-05 sampai P5-08 dan P5-18 sampai P5-19 diimplementasikan:

- Menjalankan transient switching.
- Mengukur rise time 10-90 persen.
- Mengukur fall time 90-10 persen.
- Mengukur propagation delay.
- Mengukur input capacitance dari rise time RC.
- Mengukur drive strength statis.
- Memverifikasi osilasi ring oscillator analog.

Belum ada perbandingan ngspice.

---

## 2. API

```rust
pub struct SwitchingTiming {
    pub rise_time: f64,
    pub fall_time: f64,
    pub delay_rise: f64,
    pub delay_fall: f64,
}

pub fn measure_switching(
    circuit: &Circuit,
    input: NodeId,
    output: NodeId,
    config: &TransientConfig,
) -> Result<SwitchingTiming, SimError>;
```

Semua waktu dalam detik.

```rust
pub fn measure_input_capacitance(
    circuit: &Circuit,
    drive: NodeId,
    input: NodeId,
    resistance: f64,
    config: &TransientConfig,
) -> Result<f64, SimError>;

pub struct DriveStrength {
    pub open_voltage: f64,
    pub loaded_voltage: f64,
    pub current: f64,
    pub output_resistance: f64,
}

pub fn measure_drive_strength(
    circuit: &Circuit,
    output: NodeId,
    load: f64,
    config: &DcConfig,
) -> Result<DriveStrength, SimError>;
```

---

## 3. Metode

1. Jalankan transient pada rangkaian.
2. Ambil waveform input dan output.
3. Hitung level minimum dan maksimum output.
4. Tentukan threshold 10, 50, dan 90 persen.
5. Cari crossing dengan interpolasi linear antar sampel.
6. Hitung rise time dan fall time.
7. Hitung delay terhadap edge input terakhir sebelum edge output.

Untuk output rise, input edge yang dipakai adalah crossing terakhir sebelum
output rise. Untuk output fall, dipakai crossing terakhir sebelum output
fall. Dengan cara ini fungsi bekerja untuk rangkaian inverting maupun non
inverting.

---

## 4. Test

Rangkaian uji:

- Pulse source `0` ke `5 V`.
- Resistor `1 kOhm`.
- Capacitor `1 nF`.
- Time constant `1 us`.

Hasil yang diharapkan:

- Rise time sekitar `2.2 us`.
- Fall time sekitar `2.2 us`.
- Propagation delay sekitar `0.69 us`.

Input capacitance memakai rise time 10-90 persen:

```
C = rise_time / (R * ln(9))
```

Drive strength membandingkan tegangan open-circuit dan tegangan dengan
load resistor, lalu menghitung resistansi output ekivalen.

MOS capacitor dengan `cox = 1 pF` menghasilkan capacitance sekitar `1 pF`.
Voltage divider `500 Ohm` menghasilkan resistansi output sekitar `500 Ohm`.

Test juga menolak node yang sama, output yang tidak switching, dan load
nol.

Ring oscillator tiga inverter dengan beban `1 nF` per node dan initial
voltage `5 V` menghasilkan lebih dari empat crossing pada node output.

Total test `sim-analysis`: 108.
Total test `sim-cells`: 71.
Total workspace: 617.

---

## 5. Batas

- Hanya edge pertama yang diukur.
- Tidak ada pengukuran slew rate atau overshoot.
- Tidak ada de-embedding probe.
- Tidak ada kalibrasi CMOS terhadap datasheet.
- Input harus benar-benar melakukan transisi.
- Threshold memakai persentase dari swing output, bukan level absolut.
- Input capacitance mengasumsikan rangkaian RC orde satu.
- Drive strength diukur pada satu titik operasi dan satu load.
- Belum ada perbandingan dengan ngspice.
- Osilasi diverifikasi pada satu titik beban, bukan seluruh corner.
