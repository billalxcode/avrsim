# macOS Driver

Rencana komponen driver USB virtual untuk macOS.

**Status:** rencana. Belum ada kode.

Jalur yang dievaluasi:

- PTY dan custom Arduino discovery plugin.
- DriverKit atau USBSerialDriverKit untuk USB virtual native.
- Client USB/IP pihak ketiga.

Catatan:

- Tidak ada client USB/IP bawaan di macOS.
- DriverKit membutuhkan signing dan entitlement.
- PTY adalah fallback yang paling realistis untuk tahap awal.

Todolist ada di [`TODO.md`](../../TODO.md), bagian P19.6.
