# 57 - GUI egui untuk Pengembangan dan Pengujian

Dokumen ini merancang GUI interaktif berbasis egui untuk eksplorasi,
penempatan komponen, probing, dan pengujian simulasi.

**Status:** G0 dan G1 awal sudah diimplementasikan: crate `sim-gui`, app
shell, canvas, grid, pan, zoom, status bar, dan test kamera. Library
komponen awal, placement ghost, selection, drag move, rotate, mirror,
inspector dasar, wire ortogonal, junction otomatis, warna net,
selection wire, delete wire, segment drag, saran jalur L, saran jalur Z, save/load
JSON, undo/redo, autosave, backup, recent files, konfigurasi XDG,
deteksi perubahan file eksternal, label net, legenda warna, command
palette, multi-select, selection box, copy/paste/duplicate, rotate/mirror
grup, context menu, monitor UART TX, pola garis net, toast, konfirmasi
hapus, export PNG, minimap, log panel, tooltip pin, tema terang,
recently used, net highlight, simulasi PB5, dan graph digital PB5/LED
dengan cursor juga sudah ada. Probe generik, UART input ke
simulasi, dan grafik analog belum ada. Dependensi
GUI yang disetujui hanya untuk crate `sim-gui`: `egui`, `eframe`,
`serde`, `serde_json`, `rfd`, dan `png`. `egui_plot`,
`libloading`, dan crate lain tidak diizinkan tanpa keputusan baru.

**Rekomendasi:** buat crate baru `sim-gui` sebagai adapter/composition,
mirip `sim-cli`. Semua crate simulasi tetap zero external dependency.
`egui_plot` dan crate eksternal lain tidak termasuk pengecualian ini.

---

## 1. Keputusan dan Batas

### 1.1 Konflik dependensi

Aturan repository sebelumnya:

> Zero external dependency. Hanya Rust standard library dan crate
> internal workspace.

`egui` dan `eframe` adalah dependensi eksternal.

**Keputusan:** dependensi eksternal hanya diizinkan di crate `sim-gui`,
yaitu `egui`, `eframe`, `serde`, `serde_json`, `rfd`, dan `png`.
Semua crate lain tetap zero external dependency.
`egui_plot`, `libloading`, dan crate eksternal lain tidak diizinkan tanpa
keputusan baru.

**Keputusan tambahan:**

- Format proyek memakai JSON melalui `serde_json`.
- Dialog file memakai `rfd`.
- Export PNG memakai crate `png`.
- Plugin memakai trait internal, bukan dynamic library.

### 1.2 Backend grafik

**Keputusan:** gunakan widget plot internal berbasis `egui::Painter`.
Tidak memakai `egui_plot` pada MVP. Jika kebutuhan grafik terbukti
melebihi widget internal, penambahan `egui_plot` harus disetujui
terpisah.

### 1.3 Keputusan MVP

Keputusan yang sudah dikunci:

- Linux desktop.
- Editor skematik, simulasi, dan grafik.
- Canvas 90 persen dengan overlay sheet.
- Library kategori dan pencarian.
- Klik dan drag-and-drop untuk penempatan.
- Routing manual dengan saran L/Z.
- Satu kabel per edge grid.
- Persilangan tanpa junction dilarang.
- Junction otomatis pada 3 atau 4 arah.
- Label pin memakai level of detail dan hover.
- Warna kabel mengikuti kelas net dan dapat dioverride.
- Grafik digital dan analog.
- Drawer dan floating window untuk grafik.
- Simulasi pada worker thread dengan batch kecil dan channel.
- Run, pause, step, reset.
- Ring buffer grafik 10 detik.
- JSON untuk format proyek.
- Dialog file OS melalui `rfd`.
- Undo 100 langkah.
- Autosave debounce dan backup.
- Tema gelap default.
- Satu sheet.
- Grafik dibuka dari context menu pin.
- Target 500 komponen dan 60 FPS.
- Export PNG.
- Simbol bawaan, belum ada editor simbol.
- Command palette Ctrl+K.
- Sheet dapat dipin atau dilepas.
- Test model, interaksi, dan golden image RGBA.
- Plugin API berupa trait internal untuk komponen dan analisis.

---

## 2. Tujuan

GUI harus membantu pengembangan dan pengujian, bukan menjadi pengganti
penuh aplikasi EDA.

Fokus:

- Canvas besar untuk membaca rangkaian dan mengamati perilaku.
- Penempatan komponen cepat dari library.
- Kabel rapi karena semua titik berada di grid.
- Pin dan net mudah diperiksa.
- Grafik per komponen atau per pin.
- Simulasi dapat dijalankan, dijeda, dan dilangkah.
- Kesalahan wiring terlihat jelas sebelum simulasi.

Bukan fokus awal:

- Autorouting kompleks seperti PCB.
- Editing simbol tingkat lanjut.
- Kolaborasi real-time.
- Render 3D.
- Import/export format EDA eksternal.

---

## 3. Prinsip Desain

1. Canvas adalah pusat. Target 90 persen area kerja.
2. Panel lain muncul sebagai sheet, drawer, atau floating window.
3. Semua titik listrik berada di grid integer.
4. Kabel hanya horizontal dan vertikal.
5. Satu edge grid hanya boleh memuat satu kabel, kecuali keputusan
   desain kemudian mengizinkan bundling.
6. Persimpangan harus eksplisit. Kabel yang menyilang tanpa junction
   tidak dianggap terhubung.
7. Komponen dan pin memiliki label yang dapat disembunyikan.
8. Warna kabel mengikuti kelas net, bukan warna acak.
9. Grafik tidak boleh mengurangi canvas secara permanen.
10. Semua aksi editing harus dapat di-undo dan di-redo.
11. Model dokumen tidak boleh bergantung pada tipe egui.
12. Simulasi tetap deterministik dan tidak memakai wall-clock.

---

## 4. Layout

```text
+--------------------------------------------------------------+
| Toolbar: file, undo, run, step, zoom, grid, wire, probe       |
+---------+----------------------------------------------------+
| Library |                                                    |
| sheet   |                    Canvas 90%                      |
| (overlay|                                                    |
|  atau   |                                                    |
|  pin)   |                                                    |
|         +----------------------------------------------------+
|         | Inspector sheet (kanan, collapsible)               |
+---------+----------------------------------------------------+
| Status: grid, koordinat, seleksi, sim time, FPS, status       |
+--------------------------------------------------------------+
```

Aturan layout:

- Central panel mengambil seluruh area yang tersisa.
- Toolbar atas sekitar 44 px.
- Status bar bawah sekitar 24 px.
- Library dan Inspector adalah sheet overlay yang dapat di-pin.
- Sheet default tertutup agar canvas tetap 90 persen.
- Analysis drawer di bawah canvas, default tertutup, tinggi maksimum
  30 persen saat terbuka.
- Grafik dapat menjadi tab di analysis drawer atau floating window.
- Layout terakhir disimpan di file konfigurasi.
- Tema default gelap, dengan opsi tema terang.
- Aksen UI cyan.
- UI berbahasa Inggris; dokumentasi tetap Indonesia.
- Font UI monospace untuk seluruh antarmuka.
- Target kontras WCAG AA.
- Pin/unpin sheet mengubah overlay menjadi sidebar tanpa docking penuh.

---

## 5. Grid dan Kanvas

### 5.1 Koordinat

- Model memakai `GridPoint { x: i32, y: i32 }`.
- Satu unit grid adalah satuan logis, bukan piksel.
- Piksel hanya dipakai saat rendering.
- Semua pin, vertex kabel, junction, dan label anchor berada di grid.
- Body komponen boleh digambar setengah sel untuk simetri visual, tetapi
  hit box dan anchor listrik tetap di grid.

### 5.2 Grid simetris

- Komponen memiliki anchor di titik grid.
- Pin didefinisikan sebagai offset integer dari anchor.
- Rotasi 0, 90, 180, 270 derajat dan mirror X/Y harus mempertahankan
  posisi pin pada grid.
- Komponen dengan pin kiri-kanan atau atas-bawah harus simetris secara
  elektrik dan visual.
- Alignment guide muncul saat komponen mendekati pusat atau pin lain.

### 5.3 Kamera

- Pan dengan drag tengah, Space + drag, atau dua jari.
- Zoom dengan wheel di sekitar kursor.
- Preset zoom: 25, 50, 75, 100, 150, 200, 400 persen.
- Zoom halus tersedia di antara preset.
- Fit selection, fit all, dan center origin.
- Level of detail: label kecil dan grid minor disembunyikan saat zoom
  rendah.

### 5.4 Rendering grid

- Ukuran sel default 20 px pada zoom 100 persen.
- Grid minor setiap sel.
- Grid major setiap 5 atau 10 sel.
- Latar gelap dengan titik grid halus.
- Sumbu X dan Y berbeda warna.
- Culling hanya menggambar garis yang terlihat.
- Grid dapat dimatikan sementara untuk melihat rangkaian.

### 5.5 Minimap

- Minimap opsional di sudut kanan bawah canvas.
- Default tersembunyi.
- Menampilkan viewport, komponen, dan kabel secara ringkas.
- Klik minimap memindahkan kamera.

---

## 6. Komponen dan Library

### 6.1 Definisi komponen

Setiap definisi memiliki:

- ID stabil.
- Nama dan kategori.
- Simbol vektor datar.
- Daftar pin: nama, nomor, arah, tipe listrik, offset grid.
- Pin digambar sebagai garis dengan titik di ujung.
- Parameter default.
- Probe yang tersedia.
- Batas ukuran dan anchor.

### 6.2 Instance komponen

Setiap instance memiliki:

- ID unik.
- Referensi definition.
- Posisi grid.
- Rotasi dan mirror.
- Label referensi dan nilai ditampilkan secara default, misalnya `R1 220`.
- Label referensi, nilai, dan parameter.
- Status lock dan visibility.
- Probe yang aktif.

### 6.3 Library sheet

- Library awal: ATmega328P, board Arduino, passive, LED, button, VCC,
  dan GND.
- Definisi komponen disimpan sebagai kode Rust internal.
- Kategori tetap disiapkan untuk MCU, board, passive, diode, transistor,
  digital, source, probe, ground, dan power.
- Pencarian nama dan kategori.
- Preview simbol.
- Drag-and-drop ke canvas atau klik lalu tempatkan.
- Recently used.
- Filter berdasarkan tipe pin atau domain sinyal.
- Simbol bawaan untuk MVP; editor simbol belum tersedia.

### 6.4 Penempatan

- Ghost komponen mengikuti kursor.
- Snap selalu aktif; Alt menonaktifkan snap sementara.
- Hijau jika valid, merah jika bertabrakan.
- Komponen tidak boleh saling menimpa.
- R rotate, M mirror, Esc cancel.
- Setelah ditempatkan, komponen langsung dapat dipilih.

### 6.5 Pin

- Pin menampilkan nama dan nomor sesuai level of detail.
- Hover menampilkan tooltip: nama, net, arah, tegangan, arus, state.
- Klik pin memilih net.
- Drag dari pin memulai kabel.
- Klik kanan membuka menu probe, graph, disconnect, dan properties.
- Pin terhubung dan belum terhubung dibedakan secara visual.

---

## 7. Kabel

### 7.1 Aturan geometri

- Semua segmen horizontal atau vertikal.
- Semua vertex berada di grid.
- Tidak ada segmen diagonal.
- Belokan memakai sudut 90 derajat tegas.
- Panjang minimum satu sel.
- Kabel hanya boleh berakhir di pin, junction, atau vertex grid yang valid.
- Kabel menggantung tanpa koneksi tidak diizinkan.
- Kabel tidak boleh melewati body komponen.
- Satu edge grid hanya boleh dipakai satu kabel pada MVP.
- Persilangan hanya boleh di junction eksplisit.
- Jika edge sudah terpakai, kabel harus berbelok atau mencari jalur lain.
- Animasi aliran sinyal opsional dan default mati.

### 7.2 Mode menggambar

- Manual: klik pin, klik titik grid, klik titik grid berikutnya,
  double-click atau Enter untuk selesai, Esc untuk batal.
- Klik pin tujuan kedua menyelesaikan kabel secara otomatis.
- L-route: pilih dua titik, sistem menawarkan jalur L.
- Z-route: pilih dua titik, sistem menawarkan jalur Z.
- Auto-route sederhana ditunda sampai setelah MVP.
- Pengguna selalu dapat memilih vertex dan menggesernya selama tetap
  di grid.

### 7.3 Junction

- Junction adalah objek eksplisit di titik grid.
- Tiga atau empat segmen pada satu titik wajib memiliki junction.
- Sistem membuat junction otomatis saat tiga atau empat arah bertemu.
- Pengguna dapat menghapus atau menambah junction secara manual.
- Menghapus junction memutus koneksi.
- Junction memiliki ukuran dan warna yang konsisten.

### 7.4 Warna kabel

Warna mengikuti kelas net:

| Kelas | Warna usulan |
|---|---|
| Power | Merah |
| Ground | Abu gelap |
| Digital | Hijau |
| Analog | Biru |
| Clock | Oranye |
| UART | Cyan |
| SPI | Ungu |
| I2C | Kuning |
| PWM | Magenta |
| Reset | Coklat |
| User | Abu terang |

Aturan:

- Warna harus ramah buta warna dan memiliki pola tambahan, misalnya
  ketebalan atau dash, jika diperlukan.
- Pola dash per kelas sudah diterapkan dan ditampilkan di legenda.
- Kontras semua warna net terhadap latar kanvas diuji minimal 4.5:1.
- Legenda warna tersedia melalui tombol `Legend` di toolbar.
- Warna default mengikuti kelas net.
- Warna net dapat dioverride per net.
- Kabel terpilih memiliki halo, bukan mengganti warna kelas.

### 7.5 Label net

- Label default muncul saat hover, seleksi, atau zoom minimal 1.5.
- Label dapat ditempel di pin, midpoint, atau vertex.
- Label mengikuti rotasi kabel.
- Label dapat disembunyikan per net atau secara global.
- Nama net otomatis dibuat jika belum ada, misalnya `N$001`.

---

## 8. Interaksi

### 8.1 Tool mode

- Select: klik, shift-click atau Ctrl-click, dan box select dari area kosong.
- Drag komponen terpilih memindahkan seluruh grup.
- Place: menempatkan komponen dari library.
- Wire: menggambar kabel ortogonal.
- Probe: memilih pin atau net untuk grafik.
- Pan: drag kanvas.
- Measure: ruler grid dan probe tegangan.
- Toolbar memakai ikon dan teks singkat.
- Error ditampilkan sebagai toast singkat dan status bar.
- Log error opsional tersedia sebagai tab di analysis drawer.
- Font UI monospace untuk seluruh antarmuka.

### 8.2 Shortcut

| Tombol | Aksi |
|---|---|
| V | Select |
| P | Place |
| W | Wire |
| G | Probe |
| Space + drag | Pan |
| Wheel | Zoom |
| R | Rotate |
| M | Mirror |
| Delete | Hapus seleksi |
| Ctrl+C | Copy seleksi |
| Ctrl+V | Paste |
| Ctrl+D | Duplicate |
| Ctrl+Z | Undo |
| Ctrl+Shift+Z | Redo |
| Ctrl+S | Save |
| Ctrl+O | Open |
| Ctrl+K | Command palette |
| Klik kanan | Context menu |
| Esc | Batal tool |
| Enter | Selesai wire |

### 8.3 Seleksi

- Klik memilih satu objek.
- Shift-click menambah atau mengurangi seleksi.
- Drag kosong membuat selection box.
- Selection box memilih objek yang seluruhnya berada di dalam kotak.
- Multi-select dapat dipindahkan, dirotasi, dan dimirror bersama.
- Klik net memilih seluruh net dan menghighlight semua pin serta kabel.
- Hapus satu objek langsung; komponen dengan kabel atau seleksi besar
  meminta konfirmasi.
- Alignment guide dan distribusi horizontal/vertikal.

### 8.4 Context menu

- MVP saat ini: copy, paste, duplicate, delete, rotate, mirror, dan
  pilih tool Select atau Wire.
- Komponen: properties, rotate, mirror, duplicate, delete, add graph.
- Pin: show graph, add probe, disconnect, highlight net.
- Kabel: select net, change color, add label, delete.
- Kanvas: paste, fit all, toggle grid, add component.

### 8.5 Inspector

- Section Properties: ID, tipe, posisi, rotasi, mirror, nilai.
- Section Pins: nama, nomor, net, arah, state.
- Section Simulation: pin voltage, current, probe, dan graph.
- Section mengikuti objek yang dipilih.
- Objek ganda menampilkan ringkasan dan aksi massal.

---

## 9. Simulasi

### 9.1 Kontrol

- Run, pause, step one instruction, reset.
- Monitor UART menampilkan byte TX dari CPU, buffer 16 KiB, dan auto-scroll.
- Simulation time dan cycle count terlihat di status bar.
- Speed target dalam cycle per frame, bukan klaim real-time.
- Status menunjukkan apakah simulasi tertinggal dari target.

### 9.2 Sinkronisasi

- Worker berjalan di background thread dan berkomunikasi lewat channel.
- Worker mengeksekusi batch 100 instruksi, lalu memeriksa command
  dengan timeout 2 ms agar GUI tetap responsif.
- GUI mengambil event status dan sampel sekali per frame.
- Reset mengosongkan cycle, LED, dan waveform.
- GUI tidak boleh memakai wall-clock sebagai bagian dari model simulasi.

### 9.3 Sumber data

- AVR: pin state, register, UART, timer, interrupt.
- Board: LED, tombol, UART host.
- Analog: node voltage dan branch current dari solver.
- Setiap probe memiliki satuan dan tipe sinyal.
- Probe mengambil sampel setiap N cycle; nilai default N adalah 100.
- N dapat dikonfigurasi per probe.
- Decimation memakai min/max per pixel waktu.

---

## 10. Grafik

### 10.1 Model probe

Setiap probe memiliki:

- ID sinyal.
- Tipe: analog, digital, bus, power.
- Satuan.
- Ring buffer sampel 10 detik simulasi.
- Decimation min/max per pixel waktu.
- Warna dan label.

Grafik MVP mencakup sinyal digital dan analog. Bus dan power dapat
menyusul setelah alur dasar stabil.

### 10.2 Tampilan

- Satu grafik dapat memuat maksimal 8 series.
- Implementasi saat ini: lane PB5 dan LED dengan cursor cycle di floating window.
- Sumbu X memakai waktu simulasi.
- Warna series mengikuti warna net atau pin.
- Digital ditampilkan sebagai lane logika.
- Bus ditampilkan sebagai nilai heksadesimal pada transisi.
- Cursor vertikal menampilkan waktu dan nilai.
- Autoscale dan manual scale.
- Zoom, pan, dan legend toggle.
- Pause membekukan data tanpa menghentikan simulasi jika dipilih.
- Legend dapat disembunyikan.

### 10.3 Penempatan

- Grafik default muncul di analysis drawer.
- Grafik dapat dijadikan floating window.
- Setiap komponen dapat memiliki tab grafik sendiri.
- Grafik terakhir yang dibuka disimpan di file.
- Grafik tidak boleh memaksa canvas mengecil saat tertutup.

---

## 11. Arsitektur

```text
sim-gui (binary embed-emu-gui)
  +-- sim-board
  +-- sim-avr
  +-- sim-analysis
  +-- sim-device
  `-- sim-core
```

Aturan:

- `sim-gui` adalah adapter/composition.
- Runtime GUI tetap di `sim-gui`; ekstraksi ke crate bersama ditunda
  karena monitor CLI sinkron dan worker GUI memakai batch serta channel.
- Tidak ada crate simulasi yang bergantung pada `sim-gui`.
- Model dokumen tidak memakai tipe egui.
- Rendering egui hanya ada di layer UI.
- Fungsi maksimal 20 baris, guard clause, dan test untuk logika.

Modul usulan:

| Modul | Tanggung jawab |
|---|---|
| `app` | App state, panel, command dispatch |
| `canvas` | Kamera, culling, painting, hit testing |
| `grid` | Konversi koordinat, snap, zoom |
| `model` | Dokumen, komponen, kabel, net, junction |
| `library` | Definisi komponen dan kategori |
| `edit` | Tool, seleksi, undo/redo |
| `route` | Orthogonal routing dan collision |
| `probe` | Ring buffer dan decimation |
| `sim` | Sinkronisasi simulasi |
| `persist` | Format simpan dan muat |
| `config` | XDG config dan recent files |
| `theme` | Warna, LOD, aksesibilitas |

Dependensi eksternal `sim-gui`:

- `egui` dan `eframe` untuk UI.
- `serde` dan `serde_json` untuk format proyek.
- `rfd` untuk dialog file.
- `png` untuk export gambar.

Plugin API MVP berupa trait internal yang dikompilasi bersama GUI:

- Provider komponen untuk menambah definisi library.
- Provider analisis untuk menambah probe atau panel.
- Tidak ada dynamic loading, `libloading`, atau `unsafe`.

Data analog dibaca melalui API read-only:

- Node voltage.
- Branch current.
- Tidak ada mutasi state solver dari GUI.
- Sampling dilakukan oleh layer GUI setelah solver menghasilkan solusi.

---

## 12. Format Simpan

Format proyek memakai JSON melalui `serde_json`. Ekstensi file:
`.emu.json`. Output memakai pretty print 2 spasi.

Contoh skema:

```json
{
  "format": "embed-emu-gui",
  "version": 1,
  "view": {
    "camera": [0.0, 0.0, 1.0],
    "grid_visible": true
  },
  "components": [
    {
      "id": "U1",
      "type": "atmega328p",
      "position": [10, 10],
      "rotation": 0,
      "mirror": "none",
      "label": "U1"
    }
  ],
  "wires": [
    {
      "net": "LED_A",
      "class": "digital",
      "points": [[10, 12], [14, 12], [14, 16]]
    }
  ],
  "probes": []
}
```

Aturan:

- Versi skema eksplisit dan wajib.
- Field tidak dikenal ditolak pada versi 1.
- Skema lama dimigrasikan otomatis setelah backup.
- Array komponen dan wire diurutkan berdasarkan ID agar output stabil.
- Satu file JSON berisi seluruh proyek.
- Satu window menampilkan satu proyek; proyek lain dibuka di window baru.
- Simpan memakai file sementara lalu rename.
- Backup `<file>.bak` dibuat sebelum menimpa file yang sudah ada.
- Autosave memakai debounce 2 detik, hanya aktif setelah proyek punya
  path, dan di-flush saat membuka proyek lain atau keluar.
- Proyek baru yang belum pernah disimpan harus disimpan sebelum proyek
  lain dibuka.
- Undo/redo menyimpan 100 langkah.
- Dialog buka dan simpan memakai `rfd`.
- Recent files menyimpan 10 file terakhir dan tampil di menu toolbar.
- Konfigurasi aplikasi disimpan di XDG config, yaitu
  `~/.config/embed-emu/gui.json` atau `$XDG_CONFIG_HOME/embed-emu/gui.json`.
- Jika file berubah di luar aplikasi, GUI memantau metadata setiap
  1 detik dan menampilkan modal `Reload` atau `Save Mine`.
- Autosave dan flush ditahan selama konflik belum diselesaikan.
- Path absolut tidak disimpan di proyek.

---

## 13. Performa

Target awal:

- 500 komponen dan 2000 segmen kabel pada 60 FPS.
- Grid dan komponen di-cull berdasarkan viewport.
- Hit testing memakai grid bucket atau spatial hash.
- Grafik memakai decimation dan ring buffer 10 detik.
- Alokasi per frame diminimalkan.
- Repaint hanya saat input, animasi, atau simulasi berjalan.

Export:

- Screenshot canvas ke PNG melalui crate `png`.
- Ukuran gambar mengikuti resolusi viewport atau skala ekspor.
- Export tidak memakai wall-clock dan tidak mengubah state simulasi.

Optimasi lanjutan:

- Cache tessellation untuk simbol yang sering dipakai.
- Instanced rendering untuk grid.
- Background simulation thread.

---

## 14. Test

Test model tanpa GPU:

- Konversi world ke screen dan sebaliknya.
- Snap dan zoom.
- Rotasi dan mirror pin.
- Validasi tabrakan komponen.
- Routing ortogonal dan collision.
- Junction dan konektivitas net.
- Selection dan undo/redo.
- Save/load round-trip.
- Decimation grafik.

Test interaksi:

- Klik, drag, box select, keyboard shortcut.
- Place, move, rotate, wire, delete.
- Context menu.

Test integrasi:

- Tempatkan ATmega dan LED.
- Hubungkan pin.
- Jalankan simulasi headless.
- Assert nilai pin dan sampel grafik.

Test visual:

- Snapshot RGBA mentah dari renderer headless atau screenshot test.
- Golden image membandingkan byte RGBA, bukan file PNG.
- Perubahan yang disengaja harus memperbarui golden dengan review manual.

---

## 15. Risiko

| Risiko | Dampak | Mitigasi |
|---|---|---|
| Dependensi GUI bertambah | Supply chain dan build | Daftar terbatas, pin versi, audit |
| Skema JSON berubah | Proyek lama gagal dibuka | Versi skema dan migrasi |
| API egui berubah | Build rusak | Pin versi, layer UI tipis |
| Routing terlalu kompleks | UI macet | Manual orthogonal dulu |
| Simulasi tidak real-time | Ekspektasi pengguna | Tampilkan sim time dan budget |
| Grafik memori besar | OOM | Ring buffer dan decimation |
| Duplikasi session CLI | Perawatan sulit | Pertimbangkan runtime bersama |
| Input konflik | UX buruk | Tool mode eksplisit |
| Wayland/GPU | Render gagal | Uji backend eframe |
| Dialog rfd berbeda per OS | UX tidak konsisten | Fallback dialog internal |
| Aksesibilitas warna | Sulit dibaca | Palet ramah buta warna |

---

## 16. Milestone

| Milestone | Hasil |
|---|---|
| G0 | Keputusan dependensi, crate, CI, dan kerangka |
| G1 | Canvas, grid, pan, zoom, status bar |
| G2 | Library, place, move, rotate, mirror |
| G3 | Pin, label, selection, inspector |
| G4 | Wire orthogonal dan net |
| G5 | Junction, collision, warna net |
| G6 | Simulasi run/pause/step/reset |
| G7 | Probe dan grafik |
| G8 | Undo/redo, save/load |
| G9 | Performa, aksesibilitas, test |
| G10 | Packaging dan dokumentasi |

---

## 17. Keputusan

### 17.1 Keputusan utama

1. `sim-gui` memakai `egui`, `eframe`, `serde`, `serde_json`, `rfd`,
   dan `png`.
2. Grafik memakai widget internal, bukan `egui_plot`.
3. Satu edge grid hanya boleh satu kabel.
4. Persilangan tanpa junction dilarang.
5. GUI pertama hanya Linux.
6. Format proyek JSON dengan `serde_json`.
7. Session simulasi belum diekstrak ke crate bersama.
8. Plugin memakai trait internal.
9. Export PNG memakai crate `png`.
10. Golden image memakai snapshot RGBA.

### 17.2 Detail visual dan interaksi

1. Grid default 20 px; zoom preset dan halus.
2. Latar gelap dengan titik grid.
3. Simbol komponen vektor datar.
4. Pin berupa garis dengan titik.
5. Label pin berisi nama dan nomor.
6. Kabel memakai sudut 90 derajat tegas.
7. Animasi aliran sinyal opsional dan default mati.
8. Ruler dan probe tersedia.
9. Minimap opsional dan default tersembunyi.
10. Library awal board dan passive.
11. Grafik maksimal 8 series.
12. Grafik mendukung cursor, zoom, pan, autoscale, dan legend.
13. Probe mengambil sampel setiap N cycle.
14. Proyek berupa satu file JSON.
15. Konfigurasi di XDG config.
16. Autosave debounce 2 detik.
17. Recent files 10.
18. Aksen UI cyan.
19. Toolbar ikon dan teks.
20. Shortcut CAD-like.
21. Error berupa toast dan status bar.
22. Label komponen referensi dan nilai.
23. Label net muncul saat hover atau seleksi.
24. Rotasi mengelilingi pusat komponen.
25. Undo dikelompokkan per aksi.
26. UI berbahasa Inggris.
27. Target aksesibilitas WCAG AA.
28. Ekstensi `.emu.json`, pretty print 2 spasi.
29. Migrasi skema otomatis setelah backup.
30. Sampling default 100 cycle dan decimation min/max.
31. Inspector tiga section: properties, pins, simulation.
32. Plugin dua trait internal: komponen dan analisis.
33. Probe analog memakai API read-only.
34. Satu proyek per window dan prompt reload saat file berubah.
35. Library komponen berupa kode Rust internal.
36. Font UI monospace untuk seluruh antarmuka.
37. Minimap di kanan bawah, default tersembunyi.
38. Multi-select dipindahkan bersama.
39. Snap selalu aktif, Alt menonaktifkan sementara.
40. Overlap komponen ditolak.
41. Selection box memakai full containment.
42. Zoom berpusat di kursor.

### 17.3 Detail teknis yang belum dikunci

1. Palet warna final untuk setiap kelas net.
2. Field dan struktur lengkap skema JSON.
3. Signature method trait komponen dan analisis.
4. Bentuk konkret API probe analog.
5. Strategi update golden image saat tampilan berubah.
6. Jumlah maksimum recent files saat proyek sangat banyak.
7. Perilaku undo saat file eksternal berubah.

Detail ini dibahas saat implementasi G0 dan G7.

## 18. Menjalankan

Build dan jalankan GUI:

```bash
cargo run -p sim-gui
```

Build release:

```bash
cargo build --release -p sim-gui
./target/release/embed-emu-gui
```

Crate berada di `crates/sim-gui` dengan binary `embed-emu-gui`.

Backlog eksekusi ada di [`TODO-GUI.md`](../TODO-GUI.md).
