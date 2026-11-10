# Review dan Refactor Playbook

Panduan langkah demi langkah untuk review dan refactor.

---

## 1. Jenis Pekerjaan

| Jenis | Fokus |
|---|---|
| Review PR | Temukan masalah, jangan menulis ulang seluruh kode |
| Refactor | Perbaiki struktur tanpa mengubah perilaku |
| Feature | Tambah perilaku + test |
| Bug fix | Perbaiki perilaku + test regresi |
| Architecture review | Boundary, dependensi, testability |

Jangan mencampur refactor dan feature dalam satu perubahan besar.

---

## 2. Review Workflow

### Langkah 1 — Pahami tujuan

- Apa yang ingin dicapai perubahan ini?
- Apakah ada issue/PR description?
- Apakah ada test yang menyertainya?

### Langkah 2 — Baca diff, bukan hanya file

- Apa yang berubah?
- Apa yang tidak berubah tetapi seharusnya?
- Apakah ada perubahan tak terkait?

### Langkah 3 — Cek correctness

- Apakah logika benar?
- Apakah edge case ditangani?
- Apakah error ditangani?
- Apakah ada panic/unwrap?
- Apakah ada overflow/underflow?
- Apakah ada race/global state?

### Langkah 4 — Cek desain

- Apakah fungsi/modul terlalu besar?
- Apakah nama jelas?
- Apakah ada duplikasi?
- Apakah boundary bocor?
- Apakah abstraksi prematur?
- Apakah dependensi eksternal ditambahkan?

### Langkah 5 — Cek test

- Apakah test menguji perilaku?
- Apakah test independen?
- Apakah test deterministik?
- Apakah edge case diuji?
- Apakah test akan gagal jika perilaku rusak?

### Langkah 6 — Jalankan verifikasi

```bash
bash .opencode/skills/clean-code/scripts/check.sh
```

### Langkah 7 — Tulis temuan

Gunakan format di bawah. Jangan menulis “kode ini buruk” tanpa alasan dan perbaikan.

---

## 3. Format Temuan

```
[Severity] file:line — judul singkat

Masalah:
...

Dampak:
...

Perbaikan yang disarankan:
...

Contoh (opsional):
```rust
...
```
```

Severity:

- **Blocker** — bug, panic, data loss, security.
- **High** — boundary bocor, error handling hilang, test penting tidak ada.
- **Medium** — fungsi besar, nama buruk, duplikasi.
- **Low** — formatting, komentar, konsistensi.
- **Nit** — preferensi gaya.

---

## 4. Refactor Workflow

### Langkah 1 — Kunci perilaku

- Pastikan ada test yang mencakup perilaku saat ini.
- Jika belum ada, tulis test karakterisasi.
- Jalankan test; pastikan hijau.

### Langkah 2 — Pilih satu target kecil

Contoh:

- Pecah satu fungsi.
- Perbaiki satu nama.
- Ganti satu `unwrap()`.
- Pindahkan satu tipe ke modul yang tepat.
- Hapus satu duplikasi.

Jangan memilih lima target sekaligus.

### Langkah 3 — Refactor

- Perubahan kecil.
- Jangan reformat seluruh file.
- Jangan ubah API publik tanpa migrasi.
- Jangan ubah perilaku.

### Langkah 4 — Verifikasi

```bash
bash .opencode/skills/clean-code/scripts/check.sh
```

### Langkah 5 — Commit

- Satu commit untuk satu tujuan.
- Pesan commit jelas.
- Jelaskan mengapa, bukan hanya apa.

---

## 5. Contoh Refactor Rust

### Sebelum

```rust
pub fn run(c: &Circuit, flag: bool) -> Result<(), String> {
    let mut m = build_matrix(c);
    solve(&mut m)?;
    if flag {
        write_vcd(&m);
    }
    Ok(())
}
```

### Sesudah

```rust
pub fn run_dc(circuit: &Circuit) -> Result<DcReport, SimError> {
    let matrix = build_matrix(circuit);
    solve_dc(&matrix)
}

pub fn run_transient(
    circuit: &Circuit,
    config: TransientConfig,
) -> Result<TransientReport, SimError> {
    let matrix = build_matrix(circuit);
    solve_transient(&matrix, config)
}
```

Alasan:

- Boolean flag dihilangkan.
- Setiap fungsi punya satu use case.
- Error type jelas.
- Output writer tidak dicampur dengan solver.

---

## 6. Kapan Tidak Refactor

- Tidak ada test dan perilaku belum dipahami.
- Perubahan tidak diminta dan tidak relevan.
- Kode akan segera dihapus.
- Refactor akan memperbesar risiko bug.
- Batas waktu tidak memungkinkan verifikasi.

Jika ragu, tulis temuan dan tunda.

---

## 7. Checklist Review Akhir

- [ ] Tujuan perubahan jelas.
- [ ] Diff minimal.
- [ ] Tidak ada perubahan tak terkait.
- [ ] Test ada dan lulus.
- [ ] Error handling benar.
- [ ] Tidak ada `unwrap()`/panic yang tidak perlu.
- [ ] Nama jelas.
- [ ] Fungsi kecil.
- [ ] Tidak ada duplikasi.
- [ ] Boundary crate tidak bocor.
- [ ] Tidak ada dependensi eksternal.
- [ ] `fmt`, `clippy`, `test` bersih.
- [ ] Dokumentasi diperbarui jika perlu.
- [ ] Temuan ditulis dengan format yang jelas.

---

## 8. Anti-Pattern Review

- Menulis ulang seluruh file tanpa diminta.
- Menambahkan abstraksi untuk satu implementasi.
- Menambahkan trait “biar clean” tanpa variasi.
- Menghapus test karena “terlalu ketat”.
- Menambahkan `unwrap()` untuk mempercepat.
- Menambahkan dependensi eksternal untuk masalah kecil.
- Menghina kode tanpa solusi.
- Menandai semua hal sebagai Blocker.
- Mengabaikan konteks proyek.

---

## 9. Prinsip Penutup

> Review yang baik mengurangi risiko, bukan menambah pekerjaan.
> Refactor yang baik menjaga perilaku, bukan mengubahnya.
> Kode yang baik adalah kode yang mudah diubah dan diuji.
