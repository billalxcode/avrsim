# 09 — Risiko dan Realisme

Dokumen ini adalah pemeriksaan jujur terhadap visi, bukan daftar alasan untuk berhenti.

---

## 1. Ringkasan

Scope yang dipilih adalah scope maksimal:

- Seluruh ATmega328P transistor-level.
- Seluruh peripheral struktural.
- Pustaka SPICE lengkap.
- Seluruh analisis SPICE.
- Seluruh IC aktif board struktural.
- Solo developer.
- Zero external dependency.
- Cross-platform.
- Kriteria sukses fungsional + timing + elektrik.

Ini setara **program riset multi-person-decade**. Proyek tetap bisa dimulai, tetapi harus dijalankan sebagai rangkaian milestone kecil dengan ekspektasi yang jelas.

---

## 2. Risk Register

| ID | Risiko | Probabilitas | Dampak | Mitigasi |
|---|---|---|---|---|
| R01 | Scope terlalu besar | Tinggi | Proyek tidak selesai | Milestone kecil, phase gate, fidelity matrix |
| R02 | Simulasi terlalu lambat | Tinggi | Tidak praktis | Offline target, mulai rangkaian kecil, optimasi bertahap |
| R03 | Konvergensi Newton gagal | Sedang | Simulasi berhenti | Limiting, gmin, source stepping, damping |
| R04 | Model MOSFET tidak akurat | Sedang | Hasil analog menyimpang | Level 1 untuk digital; dokumentasikan batasan |
| R05 | Kalibrasi tanpa lab tidak valid | Tinggi | Klaim elektrik lemah | Kalibrasi pin-level datasheet; toleransi; fidelity matrix |
| R06 | Mixed-signal synchronization kompleks | Tinggi | Bug halus | Breakpoint, interpolasi, delta cycle, test ketat |
| R07 | Timing 16 MHz tidak tercapai | Sedang | Firmware timing salah | Ukuran transistor, pipeline, verifikasi delay |
| R08 | ISA edge case | Sedang | Firmware hang | Test vector, differential simavr |
| R09 | Peripheral structural terlalu besar | Tinggi | Milestone molor | Urutan bertahap; GPIO/timer/UART dulu |
| R10 | Zero dependency memperlambat | Sedang | Development lambat | Workspace modular; alat dev eksternal untuk test |
| R11 | Cross-platform floating-point | Sedang | Golden test gagal | Toleransi, bukan bit-identik |
| R12 | Lisensi/model pihak ketiga | Rendah | Masalah hukum | Clean-room, PTM publik, dokumentasi lisensi |
| R13 | Data golden susah direproduksi | Sedang | Regresi tidak terdeteksi | Simpan metadata dan skrip regenerasi |
| R14 | Dokumentasi tertinggal | Sedang | Proyek sulit dilanjutkan | Dokumentasi bersama kode; phase gate |
| R15 | Burnout solo | Sedang | Proyek berhenti | Milestone kecil, hasil nyata tiap fase |

---

## 3. Realisme Teknis

### 3.1 Kecepatan

Pada 16 MHz:

- Satu siklus = 62.5 ns.
- Time step 0.1 ns → 625 step per siklus.
- Satu instruksi bisa melibatkan ribuan node.
- Setiap step bisa membutuhkan beberapa iterasi Newton.
- Setiap iterasi menyelesaikan sistem linear.

Konsekuensi:

- Satu detik firmware bisa memakan menit, jam, atau lebih.
- “Blink 1 detik” secara struktural bukan demo cepat.
- Target offline sudah tepat.

### 3.2 Jumlah Node

- Register file 32×8 DFF → 256 flip-flop, ribuan transistor.
- ALU 8-bit ripple-carry → ratusan transistor.
- Control PLA → ribuan gerbang.
- Peripheral structural → ribuan hingga puluhan ribu transistor.
- SRAM/flash behavioral → menghindari ledakan node.

### 3.3 Akurasi

- Shichman-Hodges Level 1 tidak akurat untuk short-channel.
- Meyer tidak konservasi muatan.
- Tanpa lab, kalibrasi internal tidak mungkin.
- “100% elektrik” tidak dapat diverifikasi; hanya “dalam toleransi pada pin”.

---

## 4. Kontradiksi yang Sudah Dijembatani

| Kontradiksi | Resolusi |
|---|---|
| Seluruh MCU structural vs SRAM/flash behavioral | Pengecualian fidelity eksplisit |
| Semua peripheral structural vs milestone GPIO/timer/UART | Target akhir vs milestone |
| Semua analisis SPICE vs milestone DC/transient | Target akhir vs milestone |
| Kalibrasi chip asli vs tanpa lab | Kalibrasi pin-level datasheet |
| Power ideal vs reset structural | Tahap primitif vs tahap lanjut |
| Seluruh board transistor-level vs host UART behavioral | Host di luar board |
| Zero dependency vs workspace | Crate internal boleh, eksternal tidak |
| 100% semua aspek vs keterbatasan verifikasi | Fidelity matrix + toleransi |

---

## 5. Guardrail Scope

Aturan yang harus dijaga:

1. **Satu milestone, satu hasil nyata.**
2. **Tidak menambah peripheral baru sebelum yang lama lulus test.**
3. **Tidak menambah analisis baru sebelum solver stabil.**
4. **Tidak mengejar real-time.**
5. **Tidak mengklaim 100% tanpa fidelity matrix.**
6. **Tidak menambah dependensi eksternal.**
7. **Tidak memulai GUI sebelum CLI stabil.**
8. **Tidak memulai board penuh sebelum MCU inti berjalan.**
9. **Tidak mengoptimasi sebelum profiling.**
10. **Tidak menghapus test untuk mempercepat build.**

---

## 6. Decision Triggers

Kapan harus mengubah rencana:

| Trigger | Tindakan |
|---|---|
| M1 tidak selesai dalam waktu wajar | Perkecil scope solver; fokus DC saja |
| Konvergensi terus gagal | Tambah gmin/source stepping; sederhanakan device |
| Simulasi blink terlalu lambat | Pertimbangkan model behavioral untuk SRAM/flash, optimasi flatten |
| Timing 16 MHz tidak tercapai | Evaluasi ukuran transistor, pipeline, atau clock lebih rendah |
| Peripheral structural terlalu besar | Prioritaskan GPIO/timer/UART; tunda SPI/I2C/ADC |
| Kalibrasi tidak mungkin | Dokumentasikan batasan; fokus fungsional |
| Solo burnout | Berhenti di milestone yang stabil; publikasikan hasil |
| Zero dependency menghambat | Evaluasi ulang; tetapi keputusan saat ini tetap zero |

---

## 7. Legal dan Etika

- Netlist internal adalah clean-room.
- Tidak memakai netlist proprietary.
- Tidak mendistribusikan ROM/BIOS/firmware proprietary.
- Datasheet digunakan sebagai referensi, bukan disalin mentah.
- Model PTM publik digunakan sesuai lisensi.
- Lisensi proyek MIT/Apache-2.0.
- Kode dan dokumentasi menyebut sumber data.

---

## 8. Definisi Berhasil yang Realistis

Keberhasilan tidak harus berarti “seluruh visi selesai”. Beberapa definisi yang valid:

1. **M1 selesai** — solver dan inverter VTC terbukti.
2. **M2 selesai** — pustaka CMOS dan ring oscillator berjalan.
3. **M3 selesai** — CPU uji kecil berjalan.
4. **M4 selesai** — blink AVR berjalan di CPU struktural.
5. **M5 selesai** — blink + UART hello.
6. **M7 selesai** — board demo end-to-end.
7. **M8 selesai** — full SPICE.

Setiap level adalah hasil yang bernilai, bahkan jika level berikutnya belum tercapai.

---

## 9. Kesimpulan

Visi ini **layak sebagai program riset jangka panjang**, tetapi **tidak realistis sebagai proyek solo dalam waktu singkat** jika seluruh scope maksimal dipertahankan.

Rekomendasi:

- Mulai dari M1.
- Jangan menunggu seluruh sistem selesai untuk mendapatkan hasil.
- Dokumentasikan setiap batasan.
- Perlakukan fidelity sebagai properti yang diukur, bukan slogan.
- Rayakan setiap milestone yang lulus test.

Yang paling penting: **jangan mengorbankan kebenaran fondasi demi mengejar cakupan**.
