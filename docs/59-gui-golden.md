# 59 - Golden RGBA GUI

Dokumen ini menjelaskan golden image RGBA dan prosedur update untuk
`sim-gui`.

## Status

Golden RGBA sudah tersedia dan berjalan headless di test Linux:

- Renderer: `crates/sim-gui/src/golden.rs`, hanya dikompilasi untuk test.
- Ukuran: 320 x 240 piksel, skala piksel 1.0.
- Scene: resistor, LED, label, dan teks tetap pada tema gelap.
- Golden: `crates/sim-gui/tests/golden/scene-320x240.rgba`.
- Perbandingan: byte RGBA exact, tanpa toleransi.
- Diagnostik: piksel, channel, dan nilai byte pertama yang berbeda.
- Determinisme: test merender dua kali dan memastikan hasil identik.

Renderer memakai `egui::Context::tessellate` untuk menghasilkan triangle
mesh, lalu merasterisasi triangle dan font atlas ke buffer RGBA perangkat
lunak. Tidak ada GPU, jaringan, wall-clock, atau dependensi baru.

## Menjalankan Test

```bash
cargo test -p sim-gui golden
```

Test yang berjalan:

- `golden_render_is_deterministic`
- `golden_scene_matches_snapshot`

## Prosedur Update

Jika perubahan visual disengaja:

1. Jalankan renderer untuk menulis golden baru:

   ```bash
   UPDATE_GOLDEN=1 cargo test -p sim-gui golden_scene_matches_snapshot
   ```

2. Periksa file `crates/sim-gui/tests/golden/scene-320x240.rgba`.
3. Konversi atau buka file sebagai RGBA 320 x 240 untuk review visual.
4. Pastikan hanya perubahan yang dimaksud yang terlihat.
5. Commit golden dan perubahan renderer dalam satu perubahan terpisah.
6. Jalankan `cargo test -p sim-gui golden` tanpa env update.

## Batasan

- Golden hanya divalidasi di Linux.
- Perubahan versi `egui`, font default, atau tessellator dapat mengubah
  output dan memerlukan update golden.
- Renderer test memakai rasterizer internal, bukan backend GPU `eframe`.
  Golden menjaga regresi scene dan teks, bukan kesamaan piksel dengan GPU.
- Scene golden harus tetap deterministik: tanpa animasi, pointer, atau
  waktu.

## Kriteria Lulus

- Test deterministik.
- Tidak ada akses jaringan atau wall-clock.
- Golden dapat diregenerasi dari commit yang sama.
- Perbedaan byte dilaporkan dengan jumlah byte yang berbeda.
- Ukuran file harus sama dengan 320 x 240 x 4 byte.

## Referensi

- `docs/57-gui-egui.md`: desain renderer dan test visual.
- `crates/sim-gui/src/golden.rs`: renderer dan test golden.
- `docs/07-verification.md`: kebijakan verifikasi umum.
