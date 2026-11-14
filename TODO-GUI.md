# TODO - GUI egui

Backlog eksekusi untuk GUI interaktif berbasis `egui`/`eframe`.

**Status:** implementasi inti G0 sampai G10 selesai. Item yang tertunda
karena batas netlist analog, renderer golden, atau native runner dicatat
di bagian Status Akhir.

**Desain:** [`docs/57-gui-egui.md`](docs/57-gui-egui.md)

**Keputusan yang sudah disetujui:**

- `egui`, `eframe`, `serde`, `serde_json`, `rfd`, dan `png` hanya boleh
  dipakai di crate `sim-gui`.
- Crate simulasi lain tetap zero external dependency.
- `egui_plot`, `libloading`, dan crate eksternal lain tidak diizinkan
  tanpa keputusan baru.
- Grafik memakai widget internal berbasis `egui::Painter`.
- Format proyek JSON melalui `serde_json`.
- Dialog file memakai `rfd`.
- Export PNG memakai crate `png`.
- Plugin memakai trait internal, bukan dynamic library.
- Golden image memakai snapshot RGBA.

**Keputusan desain yang sudah dikunci:**

- Linux desktop lebih dulu.
- Editor, simulasi, dan grafik digital serta analog.
- Canvas 90 persen dengan overlay sheet.
- Library kategori dan pencarian.
- Klik dan drag-and-drop untuk penempatan.
- Routing manual dengan saran L/Z.
- Satu edge grid hanya satu kabel.
- Persilangan tanpa junction dilarang.
- Junction otomatis pada 3 atau 4 arah.
- Label pin memakai LOD dan hover.
- Warna net mengikuti kelas dan dapat dioverride.
- Grafik di drawer dan floating window.
- Simulasi pada UI thread dengan budget per frame.
- Run, pause, step, reset.
- Ring buffer grafik 10 detik.
- Undo 100 langkah.
- Autosave debounce dan backup.
- Tema gelap default.
- Satu sheet.
- Probe dibuka dari context menu pin.
- Target 500 komponen dan 60 FPS.
- Simbol bawaan.
- Command palette Ctrl+K.
- Sheet dapat dipin atau dilepas.
- Format `.emu.json`, pretty print 2 spasi, migrasi otomatis.
- Sampling default 100 cycle, decimation min/max.
- Inspector tiga section.
- Plugin dua trait internal, probe analog read-only.
- Satu proyek per window, prompt reload saat file berubah.
- Library komponen berupa kode Rust internal.
- Multi-select dipindahkan bersama.
- Snap selalu aktif, Alt menonaktifkan sementara.
- Overlap komponen ditolak.
- Selection box memakai full containment.
- Zoom berpusat di kursor.
- Font UI monospace.
- Minimap di kanan bawah, default tersembunyi.

---

## G0 - Fondasi dan Keputusan

- [x] G0-01 Setujui pengecualian dependensi GUI di `sim-gui`.
- [x] G0-02 Catat keputusan di `AGENTS.md` dan `docs/57`.
- [x] G0-03 Buat crate `crates/sim-gui` dengan binary `embed-emu-gui`.
- [x] G0-04 Tambahkan `sim-gui` ke workspace dan dependency graph.
- [x] G0-05 Pin versi semua dependensi `sim-gui`, commit `Cargo.lock`.
- [ ] G0-06 Buat app error type dan aturan no-panic untuk input GUI.
- [x] G0-07 Tambahkan build dan test `sim-gui` ke CI Linux.
- [x] G0-08 Dokumentasikan perintah build dan run GUI.
- [x] G0-09 Kunci default routing, persilangan, platform, dan format simpan.
- [x] G0-10 Perbarui `docs/02-architecture.md` untuk `sim-gui` dan daftar dependensi.
- [x] G0-11 Kunci grid default 20 px, zoom preset dan halus.
- [x] G0-12 Kunci palet warna final untuk setiap kelas net.
- [x] G0-13 Kunci field skema JSON versi 1.
- [x] G0-14 Kunci signature trait `ComponentProvider`.
- [ ] G0-15 Kunci API probe analog read-only dan default N 100 cycle.
- [x] G0-16 Ekstensi file `.emu.json`; konfigurasi di XDG config.

**Acceptance G0:** `cargo run -p sim-gui` membuka window kosong, CI
membangun crate, dan tidak ada dependensi eksternal di luar daftar
`sim-gui`.

---

## G1 - Canvas, Grid, dan Layout

- [x] G1-01 Buat app shell dengan central panel, toolbar, dan status bar.
- [x] G1-02 Implementasikan kamera world ke screen dan screen ke world.
- [x] G1-03 Implementasikan pan, zoom preset, zoom halus, fit all, dan fit selection.
- [x] G1-04 Gambar grid minor 20 px, grid major, sumbu X/Y, dan titik grid.
- [x] G1-05 Terapkan level of detail grid dan label.
- [x] G1-06 Buat layout canvas dominan dengan sheet overlay.
- [x] G1-07 Implementasikan culling grid dan objek di luar viewport.
- [x] G1-08 Tampilkan koordinat grid dan status snap di status bar.
- [x] G1-09 Tambahkan tema gelap, terang, high-DPI, dan aksen cyan.
- [x] G1-10 Test transformasi, zoom, snap, dan culling.
- [x] G1-11 Implementasikan minimap opsional di kanan bawah.
- [ ] G1-12 Terapkan latar dark dotted dan toolbar ikon serta teks.
- [x] G1-13 Tampilkan koordinat, zoom, seleksi, sim time, dan FPS di status bar.
- [x] G1-14 Gunakan font monospace untuk seluruh UI.
- [x] G1-15 Implementasikan zoom berpusat di kursor.

**Acceptance G1:** canvas kosong berjalan 60 FPS, grid simetris terlihat,
pan dan zoom stabil, dan area canvas mendominasi layout.

---

## G2 - Model Dokumen dan Library

- [x] G2-01 Definisikan tipe dokumen: komponen, pin, kabel, net, junction.
- [x] G2-02 Definisikan metadata library: simbol vektor datar, pin, parameter, probe.
- [x] G2-03 Buat library awal: ATmega328P, board Arduino, passive, LED, button, VCC, GND.
- [x] G2-04 Buat library sheet dengan pencarian dan kategori.
- [x] G2-05 Buat daftar recently used di library.
- [x] G2-15 Buat preview simbol di library.
- [x] G2-06 Implementasikan ghost placement dan klik untuk menempatkan.
- [ ] G2-16 Implementasikan drag-and-drop dari library ke kanvas.
- [x] G2-07 Implementasikan move, rotate, dan mirror di grid dengan pusat rotasi.
- [x] G2-08 Implementasikan selection dan inspector dasar.
- [x] G2-09 Test transformasi pin, tabrakan, dan model dokumen.
- [x] G2-10 Definisikan trait internal `ComponentProvider` untuk plugin komponen.
- [x] G2-11 Tampilkan label referensi dan nilai komponen secara default.
- [x] G2-12 Simpan definisi library sebagai kode Rust internal.
- [ ] G2-13 Terapkan snap selalu aktif dan Alt menonaktifkan sementara.
- [x] G2-14 Tolak penempatan komponen yang saling menimpa.

**Acceptance G2:** pengguna dapat memilih komponen dari library,
menempatkannya di grid, memindahkan, memutar, dan mencerminkannya.

---

## G3 - Pin, Label, dan Net

- [x] G3-01 Gambar pin sebagai garis dengan titik, nama, nomor, dan arah.
- [x] G3-02 Implementasikan hover tooltip pin.
- [x] G3-03 Implementasikan klik pin untuk memilih net.
- [x] G3-04 Implementasikan highlight net dan endpoint terhubung.
- [x] G3-05 Tampilkan reference designator dan value komponen.
- [x] G3-06 Terapkan label pin nama dan nomor dengan LOD dan hover.
- [x] G3-07 Test hit testing dan pemetaan net wire.
- [x] G3-08 Tampilkan label net saat hover atau seleksi.
- [x] G3-09 Buat inspector tiga section: properties, pins, simulation.
- [x] G3-10 Pindahkan semua komponen terpilih saat multi-select digeser.
- [x] G3-11 Highlight seluruh net saat wire net dipilih.
- [x] G3-12 Gunakan selection box full containment.

**Acceptance G3:** setiap pin dapat diperiksa, dipilih, dan dihubungkan
ke net dengan umpan balik visual yang jelas.

---

## G4 - Kabel Ortogonal

- [x] G4-01 Definisikan model wire, segment, vertex, dan konektivitas.
- [x] G4-02 Implementasikan wire tool manual dengan vertex di grid.
- [x] G4-03 Implementasikan occupancy map satu kabel per edge.
- [x] G4-04 Tolak kabel yang melewati body komponen.
- [x] G4-05 Tolak segmen diagonal dan vertex di luar grid.
- [x] G4-06 Implementasikan junction otomatis 3 atau 4 arah.
- [x] G4-07 Implementasikan segment drag untuk mengedit kabel.
- [x] G4-08 Test konektivitas, collision, dan junction.
- [x] G4-09 Terapkan belokan 90 derajat tegas.
- [ ] G4-10 Implementasikan animasi aliran sinyal opsional, default mati.
- [x] G4-11 Tolak ujung kabel yang bukan pin, junction, atau vertex valid.
- [x] G4-12 Implementasikan selection dan delete wire.

**Acceptance G4:** dua pin dapat dihubungkan dengan kabel ortogonal rapi,
tanpa segmen diagonal, dan tanpa menembus komponen.

---

## G5 - Routing, Warna, dan Label Net

- [x] G5-01 Implementasikan saran jalur L.
- [x] G5-02 Dokumentasikan keputusan menunda auto-router penuh.
- [x] G5-03 Terapkan aturan persilangan tanpa junction.
- [x] G5-04 Deteksi kelas net dan petakan ke warna.
- [x] G5-05 Implementasikan label net saat hover, seleksi, atau zoom tinggi.
- [x] G5-06 Tambahkan legenda warna dan pola ramah buta warna.
- [x] G5-07 Test routing, warna, label, dan aturan persilangan.
- [x] G5-08 Implementasikan override warna per wire.
- [x] G5-09 Kunci palet warna final dan target kontras WCAG AA.
- [x] G5-10 Implementasikan ruler grid dan probe tegangan PB5.
- [x] G5-11 Implementasikan saran jalur Z.

**Acceptance G5:** kabel mengikuti jalur grid yang tersedia, warna sesuai
fungsi net, dan persilangan tidak menimbulkan koneksi palsu.

---

## G6 - Integrasi Simulasi

- [x] G6-01 Buat `SimMachine` untuk CPU dan board.
- [x] G6-02 Implementasikan run, pause, step, dan reset.
- [x] G6-03 Terapkan batch 100 instruksi per iterasi worker.
- [x] G6-04 Sampel pin PB5 dan state LED board.
- [x] G6-05 Tampilkan cycle dan status running atau paused.
- [x] G6-06 Tampilkan error simulasi di UI tanpa panic.
- [x] G6-07 Test integrasi ATmega328P dan LED.
- [x] G6-08 Evaluasi ekstraksi runtime bersama dengan `sim-cli`.
- [ ] G6-09 Implementasikan sampling probe setiap N cycle yang dapat dikonfigurasi.
- [x] G6-10 Tampilkan output UART TX di monitor GUI.
- [x] G6-11 Implementasikan input UART RX dari GUI.

**Acceptance G6:** sketsa blink dan UART hello dapat dijalankan, dijeda,
dan dilangkah dari GUI.

---

## G7 - Probe dan Grafik

- [ ] G7-01 Definisikan model probe dan ring buffer 10 detik.
- [ ] G7-02 Implementasikan decimation min/max.
- [ ] G7-03 Buat widget plot analog internal.
- [x] G7-04 Buat widget waveform digital PB5 dan LED.
- [x] G7-16 Tampilkan cursor cycle pada graph digital.
- [x] G7-05 Dokumentasikan keputusan menunda decode bus.
- [ ] G7-06 Buat graph drawer, tab, dan floating window.
- [ ] G7-07 Tambahkan aksi add graph dari pin atau komponen.
- [ ] G7-08 Sampel node voltage dan branch current melalui API read-only.
- [ ] G7-09 Test buffer, decimation, skala, cursor, dan render.
- [ ] G7-10 Definisikan trait internal `AnalysisProvider` untuk plugin analisis.
- [ ] G7-11 Batasi grafik maksimal 8 series.
- [ ] G7-12 Implementasikan cursor, zoom, pan, autoscale, dan legend toggle.
- [ ] G7-13 Gunakan default sampling setiap 100 cycle, dapat dikonfigurasi per probe.
- [x] G7-14 Gunakan waktu simulasi sebagai sumbu X.
- [ ] G7-15 Warna series mengikuti warna net atau pin.

**Acceptance G7:** pengguna dapat membuka grafik per pin atau komponen,
melihat sinyal digital dan analog, serta memeriksa nilai pada cursor.

---

## G8 - Editing, Undo, dan Persistence

- [x] G8-01 Implementasikan undo/redo berbasis snapshot.
- [x] G8-02 Implementasikan multi-select, shift-click, dan selection box.
- [x] G8-03 Implementasikan copy, paste, dan duplicate.
- [x] G8-04 Implementasikan align dan distribute.
- [x] G8-05 Implementasikan format `.emu.json` versi 1 dengan pretty print 2 spasi.
- [x] G8-06 Implementasikan load, validasi, dan round-trip.
- [x] G8-07 Implementasikan autosave debounce 2 detik, backup, 10 recent files, dan `rfd`.
- [x] G8-08 Implementasikan command palette Ctrl+K.
- [x] G8-20 Implementasikan context menu dasar untuk edit dan tool.
- [x] G8-22 Tambahkan properties dan graph ke context menu.
- [x] G8-23 Tambahkan aksi warna dan label net ke context menu.
- [x] G8-21 Implementasikan rotate dan mirror untuk seluruh seleksi.
- [x] G8-09 Test undo/redo dan persistence.
- [x] G8-10 Kelompokkan undo per aksi drag atau klik.
- [x] G8-11 Simpan konfigurasi aplikasi di XDG config.
- [x] G8-12 Simpan 10 recent files.
- [x] G8-13 Tampilkan status sebagai toast singkat dan status bar.
- [x] G8-14 Gunakan UI berbahasa Inggris dan dokumentasi Indonesia.
- [x] G8-15 Satu window menampilkan satu proyek.
- [x] G8-16 Prompt reload jika file berubah di luar aplikasi.
- [x] G8-17 Konfirmasi hapus untuk komponen dengan kabel atau seleksi besar.
- [x] G8-18 Implementasikan migrasi otomatis skema JSON v1 ke v2.
- [x] G8-19 Test selection komponen, wire, dan selection box.

**Acceptance G8:** proyek dapat disimpan, dimuat kembali tanpa kehilangan
data, dan semua aksi editing dapat di-undo.

---

## G9 - Performa, Aksesibilitas, dan Polish

- [ ] G9-01 Implementasikan spatial hash untuk hit testing.
- [ ] G9-02 Optimalkan culling, cache, dan alokasi per frame.
- [ ] G9-03 Buat benchmark 500 komponen dan 2000 segmen kabel.
- [ ] G9-04 Tambahkan navigasi keyboard dan fokus yang jelas.
- [x] G9-05 Audit palet warna dan kontras.
- [x] G9-06 Tambahkan interaction test dengan `egui::Context`.
- [x] G9-07 Tulis user guide GUI dan screenshot.
- [x] G9-08 Implementasikan export PNG melalui crate `png`.
- [ ] G9-09 Tambahkan golden image RGBA snapshot.
- [ ] G9-10 Lengkapi test model, interaction, integrasi, dan visual.
- [x] G9-11 Verifikasi kontras WCAG AA dan shortcut keyboard.
- [x] G9-12 Sediakan minimap opsional dengan toggle.
- [ ] G9-13 Dokumentasikan prosedur update golden RGBA.
- [x] G9-14 Sediakan panel log status dan error opsional.

**Acceptance G9:** target 60 FPS terukur, semua tool dapat dipakai dengan
keyboard, dan tidak ada panic pada input tidak valid.

---

## G10 - Release GUI

- [x] G10-01 Buat paket Linux dan desktop entry.
- [ ] G10-02 Verifikasi build Windows (perlu native runner).
- [ ] G10-03 Verifikasi build macOS (perlu native runner).
- [x] G10-04 Buat contoh proyek: LED blink dan UART hello.
- [x] G10-05 Perbarui README, docs, dan TODO.
- [x] G10-06 Tag preview GUI.

**Acceptance G10:** binary GUI dapat dijalankan di Linux, contoh proyek
terbuka, dan dokumentasi instalasi tersedia.

---

## Status Akhir G0-G10

Selesai dan teruji:

- Canvas, grid, zoom, fit, culling, minimap, selection box, multi-select.
- Library, placement, ghost, rotate, mirror grup, copy, paste, duplicate,
  align, distribute.
- Wire ortogonal, L/Z route, junction, pola net, label, legenda, net
  highlight, aturan persilangan.
- Simulasi PB5/LED, run, pause, step, reset, UART TX, graph digital,
  cursor cycle.
- Undo/redo, JSON, autosave, backup, recent files, XDG config, prompt
  reload, command palette, context menu, toast, log, PNG export.
- CI workspace Linux, paket Linux, desktop entry, contoh proyek, panduan
  pengguna.

Tertunda karena batas teknis atau lingkungan:

- Probe generik dan grafik analog: skematik GUI belum memiliki netlist
  elektrik yang terhubung ke solver analog.
- Decimation, sampling analog, dan branch current belum ada karena alasan
  yang sama.
- Golden image RGBA dan interaction visual penuh memerlukan renderer
  headless yang stabil.
- Build Windows dan macOS belum dijalankan pada native runner.
- Drag-and-drop library, preview simbol, Alt snap, animasi aliran, dan
  override warna net belum diimplementasikan.
- Migrasi skema JSON lama belum ada karena baru ada versi 1.

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
- [ ] Tidak ada dependensi eksternal di luar daftar `sim-gui`.
- [ ] Tidak ada `unsafe`.
- [ ] Setiap fungsi maksimal 20 baris.
- [ ] Guard clause dipakai.
- [ ] ASCII-only.
- [ ] fmt, clippy, dan test lulus.
- [ ] Dokumentasi diperbarui.
- [ ] Commit memakai conventional commit.
