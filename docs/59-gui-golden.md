# 59 - Prosedur Golden RGBA GUI

Dokumen ini menjelaskan status dan prosedur update golden image RGBA untuk
`sim-gui`.

## Status

Golden image RGBA belum tersedia. Test visual saat ini memakai
`egui::Context` untuk merender scene dan memeriksa jumlah shape, bukan
piksel. Alasannya:

- Renderer final bergantung pada backend `eframe` dan GPU.
- Ukuran viewport, DPI, tema, dan font dapat berbeda antar mesin.
- Belum ada jalur render headless yang stabil di CI Linux.

Karena itu G9-09 tetap terbuka. Jangan menambahkan file golden tanpa
renderer headless yang deterministik.

## Prasyarat Renderer

Renderer golden harus memenuhi syarat berikut:

- Ukuran viewport tetap, misalnya 640 x 480.
- Skala piksel tetap 1.0.
- Tema dan palet warna dikunci.
- Font monospace yang sama tersedia di CI.
- Tidak memakai waktu, animasi, atau input pointer.
- Output RGBA mentah dengan stride yang jelas.
- Tidak menulis file di dalam core simulation.

## Prosedur Update

Jika renderer headless sudah tersedia:

1. Render scene referensi ke `crates/sim-gui/tests/golden/<name>.rgba`.
2. Simpan metadata di `<name>.json`: lebar, tinggi, dan hash sumber.
3. Jalankan test dengan `UPDATE_GOLDEN=1` untuk menulis ulang golden.
4. Jalankan test tanpa env tersebut untuk membandingkan piksel.
5. Gunakan toleransi kecil, misalnya maksimal 2 channel berbeda per
   piksel, untuk anti-aliasing.
6. Review diff gambar secara manual sebelum commit.
7. Commit golden dan metadata dalam perubahan terpisah dari perubahan
   renderer.

## Kriteria Lulus

- Test deterministik di Linux.
- Tidak ada akses jaringan atau wall-clock.
- Golden dapat diregenerasi dari commit yang sama.
- Perbedaan piksel dilaporkan dengan koordinat dan nilai channel.
- Test gagal jika ukuran atau hash metadata tidak cocok.

## Referensi

- `docs/57-gui-egui.md`: desain renderer dan test visual.
- `crates/sim-gui/src/scene.rs`: test render berbasis shape.
- `docs/07-verification.md`: kebijakan verifikasi umum.
