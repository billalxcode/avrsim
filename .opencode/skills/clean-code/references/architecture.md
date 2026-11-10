# Clean Architecture dan Boundary Proyek

Referensi untuk memutuskan batas modul/crate dan arah dependensi.

---

## 1. The Dependency Rule

> Source code dependencies hanya boleh menunjuk ke dalam, menuju kebijakan tingkat tinggi.

- Lingkaran dalam tidak tahu lingkaran luar.
- Nama/tipe dari luar tidak boleh disebut di dalam.
- Format data luar tidak boleh masuk ke dalam.
- Data yang menyeberang boundary adalah struct/DTO sederhana.
- Alur kontrol boleh keluar-masuk; dependensi source code tetap masuk.

Jika alur kontrol berlawanan, gunakan **Dependency Inversion**:

- Lapisan dalam mendefinisikan trait/port.
- Lapisan luar mengimplementasikannya.
- `main` merakit implementasi.

---

## 2. Pemetaan ke `embed-emulation`

```
sim-cli
  +-- sim-board
  |     +-- sim-avr
  |     |     +-- sim-cells
  |     |     |     +-- sim-device
  |     |     |     |     `-- sim-core
  |     |     |     `-- sim-core
  |     |     `-- sim-core
  |     `-- sim-device
  `-- sim-analysis
        +-- sim-device
        `-- sim-core
```

| Konsep | Padanan |
|---|---|
| Entities | `sim-core`: waktu, node, netlist, matrix, error, port |
| Use Cases | `sim-analysis`: run DC/sweep/transient; `sim-avr`: step CPU |
| Interface Adapters | `sim-board`: wiring; `sim-cli`: CSV/VCD/CLI; loader firmware |
| Frameworks & Drivers | `std`, filesystem, terminal, GTKWave, `avr-gcc` |
| Main | `crates/sim-cli/src/main.rs` |
| Ports | Trait di core/analysis (`WaveformSink`, `FirmwareSource`, `UartSink`) |
| Adapters | Implementasi di CLI/board |

### Aturan crate

- `sim-core` tidak bergantung pada crate lain.
- `sim-device` hanya `sim-core`.
- `sim-analysis` hanya `sim-core` + `sim-device`.
- `sim-cells` hanya `sim-core` + `sim-device`.
- `sim-avr` hanya `sim-core` + `sim-device` + `sim-cells`.
- `sim-board` hanya `sim-core` + `sim-device` + `sim-avr`.
- `sim-cli` boleh semua.
- Tidak ada crate yang bergantung pada `sim-cli`.
- Tidak ada siklus.
- Tidak ada dependensi eksternal.

---

## 3. SOLID Ringkas

### SRP

- Satu modul, satu alasan perubahan.
- "Reason to change" = actor/stakeholder.
- Gejala: merge conflict, accidental duplication.

### OCP

- Terbuka untuk ekstensi, tertutup untuk modifikasi.
- Tambah device/peripheral tanpa mengubah inti.
- Gunakan trait/registry.

### LSP

- Implementasi trait harus menghormati kontrak.
- Jangan panik pada input valid.
- Jangan mengubah invariant.

### ISP

- Trait kecil.
- Jangan bergantung pada method yang tidak dipakai.
- Pisahkan `Clock`, `WaveformSink`, `FirmwareSource`, `UartSink`.

### DIP

- High-level dan low-level bergantung pada abstraksi.
- Jangan bergantung pada volatile concrete.
- `main` merakit dependency.

---

## 4. Component Principles

- **REP** - satu crate = satu unit rilis.
- **CCP** - yang berubah bersama masuk bersama.
- **CRP** - yang tidak dipakai bersama jangan digabung.
- **ADP** - graph crate harus DAG.
- **SDP** - dependensi menuju stabilitas.
- **SAP** - crate stabil berisi abstraksi yang tepat.

Hindari:

- Crate `utils` gemuk.
- Terlalu banyak crate kecil.
- Siklus logis.
- Abstraksi tanpa variasi.

---

## 5. Boundary dan Biaya

- Boundary memisahkan kebijakan dari detail.
- Boundary punya biaya: interface, DTO, file, build.
- Pasang boundary ketika biaya mengabaikannya lebih besar.
- Gunakan **partial boundary** jika strict boundary terlalu mahal:
  - Source-level decoupling.
  - Gabungkan interface/implementasi.
  - Facade.
- Jangan over-engineer.

---

## 6. Humble Object

- Pisahkan bagian sulit diuji (I/O, terminal, file, GUI) dari logika.
- CLI/VCD/CSV writer harus "bodoh".
- Logika ada di core/analysis.
- View/CLI hanya menampilkan data dari view model/DTO.

---

## 7. Main

- `main.rs` adalah komposisi.
- Membuat board, memilih analysis, menyambungkan adapter.
- Tidak ada logika simulasi.
- Bisa ada beberapa `main` untuk konfigurasi berbeda.

---

## 8. Embedded/Target Bottleneck

- Inti harus bisa diuji tanpa board.
- Inti harus bisa diuji tanpa CLI.
- Inti harus bisa diuji tanpa file.
- Inti harus bisa diuji tanpa `avr-gcc`.
- Pin/UART/timer adalah port/adapter.
- "A clean embedded architecture is a testable embedded architecture."

---

## 9. Red Flags Arsitektur

- `sim-core` mengimpor `sim-cli`.
- `sim-device` menulis CSV.
- `sim-analysis` membaca file.
- `sim-avr` tahu `sim-board`.
- `sim-board` mem-parsing HEX.
- `main.rs` berisi logika.
- Trait port didefinisikan di crate luar.
- Siklus crate.
- Global mutable state.
- Wall-clock di simulation.

---

## 10. Cara Memutuskan

1. Apa kebijakan inti yang harus dilindungi?
2. Apa detail yang bisa berubah?
3. Apa yang akan berubah bersama?
4. Apa yang akan diuji tanpa I/O?
5. Berapa biaya boundary?
6. Apakah ada variasi implementasi nyata?
7. Apakah boundary bisa ditegakkan compiler (`pub(crate)`/crate)?

Jika tidak ada variasi nyata, tunda abstraksi.
