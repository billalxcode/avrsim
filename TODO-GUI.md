# TODO - GUI egui

Backlog eksekusi untuk GUI interaktif berbasis `egui`/`eframe`.

**Status:** desain selesai, implementasi belum dimulai.

**Desain:** [`docs/57-gui-egui.md`](docs/57-gui-egui.md)

**Keputusan yang sudah disetujui:**

- `egui` dan `eframe` hanya boleh dipakai di crate `sim-gui`.
- Crate simulasi lain tetap zero external dependency.
- `egui_plot`, `serde`, `rfd`, dan crate eksternal lain tidak diizinkan
  tanpa keputusan baru.
- Grafik memakai widget internal berbasis `egui::Painter`.

**Default usulan yang belum dikunci:**

- Satu edge grid hanya boleh memuat satu kabel.
- Persilangan tanpa junction dilarang.
- GUI pertama hanya Linux.
- Format simpan berupa teks ASCII kustom.
- Session simulasi belum diekstrak ke crate bersama.

---

## G0 - Fondasi dan Keputusan

- [x] G0-01 Setujui pengecualian `egui`/`eframe` untuk `sim-gui`.
- [x] G0-02 Catat keputusan di `AGENTS.md` dan `docs/57`.
- [ ] G0-03 Buat crate `crates/sim-gui` dengan binary `embed-emu-gui`.
- [ ] G0-04 Tambahkan `sim-gui` ke workspace dan dependency graph.
- [ ] G0-05 Pin versi `egui`/`eframe`, commit `Cargo.lock`.
- [ ] G0-06 Buat app error type dan aturan no-panic untuk input GUI.
- [ ] G0-07 Tambahkan build dan test `sim-gui` ke CI Linux.
- [ ] G0-08 Dokumentasikan perintah build dan run GUI.
- [ ] G0-09 Kunci default routing, persilangan, platform, dan format simpan.
- [ ] G0-10 Perbarui `docs/02-architecture.md` untuk `sim-gui` dan pengecualian dependensi.

**Acceptance G0:** `cargo run -p sim-gui` membuka window kosong, CI
membangun crate, dan tidak ada dependensi eksternal di luar
`egui`/`eframe`.

---

## G1 - Canvas, Grid, dan Layout

- [ ] G1-01 Buat app shell dengan central panel, toolbar, dan status bar.
- [ ] G1-02 Implementasikan kamera world ke screen dan screen ke world.
- [ ] G1-03 Implementasikan pan, zoom, fit all, dan fit selection.
- [ ] G1-04 Gambar grid minor, grid major, dan sumbu X/Y.
- [ ] G1-05 Terapkan level of detail grid dan label.
- [ ] G1-06 Buat layout canvas 90 persen dengan sheet overlay.
- [ ] G1-07 Implementasikan culling grid dan objek di luar viewport.
- [ ] G1-08 Tampilkan koordinat grid dan status snap di status bar.
- [ ] G1-09 Tambahkan tema gelap, terang, dan high-DPI.
- [ ] G1-10 Test transformasi, zoom, snap, dan culling.

**Acceptance G1:** canvas kosong berjalan 60 FPS, grid simetris terlihat,
pan dan zoom stabil, dan area canvas mendominasi layout.

---

## G2 - Model Dokumen dan Library

- [ ] G2-01 Definisikan tipe dokumen: komponen, pin, kabel, net, junction.
- [ ] G2-02 Definisikan metadata library: simbol, pin, parameter, probe.
- [ ] G2-03 Buat library awal: ATmega328P, LED, resistor, button, VCC, GND.
- [ ] G2-04 Buat library sheet dengan pencarian dan kategori.
- [ ] G2-05 Buat preview simbol dan recently used.
- [ ] G2-06 Implementasikan ghost placement dan validasi tabrakan.
- [ ] G2-07 Implementasikan move, rotate, dan mirror di grid.
- [ ] G2-08 Implementasikan selection dan inspector dasar.
- [ ] G2-09 Test transformasi pin, tabrakan, dan model dokumen.

**Acceptance G2:** pengguna dapat memilih komponen dari library,
menempatkannya di grid, memindahkan, memutar, dan mencerminkannya.

---

## G3 - Pin, Label, dan Net

- [ ] G3-01 Gambar pin, nama pin, nomor pin, dan arah pin.
- [ ] G3-02 Implementasikan hover tooltip pin.
- [ ] G3-03 Implementasikan klik pin untuk memilih net.
- [ ] G3-04 Implementasikan highlight net dan pin terhubung.
- [ ] G3-05 Tampilkan reference designator dan value komponen.
- [ ] G3-06 Tambahkan toggle visibility label dan level of detail.
- [ ] G3-07 Test hit testing dan pemetaan pin ke net.

**Acceptance G3:** setiap pin dapat diperiksa, dipilih, dan dihubungkan
ke net dengan umpan balik visual yang jelas.

---

## G4 - Kabel Ortogonal

- [ ] G4-01 Definisikan model wire, segment, vertex, dan konektivitas.
- [ ] G4-02 Implementasikan wire tool manual dengan vertex di grid.
- [ ] G4-03 Implementasikan occupancy map satu kabel per edge.
- [ ] G4-04 Tolak kabel yang melewati body komponen.
- [ ] G4-05 Tolak segmen diagonal dan vertex di luar grid.
- [ ] G4-06 Implementasikan junction eksplisit.
- [ ] G4-07 Implementasikan edit vertex dan delete wire.
- [ ] G4-08 Test konektivitas, collision, dan junction.

**Acceptance G4:** dua pin dapat dihubungkan dengan kabel ortogonal rapi,
tanpa segmen diagonal, dan tanpa menembus komponen.

---

## G5 - Routing, Warna, dan Label Net

- [ ] G5-01 Implementasikan saran jalur L dan Z.
- [ ] G5-02 Implementasikan maze router Manhattan dengan batas eksplorasi.
- [ ] G5-03 Terapkan aturan persilangan tanpa junction.
- [ ] G5-04 Deteksi kelas net dan petakan ke warna.
- [ ] G5-05 Implementasikan label net di pin atau midpoint.
- [ ] G5-06 Tambahkan legenda warna dan pola ramah buta warna.
- [ ] G5-07 Test routing, warna, label, dan aturan persilangan.

**Acceptance G5:** kabel mengikuti jalur grid yang tersedia, warna sesuai
fungsi net, dan persilangan tidak menimbulkan koneksi palsu.

---

## G6 - Integrasi Simulasi

- [ ] G6-01 Buat `SimSession` untuk CPU dan board.
- [ ] G6-02 Implementasikan run, pause, step, reset, dan stop.
- [ ] G6-03 Terapkan budget cycle per frame.
- [ ] G6-04 Sampel pin digital dan state board.
- [ ] G6-05 Tampilkan simulation time, cycle, dan status tertinggal.
- [ ] G6-06 Tampilkan error simulasi di UI tanpa panic.
- [ ] G6-07 Test integrasi ATmega328P dan LED.
- [ ] G6-08 Evaluasi ekstraksi runtime bersama dengan `sim-cli`.

**Acceptance G6:** sketsa blink dan UART hello dapat dijalankan, dijeda,
dan dilangkah dari GUI.

---

## G7 - Probe dan Grafik

- [ ] G7-01 Definisikan model probe dan ring buffer.
- [ ] G7-02 Implementasikan decimation min/max.
- [ ] G7-03 Buat widget plot analog internal.
- [ ] G7-04 Buat widget waveform digital.
- [ ] G7-05 Implementasikan decode bus digital.
- [ ] G7-06 Buat graph drawer, tab, dan floating window.
- [ ] G7-07 Tambahkan aksi add graph dari pin atau komponen.
- [ ] G7-08 Sampel node voltage dan branch current analog.
- [ ] G7-09 Test buffer, decimation, skala, cursor, dan render.

**Acceptance G7:** pengguna dapat membuka grafik per pin atau komponen,
melihat sinyal digital dan analog, serta memeriksa nilai pada cursor.

---

## G8 - Editing, Undo, dan Persistence

- [ ] G8-01 Implementasikan command pattern dan undo/redo.
- [ ] G8-02 Implementasikan multi-select dan selection box.
- [ ] G8-03 Implementasikan copy, paste, duplicate, dan delete.
- [ ] G8-04 Implementasikan align dan distribute.
- [ ] G8-05 Implementasikan format simpan teks ASCII versi 1.
- [ ] G8-06 Implementasikan load, validasi, dan round-trip.
- [ ] G8-07 Implementasikan autosave, backup, dan recent files.
- [ ] G8-08 Implementasikan shortcut dan context menu lengkap.
- [ ] G8-09 Test undo/redo, selection, dan persistence.

**Acceptance G8:** proyek dapat disimpan, dimuat kembali tanpa kehilangan
data, dan semua aksi editing dapat di-undo.

---

## G9 - Performa, Aksesibilitas, dan Polish

- [ ] G9-01 Implementasikan spatial hash untuk hit testing.
- [ ] G9-02 Optimalkan culling, cache, dan alokasi per frame.
- [ ] G9-03 Buat benchmark 500 komponen dan 2000 segmen kabel.
- [ ] G9-04 Tambahkan navigasi keyboard dan fokus yang jelas.
- [ ] G9-05 Audit palet warna dan kontras.
- [ ] G9-06 Tambahkan interaction test dengan `egui::Context`.
- [ ] G9-07 Tulis user guide GUI dan screenshot.

**Acceptance G9:** target 60 FPS terukur, semua tool dapat dipakai dengan
keyboard, dan tidak ada panic pada input tidak valid.

---

## G10 - Release GUI

- [ ] G10-01 Buat paket Linux dan desktop entry.
- [ ] G10-02 Verifikasi build Windows.
- [ ] G10-03 Verifikasi build macOS.
- [ ] G10-04 Buat contoh proyek: LED blink dan UART hello.
- [ ] G10-05 Perbarui README, docs, dan TODO.
- [ ] G10-06 Tag preview GUI.

**Acceptance G10:** binary GUI dapat dijalankan di Linux, contoh proyek
terbuka, dan dokumentasi instalasi tersedia.

---

## Non-Goals MVP

- Autorouting PCB penuh.
- Editing simbol dari dalam GUI.
- Kolaborasi real-time.
- Import dan export EDA eksternal.
- Render 3D.
- Analog device lengkap sebelum digital stabil.

---

## Definition of Done GUI

- [ ] Perubahan sesuai task.
- [ ] Test model atau interaction ditambahkan.
- [ ] Tidak ada dependensi eksternal selain `egui`/`eframe`.
- [ ] Tidak ada `unsafe`.
- [ ] Setiap fungsi maksimal 20 baris.
- [ ] Guard clause dipakai.
- [ ] ASCII-only.
- [ ] fmt, clippy, dan test lulus.
- [ ] Dokumentasi diperbarui.
- [ ] Commit memakai conventional commit.
