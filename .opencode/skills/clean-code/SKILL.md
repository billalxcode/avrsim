---
name: Clean Code
description: Write, review, and refactor code using Clean Code and Clean Architecture principles, with Rust-specific checks and project boundary rules.
---

# Clean Code

Gunakan skill ini saat:

- Menulis kode baru.
- Mereview PR atau diff.
- Refactor fungsi/modul yang sulit dibaca.
- Memperbaiki nama, error handling, test, atau struktur modul.
- Memutuskan boundary antar-crate/modul.
- Menghadapi kode yang "bekerja tetapi sulit diubah".

Jangan gunakan skill ini untuk perubahan sepele yang tidak menyentuh desain (mis. typo komentar), kecuali pengguna meminta review menyeluruh.

---

## Prinsip Utama

> **Preserve behavior, improve clarity, keep changes small, prove with tests.**

1. **Jangan mengubah perilaku tanpa test.**
2. **Jangan mencampur refactor dengan fitur baru.**
3. **Jangan menambah abstraksi sebelum ada variasi nyata.**
4. **Jangan menambah dependensi eksternal.**
5. **Jangan memakai `unsafe`.**
6. **Jangan melakukan perubahan yang tidak diminta.**
7. **Jika ragu, berhenti dan tanya.**

---

## Aturan Fungsi Wajib

Aturan ini berlaku untuk semua fungsi, termasuk test helper, adapter, dan fungsi privat.

1. **Guard clause dulu.**
   - Validasi precondition di awal fungsi.
   - Return atau `Err` lebih awal.
   - Hindari `else` setelah `return`.
   - Happy path tidak boleh berada di dalam sarang `if`.

2. **Maksimal 20 baris per fungsi.**
   - Hitung dari baris `fn` sampai `}` penutup.
   - Doc comment dan attribute di atas `fn` tidak dihitung.
   - Blank line di dalam body dihitung.
   - Jika lebih dari 20 baris, pecah.

3. **Pecah menjadi fungsi terkecil yang reusable.**
   - Satu fungsi, satu langkah atau satu ide.
   - Ekstrak validasi, parsing, transformasi, keputusan, formatting, dan I/O.
   - Pecah selama pemecahan memberi nama yang jelas dan menghilangkan duplikasi.
   - Fungsi satu baris yang hanya memindahkan nama bukan pemecahan yang berguna.

4. **Cari sebelum membuat.**
   - Cari fungsi yang sudah ada dengan `rg`, `grep`, atau inspeksi modul.
   - Cek modul tetangga, trait, helper, dan test utility.
   - Gunakan ulang atau perluas fungsi yang ada.
   - Jangan membuat duplikat dekat seperti `foo2`, `foo_new`, atau `helper_foo`.

5. **Jangan membuat fungsi yang sudah ada.**
   - Jika perilaku sama, reuse fungsi yang ada.
   - Jika perilaku mirip, parameterisasi atau refactor fungsi yang ada.
   - Jika perilaku berbeda, beri nama yang membedakan dengan jelas.
   - Duplikasi logika adalah pelanggaran, bukan sekadar preferensi gaya.

---

## Workflow

### 1. Pahami konteks

- Baca file yang relevan, test, dan dokumen arsitektur.
- Identifikasi tanggung jawab modul dan boundary-nya.
- Cek apakah ada aturan khusus di `AGENTS.md`, `README.md`, atau docs proyek.

### 2. Muat referensi

Baca sesuai kebutuhan:

- `references/principles.md` - prinsip Clean Code dan smells.
- `references/rust-checklist.md` - checklist Rust.
- `references/architecture.md` - dependency rule, SOLID, component principles, boundary proyek.
- `references/review-playbook.md` - cara review dan refactor.
- `references/sources.md` - sumber dan batas klaim.

Jangan mengklaim isi buku tanpa merujuk referensi/cache.

### 3. Diagnosis

Tulis temuan dalam format:

```
[Severity] file:line - masalah
Dampak: ...
Perbaikan: ...
```

Severity:

- **Blocker** - bug, panic, data loss, security.
- **High** - pelanggaran boundary, error handling hilang, test tidak ada, fungsi > 20 baris, duplikasi fungsi.
- **Medium** - nama menyesatkan, nesting > 3, komentar redundan.
- **Low** - formatting, komentar, konsistensi.
- **Nit** - preferensi gaya.

### 4. Refactor

- Mulai dari perubahan terkecil yang aman.
- Tambah/ubah test lebih dulu jika perilaku berubah.
- Satu commit/PR untuk satu tujuan.
- Jangan reformat seluruh file jika hanya mengubah satu fungsi.
- Jangan mengganti API publik tanpa alasan dan migrasi.

### 5. Verifikasi

Jalankan:

```bash
bash .opencode/skills/clean-code/scripts/check.sh
```

Atau manual:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Jika ada yang gagal, perbaiki sebelum melapor.

### 6. Lapor

Gunakan `templates/review-report.md` untuk review. Untuk refactor, jelaskan:

- Apa yang diubah.
- Mengapa.
- Perilaku apa yang dipertahankan.
- Test apa yang membuktikan.
- Risiko/limitasi.

---

## Aturan Proyek EmuCAD

- `sim-core` tidak boleh bergantung pada crate lain.
- `sim-device` tidak boleh tahu analysis/CLI.
- `sim-analysis` tidak boleh tahu CLI/file.
- `sim-avr` tidak boleh tahu board.
- `sim-board` tidak boleh mem-parsing firmware.
- `sim-cli` adalah adapter/komposisi.
- Tidak ada crate eksternal.
- Tidak ada `unsafe`.
- Tidak ada wall-clock di simulation.
- Tidak ada `println!` di core.
- Tidak ada `unwrap()` pada input pengguna.

Jika perubahan melanggar aturan ini, tandai sebagai **High** dan tawarkan alternatif.

---

## Checklist Cepat

- [ ] Nama mengungkap maksud.
- [ ] Fungsi kecil dan satu tanggung jawab.
- [ ] Guard clause digunakan di awal.
- [ ] Setiap fungsi maksimal 20 baris.
- [ ] Fungsi dipecah menjadi unit kecil dan reusable.
- [ ] Fungsi yang sudah ada dicari dan digunakan ulang.
- [ ] Tidak ada duplikat dekat.
- [ ] Tidak ada side effect tersembunyi.
- [ ] Error memakai `Result`/`Option` dengan konteks.
- [ ] Test ada dan lulus.
- [ ] Tidak ada duplikasi yang jelas.
- [ ] Tidak ada abstraksi prematur.
- [ ] Boundary crate tidak bocor.
- [ ] `fmt`, `clippy`, `test` bersih.
- [ ] Perubahan minimal dan sesuai permintaan.

---

## References

- `references/principles.md`
- `references/rust-checklist.md`
- `references/architecture.md`
- `references/review-playbook.md`
- `references/sources.md`
- `templates/review-report.md`
- `scripts/check.sh`

---

## Sumber Belajar

Cache riset proyek:

- `.caches/clean-architecture/by-chapter/` - 34 bab Clean Architecture.
- `.caches/clean-architecture/clean-code-by-chapter/` - 17 bab Clean Code.
- `.caches/clean-architecture/scraped/` - hasil scrape mentah.

Gunakan cache sebagai referensi, bukan sebagai pengganti buku. Jangan menyalin isi buku ke kode atau dokumen publik.
