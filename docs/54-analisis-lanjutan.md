# 54 - Analisis Lanjutan

Dokumen ini merangkum analisis menyeluruh kondisi proyek dan rencana TODO
berikutnya.

---

## 1. Ringkasan

Proyek sudah berada pada tahap implementasi matang:

- 8 crate Rust.
- 728 test.
- Nol dependensi Rust eksternal di crate simulasi; `sim-gui` memakai
  daftar dependensi yang disetujui.
- Dokumentasi lengkap di `docs/`.
- Release archive tersedia di `dist/`.

Semua quality gate lokal lulus:

- `cargo fmt`.
- `cargo clippy`.
- `cargo test --workspace`.
- `cargo build --workspace --release`.
- Pemeriksaan ASCII.

---

## 2. Kekuatan

- Batas crate konsisten dan tidak melingkar.
- Device inti dan lanjutan sudah luas.
- Solver DC, transient, AC linear, noise, distorsi, dan sensitivitas tersedia.
- CPU AVR, peripheral, board, pin elektrik, HEX, dan ELF tersedia.
- Test tersebar dari device sampai end-to-end.
- Dokumentasi keputusan, fidelity, risiko, dan batasan tersedia.
- CI workflow, script lokal, packaging, dan contoh runnable tersedia.

---

## 3. Temuan Teknis

### 3.1 Transient nonlinier hanya satu solves per langkah

`transient.rs::advance` memanggil `solve_with_method` satu kali per langkah,
lalu `update`. Tidak ada loop Newton per langkah.

Dampak:

- Akurasi transient diode, MOSFET, BJT, dan JFET tidak dijamin.
- Tidak ada kriteria konvergensi per langkah.
- Error dapat menumpuk pada switching cepat.

Prioritas: tinggi.

### 3.2 Adaptive transient tidak menginisialisasi state device

`transient_adaptive` tidak memanggil `system.update` setelah
`initial_solution`.

Dampak:

- Initial voltage kapasitor tidak masuk ke state device.
- Hasil berbeda dari `transient` biasa pada kondisi awal yang sama.

Prioritas: tinggi, perbaikan kecil.

### 3.3 AC hanya untuk device linear

AC menolak MOSFET, diode, BJT, dan JFET.

Dampak:

- Rangkaian nyata belum bisa dianalisis pada domain frekuensi.
- Noise dan sensitivitas AC juga terbatas.

Prioritas: tinggi setelah transient Newton.

### 3.4 Verifikasi eksternal

Sudah:

- Firmware `avr-gcc` dan sketsa Arduino core dijalankan lewat
  `/dev/ttyACM0`.

Belum:

- Golden ngspice belum dijalankan.
- Upload dari GUI Arduino IDE belum diuji ulang.
- Differential test simavr belum ada.
- Determinisme macOS/Windows belum dijalankan.

Prioritas: tinggi, tetapi terblokir tool.

### 3.5 Dokumentasi belum sepenuhnya akurat

- `README.md` masih menyebut proyek belum memiliki kode.
- `docs/07-verification.md` mencantumkan bukti ngspice yang belum dijalankan.
- `docs/13-analysis.md` belum menyebut batas transient satu solves.

Prioritas: menengah, wajib sebelum release.

### 3.6 Release belum dieksekusi

- CI workflow belum dijalankan di GitHub.
- Tag dan publikasi belum dibuat.
- Packaging lokal sudah berhasil.

Prioritas: setelah correctness diperbaiki.

---

## 4. Analisis TODO

Sisa TODO lama seluruhnya terblokir tool atau keputusan:

| Blocker | Item |
|---|---|
| ngspice | P4-33, P4-34, P5-26, P10-21, X-10, X-11 |
| `avr-gcc` | P7-105, P9-14, P9-15, P9-18, P9-19 |
| macOS/Windows | P10-25, P10-26, P11-09, P11-10 |
| GitHub Actions | P11-08 |
| Keputusan rilis | P11-14, P11-15, P11-17 |
| simavr | P7-103 |

Karena itu, pekerjaan berikutnya harus berupa task baru yang tidak
terblokir.

---

## 5. Rencana P12

Urutan pekerjaan:

1. P12-01 sampai P12-04: adaptive initial state.
2. P12-05 sampai P12-14: Newton transient.
3. P12-15 sampai P12-17: koreksi dokumentasi.
4. P12-18 sampai P12-26: AC nonlinier.
5. P11 release setelah P12 stabil.

### 5.1 Adaptive initial state

- Tulis test RC discharge adaptive dengan initial voltage.
- Panggil `system.update` setelah `initial_solution`.
- Bandingkan fixed-step dan adaptive-step.
- Dokumentasikan perilaku initial state.

### 5.2 Newton transient

- Definisikan parameter Newton transient.
- Ekstrak helper damping dan convergence agar tidak duplikat.
- Tambah satu iterasi Newton per langkah.
- Tambah loop sampai konvergen.
- Kembalikan error konvergensi per langkah.
- Test diode rectifier.
- Test inverter switching dengan beban kapasitif.
- Test BJT common emitter transient.
- Dokumentasikan parameter dan batas.

### 5.3 Dokumentasi

- Perbarui status README.
- Perbaiki klaim ngspice di docs/07.
- Catat batas transient dan AC di docs/13 dan docs/24.

### 5.4 AC nonlinier

- Definisikan linearisasi small-signal.
- Implementasikan diode AC.
- Implementasikan MOSFET AC.
- Implementasikan BJT AC.
- Implementasikan JFET AC.
- Test diode resistance.
- Test common source gain.
- Test common emitter gain.
- Dokumentasikan AC nonlinier.

---

## 6. Kriteria Keberhasilan

- 728 test lama tetap lulus.
- Test baru untuk setiap perilaku.
- Tidak ada dependensi eksternal.
- Tidak ada `unsafe`.
- Fungsi maksimal 20 baris.
- Dokumentasi tidak overclaim.
- Release baru dilakukan setelah P12 selesai.
