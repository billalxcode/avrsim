# 57 - GUI egui untuk Pengembangan dan Pengujian

Dokumen ini merancang GUI interaktif berbasis egui untuk eksplorasi,
penempatan komponen, probing, dan pengujian simulasi.

**Status:** desain dan backlog. Dependensi `egui`/`eframe` disetujui hanya
untuk crate `sim-gui`. Belum ada implementasi.

**Rekomendasi:** buat crate baru `sim-gui` sebagai adapter/composition,
mirip `sim-cli`. Semua crate simulasi tetap zero external dependency.
`egui_plot` dan crate eksternal lain tidak termasuk pengecualian ini.

---

## 1. Keputusan yang Perlu Disetujui

### 1.1 Konflik dependensi

Aturan repository sebelumnya:

> Zero external dependency. Hanya Rust standard library dan crate
> internal workspace.

`egui` dan `eframe` adalah dependensi eksternal.

**Keputusan:** `egui` dan `eframe` diizinkan hanya di crate `sim-gui`.
Semua crate lain tetap zero external dependency. `egui_plot`, `serde`,
`rfd`, dan crate eksternal lain tidak diizinkan tanpa keputusan baru.

### 1.2 Backend grafik

**Keputusan:** gunakan widget plot internal berbasis `egui::Painter`.
Tidak memakai `egui_plot` pada MVP. Jika kebutuhan grafik terbukti
melebihi widget internal, penambahan `egui_plot` harus disetujui
terpisah.

### 1.3 Batas GUI pertama

Rekomendasi rilis GUI pertama:

- Linux desktop.
- Editor skematik dasar: pilih, tempatkan, pindahkan, rotasi, mirror,
  hubungkan dengan kabel ortogonal, simpan, dan muat.
- Simulasi AVR dan board.
- Probe pin digital dan grafik sederhana.
- Belum ada autorouting bebas, analog penuh, atau kolaborasi.

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
- Analysis drawer di bawah canvas, default tertutup, tinggi maksimum
  30 persen saat terbuka.
- Grafik dapat menjadi tab di analysis drawer atau floating window.
- Layout terakhir disimpan di file konfigurasi.

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
- Zoom level diskrit: 25, 50, 75, 100, 150, 200, 400 persen.
- Fit selection, fit all, dan center origin.
- Level of detail: label kecil dan grid minor disembunyikan saat zoom
  rendah.

### 5.4 Rendering grid

- Grid minor setiap sel.
- Grid major setiap 5 atau 10 sel.
- Sumbu X dan Y berbeda warna.
- Culling hanya menggambar garis yang terlihat.
- Grid dapat dimatikan sementara untuk melihat rangkaian.

---

## 6. Komponen dan Library

### 6.1 Definisi komponen

Setiap definisi memiliki:

- ID stabil.
- Nama dan kategori.
- Simbol vektor.
- Daftar pin: nama, nomor, arah, tipe listrik, offset grid.
- Parameter default.
- Probe yang tersedia.
- Batas ukuran dan anchor.

### 6.2 Instance komponen

Setiap instance memiliki:

- ID unik.
- Referensi definition.
- Posisi grid.
- Rotasi dan mirror.
- Label referensi, nilai, dan parameter.
- Status lock dan visibility.
- Probe yang aktif.

### 6.3 Library sheet

- Kategori: MCU, board, passive, diode, transistor, digital, source,
  probe, ground, dan power.
- Pencarian nama dan kategori.
- Preview simbol.
- Drag-and-drop ke canvas atau klik lalu tempatkan.
- Recently used.
- Filter berdasarkan tipe pin atau domain sinyal.

### 6.4 Penempatan

- Ghost komponen mengikuti kursor.
- Snap ke grid.
- Hijau jika valid, merah jika bertabrakan.
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
- Panjang minimum satu sel.
- Kabel tidak boleh melewati body komponen.
- Satu edge grid hanya boleh dipakai satu kabel pada MVP.
- Persilangan hanya boleh di junction eksplisit.
- Jika edge sudah terpakai, kabel harus berbelok atau mencari jalur lain.

### 7.2 Mode menggambar

- Manual: klik pin, klik titik grid, klik titik grid berikutnya,
  double-click atau Enter untuk selesai, Esc untuk batal.
- L-route: pilih dua titik, sistem menawarkan jalur L.
- Z-route: pilih dua titik, sistem menawarkan jalur Z.
- Auto-route sederhana: cari jalur Manhattan terpendek yang tidak
  bertabrakan, dengan batas eksplorasi.
- Pengguna selalu dapat memilih vertex dan menggesernya selama tetap
  di grid.

### 7.3 Junction

- Junction adalah objek eksplisit di titik grid.
- Tiga atau empat segmen pada satu titik wajib memiliki junction.
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
- Legenda warna tersedia di panel bantu.
- Warna net dapat dioverride per net, tetapi default tetap kelas.
- Kabel terpilih memiliki halo, bukan mengganti warna kelas.

### 7.5 Label net

- Label dapat ditempel di pin, midpoint, atau vertex.
- Label mengikuti rotasi kabel.
- Label dapat disembunyikan per net atau secara global.
- Nama net otomatis dibuat jika belum ada, misalnya `N$001`.

---

## 8. Interaksi

### 8.1 Tool mode

- Select: klik, shift-click, box select.
- Place: menempatkan komponen dari library.
- Wire: menggambar kabel ortogonal.
- Probe: memilih pin atau net untuk grafik.
- Pan: drag kanvas.
- Measure: opsional, untuk mengukur jarak grid.

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
| Ctrl+Z | Undo |
| Ctrl+Shift+Z | Redo |
| Ctrl+S | Save |
| Ctrl+O | Open |
| Esc | Batal tool |
| Enter | Selesai wire |

### 8.3 Seleksi

- Klik memilih satu objek.
- Shift-click menambah atau mengurangi seleksi.
- Drag kosong membuat selection box.
- Selection box hanya memilih objek yang seluruhnya berada di dalam.
- Multi-select dapat dipindahkan bersama.
- Alignment guide dan distribusi horizontal/vertikal.

### 8.4 Context menu

- Komponen: properties, rotate, mirror, duplicate, delete, add graph.
- Pin: show graph, add probe, disconnect, highlight net.
- Kabel: select net, change color, add label, delete.
- Kanvas: paste, fit all, toggle grid, add component.

---

## 9. Simulasi

### 9.1 Kontrol

- Run, pause, step one instruction, step one cycle, reset.
- Simulation time dan cycle count terlihat di toolbar.
- Speed target dalam cycle per frame, bukan klaim real-time.
- Status menunjukkan apakah simulasi tertinggal dari target.

### 9.2 Sinkronisasi

- Untuk MVP, simulation berjalan di UI thread dengan budget waktu per
  frame.
- Jika budget habis, frame berikutnya melanjutkan.
- Background thread dan channel adalah optimasi lanjutan.
- GUI tidak boleh memakai wall-clock sebagai bagian dari model simulasi.

### 9.3 Sumber data

- AVR: pin state, register, UART, timer, interrupt.
- Board: LED, tombol, UART host.
- Analog: node voltage dan branch current dari solver.
- Setiap probe memiliki satuan dan tipe sinyal.

---

## 10. Grafik

### 10.1 Model probe

Setiap probe memiliki:

- ID sinyal.
- Tipe: analog, digital, bus, power.
- Satuan.
- Ring buffer sampel.
- Decimation policy.
- Warna dan label.

### 10.2 Tampilan

- Satu grafik dapat memuat beberapa series.
- Digital ditampilkan sebagai lane logika.
- Bus ditampilkan sebagai nilai heksadesimal pada transisi.
- Cursor vertikal menampilkan waktu dan nilai.
- Autoscale dan manual scale.
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
| `theme` | Warna, LOD, aksesibilitas |

---

## 12. Format Simpan

Format awal berupa teks ASCII dengan versi eksplisit.

```text
format=embed-emu-gui
version=1

[view]
camera=0,0,1.0
grid=visible

[component]
id=U1
type=atmega328p
pos=10,10
rot=0
mirror=none
label=U1

[wire]
net=LED_A
class=digital
points=10,12 14,12 14,16
```

Aturan:

- Urutan baris deterministik.
- File mudah di-diff.
- Parser memvalidasi versi dan nilai.
- Unknown key ditolak pada versi 1.
- Save memakai file sementara lalu rename.
- Autosave dan backup terpisah dari file utama.

---

## 13. Performa

Target awal:

- 60 FPS pada 500 komponen dan 2000 segmen kabel.
- Grid dan komponen di-cull berdasarkan viewport.
- Hit testing memakai grid bucket atau spatial hash.
- Grafik memakai decimation.
- Alokasi per frame diminimalkan.
- Repaint hanya saat input, animasi, atau simulasi berjalan.

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

- Snapshot geometry atau layout.
- Golden image hanya jika renderer stabil.

---

## 15. Risiko

| Risiko | Dampak | Mitigasi |
|---|---|---|
| Konflik zero dependency | Blocker aturan | Keputusan scoped exception |
| API egui berubah | Build rusak | Pin versi, layer UI tipis |
| Routing terlalu kompleks | UI macet | Manual orthogonal dulu |
| Simulasi tidak real-time | Ekspektasi pengguna | Tampilkan sim time dan budget |
| Grafik memori besar | OOM | Ring buffer dan decimation |
| Duplikasi session CLI | Perawatan sulit | Pertimbangkan runtime bersama |
| Input konflik | UX buruk | Tool mode eksplisit |
| Wayland/GPU | Render gagal | Uji backend eframe |
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

## 17. Keputusan Terbuka

Sudah diputuskan:

1. `sim-gui` boleh memakai `eframe`/`egui`.
2. Grafik memakai widget internal, bukan `egui_plot`.

Belum diputuskan:

3. Apakah satu edge grid hanya boleh satu kabel? Default usulan: ya.
4. Apakah persilangan tanpa junction dilarang total? Default usulan: ya.
5. Apakah GUI pertama hanya Linux? Default usulan: ya.
6. Apakah format simpan memakai teks kustom? Default usulan: ya.
7. Apakah session simulasi diekstrak ke crate bersama? Default usulan:
   tunda sampai ada duplikasi nyata dengan `sim-cli`.

Backlog eksekusi ada di [`TODO-GUI.md`](../TODO-GUI.md).
