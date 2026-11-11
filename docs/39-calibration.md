# 39 - Kalibrasi

Dokumen ini menjelaskan prosedur kalibrasi parameter device.

---

## 1. Status

P10-20 diimplementasikan sebagai dokumentasi prosedur dan inventaris
parameter.

P10-19 belum dijalankan karena:

- Model pin elektrik P8-09 belum ada.
- Data pengukuran pin-level belum tersedia.
- Datasheet target belum dipilih.

Dokumen ini tidak memuat angka datasheet yang belum diverifikasi.

---

## 2. Inventaris Parameter

### 2.1 Resistor

- `resistance`.
- `tc1`, `tc2`.
- `temp`.

Kalibrasi:

1. Ukur resistansi pada beberapa suhu.
2. Fit polinomial orde dua terhadap `T - Tnom`.
3. Simpan `tc1` dan `tc2`.

### 2.2 Diode

- `is`, `n`, `vt`.
- `temp`.

Kalibrasi:

1. Ukur kurva `I` terhadap `V` pada suhu referensi.
2. Ambil daerah forward bias rendah untuk `is`.
3. Hitung slope logaritmik untuk `n`.
4. Ulangi pada suhu lain untuk memisahkan efek `vt`.

### 2.3 MOSFET

- `vto`, `kp`, `lambda`, `w`, `l`.
- `tcv`, `temp`.

Kalibrasi:

1. Ukur kurva transfer pada `Vds` tetap.
2. Ekstrak `vto` dari titik onset.
3. Ekstrak `kp` dari slope daerah linear atau saturasi.
4. Ekstrak `lambda` dari kemiringan saturasi.
5. Ukur pergeseran `vto` terhadap suhu untuk `tcv`.

### 2.4 BJT

- `is`, `bf`, `br`.
- `vt`, `temp`.

Kalibrasi:

1. Ukur `Ic` terhadap `Vbe` pada `Vbc` reverse.
2. Ekstrak `is` dan slope junction.
3. Ukur `Ic / Ib` pada forward active untuk `bf`.
4. Ukur operasi reverse untuk `br`.

### 2.5 JFET

- `vto`, `beta`, `lambda`.

Kalibrasi:

1. Ukur `Id` terhadap `Vgs` pada `Vds` tetap.
2. Ekstrak `vto` dari cutoff.
3. Ekstrak `beta` dari arus saturasi.
4. Ekstrak `lambda` dari slope saturasi.

### 2.6 Transmission Line

- `z0`, `delay`.

Kalibrasi:

1. Ukur impedansi karakteristik dengan terminasi matched.
2. Ukur delay dari step response.
3. Pastikan `delay` adalah kelipatan `dt` atau catat error kuantisasi.

---

## 3. Prosedur Umum

1. Tentukan device dan parameter target.
2. Siapkan data pengukuran atau datasheet yang boleh dipakai.
3. Jalankan sweep DC atau transient pada kondisi yang sama.
4. Hitung error relatif terhadap data.
5. Ubah satu parameter setiap langkah.
6. Catat nilai dan error akhir.
7. Simpan konfigurasi sebagai fixture test.

Jangan menyesuaikan beberapa parameter sekaligus tanpa catatan, karena
sumber error menjadi tidak jelas.

---

## 4. Kriteria

Kriteria numerik harus ditentukan per device dan per datasheet. Repository
ini belum menetapkan ambang global karena belum ada data target.

Saran awal:

- Parameter DC: error relatif di bawah 5 persen pada daerah operasi.
- Parameter suhu: error di bawah 10 persen pada rentang yang diukur.
- Delay line: error di bawah satu langkah waktu.

Angka di atas adalah saran, bukan hasil kalibrasi.

---

## 5. Batas

- Belum ada fixture datasheet.
- Belum ada model pin elektrik.
- Belum ada pengukuran laboratorium.
- Belum ada validasi silang dengan ngspice.
- Semua parameter masih default atau diberikan pengguna.
