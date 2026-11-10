# Structures

Template struktur untuk berbagai jenis tulisan. Semua contoh ASCII-only.

---

## 1. Ringkasan Eksekutif

```
Tujuan:
[1 kalimat]

Temuan:
- [temuan 1]
- [temuan 2]
- [temuan 3]

Rekomendasi:
[1 kalimat]

Dampak:
[1 kalimat]

Langkah berikutnya:
[1 kalimat]
```

Aturan:

- Maksimal satu halaman.
- Kesimpulan di awal.
- Angka jika ada.
- Tidak ada latar belakang panjang.
- Tidak ada jargon tanpa definisi.

---

## 2. Memo

```
Kepada:
Dari:
Tanggal:
Perihal:

Ringkasan:
[1 sampai 2 kalimat]

Konteks:
[1 paragraf]

Analisis:
[2 sampai 4 paragraf]

Rekomendasi:
[1 paragraf]

Risiko:
[1 paragraf]

Langkah berikutnya:
[1 paragraf]
```

Gunakan memo untuk keputusan internal yang butuh jejak tertulis.

---

## 3. Dokumen Teknis

```
# Judul

## Tujuan
## Latar Belakang
## Ruang Lingkup
## Keputusan
## Alternatif yang Dipertimbangkan
## Risiko dan Batasan
## Rencana Implementasi
## Verifikasi
## Pertanyaan Terbuka
```

Aturan:

- Tujuan maksimal 3 kalimat.
- Keputusan harus eksplisit.
- Alternatif harus punya alasan ditolak.
- Risiko harus konkret.
- Rencana harus punya urutan dan pemilik.

---

## 4. RFC

```
# RFC: [judul]

## Ringkasan
## Motivasi
## Panduan Tingkat Tinggi
## Desain Detail
## Alternatif
## Dampak
## Rencana Migrasi
## Pertanyaan Terbuka
```

Aturan:

- Ringkasan harus bisa dibaca tanpa bagian lain.
- Desain detail harus cukup untuk implementasi.
- Alternatif harus jujur.
- Dampak harus mencakup biaya dan risiko.

---

## 5. Review

```
# Review: [objek]

## Ringkasan
## Temuan
### [Severity] [lokasi] - [judul]
**Masalah:**
**Dampak:**
**Perbaikan:**
## Verifikasi
## Catatan
```

Severity:

- Blocker.
- High.
- Medium.
- Low.
- Nit.

Aturan:

- Setiap temuan punya lokasi.
- Setiap temuan punya dampak.
- Setiap temuan punya saran.
- Jangan menulis "buruk" tanpa alasan.

---

## 6. Email

```
Subjek: [tujuan atau keputusan]

[1 kalimat tujuan]

[1 paragraf konteks]

[1 paragraf permintaan atau rekomendasi]

[1 kalimat tenggat]

[Lampiran atau tautan]
```

Aturan:

- Tujuan di baris pertama.
- Jangan menyembunyikan permintaan di akhir.
- Sebut tenggat dengan tanggal.
- Jangan memakai huruf kapital semua.

---

## 7. Laporan Insiden

```
# Insiden: [judul]

## Ringkasan
## Dampak
## Garis Waktu
## Penyebab
## Tindakan Perbaikan
## Pencegahan
## Pelajaran
```

Aturan:

- Tulis fakta, bukan spekulasi.
- Pisahkan penyebab langsung dan akar masalah.
- Tindakan harus punya pemilik dan tanggal.
- Jangan menyalahkan individu.

---

## 8. Catatan Keputusan

```
# Keputusan: [judul]

Tanggal:
Status: proposed, accepted, rejected, superseded

## Konteks
## Keputusan
## Konsekuensi
## Alternatif
## Referensi
```

Aturan:

- Satu keputusan per catatan.
- Status harus jelas.
- Konsekuensi harus mencakup yang negatif.
- Referensi harus dapat dilacak.

---

## 9. Dokumentasi API

```
# [Nama API]

## Tujuan
## Autentikasi
## Endpoint
### [METHOD] [path]
Deskripsi:
Parameter:
Response:
Error:
Contoh:
```

Aturan:

- Setiap endpoint punya contoh.
- Setiap error punya kode dan arti.
- Jangan mengarang perilaku.
- Tandai bagian yang belum stabil.

---

## 10. Ringkasan Keputusan untuk Eksekutif

```
Keputusan yang diminta:
[1 kalimat]

Pilihan:
- Opsi A: [biaya, manfaat, risiko]
- Opsi B: [biaya, manfaat, risiko]

Rekomendasi:
[1 kalimat]

Alasan:
- [alasan 1]
- [alasan 2]

Jika tidak ada keputusan:
[konsekuensi]
```

Aturan:

- Maksimal satu halaman.
- Fokus pada keputusan.
- Jangan memulai dengan sejarah.
- Sertakan konsekuensi jika ditunda.

---

## 11. Pola Kalimat yang Berguna

- "Tujuannya adalah ..."
- "Keputusannya adalah ..."
- "Kami merekomendasikan ... karena ..."
- "Risiko utamanya adalah ..."
- "Asumsi yang dipakai adalah ..."
- "Data yang tersedia menunjukkan ..."
- "Data belum tersedia untuk ..."
- "Langkah berikutnya adalah ... pada [tanggal]."
- "Jika [kondisi], maka [konsekuensi]."
- "Kami belum memutuskan [hal] karena [alasan]."

---

## 12. Pola yang Harus Dihindari

- "Sehubungan dengan hal tersebut di atas."
- "Demikian kami sampaikan."
- "Untuk itu, maka."
- "Sangat jelas bahwa."
- "Semua orang tahu."
- "Tidak diragukan lagi."
- "Akan dilakukan optimalisasi."
- "Segera."
- "Banyak."
- "Signifikan" tanpa angka.

---

## 13. Format ASCII

- Gunakan hyphen (-) untuk bullet.
- Gunakan `->` untuk panah.
- Gunakan `...` untuk elipsis.
- Gunakan kutip lurus.
- Gunakan `deg` untuk derajat.
- Gunakan `+/-` untuk plus-minus.
- Jangan memakai em dash.
- Jangan memakai karakter non-ASCII.
