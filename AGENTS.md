# AGENTS.md

Panduan wajib untuk agent dan kontributor yang bekerja di repository ini.

Baca file ini sebelum mengubah kode, dokumen, atau konfigurasi.

---

## 1. Status Proyek

- Fase saat ini: desain dan fondasi workspace.
- Workspace Rust sudah ada, tetapi implementasi simulator belum dimulai.
- Jangan memulai implementasi besar tanpa instruksi eksplisit.
- Sumber kebenaran: `README.md`, `docs/`, dan `TODO.md`.
- `.caches/` adalah riset lokal, bukan bagian dari kode produksi.
- Cache tidak di-commit dan tidak boleh dijadikan dependensi runtime.

---

## 2. Aturan Tidak Bisa Dinegosiasikan

1. Zero external dependency untuk semua crate, dengan pengecualian `sim-gui` yang boleh memakai `egui` dan `eframe`. `egui_plot`, `serde`, `rfd`, dan crate eksternal lain tidak diizinkan tanpa keputusan baru.
2. Tidak ada `unsafe`.
3. Semua teks baru atau yang diubah harus ASCII.
4. Tidak ada em dash, en dash, kutip melengkung, elipsis Unicode, panah Unicode, atau bullet Unicode.
5. Setiap fungsi maksimal 20 baris.
6. Guard clause wajib di awal fungsi.
7. Fungsi harus dipecah menjadi unit kecil yang reusable.
8. Cari fungsi yang sudah ada sebelum membuat fungsi baru.
9. Jangan membuat fungsi yang menduplikasi fungsi yang sudah ada.
10. Setiap perubahan perilaku harus punya test.
11. Jangan mencampur refactor dan fitur baru dalam satu perubahan.
12. Jangan mengubah file yang tidak terkait.
13. Jangan mengarang data, kutipan, atau hasil pengukuran.
14. Jika ragu, berhenti dan tanya.

Jika permintaan pengguna bertentangan dengan aturan ini, jelaskan konfliknya dan minta keputusan.

---

## 3. Aturan Fungsi

### 3.1 Guard Clause

- Validasi precondition di awal.
- Return atau `Err` lebih awal.
- Hindari `else` setelah `return`.
- Happy path tidak boleh berada di dalam sarang `if`.
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

### 3.2 Maksimal 20 Baris

- Hitung dari baris `fn` sampai `}` penutup.
- Doc comment dan attribute di atas `fn` tidak dihitung.
- Blank line di dalam body dihitung.
- Test helper, adapter, dan fungsi privat juga berlaku.
- Jika lebih dari 20 baris, pecah sebelum menambah logika baru.

Cara memecah:

1. Ekstrak guard clause.
2. Ekstrak body loop.
3. Ekstrak error mapping.
4. Ekstrak formatting.
5. Ekstrak satu langkah domain.

### 3.3 Fungsi Kecil dan Reusable

- Satu fungsi, satu langkah atau satu ide.
- Nama fungsi menjelaskan langkah, bukan implementasi.
- Pure function lebih baik jika memungkinkan.
- Parameter sedikit; gunakan struct jika perlu.
- Jangan membuat wrapper satu baris yang hanya memindahkan nama.
- Jangan over-fragmentasi sampai alur sulit dibaca.

### 3.4 Cari Sebelum Membuat

- Cari dengan `rg`, `grep`, atau inspeksi modul.
- Cek modul tetangga, trait, helper, dan test utility.
- Reuse, perluas, atau parameterisasi fungsi yang ada.
- Jangan membuat `foo2`, `foo_new`, atau `helper_foo`.
- Jika perilaku berbeda, beri nama yang membedakan dengan jelas.

---

## 4. Arsitektur

Graph dependensi crate:

```
sim-cli
  +-- sim-board
  |   +-- sim-avr
  |   |   +-- sim-cells
  |   |   |   +-- sim-device
  |   |   |   |   `-- sim-core
  |   |   |   `-- sim-core
  |   |   `-- sim-core
  |   `-- sim-device
  `-- sim-analysis
      +-- sim-device
      `-- sim-core

sim-gui
  +-- sim-board
  +-- sim-avr
  +-- sim-analysis
  +-- sim-device
  `-- sim-core
```

Aturan:

- `sim-core` tidak bergantung pada crate lain.
- `sim-device` hanya bergantung pada `sim-core`.
- `sim-analysis` hanya bergantung pada `sim-core` dan `sim-device`.
- `sim-cells` hanya bergantung pada `sim-core` dan `sim-device`.
- `sim-avr` hanya bergantung pada `sim-core`, `sim-device`, dan `sim-cells`.
- `sim-board` hanya bergantung pada `sim-core`, `sim-device`, dan `sim-avr`.
- `sim-cli` adalah adapter dan komposisi.
- `sim-gui` adalah adapter dan komposisi.
- `sim-gui` boleh memakai `egui` dan `eframe`; crate lain tidak boleh.
- Tidak ada crate yang bergantung pada `sim-cli` atau `sim-gui`.
- Tidak ada siklus.
- `main.rs` tidak berisi logika simulasi.
- `sim-core` tidak memakai `println!`, filesystem, atau wall-clock.
- Format CSV, VCD, HEX, dan ELF berada di adapter, bukan di core.
- Pin, UART, timer, dan firmware loader adalah port/adapter.
- Inti harus bisa diuji tanpa board, CLI, file, atau `avr-gcc`.

Jika boundary perlu diubah, jelaskan alasan dan dampaknya sebelum menulis kode.

---

## 5. Workflow Agent

### Langkah 1 - Baca konteks

- Baca `AGENTS.md`, `README.md`, dan dokumen yang relevan.
- Baca test dan kode di sekitar perubahan.
- Cek `TODO.md` untuk status milestone.

### Langkah 2 - Muat skill

- Untuk menulis atau mereview kode: gunakan skill `clean-code`.
- Untuk menulis dokumen: gunakan skill `senior-writing`.
- Jangan mengklaim isi buku tanpa merujuk `.caches/` atau sumber resmi.

### Langkah 3 - Cari sebelum mengubah

- Cari fungsi, tipe, dan test yang sudah ada.
- Cari apakah masalah sudah pernah diselesaikan.
- Jangan membuat solusi paralel.

### Langkah 4 - Rencanakan perubahan terkecil

- Satu tujuan per perubahan.
- Pecah pekerjaan besar menjadi langkah kecil.
- Tentukan test yang membuktikan perilaku.
- Tentukan file yang tidak boleh disentuh.

### Langkah 5 - Implementasi

- Tulis test lebih dulu jika perilaku berubah.
- Gunakan guard clause.
- Jaga fungsi maksimal 20 baris.
- Reuse fungsi yang ada.
- Jangan menambah dependensi eksternal di luar `egui`/`eframe` pada `sim-gui`.
- Jangan memakai `unsafe`.

### Langkah 6 - Verifikasi

Jalankan:

```bash
bash .opencode/skills/clean-code/scripts/check.sh
```

Periksa ASCII untuk file yang diubah:

```bash
python3 .opencode/skills/senior-writing/scripts/check_ascii.py <file>
```

Jika gagal, perbaiki sebelum melapor.

### Langkah 7 - Lapor

Laporkan:

- Apa yang diubah.
- Mengapa.
- Perilaku apa yang dipertahankan.
- Test apa yang membuktikan.
- Risiko atau batasan.
- Langkah berikutnya.

---

## 6. Early Exit

Berhenti dan tanya jika:

- Permintaan bertentangan dengan aturan di file ini.
- Perubahan akan melanggar boundary crate.
- Perlu menambah dependensi eksternal di luar pengecualian `sim-gui`.
- Perlu mengubah API publik tanpa migrasi.
- Perilaku saat ini tidak dipahami dan belum ada test.
- Fungsi yang akan diubah sudah lebih dari 20 baris dan pemecahannya tidak jelas.
- Ada konflik antara instruksi pengguna dan sumber kebenaran.
- Data atau sumber untuk klaim tidak tersedia.

Jangan menebak. Jangan mengarang. Jangan memaksa.

---

## 7. Definition of Done

- [ ] Perubahan sesuai permintaan.
- [ ] Tidak ada perubahan tak terkait.
- [ ] Test baru atau test yang diperbarui ada.
- [ ] Guard clause digunakan.
- [ ] Setiap fungsi maksimal 20 baris.
- [ ] Fungsi existing sudah dicari dan digunakan ulang.
- [ ] Tidak ada duplikasi fungsi.
- [ ] Tidak ada dependensi eksternal selain `egui`/`eframe` di `sim-gui`.
- [ ] Tidak ada `unsafe`.
- [ ] ASCII-only untuk file yang diubah.
- [ ] `fmt`, `clippy`, dan `test` lulus.
- [ ] Dokumentasi diperbarui jika perlu.
- [ ] Commit memakai conventional commit.

---

## 8. Gaya Kode Rust

- `snake_case` untuk fungsi, variabel, dan modul.
- `PascalCase` untuk tipe, trait, dan enum.
- `SCREAMING_SNAKE_CASE` untuk konstanta.
- Gunakan `Result<T, E>` untuk operasi yang bisa gagal.
- Gunakan `Option<T>` hanya untuk nilai yang mungkin tidak ada.
- Jangan `unwrap()` pada input pengguna.
- `expect()` hanya untuk invariant dengan pesan jelas.
- Field privat; gunakan constructor dan invariant.
- `pub` hanya untuk API yang stabil.
- `pub(crate)` untuk internal.
- Trait kecil dan kohesif.
- Generic untuk hot path; `dyn` hanya di boundary.
- Test deterministik; jangan memakai `sleep`.
- Jangan memakai `Rc<RefCell<T>>` tanpa alasan.
- Jangan memakai `clone()` untuk melawan borrow checker.
- Dokumentasikan error dan invariant.
- Gunakan `rustfmt` dan `clippy`.

---

## 9. ASCII Policy

Semua teks baru atau yang diubah harus ASCII, U+0000 sampai U+007F.

Dilarang:

- Em dash, en dash, horizontal bar.
- Kutip melengkung.
- Elipsis Unicode.
- Panah Unicode.
- Bullet Unicode.
- Non-breaking space.
- Huruf beraksen.
- Simbol derajat, plus-minus, micro, kali, hak cipta, trademark.
- Karakter non-ASCII lain.

Gunakan:

| Dilarang | Gunakan |
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
| Kali | x |
| Hak cipta | (c) |

Jika file yang disentuh mengandung non-ASCII, perbaiki karakter tersebut dalam perubahan yang sama.

Jika pengguna meminta karakter non-ASCII, jelaskan konflik kebijakan dan minta konfirmasi.

---

## 10. Penulisan Dokumen

- Kode dan identifier: bahasa Inggris.
- Dokumentasi dan penjelasan: bahasa Indonesia.
- Ringkas, jelas, dan dapat dipindai.
- Kesimpulan di awal.
- Pisahkan fakta, asumsi, dan opini.
- Sebut sumber untuk klaim yang tidak jelas.
- Jangan memakai jargon tanpa definisi.
- Jangan memakai hype.
- Gunakan `senior-writing` untuk dokumen penting.

---

## 11. Git dan Commit

Format conventional commit:

```
<type>(<scope>): <description>
```

Type yang dipakai:

- `feat`
- `fix`
- `docs`
- `refactor`
- `test`
- `chore`
- `perf`
- `build`
- `ci`

Aturan:

- Deskripsi lowercase.
- Imperatif.
- Tanpa titik di akhir.
- Satu tujuan per commit.
- Jangan commit `target/`, `.caches/`, atau file rahasia.
- Jangan commit perubahan yang tidak terkait.

Contoh:

```
feat(sim-core): add time conversion
fix(sim-analysis): handle singular matrix
docs(agents): add workflow rules
refactor(sim-device): extract mosfet region helpers
```

---

## 12. Review dan Severity

Severity:

- Blocker: bug, panic, data loss, security.
- High: boundary bocor, error handling hilang, test tidak ada, fungsi > 20 baris, duplikasi fungsi.
- Medium: nama menyesatkan, nesting > 3, komentar redundan.
- Low: formatting, komentar, konsistensi.
- Nit: preferensi gaya.

Format temuan:

```
[Severity] file:line - judul

Masalah:
Dampak:
Perbaikan:
```

Jangan menulis "kode ini buruk" tanpa alasan dan solusi.

---

## 13. Larangan

- Jangan menambah crate eksternal selain `egui`/`eframe` di `sim-gui`.
- Jangan memakai `unsafe`.
- Jangan memakai wall-clock di simulation.
- Jangan memakai `println!` di core.
- Jangan memakai `unwrap()` pada input pengguna.
- Jangan membuat fungsi duplikat.
- Jangan membuat fungsi lebih dari 20 baris.
- Jangan membuat abstraksi tanpa variasi nyata.
- Jangan mengubah banyak file sekaligus tanpa alasan.
- Jangan menghapus test untuk mempercepat build.
- Jangan menyalin isi buku ke repository.
- Jangan mengarang hasil scrape atau kutipan.
- Jangan mengabaikan aturan ASCII.

---

## 14. Referensi

- `README.md` - ikhtisar proyek.
- `TODO.md` - task list dan milestone.
- `TODO-GUI.md` - backlog GUI egui.
- `docs/57-gui-egui.md` - desain GUI egui.
- `docs/00-vision.md` - visi dan batasan.
- `docs/01-decisions.md` - decision log.
- `docs/02-architecture.md` - arsitektur.
- `docs/07-verification.md` - verifikasi dan fidelity.
- `docs/09-risks.md` - risiko dan realisme.
- `.opencode/skills/clean-code/` - aturan kode.
- `.opencode/skills/senior-writing/` - aturan tulisan.
- `.caches/clean-architecture/` - riset Clean Code dan Clean Architecture.

---

## 15. Prinsip Penutup

> Perubahan kecil yang teruji lebih baik daripada perubahan besar yang tidak dipahami.
> Reuse lebih baik daripada duplikasi.
> Kejelasan lebih baik daripada kepintaran.
> ASCII lebih baik daripada karakter yang tidak perlu.
