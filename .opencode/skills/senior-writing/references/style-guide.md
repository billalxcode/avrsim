# Style Guide

Panduan gaya untuk tulisan senior. Semua contoh dan aturan di sini memakai ASCII saja.

---

## 1. Pembaca dan Tujuan

Sebelum menulis, tentukan:

- Pembaca utama.
- Pengetahuan awal mereka.
- Keputusan yang ingin mereka ambil.
- Batas waktu atau panjang.
- Format yang mereka harapkan.

Tulisan yang baik menjawab pertanyaan pembaca, bukan pertanyaan penulis.

---

## 2. Struktur Paragraf

- Satu paragraf, satu ide.
- Kalimat pertama paragraf menyatakan ide utama.
- Kalimat berikutnya mendukung, menjelaskan, atau memberi contoh.
- Panjang paragraf ideal 2 sampai 5 kalimat.
- Hindari paragraf satu kalimat berulang tanpa alasan.
- Hindari paragraf lebih dari 8 kalimat.

---

## 3. Kalimat

- Satu kalimat, satu klaim utama.
- Panjang ideal 12 sampai 25 kata.
- Jika lebih dari 35 kata, pertimbangkan pecah.
- Letakkan informasi penting di awal.
- Hindari anak kalimat bertingkat.
- Hindari "yang yang".
- Hindari kata ganti yang ambigu.
- Gunakan kata kerja aktif.
- Hindari nominalisasi berlebihan.

Contoh:

Buruk:

```
Pengambilan keputusan mengenai implementasi arsitektur dilakukan oleh tim.
```

Baik:

```
Tim memutuskan arsitektur yang akan diimplementasikan.
```

---

## 4. Ringkas

Hapus:

- Kata pengisi: sangat, cukup, sebenarnya, pada dasarnya, tentu saja.
- Hedge berlebihan: mungkin, bisa jadi, barangkali, sepertinya.
- Pengulangan konsep.
- Kalimat yang hanya mengulang judul.
- Basa-basi yang tidak menambah informasi.
- Jargon yang tidak dipakai pembaca.

Jangan menghapus:

- Batasan.
- Angka.
- Sumber.
- Peringatan.
- Asumsi.

---

## 5. Nada

- Langsung.
- Hormat.
- Tenang.
- Tidak defensif.
- Tidak sloganis.
- Tidak menggurui.
- Tidak memakai hype.

Hindari:

- "Revolusioner".
- "Terbaik di kelasnya".
- "Tidak diragukan lagi".
- "Semua orang tahu".
- "Jelas sekali".

---

## 6. Istilah

- Gunakan istilah yang sama untuk konsep yang sama.
- Definisikan istilah baru saat pertama muncul.
- Hindari sinonim bergantian untuk konsep teknis.
- Hindari singkatan tanpa definisi.
- Gunakan istilah domain yang dikenal pembaca.
- Jika memakai istilah asing, jelaskan sekali.

---

## 7. Angka dan Satuan

- Gunakan angka Arab.
- Tulis satuan dengan jelas.
- Gunakan "ns", "us", "ms", "s" untuk waktu.
- Gunakan "Hz", "kHz", "MHz" untuk frekuensi.
- Gunakan "V", "A", "W" untuk listrik.
- Gunakan "deg C" untuk suhu.
- Gunakan "+/-" untuk toleransi.
- Gunakan "x" untuk perkalian.
- Jangan memakai simbol non-ASCII.

---

## 8. Daftar

Gunakan daftar jika:

- Ada 3 item atau lebih.
- Urutan penting.
- Item perlu dipindai cepat.

Jangan gunakan daftar jika:

- Hanya satu item.
- Prosa lebih jelas.
- Item sangat panjang dan saling bergantung.

Gunakan hyphen (-) untuk bullet. Jangan memakai bullet Unicode.

---

## 9. Tabel

- Gunakan tabel untuk perbandingan.
- Maksimal 5 kolom jika memungkinkan.
- Setiap kolom punya makna jelas.
- Jangan memakai tabel untuk prosa panjang.
- Sel kosong harus disengaja.

---

## 10. Kode dan Perintah

- Gunakan blok kode dengan bahasa.
- Jelaskan perintah yang tidak umum.
- Tulis nama file dengan path lengkap jika relevan.
- Jangan memakai karakter non-ASCII di kode.
- Untuk waktu, gunakan "us" atau "microsecond", bukan simbol micro.

---

## 11. Bukti

- Setiap klaim penting harus punya dasar.
- Sebut sumber data.
- Pisahkan fakta, asumsi, dan opini.
- Tandai ketidakpastian.
- Jelaskan batasan.
- Jangan mengarang angka.
- Jika data tidak ada, tulis "data belum tersedia".

---

## 12. Tiga Pass Editing

### Pass 1 - Struktur

- Apakah urutan logis?
- Apakah bagian penting terlalu jauh di belakang?
- Apakah ada bagian yang tidak perlu?
- Apakah kesimpulan muncul lebih dulu?

### Pass 2 - Isi

- Apakah setiap klaim didukung?
- Apakah ada asumsi tersembunyi?
- Apakah ada data yang hilang?
- Apakah rekomendasi dapat ditindaklanjuti?

### Pass 3 - Gaya

- Apakah kalimat terlalu panjang?
- Apakah ada kata pengisi?
- Apakah istilah konsisten?
- Apakah ada karakter non-ASCII?

---

## 13. Kesalahan Umum

- Menulis latar belakang terlalu panjang sebelum kesimpulan.
- Menyembunyikan rekomendasi di akhir.
- Memakai jargon tanpa definisi.
- Menyebut "kami" tanpa menjelaskan siapa.
- Menyalin template tanpa menyesuaikan konteks.
- Menulis "akan dilakukan" tanpa pelaku.
- Menyebut "segera" tanpa tanggal.
- Menyebut "banyak" tanpa angka.
- Menyebut "optimal" tanpa ukuran.
- Memakai em dash atau karakter non-ASCII.

---

## 14. Pengganti ASCII

| Jika ingin memakai | Tulis |
|---|---|
| Em dash | Hyphen, koma, titik dua, atau tulis ulang |
| En dash | Hyphen atau "sampai" |
| Kutip melengkung | Kutip lurus |
| Elipsis Unicode | ... |
| Panah Unicode | -> |
| Bullet Unicode | - |
| Derajat | deg |
| Plus-minus | +/- |
| Micro | micro atau u |
| Hak cipta | (c) |

---

## 15. Contoh Sebelum dan Sesudah

Sebelum:

```
Sehubungan dengan hal tersebut di atas, maka dapat disampaikan bahwa
implementasi yang akan dilakukan pada tahap berikutnya adalah
melakukan optimalisasi pada modul analisis.
```

Sesudah:

```
Tahap berikutnya mengoptimalkan modul analisis.
```

Sebelum:

```
Kami sangat yakin bahwa pendekatan ini merupakan pendekatan terbaik
dan tidak diragukan lagi akan memberikan hasil yang signifikan.
```

Sesudah:

```
Kami merekomendasikan pendekatan ini karena mengurangi waktu simulasi
sekitar 30 persen pada data uji kami.
```
