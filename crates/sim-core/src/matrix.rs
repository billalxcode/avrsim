//! Dense row-major matrix and LU solver.

use crate::error::SimError;

const SINGULAR_EPSILON: f64 = 1e-18;

/// A dense row-major matrix of `f64` values.
#[derive(Clone, Debug)]
pub struct DenseMatrix {
    rows: usize,
    cols: usize,
    data: Vec<f64>,
}

impl DenseMatrix {
    /// Creates a zero-filled matrix.
    pub fn zeros(rows: usize, cols: usize) -> Result<Self, SimError> {
        let len = rows
            .checked_mul(cols)
            .ok_or_else(|| SimError::Internal("matrix size overflow".to_string()))?;
        Ok(Self {
            rows,
            cols,
            data: vec![0.0; len],
        })
    }

    /// Creates a matrix from row-major data.
    pub fn from_vec(rows: usize, cols: usize, data: Vec<f64>) -> Result<Self, SimError> {
        let expected = rows
            .checked_mul(cols)
            .ok_or_else(|| SimError::Internal("matrix size overflow".to_string()))?;
        if data.len() != expected {
            return Err(SimError::DimensionMismatch {
                expected,
                actual: data.len(),
            });
        }
        Ok(Self { rows, cols, data })
    }

    /// Creates an identity matrix.
    pub fn identity(size: usize) -> Result<Self, SimError> {
        let mut matrix = Self::zeros(size, size)?;
        for index in 0..size {
            matrix.set(index, index, 1.0)?;
        }
        Ok(matrix)
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

    /// Returns a value, or `None` when the index is out of bounds.
    #[must_use]
    pub fn get(&self, row: usize, col: usize) -> Option<f64> {
        if row >= self.rows || col >= self.cols {
            return None;
        }
        Some(self.get_unchecked(row, col))
    }

    /// Sets a value, returning an error when the index is out of bounds.
    pub fn set(&mut self, row: usize, col: usize, value: f64) -> Result<(), SimError> {
        if row >= self.rows || col >= self.cols {
            return Err(SimError::IndexOutOfBounds { row, col });
        }
        self.set_unchecked(row, col, value);
        Ok(())
    }

    /// Returns a row slice, or `None` when the row is out of bounds.
    #[must_use]
    pub fn row(&self, row: usize) -> Option<&[f64]> {
        if row >= self.rows {
            return None;
        }
        let start = row * self.cols;
        Some(&self.data[start..start + self.cols])
    }

    /// Adds a value to an existing matrix entry.
    pub fn add(&mut self, row: usize, col: usize, value: f64) -> Result<(), SimError> {
        let current = self
            .get(row, col)
            .ok_or(SimError::IndexOutOfBounds { row, col })?;
        self.set(row, col, current + value)
    }

    /// Returns a column as a new vector, or `None` when out of bounds.
    #[must_use]
    pub fn column(&self, col: usize) -> Option<Vec<f64>> {
        if col >= self.cols {
            return None;
        }
        let values = (0..self.rows).map(|row| self.get_unchecked(row, col));
        Some(values.collect())
    }

    /// Solves `A * x = rhs` using LU decomposition with partial pivoting.
    pub fn solve(&self, rhs: &[f64]) -> Result<Vec<f64>, SimError> {
        self.ensure_square()?;
        if rhs.len() != self.rows {
            return Err(SimError::DimensionMismatch {
                expected: self.rows,
                actual: rhs.len(),
            });
        }
        if self.rows == 0 {
            return Ok(Vec::new());
        }
        let mut lu = self.clone();
        let pivots = lu.lu_decompose()?;
        let mut y = rhs.to_vec();
        apply_pivots(&mut y, &pivots);
        let y = forward_substitute(&lu, &y);
        back_substitute(&lu, &y)
    }

    fn ensure_square(&self) -> Result<(), SimError> {
        if self.rows != self.cols {
            return Err(SimError::NotSquare {
                rows: self.rows,
                cols: self.cols,
            });
        }
        Ok(())
    }

    fn lu_decompose(&mut self) -> Result<Vec<usize>, SimError> {
        let size = self.rows;
        let mut pivots = vec![0; size];
        for (column, pivot_slot) in pivots.iter_mut().enumerate() {
            let pivot = self.find_pivot(column);
            *pivot_slot = pivot;
            let value = self.get_unchecked(pivot, column);
            if value.abs() < SINGULAR_EPSILON {
                return Err(SimError::SingularMatrix { row: column });
            }
            self.swap_rows(column, pivot);
            self.eliminate_below(column);
        }
        Ok(pivots)
    }

    fn find_pivot(&self, column: usize) -> usize {
        let mut pivot = column;
        let mut best = self.get_unchecked(column, column).abs();
        for row in (column + 1)..self.rows {
            let value = self.get_unchecked(row, column).abs();
            if value > best {
                best = value;
                pivot = row;
            }
        }
        pivot
    }

    fn swap_rows(&mut self, first: usize, second: usize) {
        if first == second {
            return;
        }
        for col in 0..self.cols {
            let a = first * self.cols + col;
            let b = second * self.cols + col;
            self.data.swap(a, b);
        }
    }

    fn eliminate_below(&mut self, column: usize) {
        let pivot = self.get_unchecked(column, column);
        for row in (column + 1)..self.rows {
            let factor = self.get_unchecked(row, column) / pivot;
            self.set_unchecked(row, column, factor);
            for col in (column + 1)..self.cols {
                let value = self.get_unchecked(row, col) - factor * self.get_unchecked(column, col);
                self.set_unchecked(row, col, value);
            }
        }
    }

    fn get_unchecked(&self, row: usize, col: usize) -> f64 {
        self.data[row * self.cols + col]
    }

    fn set_unchecked(&mut self, row: usize, col: usize, value: f64) {
        self.data[row * self.cols + col] = value;
    }
}

fn apply_pivots(rhs: &mut [f64], pivots: &[usize]) {
    for (index, &pivot) in pivots.iter().enumerate() {
        if index != pivot {
            rhs.swap(index, pivot);
        }
    }
}

fn forward_substitute(lu: &DenseMatrix, rhs: &[f64]) -> Vec<f64> {
    let mut result = rhs.to_vec();
    for row in 0..lu.rows() {
        let mut sum = result[row];
        for (col, &value) in result.iter().enumerate().take(row) {
            sum -= lu.get_unchecked(row, col) * value;
        }
        result[row] = sum;
    }
    result
}

fn back_substitute(lu: &DenseMatrix, rhs: &[f64]) -> Result<Vec<f64>, SimError> {
    let size = lu.rows();
    let mut result = vec![0.0; size];
    for row in (0..size).rev() {
        let mut sum = rhs[row];
        for (col, &value) in result.iter().enumerate().skip(row + 1) {
            sum -= lu.get_unchecked(row, col) * value;
        }
        let diagonal = lu.get_unchecked(row, row);
        if diagonal.abs() < SINGULAR_EPSILON {
            return Err(SimError::SingularMatrix { row });
        }
        result[row] = sum / diagonal;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::DenseMatrix;
    use crate::error::SimError;

    fn assert_close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 1e-9, "{actual} != {expected}");
    }

    #[test]
    fn solves_two_by_two_system() {
        let matrix = DenseMatrix::from_vec(2, 2, vec![4.0, 3.0, 6.0, 3.0]).expect("matrix");
        let solution = matrix.solve(&[10.0, 12.0]).expect("solution");
        assert_close(solution[0], 1.0);
        assert_close(solution[1], 2.0);
    }

    #[test]
    fn solves_three_by_three_system() {
        let data = vec![2.0, 1.0, -1.0, -3.0, -1.0, 2.0, -2.0, 1.0, 2.0];
        let matrix = DenseMatrix::from_vec(3, 3, data).expect("matrix");
        let solution = matrix.solve(&[8.0, -11.0, -3.0]).expect("solution");
        assert_close(solution[0], 2.0);
        assert_close(solution[1], 3.0);
        assert_close(solution[2], -1.0);
    }

    #[test]
    fn pivots_when_the_first_pivot_is_zero() {
        let matrix = DenseMatrix::from_vec(2, 2, vec![0.0, 1.0, 1.0, 0.0]).expect("matrix");
        let solution = matrix.solve(&[2.0, 3.0]).expect("solution");
        assert_close(solution[0], 3.0);
        assert_close(solution[1], 2.0);
    }

    #[test]
    fn rejects_singular_matrix() {
        let matrix = DenseMatrix::from_vec(2, 2, vec![1.0, 2.0, 2.0, 4.0]).expect("matrix");
        let result = matrix.solve(&[3.0, 6.0]);
        assert!(matches!(result, Err(SimError::SingularMatrix { .. })));
    }

    #[test]
    fn rejects_dimension_mismatch() {
        let matrix = DenseMatrix::from_vec(2, 2, vec![1.0, 0.0, 0.0, 1.0]).expect("matrix");
        let result = matrix.solve(&[1.0]);
        assert!(matches!(result, Err(SimError::DimensionMismatch { .. })));
    }

    #[test]
    fn rejects_non_square_matrix() {
        let matrix = DenseMatrix::from_vec(2, 3, vec![0.0; 6]).expect("matrix");
        let result = matrix.solve(&[1.0, 2.0]);
        assert!(matches!(result, Err(SimError::NotSquare { .. })));
    }

    #[test]
    fn solves_empty_system() {
        let matrix = DenseMatrix::zeros(0, 0).expect("matrix");
        assert_eq!(matrix.solve(&[]).expect("solution"), Vec::<f64>::new());
    }

    #[test]
    fn solves_identity_system() {
        let matrix = DenseMatrix::identity(3).expect("matrix");
        let solution = matrix.solve(&[1.0, 2.0, 3.0]).expect("solution");
        assert_eq!(solution, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn accessors_respect_bounds() {
        let mut matrix = DenseMatrix::zeros(2, 2).expect("matrix");
        matrix.set(0, 0, 1.0).expect("set");
        matrix.set(0, 1, 2.0).expect("set");
        assert_eq!(matrix.get(0, 1), Some(2.0));
        assert_eq!(matrix.get(2, 0), None);
        assert_eq!(matrix.row(0).map(|row| row.to_vec()), Some(vec![1.0, 2.0]));
        assert_eq!(matrix.column(1), Some(vec![2.0, 0.0]));
    }

    #[test]
    fn from_vec_rejects_wrong_length() {
        let result = DenseMatrix::from_vec(2, 2, vec![1.0]);
        assert!(matches!(result, Err(SimError::DimensionMismatch { .. })));
    }

    #[test]
    fn repeated_solve_is_stable() {
        let matrix = DenseMatrix::from_vec(2, 2, vec![4.0, 3.0, 6.0, 3.0]).expect("matrix");
        for _ in 0..1_000 {
            let solution = matrix.solve(&[10.0, 12.0]).expect("solution");
            assert_close(solution[0], 1.0);
            assert_close(solution[1], 2.0);
        }
    }
}
