# 50 - ELF Loader

Dokumen ini menjelaskan loader ELF32 dan pembacaan simbol.

---

## 1. Status

P9-09 dan P9-10 diimplementasikan:

- Parsing header ELF32 little-endian.
- Program header `PT_LOAD`.
- Segment flash dan metadata.
- Symbol table dan string table.
- Konversi segment flash menjadi program words.

Belum ada relocation, debug info, atau ELF64.

---

## 2. API

```rust
pub struct ElfImage {
    // fields privat
}

impl ElfImage {
    pub fn parse(data: &[u8]) -> Result<Self, SimError>;
    pub fn entry(&self) -> u32;
    pub fn segments(&self) -> &[ElfSegment];
    pub fn symbols(&self) -> &[ElfSymbol];
    pub fn load_flash(&self, image: &mut [u8]) -> Result<(), SimError>;
    pub fn program_words(&self) -> Result<Vec<u16>, SimError>;
    pub fn flash_bytes(&self) -> Result<Vec<u8>, SimError>;
    pub fn to_hex(&self) -> Result<String, SimError>;
}
```

`ElfSegment` menyimpan alamat, byte file, dan ukuran memori.

`ElfSymbol` menyimpan nama, alamat, ukuran, dan jenis:

- `Function`.
- `Object`.
- `Other`.

---

## 3. Validasi

Header harus:

- Diawali `0x7F 45 4C 46`.
- ELF class 32-bit.
- Little endian.
- Machine AVR (`83`).

Offset tabel dan segment harus berada di dalam file.

---

## 4. Flash dan Data

Segment dengan alamat di bawah `0x00800000` dianggap flash.

Segment dengan alamat `0x00800000` atau lebih diabaikan oleh
`load_flash` dan `program_words`.

`program_words` menghasilkan `u16` little-endian dan menambahkan padding
nol jika panjang ganjil.

---

## 5. Simbol

Loader mencari section `SHT_SYMTAB`, lalu memakai `sh_link` untuk menemukan
string table.

Simbol dengan section index nol atau nama kosong dilewati.

---

## 6. Test

Fixture ELF sintetis berisi:

- Satu `PT_LOAD` dengan byte `01 02 03 04`.
- Satu simbol `main` bertipe function.
- String table dan section header yang valid.

Test memverifikasi:

- Entry point.
- Segment dan isi flash.
- Nama dan jenis simbol.
- Program words little-endian.
- Konversi segment flash ke Intel HEX.
- Penolakan magic salah, file terpotong, dan machine bukan AVR.

Total test `sim-board`: 23.
Total workspace: 653.

---

## 7. Batas

- Tidak ada relocation.
- Tidak ada debug section.
- Tidak ada ELF64.
- Tidak ada validasi `e_type`.
- Tidak ada deduplikasi simbol.
- Alamat flash di atas `0x00800000` tidak didukung.
