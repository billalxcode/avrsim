---
name: Senior Writing
description: Draft and edit clear, decision-ready technical writing with strict ASCII-only output and no em dash or non-ASCII characters.
---

# Senior Writing

Gunakan skill ini saat:

- Menulis dokumen teknis, proposal, memo, RFC, laporan, atau email.
- Mengedit tulisan agar lebih jelas, ringkas, dan dapat diputuskan.
- Menyusun ringkasan eksekutif atau rekomendasi.
- Melakukan review tulisan orang lain.
- Menulis dokumentasi yang harus konsisten dan mudah dipindai.

---

## Aturan Mutlak: ASCII Saja

Semua output wajib hanya memakai karakter ASCII, yaitu U+0000 sampai U+007F. Jika karakter itu tidak ada di keyboard ASCII standar, jangan dipakai.

Dilarang keras:

- Em dash (U+2014), en dash (U+2013), horizontal bar (U+2015).
- Kutip kiri dan kanan, termasuk U+2018, U+2019, U+201C, U+201D.
- Karakter elipsis (U+2026).
- Panah Unicode (U+2190 sampai U+2199).
- Bullet Unicode (U+2022), checkmark, dan simbol dekoratif.
- Non-breaking space (U+00A0) dan spasi Unicode lainnya.
- Huruf beraksen atau karakter Latin di luar ASCII.
- Simbol seperti U+00B0, U+00B1, U+00B5, U+00D7, U+00A9, U+2122.
- Setiap karakter non-ASCII lain, tanpa pengecualian.

Ganti dengan ASCII:

| Dilarang | Gunakan |
|---|---|
| Em dash atau en dash | Hyphen, koma, titik dua, tanda kurung, atau tulis ulang kalimat |
| Kutip melengkung | Kutip lurus: ' dan " |
| Elipsis Unicode | Tiga titik: ... |
| Panah Unicode | Panah ASCII: -> atau kata "menjadi", "mengarah ke" |
| Bullet Unicode | Hyphen atau asterisk |
| Non-breaking space | Spasi biasa |
| Derajat | "deg" atau "derajat" |
| Plus-minus | "+/-" |
| Micro | "micro" atau "u" jika konteksnya jelas |
| Simbol hak cipta | "(c)" |

Aturan ini berlaku untuk:

- Prosa.
- Judul.
- Tabel.
- Kode dan contoh.
- Commit message.
- Pesan error.
- Komentar.
- File Markdown.
- Ringkasan jawaban.

Sebelum mengirim output, jalankan:

```bash
python3 .opencode/skills/senior-writing/scripts/check_ascii.py <file>
```

Atau untuk stdin:

```bash
printf '%s' "teks" | python3 .opencode/skills/senior-writing/scripts/check_ascii.py
```

Jika ada karakter non-ASCII, perbaiki dulu. Jangan mengirim output yang gagal.

---

## Standar Senior Writing

Tulisan senior:

1. Menjawab pertanyaan pembaca lebih dulu.
2. Menyatakan tujuan dan rekomendasi dengan jelas.
3. Memisahkan fakta, asumsi, dan opini.
4. Memberi bukti atau alasan.
5. Menghormati waktu pembaca.
6. Tidak memakai jargon tanpa definisi.
7. Tidak menyembunyikan ketidakpastian.
8. Dapat dipindai dalam 30 detik.
9. Dapat diputuskan oleh pembaca.
10. Tidak bertele-tele.

Prinsip:

- Tulis untuk pembaca tertentu, bukan untuk diri sendiri.
- Mulai dari kesimpulan, lalu jelaskan.
- Satu paragraf, satu ide.
- Satu kalimat, satu klaim utama.
- Aktifkan kata kerja.
- Hapus kata yang tidak menambah makna.
- Gunakan istilah yang konsisten.
- Tunjukkan contoh jika konsep abstrak.
- Batasi panjang kalimat.
- Jangan memakai hype.

---

## Workflow

### 1. Tentukan konteks

Jawab:

- Siapa pembacanya?
- Apa yang mereka butuhkan?
- Keputusan apa yang harus diambil?
- Apa batas waktu atau batas ruang?
- Apa yang sudah diketahui pembaca?

Jika konteks tidak jelas, tanyakan sebelum menulis panjang.

### 2. Buat outline

Minimal:

- Tujuan.
- Rekomendasi atau tesis.
- Alasan utama.
- Bukti.
- Risiko atau batasan.
- Langkah berikutnya.

Untuk dokumen pendek, outline bisa satu baris.

### 3. Tulis draft

- Mulai dari kesimpulan.
- Gunakan struktur yang sesuai.
- Tulis paragraf pendek.
- Masukkan angka dan contoh.
- Tandai asumsi.
- Jangan mengarang data.

### 4. Edit

Lakukan tiga pass:

1. **Struktur** - apakah urutan logis?
2. **Isi** - apakah klaim didukung?
3. **Gaya** - apakah kalimat jelas dan ringkas?

### 5. Audit ASCII

Jalankan `check_ascii.py`.

### 6. Review akhir

Gunakan `references/review-checklist.md`.

---

## Struktur Standar

### Ringkasan eksekutif

- Konteks: 1 kalimat.
- Temuan: 1 sampai 3 poin.
- Rekomendasi: 1 kalimat.
- Dampak: 1 kalimat.
- Langkah berikutnya: 1 kalimat.

### Dokumen teknis

- Tujuan.
- Latar belakang.
- Ruang lingkup.
- Keputusan atau rekomendasi.
- Alternatif.
- Risiko.
- Rencana implementasi.
- Pertanyaan terbuka.

### Review

- Ringkasan.
- Temuan dengan severity.
- Bukti.
- Perbaikan yang disarankan.
- Verifikasi.

### Email

- Tujuan di baris pertama.
- Konteks singkat.
- Permintaan atau keputusan.
- Batas waktu.
- Lampiran atau tautan.

---

## Bukti dan Ketidakpastian

- Bedakan "fakta", "asumsi", dan "opini".
- Sebut sumber untuk klaim yang tidak jelas.
- Jika data tidak tersedia, tulis "data belum tersedia".
- Jangan mengarang angka, tanggal, nama, atau kutipan.
- Gunakan rentang atau skenario jika angka pasti tidak ada.
- Tandai tingkat keyakinan: tinggi, sedang, rendah.
- Jelaskan apa yang akan mengubah rekomendasi.

---

## Gaya Bahasa

- Gunakan kalimat aktif.
- Hindari nominalisasi berlebihan.
- Hindari kata pengisi: "sangat", "cukup", "sebenarnya", "pada dasarnya".
- Hindari hedge berlebihan: "mungkin bisa jadi barangkali".
- Hindari jargon tanpa definisi.
- Hindari singkatan yang tidak dijelaskan.
- Gunakan istilah yang sama untuk konsep yang sama.
- Gunakan angka Arab.
- Gunakan format daftar hanya jika ada 3 item atau lebih.
- Jangan memakai bullet untuk satu item.
- Jangan memakai huruf kapital semua untuk penekanan.

---

## Output Contract

Setiap jawaban menulis harus:

1. ASCII-only.
2. Menyatakan tujuan.
3. Memberi struktur yang jelas.
4. Memisahkan fakta dan opini.
5. Menyebut asumsi.
6. Memberi langkah berikutnya.
7. Tidak memakai em dash atau karakter non-ASCII.
8. Lulus `check_ascii.py`.

Jika pengguna meminta gaya tertentu, ikuti selama tidak melanggar aturan ASCII.

---

## References

- `references/style-guide.md`
- `references/structures.md`
- `references/review-checklist.md`
- `references/ascii-policy.md`
- `templates/memo.md`
- `templates/technical-doc.md`
- `templates/review.md`
- `scripts/check_ascii.py`

---

## Prinsip Penutup

> Tulisan yang baik mengurangi kerja pembaca.
> Tulisan yang ringkas bukan berarti dangkal.
> Tulisan yang jujur menyebut batasannya.
