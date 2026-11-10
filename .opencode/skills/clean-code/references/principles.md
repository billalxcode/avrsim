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

- Kecil.
- Melakukan satu hal.
- Satu level abstraksi.
- Sedikit argumen (0–3).
- Hindari flag argument.
- Hindari side effect tersembunyi.
- Command-Query Separation.
- Prefer error/exception daripada kode error.
- Ekstrak blok `try`/penanganan error.
- DRY.
- Nama menjelaskan maksud.

Pertanyaan:

- Apakah fungsi > 20–30 baris? Pertimbangkan pecah.
- Apakah ada lebih dari 3 parameter? Pertimbangkan struct.
- Apakah ada boolean yang mengubah jalur? Pisahkan fungsi.
- Apakah fungsi mengubah state dan mengembalikan nilai? Pisahkan.

---

## 3. Komentar

- Komentar tidak memperbaiki kode buruk.
- Jelaskan “mengapa”, bukan “apa”.
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
- Jangan membuat modul “serba bisa”.

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
