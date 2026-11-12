# Windows Driver

Rencana komponen driver USB virtual untuk Windows.

**Status:** rencana. Belum ada kode.

Jalur yang dievaluasi:

- Server USB/IP C++ yang kompatibel dengan `usbip-win2`.
- Client `usbip-win2` membuat USB device virtual.
- `usbser.sys` membuat COM port dari CDC ACM.

Belum diputuskan:

- Apakah server USB/IP ditulis di C++ atau Rust.
- Cara instalasi driver dan signing.
- Cara packaging dan auto-start.

Todolist ada di [`TODO.md`](../../TODO.md), bagian P19.5.
