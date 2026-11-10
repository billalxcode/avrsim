# Rust Clean Code Checklist

Checklist khusus Rust untuk proyek `embed-emulation`.

---

## 1. Naming

- [ ] `snake_case` untuk fungsi, variabel, modul.
- [ ] `PascalCase` untuk tipe, trait, enum.
- [ ] `SCREAMING_SNAKE_CASE` untuk konstanta.
- [ ] Nama domain, bukan singkatan misterius.
- [ ] Tidak ada `tmp`, `data`, `foo`, `bar` tanpa konteks.
- [ ] Tidak ada `Manager`, `Helper`, `Util` yang kabur.
- [ ] Satu istilah untuk satu konsep (`device`, `node`, `net`, `sample`).

---

## 2. Functions

- [ ] Kecil dan satu tanggung jawab.
- [ ] Tidak ada boolean flag parameter.
- [ ] Tidak ada lebih dari 3 parameter tanpa alasan.
- [ ] Parameter kompleks memakai struct.
- [ ] Tidak ada side effect tersembunyi.
- [ ] Fungsi yang mengubah state tidak sekaligus mengembalikan nilai penting.
- [ ] Nama fungsi menjelaskan efek samping jika ada.

Contoh:

```rust
// Buruk
fn run(circuit: &Circuit, flag: bool) -> Result<(), SimError>

// Baik
fn run_dc(circuit: &Circuit) -> Result<DcReport, SimError>
fn run_transient(circuit: &Circuit, cfg: TransientConfig) -> Result<TransientReport, SimError>
```

---

## 3. Error Handling

- [ ] Gunakan `Result<T, E>` untuk operasi yang bisa gagal.
- [ ] Gunakan `Option<T>` hanya untuk “tidak ada nilai”, bukan error.
- [ ] Tidak ada `unwrap()` pada input pengguna.
- [ ] `expect()` hanya untuk invariant dengan pesan jelas.
- [ ] Error type punya `Display` dan `std::error::Error`.
- [ ] Error domain tidak bergantung pada CLI.
- [ ] Gunakan `?` untuk propagasi.
- [ ] Jangan menelan error dengan `let _ = ...`.
- [ ] Jangan memakai `panic!` untuk validasi input.

Contoh:

```rust
#[derive(Debug)]
pub enum SimError {
    InvalidNetlist(String),
    SingularMatrix { row: usize },
    ConvergenceFailure { iteration: usize },
}

impl std::fmt::Display for SimError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidNetlist(msg) => write!(f, "invalid netlist: {msg}"),
            Self::SingularMatrix { row } => write!(f, "singular matrix at row {row}"),
            Self::ConvergenceFailure { iteration } => {
                write!(f, "convergence failure after {iteration} iterations")
            }
        }
    }
}

impl std::error::Error for SimError {}
```

---

## 4. Ownership dan Borrowing

- [ ] Jangan `clone()` hanya untuk melawan borrow checker; pahami lifetime-nya.
- [ ] Gunakan `&str`/`&[T]` untuk input baca.
- [ ] Kembalikan `String`/`Vec<T>` hanya jika memang memiliki.
- [ ] Hindari `Rc<RefCell<T>>` kecuali ada kebutuhan nyata.
- [ ] Hindari `Arc<Mutex<T>>` di simulation single-thread.
- [ ] Gunakan `Cow<'_, T>` hanya jika benar-benar perlu.

---

## 5. Types

- [ ] Gunakan newtype untuk unit/semantik (`TimeNs`, `Voltage`, `NodeId`).
- [ ] Gunakan enum untuk state, bukan string/boolean.
- [ ] Jangan memakai `bool` untuk dua state yang punya makna domain.
- [ ] Jangan memakai `f64` untuk waktu digital; gunakan integer.
- [ ] Jangan memakai `usize` untuk ID lintas platform tanpa alasan.
- [ ] Buat field privat; gunakan constructor/invariant.
- [ ] Gunakan `#[must_use]` untuk nilai yang tidak boleh diabaikan.

---

## 6. Modules dan Visibility

- [ ] `pub` hanya untuk API yang stabil.
- [ ] `pub(crate)` untuk internal.
- [ ] Modul privat untuk detail.
- [ ] Tidak semua field `pub`.
- [ ] Tidak ada modul `utils` gemuk.
- [ ] Boundary crate sesuai `references/architecture.md`.
- [ ] `main.rs` hanya komposisi.

---

## 7. Traits

- [ ] Trait kecil dan kohesif.
- [ ] Nama trait menjelaskan capability (`Clock`, `WaveformSink`).
- [ ] Tidak ada trait “god object”.
- [ ] Gunakan generic untuk hot path.
- [ ] Gunakan `dyn` hanya di boundary/plugin.
- [ ] Dokumentasikan kontrak dan invariant.
- [ ] Jangan membuat trait jika hanya ada satu implementasi dan tidak ada rencana variasi.

---

## 8. Tests

- [ ] Unit test untuk perilaku inti.
- [ ] Integration test untuk boundary.
- [ ] Test deterministik.
- [ ] Tidak ada `sleep`.
- [ ] Tidak ada filesystem/network jika bisa dihindari.
- [ ] Test independen.
- [ ] Nama test menjelaskan perilaku.
- [ ] Edge case diuji.
- [ ] Error case diuji.
- [ ] `#[should_panic]` hanya jika panic adalah kontrak.

---

## 9. Documentation

- [ ] `//!` untuk crate/modul.
- [ ] `///` untuk API publik.
- [ ] Contoh kode di doc comment jika membantu.
- [ ] Dokumentasikan error yang mungkin.
- [ ] Dokumentasikan invariant.
- [ ] Jangan menulis komentar yang mengulang kode.

---

## 10. Formatting dan Lint

- [ ] `cargo fmt --all -- --check`.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] `cargo test --workspace`.
- [ ] Tidak ada `unsafe`.
- [ ] Tidak ada dependensi eksternal.
- [ ] Tidak ada warning yang diabaikan tanpa alasan.

---

## 11. Performance

- [ ] Jangan optimasi sebelum mengukur.
- [ ] Hindari alokasi di hot path.
- [ ] Gunakan `Vec`/arena, bukan linked list, untuk data panas.
- [ ] Jangan memakai `dyn` di loop solver jika generic memungkinkan.
- [ ] Ukur dengan benchmark sebelum dan sesudah.
- [ ] Jangan mengorbankan kejelasan tanpa bukti.

---

## 12. Project-Specific

- [ ] `sim-core` tanpa dependensi crate lain.
- [ ] `sim-device` tanpa analysis/CLI.
- [ ] `sim-analysis` tanpa CLI/file.
- [ ] `sim-avr` tanpa board.
- [ ] `sim-board` tanpa parser firmware.
- [ ] `sim-cli` hanya adapter/komposisi.
- [ ] Waktu simulasi deterministik.
- [ ] Tidak ada wall-clock di core.
- [ ] Tidak ada `println!` di core.

---

## 13. Red Flags

- `unwrap()` di mana-mana.
- `clone()` untuk menghindari desain.
- `Rc<RefCell<>>` tanpa alasan.
- `unsafe`.
- Global mutable state.
- File 1000+ baris.
- Fungsi 100+ baris.
- Boolean flag parameter.
- Trait dengan 10+ method.
- Modul `utils`.
- `String` untuk semua hal.
- Error type `String`.
- Test yang membutuhkan terminal.
- Test yang membutuhkan file.
- Abstraksi tanpa variasi.
