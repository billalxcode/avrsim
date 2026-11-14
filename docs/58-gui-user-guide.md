# 58 - Panduan Pengguna GUI

Dokumen ini menjelaskan cara memakai GUI EmuCAD (`emucad-gui`) untuk menggambar
skematik, menjalankan firmware AVR, memantau UART, dan mengekspor gambar.

**Status:** GUI Linux sudah dapat dipakai untuk editing dasar, simulasi
PB5, monitor UART TX, graph digital, graph analog PB5/LED, probe read-only
default 100 cycle, dan persistence proyek. Probe net generik menunggu
integrasi netlist; input UART RX sudah ada.

---

## 1. Build dan Jalankan

Prasyarat:

- Rust toolchain sesuai `rust-toolchain.toml`.
- Linux desktop dengan Wayland atau X11.
- Dependensi sistem untuk `eframe` dan `rfd`.

Perintah:

```bash
cargo build -p sim-gui
cargo run -p sim-gui
```

Binary hasil build:

```text
target/debug/emucad-gui
```

Release build:

```bash
cargo build --workspace --release
```

---

## 2. Tata Letak

- Toolbar atas: file, undo/redo, simulasi, tool, sheet, grid, minimap,
  animasi aliran, dan zoom.
- Kanvas tengah: grid, komponen, kabel, dan minimap opsional.
- Status bar bawah: koordinat, zoom, grid, snap, tool, seleksi, parts,
  cycle, PB5 high/low dan tegangan mV, LED, dan FPS.
- Library, Inspector, Legend, Digital graph, Analog graph, UART monitor,
  Log, dan Command palette tampil sebagai floating window.
- Toast singkat muncul di bawah tengah untuk status terbaru.

Centang `Flow` di toolbar untuk animasi aliran sinyal. Animasi default
mati, hanya memengaruhi tampilan, dan tidak mengubah data proyek.

---

## 3. Mouse dan Keyboard

| Input | Aksi |
|---|---|
| Klik kiri | Pilih komponen atau kabel |
| Shift atau Ctrl + klik | Tambah atau lepas item dari seleksi |
| Drag area kosong | Selection box |
| Drag komponen terpilih | Pindahkan seluruh grup |
| Klik kanan | Context menu |
| Space + drag | Pan kanvas |
| Wheel | Zoom di posisi kursor |
| R | Rotate |
| M | Mirror |
| Arrow | Geser seleksi 1 grid |
| Shift+Arrow | Geser seleksi 5 grid |
| Delete | Hapus seleksi, dengan konfirmasi jika perlu |
| Ctrl+C | Copy |
| Ctrl+V | Paste |
| Ctrl+D | Duplicate |
| Ctrl+Z | Undo |
| Ctrl+Shift+Z atau Ctrl+Y | Redo |
| Ctrl+S | Save |
| Ctrl+O | Open |
| Ctrl+K | Command palette |
| Escape | Batal tool atau tutup menu |
| Enter | Selesai menggambar wire |

---

## 4. Library dan Penempatan

1. Klik `Library`.
2. Cari komponen atau pilih kategori.
3. Klik komponen untuk masuk mode placement.
4. Atau drag komponen dari library ke kanvas lalu lepas.
5. Gerakkan pointer di kanvas.
6. Klik untuk menempatkan.
7. Tekan `R` atau `M` sebelum menempatkan untuk transform.
8. Preview simbol muncul di bawah daftar library saat item di-hover.
9. Escape membatalkan placement.

Komponen tidak boleh saling menimpa. Selalu snap ke grid.

---

## 5. Menggambar Kabel

1. Pilih tool `Wire`.
2. Pilih `Class` di toolbar.
3. Klik pin atau titik koneksi.
4. Klik titik grid berikutnya.
5. Gunakan `L-H`, `L-V`, `Z-H`, atau `Z-V` untuk saran jalur.
6. Klik pin kedua; wire selesai otomatis.
7. Klik `Finish` atau tekan Enter untuk menyelesaikan jalur manual.
8. Escape atau `Cancel` membatalkan.

Kelas net juga dapat diubah setelah wire dibuat:

- Pilih wire.
- Buka Inspector, lalu ganti `Class`.
- Atau klik kanan wire, lalu pilih `Net class`.
- Context menu juga menyediakan preset warna dan editor label.
- Isi `Label` di Inspector untuk nama net kustom, misalnya `LED_A`.
- Warna dan pola garis mengikuti kelas baru.
- Undo mengembalikan kelas sebelumnya.

Aturan:

- Semua segmen ortogonal.
- Satu edge grid hanya untuk satu kabel.
- Kabel tidak boleh menembus body komponen.
- Persilangan tanpa junction ditolak.
- Ujung kabel harus pin, junction, atau vertex kabel.

Warna dan pola garis mengikuti kelas net. Legenda menampilkan warna dan
pola.

---

## 6. Seleksi dan Editing

- Selection box memilih objek yang seluruhnya berada di dalam kotak.
- Inspector menampilkan properti satu objek atau ringkasan multi-select.
- Rotate dan mirror berlaku untuk seluruh komponen terpilih.
- Copy dan paste mempertahankan posisi relatif, nilai, rotasi, dan mirror.
- Label komponen hasil paste dibuat baru.
- Wire internal ikut disalin jika pin tujuan valid.
- Align dan distribute tersedia melalui command palette.
- Undo menyimpan hingga 100 langkah.

Paste memakai posisi pointer. Jika pointer tepat di origin clipboard,
offset menjadi `+2,+2`. Duplicate selalu memakai offset `+2,+2`.

---

## 7. Simulasi

Kontrol:

- `Load HEX`: memuat firmware Intel HEX.
- `Run` atau `Pause`.
- `Step`: satu instruksi.
- `Reset`: cycle, waveform, dan UART dikosongkan.
- `Wave`: membuka graph digital.
- `Analog`: membuka graph analog PB5.
- `UART`: membuka monitor TX.
- `N 100`: mengatur sampling probe setiap N cycle.
- Klik kanan lalu `Probe PB5` atau `Probe LED`: menambah probe.

Graph digital menampilkan lane PB5 dan LED pada sumbu cycle. Arahkan
pointer ke graph untuk melihat cursor dan nilai pada cycle tersebut.

Graph analog menampilkan tegangan PB5 dalam volt dan arus LED dalam
ampere. Tegangan memakai sumbu kiri, arus memakai sumbu kanan. Kontrol
`Auto` mengaktifkan autoscale, `Legend` menampilkan daftar series, `+`
dan `-` mengubah zoom, drag menggeser window, dan wheel di atas plot juga
mengubah zoom. Cursor menampilkan cycle dan nilai terdekat.

Probe memakai API read-only dan buffer 10 detik. Default sampling adalah
setiap 100 cycle. Combo `N` mengatur laju sampling global worker; model
probe menyimpan `Sampling` per probe dan menyaring sample sesuai nilai
tersebut. Maksimal 8 series per plot. Warna series mengikuti warna pin
atau net.

Monitor UART menampilkan byte TX sebagai UTF-8 lossy, dengan buffer 16 KiB
dan auto-scroll. Isi kolom input dan klik `Send` atau tekan Enter untuk
mengirim byte RX ke firmware.

Simulasi berjalan di worker thread. Batch 100 instruksi memberi
kesempatan GUI memproses input dan event.

---

## 8. Command Palette

Tekan `Ctrl+K`, lalu ketik sebagian nama perintah.

Contoh:

- `save`, `open`, `load hex`
- `run`, `pause`, `step`, `reset`
- `fit`, `zoom`, `grid`, `minimap`
- `copy`, `paste`, `duplicate`
- `align left`, `distribute horizontally`
- `uart`, `log`, `legend`, `wave`

Gunakan panah atas dan bawah, lalu Enter. Escape menutup palette.

---

## 9. Proyek dan Konfigurasi

Format proyek:

```text
.emu.json
```

Perilaku:

- JSON pretty print 2 spasi.
- Versi skema saat ini 2.
- File versi 1 dimigrasikan otomatis ke versi 2.
- Warna override wire disimpan sebagai RGB di field `color`.
- Label net kustom disimpan di field `label`.
- Save atomik melalui file sementara.
- Backup `<file>.bak` sebelum menimpa.
- Autosave setelah 2 detik tanpa edit.
- Autosave dan save manual ditahan jika file berubah di luar aplikasi.
- Konflik file menampilkan pilihan `Reload` atau `Save Mine`.
- Recent files disimpan maksimal 10.

Konfigurasi:

```text
$XDG_CONFIG_HOME/emucad/gui.json
```

Fallback:

```text
~/.config/emucad/gui.json
```

---

## 10. Export PNG

1. Siapkan tampilan kanvas.
2. Klik `PNG`.
3. Pilih lokasi file.
4. Screenshot viewport dipotong ke area kanvas dan disimpan sebagai RGBA PNG.

Overlay yang sedang terbuka dapat ikut tertangkap jika menutupi kanvas.
Tutup window yang tidak diinginkan sebelum export.

---

## 11. Troubleshooting

### Window tidak terbuka

Jalankan dari terminal untuk melihat pesan:

```bash
cargo run -p sim-gui
```

Pastikan backend grafis dan driver GPU tersedia.

### Firmware tidak berjalan

- Pastikan file Intel HEX valid.
- Cek pesan error di toolbar, toast, atau Log.
- Coba `Reset` setelah load.
- Pastikan firmware tidak berhenti pada instruksi yang tidak didukung.

### UART tidak menampilkan data

- Pastikan firmware mengaktifkan transmitter UART.
- Pastikan baud dan register UART diinisialisasi.
- Cek monitor `UART` sudah dicentang.
- Lihat `Log` untuk error simulasi.

### File proyek berubah di luar aplikasi

Modal konflik muncul. Pilih:

- `Reload`: muat versi disk, edit lokal dibuang.
- `Save Mine`: timpa file disk dengan versi GUI.

### Kinerja rendah

- Tutup graph atau minimap.
- Kurangi jumlah komponen dan kabel yang terlihat.
- Gunakan release build.

---

## 12. Batas Saat Ini

- Probe net generik menunggu integrasi netlist; probe PB5 dan arus LED
  sudah tersedia.
- Grafik analog saat ini memakai sampel PB5 dan arus LED.
- `Alt` belum menonaktifkan snap karena model dokumen memakai grid
  integer; semua posisi tetap snap ke grid.
- Clipboard hanya in-memory.
- Wire tidak ikut berubah saat grup komponen ditransformasi.
- Build Windows dan macOS belum diverifikasi.
- Golden image RGBA headless tersedia di Linux dengan renderer perangkat
  lunak; prosedur update ada di `docs/59`.

---

## 13. Referensi

- `docs/57-gui-egui.md`: desain dan keputusan GUI.
- `docs/43-user-guide.md`: panduan CLI dan simulasi.
- `TODO-GUI.md`: backlog G0 sampai G10.
