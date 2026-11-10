# Clean Code Principles

Referensi ringkas. Untuk detail per bab, lihat `.caches/clean-architecture/clean-code-by-chapter/`.

---

## 1. Nama

- Nama harus mengungkap maksud.
- Hindari disinformasi.
- Buat pembedaan yang bermakna.
- Gunakan nama yang dapat diucapkan dan dicari.
- Hindari encoding tipe/prefix.
- Gunakan kata benda untuk tipe, kata kerja untuk fungsi.
- Satu kata untuk satu konsep.
- Jangan punning.
- Gunakan nama domain.
- Tambahkan konteks secukupnya; jangan berlebihan.

Contoh buruk:

```rust
fn calc(a: u64, b: u64) -> u64 { a - b }
```

Contoh baik:

```rust
fn remaining_time_ns(total_ns: u64, elapsed_ns: u64) -> u64 {
    total_ns.saturating_sub(elapsed_ns)
}
```

---

## 2. Fungsi

### 2.1 Guard Clause

- Validasi precondition di awal.
- Return atau `Err` lebih awal.
- Hindari `else` setelah `return`.
- Happy path di akhir dengan indentasi minimal.
- Setiap cabang error harus jelas.

Contoh:

```rust
fn parse_limit(raw: &str) -> Result<usize, ParseError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(ParseError::Empty);
    }
    let value = trimmed.parse::<usize>()?;
    if value == 0 {
        return Err(ParseError::Zero);
    }
    Ok(value)
}
```

Hindari:

```rust
fn parse_limit(raw: &str) -> Result<usize, ParseError> {
    if !raw.trim().is_empty() {
        let value = raw.trim().parse::<usize>()?;
        if value != 0 {
            Ok(value)
        } else {
            Err(ParseError::Zero)
        }
    } else {
        Err(ParseError::Empty)
    }
}
```

### 2.2 Maksimal 20 Baris

- Hitung dari baris `fn` sampai `}` penutup.
- Doc comment dan attribute di atas `fn` tidak dihitung.
- Blank line di dalam body dihitung.
- Test helper, adapter, dan fungsi privat juga berlaku.
- Jika lebih dari 20 baris, pecah.

Cara memecah:

1. Ekstrak validasi dan guard clause.
2. Ekstrak body loop.
3. Ekstrak error mapping.
4. Ekstrak formatting.
5. Ekstrak satu langkah domain.

### 2.3 Fungsi Terkecil yang Reusable

- Satu fungsi, satu langkah.
- Nama fungsi menjelaskan langkah, bukan implementasi.
- Pure function lebih baik jika memungkinkan.
- Parameter sedikit; gunakan struct jika perlu.
- Kembalikan nilai yang jelas.
- Jangan membuat wrapper satu baris tanpa makna.
- Jangan over-fragmentasi sampai alur sulit dibaca.

### 2.4 Cari Sebelum Membuat

- Cari dengan `rg "fn nama"` atau `rg "nama"`.
- Cek modul tetangga.
- Cek trait dan implementasinya.
- Cek test utility.
- Cek helper yang sudah ada.
- Reuse, perluas, atau parameterisasi.
- Jangan membuat duplikat dekat.

### 2.5 Jangan Membuat Fungsi yang Sudah Ada

- Perilaku sama: reuse.
- Perilaku mirip: parameterisasi atau refactor fungsi lama.
- Perilaku berbeda: nama harus membedakan.
- Jangan membuat `foo2`, `foo_new`, atau `helper_foo`.
- Duplikasi logika adalah pelanggaran.

### 2.6 Pertanyaan

- Apakah ada guard clause?
- Apakah fungsi <= 20 baris?
- Apakah fungsi melakukan satu hal?
- Apakah ada fungsi existing yang bisa dipakai?
- Apakah ada duplikasi?
- Apakah ada lebih dari 3 parameter?
- Apakah ada boolean flag?
- Apakah ada side effect tersembunyi?
- Apakah fungsi mengubah state dan mengembalikan nilai?

---

## 3. Komentar

- Komentar tidak memperbaiki kode buruk.
- Jelaskan "mengapa", bukan "apa".
- Jangan menduplikasi kode.
- Jangan misleading.
- Jangan meninggalkan kode mati.
- Gunakan untuk:
  - Alasan hukum.
  - Maksud yang tidak jelas.
  - Peringatan konsekuensi.
  - TODO yang jelas.
  - Dokumentasi API publik.
- Dokumentasi Rust:
  - `///` untuk item publik.
  - `//!` untuk modul/crate.
  - Contoh kode di doc comment jika membantu.

---

## 4. Formatting

- Gunakan `rustfmt`.
- Dekatkan konsep terkait.
- Pisahkan konsep dengan blank line.
- Jaga file tetap fokus.
- Baris tidak terlalu panjang.
- Indentasi konsisten.
- Aturan tim lebih penting daripada selera pribadi.
- Jangan reformat file besar tanpa alasan.

---

## 5. Object dan Data Structure

- Object: menyembunyikan data, mengekspos behavior.
- Data structure: mengekspos data, tanpa behavior.
- Jangan mencampur keduanya sembarangan.
- Law of Demeter: hindari rantai `a.b().c().d()`.
- DTO sederhana untuk boundary.
- Jangan membocorkan tipe internal solver ke CLI.

---

## 6. Error Handling

- Gunakan `Result<T, E>` dan `Option<T>`.
- Jangan `unwrap()` pada input pengguna.
- `expect()` hanya untuk invariant yang benar-benar tidak mungkin gagal, dengan pesan jelas.
- Berikan konteks error.
- Definisikan error domain.
- Pisahkan error domain dari error presentasi.
- Jangan menelan error.
- Jangan memakai `String` mentah sebagai satu-satunya error type.
- Propagasi dengan `?`.
- Untuk error yang butuh konteks, bungkus error sumber.

---

## 7. Boundaries

- Isolasi pihak ketiga di adapter.
- Jangan biarkan tipe eksternal menyebar.
- Tulis learning test untuk library baru.
- Jangan biarkan format file/CLI masuk ke core.
- Boundary harus bisa diuji tanpa I/O.

---

## 8. Unit Tests

- Test adalah kode production.
- F.I.R.S.T:
  - Fast.
  - Independent.
  - Repeatable.
  - Self-validating.
  - Timely.
- Satu konsep per test.
- Nama test menjelaskan perilaku.
- Test jangan bergantung pada urutan.
- Test jangan membutuhkan filesystem/network jika bisa dihindari.
- Test harus deterministik.
- Jangan memakai sleep.
- Jangan menguji implementasi; uji perilaku.

---

## 9. Class/Modul

- Kecil.
- Kohesif.
- Satu tanggung jawab.
- Sedikit dependensi.
- Isolasi dari perubahan.
- Bergantung pada abstraksi bila ada variasi.
- Jangan membuat modul "serba bisa".

---

## 10. Systems

- Pisahkan konstruksi dari penggunaan.
- `main` adalah komposisi.
- Dependency injection lewat parameter/generic.
- Jangan memakai global state.
- Jangan memakai framework sebagai arsitektur.

---

## 11. Emergence

1. Jalankan semua test.
2. Hilangkan duplikasi.
3. Ekspresikan maksud.
4. Minimalkan jumlah tipe/fungsi.

Jangan menambah abstraksi sebelum perlu.

---

## 12. Concurrency

- Hindari concurrency sampai benar-benar perlu.
- Simulasi harus deterministik.
- Isolasi state mutable.
- Jangan memakai wall-clock.
- Jika memakai thread, gunakan channel/mutex dengan benar.
- Uji concurrency; anggap kegagalan sporadis sebagai bug.

---

## 13. Smells

### Komentar
- Redundant.
- Misleading.
- Kode mati.
- Noise.

### Fungsi
- Terlalu panjang.
- Terlalu banyak argumen.
- Flag argument.
- Side effect.
- Duplikasi.
- Nama buruk.

### Nama
- Singkatan misterius.
- Encoding.
- Punning.
- Tidak konsisten.

### Test
- Tidak ada test.
- Test lambat.
- Test tidak independen.
- Test menguji implementasi.

### Umum
- Duplikasi.
- Magic number.
- Dead code.
- Coupling berlebih.
- Global mutable state.
- Premature optimization.
- Abstraksi tanpa variasi.

---

## 14. Kaitannya dengan Clean Architecture

Clean Code adalah level bata; Clean Architecture adalah level bangunan.

- Nama/fungsi/test yang buruk tidak diselamatkan oleh boundary.
- Boundary yang buruk tidak diselamatkan oleh kode yang rapi.
- Keduanya diperlukan.

Lihat `references/architecture.md` untuk dependency rule dan boundary.
