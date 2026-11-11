//! Sparse CSR matrix, sparse LU solve, and degree ordering.

use crate::error::SimError;
use crate::matrix::DenseMatrix;
use std::collections::BTreeMap;

const PIVOT_EPSILON: f64 = 1e-18;

/// Compressed sparse row matrix.
#[derive(Clone, Debug)]
pub struct SparseMatrix {
    rows: usize,
    cols: usize,
    row_ptr: Vec<usize>,
    col_idx: Vec<usize>,
    values: Vec<f64>,
}

impl SparseMatrix {
    /// Builds a matrix from triplets. Duplicate entries are summed.
    pub fn from_triplets(
        rows: usize,
        cols: usize,
        triplets: &[(usize, usize, f64)],
    ) -> Result<Self, SimError> {
        let entries = collect_entries(rows, cols, triplets)?;
        let (row_ptr, col_idx, values) = pack_rows(entries);
        Ok(Self {
            rows,
            cols,
            row_ptr,
            col_idx,
            values,
        })
    }

    /// Returns the number of rows.
    #[must_use]
    pub const fn rows(&self) -> usize {
        self.rows
    }

    /// Returns the number of columns.
    #[must_use]
    pub const fn cols(&self) -> usize {
        self.cols
    }

    /// Returns the number of stored entries.
    #[must_use]
    pub fn nonzeros(&self) -> usize {
        self.values.len()
    }

    /// Returns a value, or None when out of bounds.
    #[must_use]
    pub fn get(&self, row: usize, col: usize) -> Option<f64> {
        if row >= self.rows || col >= self.cols {
            return None;
        }
        let start = self.row_ptr[row];
        let end = self.row_ptr[row + 1];
        let slice = &self.col_idx[start..end];
        let index = slice.binary_search(&col).ok()?;
        Some(self.values[start + index])
    }

    /// Multiplies the matrix by a vector.
    pub fn mul_vec(&self, vector: &[f64]) -> Result<Vec<f64>, SimError> {
        if vector.len() != self.cols {
            return Err(SimError::DimensionMismatch {
                expected: self.cols,
                actual: vector.len(),
            });
        }
        Ok((0..self.rows)
            .map(|row| row_product(self, row, vector))
            .collect())
    }

    /// Converts the matrix to dense form.
    pub fn to_dense(&self) -> Result<DenseMatrix, SimError> {
        let mut dense = DenseMatrix::zeros(self.rows, self.cols)?;
        for row in 0..self.rows {
            let start = self.row_ptr[row];
            let end = self.row_ptr[row + 1];
            for index in start..end {
                dense.set(row, self.col_idx[index], self.values[index])?;
            }
        }
        Ok(dense)
    }
}

fn row_product(matrix: &SparseMatrix, row: usize, vector: &[f64]) -> f64 {
    let start = matrix.row_ptr[row];
    let end = matrix.row_ptr[row + 1];
    (start..end)
        .map(|index| matrix.values[index] * vector[matrix.col_idx[index]])
        .sum()
}

fn collect_entries(
    rows: usize,
    cols: usize,
    triplets: &[(usize, usize, f64)],
) -> Result<Vec<Vec<(usize, f64)>>, SimError> {
    let mut entries = vec![Vec::new(); rows];
    for &(row, col, value) in triplets {
        if row >= rows || col >= cols {
            return Err(SimError::IndexOutOfBounds { row, col });
        }
        entries[row].push((col, value));
    }
    Ok(entries)
}

fn pack_rows(mut entries: Vec<Vec<(usize, f64)>>) -> (Vec<usize>, Vec<usize>, Vec<f64>) {
    let mut row_ptr = Vec::with_capacity(entries.len() + 1);
    let mut col_idx = Vec::new();
    let mut values = Vec::new();
    row_ptr.push(0);
    for row in &mut entries {
        pack_row(row, &mut col_idx, &mut values);
        row_ptr.push(values.len());
    }
    (row_ptr, col_idx, values)
}

fn pack_row(row: &mut [(usize, f64)], col_idx: &mut Vec<usize>, values: &mut Vec<f64>) {
    row.sort_by_key(|(col, _)| *col);
    for (col, value) in merge_row(row) {
        if value != 0.0 {
            col_idx.push(col);
            values.push(value);
        }
    }
}

fn merge_row(row: &[(usize, f64)]) -> Vec<(usize, f64)> {
    let mut merged: Vec<(usize, f64)> = Vec::new();
    for &(col, value) in row {
        if let Some(last) = merged.last_mut() {
            if last.0 == col {
                last.1 += value;
                continue;
            }
        }
        merged.push((col, value));
    }
    merged
}

/// Solves `A * x = rhs` with sparse Gaussian elimination.
pub fn solve_sparse(matrix: &SparseMatrix, rhs: &[f64]) -> Result<Vec<f64>, SimError> {
    validate_sparse(matrix, rhs)?;
    let mut rows = to_rows(matrix);
    let mut result = rhs.to_vec();
    for column in 0..matrix.rows {
        let pivot = select_pivot(&rows, column)?;
        rows.swap(column, pivot);
        result.swap(column, pivot);
        eliminate(&mut rows, &mut result, column);
    }
    Ok(back_substitute(&rows, &result))
}

fn validate_sparse(matrix: &SparseMatrix, rhs: &[f64]) -> Result<(), SimError> {
    if matrix.rows != matrix.cols {
        return Err(SimError::NotSquare {
            rows: matrix.rows,
            cols: matrix.cols,
        });
    }
    if rhs.len() != matrix.rows {
        return Err(SimError::DimensionMismatch {
            expected: matrix.rows,
            actual: rhs.len(),
        });
    }
    Ok(())
}

fn to_rows(matrix: &SparseMatrix) -> Vec<BTreeMap<usize, f64>> {
    (0..matrix.rows)
        .map(|row| {
            let start = matrix.row_ptr[row];
            let end = matrix.row_ptr[row + 1];
            (start..end)
                .map(|index| (matrix.col_idx[index], matrix.values[index]))
                .collect()
        })
        .collect()
}

fn select_pivot(rows: &[BTreeMap<usize, f64>], column: usize) -> Result<usize, SimError> {
    let mut best = column;
    let mut magnitude = 0.0;
    for (row, entries) in rows.iter().enumerate().skip(column) {
        let value = entries.get(&column).copied().unwrap_or(0.0).abs();
        if value > magnitude {
            magnitude = value;
            best = row;
        }
    }
    if magnitude < PIVOT_EPSILON {
        return Err(SimError::SingularMatrix { row: column });
    }
    Ok(best)
}

fn eliminate(rows: &mut [BTreeMap<usize, f64>], rhs: &mut [f64], column: usize) {
    let pivot = rows[column].get(&column).copied().unwrap_or(0.0);
    let pivot_row = rows[column].clone();
    for row in column + 1..rows.len() {
        let factor = rows[row].get(&column).copied().unwrap_or(0.0) / pivot;
        if factor == 0.0 {
            continue;
        }
        for (&col, &value) in &pivot_row {
            if col < column {
                continue;
            }
            let slot = rows[row].entry(col).or_insert(0.0);
            *slot -= factor * value;
        }
        rows[row].remove(&column);
        rhs[row] -= factor * rhs[column];
    }
}

fn back_substitute(rows: &[BTreeMap<usize, f64>], rhs: &[f64]) -> Vec<f64> {
    let mut result = vec![0.0; rhs.len()];
    for row in (0..rows.len()).rev() {
        let diagonal = rows[row].get(&row).copied().unwrap_or(0.0);
        let sum: f64 = rows[row]
            .iter()
            .filter(|(col, _)| **col > row)
            .map(|(col, value)| value * result[*col])
            .sum();
        result[row] = (rhs[row] - sum) / diagonal;
    }
    result
}

/// Returns a static minimum-degree ordering of matrix rows.
#[must_use]
pub fn minimum_degree_order(matrix: &SparseMatrix) -> Vec<usize> {
    let mut nodes: Vec<(usize, usize)> = (0..matrix.rows)
        .map(|row| (degree(matrix, row), row))
        .collect();
    nodes.sort_unstable();
    nodes.into_iter().map(|(_, row)| row).collect()
}

fn degree(matrix: &SparseMatrix, row: usize) -> usize {
    let start = matrix.row_ptr[row];
    let end = matrix.row_ptr[row + 1];
    (start..end)
        .filter(|index| matrix.col_idx[*index] != row)
        .count()
}

#[cfg(test)]
mod tests {
    use super::{SparseMatrix, minimum_degree_order, solve_sparse};

    fn sample() -> SparseMatrix {
        SparseMatrix::from_triplets(
            3,
            3,
            &[
                (0, 0, 4.0),
                (0, 1, 1.0),
                (1, 0, 1.0),
                (1, 1, 3.0),
                (2, 2, 2.0),
            ],
        )
        .expect("matrix")
    }

    #[test]
    fn builds_csr_and_sums_duplicates() {
        let matrix = SparseMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (0, 0, 2.0), (1, 1, 3.0)])
            .expect("matrix");
        assert_eq!(matrix.get(0, 0), Some(3.0));
        assert_eq!(matrix.get(1, 1), Some(3.0));
        assert_eq!(matrix.get(0, 1), None);
        assert_eq!(matrix.nonzeros(), 2);
    }

    #[test]
    fn multiplies_vector() {
        let result = sample().mul_vec(&[1.0, 2.0, 3.0]).expect("matvec");
        assert_eq!(result, vec![6.0, 7.0, 6.0]);
    }

    #[test]
    fn sparse_solve_matches_dense() {
        let sparse = sample();
        let dense = sparse.to_dense().expect("dense");
        let rhs = [1.0, 2.0, 4.0];
        let sparse_x = solve_sparse(&sparse, &rhs).expect("sparse");
        let dense_x = dense.solve(&rhs).expect("dense");
        for (left, right) in sparse_x.iter().zip(dense_x.iter()) {
            assert!((left - right).abs() < 1e-12);
        }
    }

    #[test]
    fn singular_matrix_is_rejected() {
        let matrix = SparseMatrix::from_triplets(
            2,
            2,
            &[(0, 0, 1.0), (0, 1, 1.0), (1, 0, 1.0), (1, 1, 1.0)],
        )
        .expect("matrix");
        assert!(solve_sparse(&matrix, &[1.0, 1.0]).is_err());
    }

    #[test]
    fn degree_order_sorts_low_degree_first() {
        let matrix = SparseMatrix::from_triplets(
            3,
            3,
            &[
                (0, 0, 1.0),
                (0, 1, 1.0),
                (0, 2, 1.0),
                (1, 0, 1.0),
                (1, 1, 1.0),
                (2, 0, 1.0),
                (2, 2, 1.0),
            ],
        )
        .expect("matrix");
        assert_eq!(minimum_degree_order(&matrix), vec![1, 2, 0]);
    }

    #[test]
    fn dense_conversion_preserves_values() {
        let matrix = sample().to_dense().expect("dense");
        assert_eq!(matrix.get(0, 1), Some(1.0));
        assert_eq!(matrix.get(2, 2), Some(2.0));
    }
}
