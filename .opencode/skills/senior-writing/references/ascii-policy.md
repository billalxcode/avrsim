# ASCII Policy

Kebijakan ini berlaku untuk semua output yang dihasilkan dengan skill `senior-writing`.

---

## 1. Definisi

ASCII adalah karakter dengan kode U+0000 sampai U+007F.

Untuk teks, karakter yang diizinkan adalah:

- Huruf A sampai Z dan a sampai z.
- Angka 0 sampai 9.
- Spasi.
- Tanda baca ASCII: ! " # $ % & ' ( ) * + , - . / : ; < = > ? @ [ \ ] ^ _ ` { | } ~
- Newline dan tab.

Karakter di luar itu dilarang, tanpa pengecualian.

---

## 2. Daftar Terlarang

| Kategori | Contoh kode | Gunakan |
|---|---|---|
| Em dash | U+2014 | Hyphen, koma, titik dua, atau tulis ulang |
| En dash | U+2013 | Hyphen atau "sampai" |
| Horizontal bar | U+2015 | Hyphen |
| Kutip kiri tunggal | U+2018 | ' |
| Kutip kanan tunggal | U+2019 | ' |
| Kutip kiri ganda | U+201C | " |
| Kutip kanan ganda | U+201D | " |
| Elipsis | U+2026 | ... |
| Bullet | U+2022 | - |
| Panah kiri | U+2190 | <- |
| Panah kanan | U+2192 | -> |
| Panah dua arah | U+2194 | <-> |
| Non-breaking space | U+00A0 | Spasi biasa |
| Derajat | U+00B0 | deg |
| Plus-minus | U+00B1 | +/- |
| Micro | U+00B5 | micro atau u |
| Kali | U+00D7 | x |
| Hak cipta | U+00A9 | (c) |
| Trademark | U+2122 | (tm) |
| Checkmark | U+2713 | [x] |
| Silang | U+2717 | [ ] |
| Setiap huruf beraksen | U+00C0 dan seterusnya | Huruf ASCII terdekat atau tulis ulang |

Daftar ini bukan daftar lengkap. Aturannya adalah: jika bukan ASCII, jangan dipakai.

---

## 3. Penggantian dalam Praktik

### Em dash

Jangan memakai em dash. Pilih salah satu:

- Hyphen: "rencana - termasuk anggaran - sudah disetujui"
- Koma: "rencana, termasuk anggaran, sudah disetujui"
- Titik dua: "ada tiga opsi: A, B, dan C"
- Tanda kurung: "rencana (termasuk anggaran) sudah disetujui"
- Tulis ulang: "anggaran sudah termasuk dalam rencana"

### Rentang

- Gunakan "sampai": "10 sampai 20 ms"
- Atau hyphen: "10-20 ms"

### Kutipan

- Gunakan kutip lurus: "teks" dan 'teks'.
- Jangan memakai kutip melengkung dari word processor.
- Jika menyalin kutipan yang mengandung karakter non-ASCII, parafrase atau ganti dengan ASCII. Jangan menyalin karakter mentah.

### Panah

- Gunakan `->` untuk "menjadi" atau "mengarah ke".
- Gunakan `<-` untuk arah sebaliknya.
- Gunakan kata jika lebih jelas: "berubah menjadi", "menghasilkan".

### Simbol

- Derajat: "85 deg C".
- Plus-minus: "5 +/- 0.2 V".
- Micro: "10 us" atau "10 microsecond".
- Kali: "3 x 4".
- Hak cipta: "(c) 2026".
- Trademark: "Foo (tm)".

---

## 4. Sumber yang Mengandung Non-ASCII

Jika harus mengutip sumber yang mengandung karakter non-ASCII:

1. Parafrase dalam ASCII.
2. Atau jelaskan karakter tersebut dengan nama dan kode: "tanda U+2014".
3. Jangan menyalin karakter mentah ke output.
4. Jangan mengubah makna kutipan.

Contoh:

```
Sumber memakai tanda pisah panjang; di sini ditulis sebagai hyphen
sesuai kebijakan ASCII.
```

---

## 5. Verifikasi

Script:

```bash
python3 .opencode/skills/senior-writing/scripts/check_ascii.py <file>
```

Atau stdin:

```bash
printf '%s' "teks" | python3 .opencode/skills/senior-writing/scripts/check_ascii.py
```

Script akan menampilkan:

- Nama file.
- Nomor baris.
- Nomor kolom.
- Kode karakter.
- Representasi karakter.

Exit code:

- 0 jika semua ASCII.
- 1 jika ada non-ASCII atau file tidak ditemukan.

---

## 6. File dan Encoding

- File boleh disimpan sebagai UTF-8, selama isinya ASCII.
- Jangan menyimpan BOM.
- Jangan memakai smart quotes dari editor.
- Jangan memakai karakter non-ASCII di nama file.
- Jangan memakai karakter non-ASCII di nama direktori.
- Jangan memakai karakter non-ASCII di commit message.

---

## 7. Pengecualian

Tidak ada pengecualian untuk output yang dihasilkan dengan skill ini.

Jika pengguna secara eksplisit meminta karakter non-ASCII, jelaskan konflik kebijakan dan minta konfirmasi. Jangan melanggar aturan tanpa persetujuan eksplisit.

---

## 8. Kegagalan

Jika audit ASCII gagal:

1. Perbaiki karakter yang dilaporkan.
2. Jalankan ulang script.
3. Jangan mengirim output sebelum lulus.
4. Jika karakter berasal dari data pengguna, jelaskan dan tawarkan versi ASCII.

---

## 9. Ringkasan

- ASCII saja.
- Tanpa em dash.
- Tanpa kutip melengkung.
- Tanpa elipsis Unicode.
- Tanpa panah Unicode.
- Tanpa bullet Unicode.
- Tanpa spasi non-breaking.
- Selalu jalankan `check_ascii.py`.
