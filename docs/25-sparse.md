# 25 - Sparse Matrix

Dokumen ini menjelaskan modul sparse matrix di `sim-core`.

---

## 1. SparseMatrix

- Format CSR: row pointer, column index, values.
- Dibangun dari triplet.
- Duplikat dijumlahkan.
- Nilai nol tidak disimpan.
- Get, nonzeros, multiply vector, dan konversi dense.

API:

```rust
pub struct SparseMatrix { /* ... */ }

impl SparseMatrix {
    pub fn from_triplets(rows, cols, triplets) -> Result<Self, SimError>;
    pub fn get(&self, row: usize, col: usize) -> Option<f64>;
    pub fn mul_vec(&self, vector: &[f64]) -> Result<Vec<f64>, SimError>;
    pub fn to_dense(&self) -> Result<DenseMatrix, SimError>;
}
```

---

## 2. Sparse LU

`solve_sparse` memakai eliminasi Gaussian sparse dengan partial pivoting.

- Baris disimpan sebagai `BTreeMap` agar deterministik.
- Pivot dipilih dari nilai absolut terbesar.
- Fill-in diperbolehkan.
- Matriks singular menghasilkan `SimError::SingularMatrix`.
- Back substitution memakai entri sparse.

---

## 3. Ordering

`minimum_degree_order` mengurutkan baris berdasarkan jumlah entri off-diagonal
sebagai heuristic statis. Ini belum melakukan update degree selama eliminasi.

---

## 4. Test

- Pembentukan CSR dan penjumlahan duplikat.
- Perkalian matriks-vektor.
- Sparse solve dibandingkan dense solve.
- Matriks singular ditolak.
- Degree ordering.
- Konversi dense.

Total test `sim-core`: 62.

---

## 5. Batas

- Sparse solver belum dipakai oleh `MnaContext`.
- Ordering belum diterapkan sebelum eliminasi.
- Belum ada symbolic factorization.
- Belum ada supernodal atau multifrontal.
- Kompleksitas masih dapat membesar karena fill-in.
- Belum ada benchmark terhadap matriks besar.
