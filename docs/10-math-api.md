# 10 - Math API

Dokumentasi API matematika di `sim-core` untuk fondasi P1.

---

## 1. Tujuan

Modul ini menyediakan primitif numerik yang dipakai oleh netlist, device, dan analysis:

- Waktu simulasi.
- Satuan fisik.
- Identifier entity.
- Bilangan kompleks.
- Dense matrix dan LU solver.
- Tolerance helper.
- Error type.

Semua modul berada di crate `sim-core` dan tidak memakai dependensi eksternal.

---

## 2. Peta Modul

| File | Isi |
|---|---|
| `time.rs` | `Time` dalam pikodetik |
| `units.rs` | `Voltage`, `Current`, `Resistance`, `Capacitance` |
| `ids.rs` | `NodeId`, `BranchId`, `DeviceId` |
| `complex.rs` | `Complex64` untuk AC |
| `matrix.rs` | `DenseMatrix` dan LU solver |
| `tolerance.rs` | `approx_eq`, `max_abs`, `l2_norm` |
| `error.rs` | `SimError` |

Re-export tersedia dari `sim_core`:

```rust
use sim_core::{
    BranchId, Capacitance, Complex64, Current, DenseMatrix, DeviceId, NodeId,
    Resistance, SimError, Time, Voltage,
};
```

---

## 3. Time

`Time` menyimpan waktu sebagai `u64` pikodetik.

Contoh:

```rust
let a = Time::from_ns(10).expect("valid time");
let b = Time::from_us(2).expect("valid time");
let total = a.checked_add(b).expect("no overflow");
assert_eq!(total.as_ps(), 12_000);
```

Operasi:

- `Time::from_ps`, `from_ns`, `from_us`, `from_ms`, `from_secs`.
- `Time::from_secs_f64` untuk waktu pecahan.
- `as_ps`, `as_ns_f64`, `as_us_f64`, `as_ms_f64`, `as_secs_f64`.
- `checked_add`, `checked_sub`, `saturating_add`, `saturating_sub`.
- `Time::ZERO` untuk waktu nol; ground adalah `NodeId::GROUND`.

Aturan:

- Waktu digital selalu integer pikodetik.
- Overflow menghasilkan `None`, bukan panic.
- Nilai negatif atau non-finite pada `from_secs_f64` menghasilkan `None`.

---

## 4. Satuan

Newtype satuan:

```rust
let v = Voltage::new(3.3);
let i = Current::new(0.001);
let r = Resistance::new(1_000.0);
let c = Capacitance::new(1e-6);
assert_eq!(v.value(), 3.3);
```

Setiap satuan memiliki:

- `new(value)`.
- `value()`.
- `is_finite()`.

Tidak ada konversi otomatis antar-satuan. Konversi eksplisit dilakukan oleh pemanggil agar tidak ada kesalahan unit.

---

## 5. Identifier

```rust
let ground = NodeId::GROUND;
let node = NodeId::new(7);
let branch = BranchId::new(3);
let device = DeviceId::new(11);
assert_eq!(ground.raw(), 0);
assert_eq!(node.raw(), 7);
```

Aturan:

- `NodeId::GROUND` selalu 0.
- ID tidak boleh dipakai untuk entity yang berbeda.
- ID tidak memiliki arti di luar netlist tempat ia dibuat.

---

## 6. Complex64

`Complex64` memiliki field `re` dan `im`.

```rust
let a = Complex64::new(1.0, 2.0);
let b = Complex64::new(3.0, 4.0);
let sum = a + b;
let product = a * b;
let magnitude = a.magnitude();
let phase = a.phase();
```

Operasi:

- `Add`, `Sub`, `Mul`, `Div`, `Neg`.
- `magnitude`, `phase`, `conjugate`, `is_finite`.
- `from_real`.

Pembagian dengan nol menghasilkan `NaN` pada kedua komponen, bukan panic.

---

## 7. DenseMatrix

Matrix row-major dengan `f64`.

### 7.1 Konstruksi

```rust
let zeros = DenseMatrix::zeros(2, 2).expect("matrix");
let identity = DenseMatrix::identity(3).expect("matrix");
let matrix = DenseMatrix::from_vec(2, 2, vec![4.0, 3.0, 6.0, 3.0])
    .expect("matrix");
```

### 7.2 Akses

```rust
let mut matrix = DenseMatrix::zeros(2, 2).expect("matrix");
matrix.set(0, 0, 1.0).expect("set");
assert_eq!(matrix.get(0, 0), Some(1.0));
assert_eq!(matrix.get(2, 0), None);
let row = matrix.row(0).expect("row");
let column = matrix.column(0).expect("column");
```

Aturan:

- `get` dan `row`/`column` mengembalikan `None` jika di luar batas.
- `set` mengembalikan `SimError::IndexOutOfBounds`.
- `from_vec` memvalidasi panjang data.

### 7.3 Solve

```rust
let matrix = DenseMatrix::from_vec(2, 2, vec![4.0, 3.0, 6.0, 3.0])
    .expect("matrix");
let solution = matrix.solve(&[10.0, 12.0]).expect("solution");
assert!((solution[0] - 1.0).abs() < 1e-9);
assert!((solution[1] - 2.0).abs() < 1e-9);
```

Algoritma:

1. Validasi matrix persegi dan panjang RHS.
2. LU decomposition dengan partial pivoting.
3. Terapkan pivot ke RHS.
4. Forward substitution.
5. Back substitution.

Error yang mungkin:

- `NotSquare`.
- `DimensionMismatch`.
- `SingularMatrix`.
- `IndexOutOfBounds`.
- `Internal` untuk overflow ukuran.

### 7.4 Batasan

- Dense matrix, kompleksitas O(n^3).
- Cocok untuk rangkaian kecil.
- Sparse solver akan ditambahkan pada tahap berikutnya.
- Deteksi singular memakai epsilon `1e-18`.
- Tidak ada scaling pivot.
- Tidak ada iterative refinement.

---

## 8. Tolerance

```rust
use sim_core::tolerance::{approx_eq, l2_norm, max_abs};

assert!(approx_eq(1.0, 1.0 + 1e-12, 1e-9, 1e-12));
assert_eq!(max_abs(&[1.0, -5.0, 3.0]), 5.0);
assert_eq!(l2_norm(&[3.0, 4.0]), 5.0);
```

Aturan:

- Nilai non-finite tidak dianggap close.
- `max_abs` mengabaikan tanda.
- `l2_norm` mengembalikan 0 untuk slice kosong.

---

## 9. SimError

Varian yang tersedia:

- `InvalidNetlist`.
- `FloatingNode`.
- `SingularMatrix`.
- `ConvergenceFailure`.
- `UnsupportedDevice`.
- `DimensionMismatch`.
- `NotSquare`.
- `IndexOutOfBounds`.
- `Io`.
- `Parse`.
- `Internal`.

Semua varian mengimplementasikan `Display` dan `std::error::Error`.

---

## 10. Prinsip yang Diterapkan

- Guard clause di awal fungsi.
- Setiap fungsi maksimal 20 baris.
- Fungsi kecil dan reusable.
- Tidak ada duplikasi.
- Tidak ada dependensi eksternal.
- Tidak ada `unsafe`.
- ASCII-only.
- Error dikembalikan sebagai `Result`, bukan panic.
- Test deterministik.

---

## 11. Test

Test unit berada di masing-masing modul.

Cakupan:

- Konversi waktu dan overflow.
- Satuan dan finite check.
- ID dan ground.
- Operasi complex, magnitude, phase, dan pembagian nol.
- Matrix accessor dan batas.
- LU 2x2, 3x3, pivot swap, singular, dimensi salah, non-square, matrix kosong.
- Solver berulang sebagai smoke test.
- Tolerance helper.

Jumlah test P1: 33.

---

## 12. Berikutnya

P2 akan menambahkan:

- Netlist builder.
- Node dan device.
- Subcircuit.
- Flatten.
- Validasi.
- Macro deklaratif.

Lihat `docs/02-architecture.md` dan `TODO.md`.
