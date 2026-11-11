//! Dense complex matrix and LU solve for AC analysis.

use crate::complex::Complex64;
use crate::error::SimError;

const PIVOT_EPSILON: f64 = 1e-18;

/// A dense complex matrix.
#[derive(Clone, Debug)]
pub struct ComplexMatrix {
    size: usize,
    data: Vec<Complex64>,
}

impl ComplexMatrix {
    /// Creates a zero matrix.
    pub fn zeros(size: usize) -> Result<Self, SimError> {
        let len = size
            .checked_mul(size)
            .ok_or_else(|| SimError::Internal("complex matrix size overflow".to_string()))?;
        Ok(Self {
            size,
            data: vec![Complex64::default(); len],
        })
    }

    /// Returns the matrix size.
    #[must_use]
    pub const fn size(&self) -> usize {
        self.size
    }

    /// Returns one entry.
    #[must_use]
    pub fn get(&self, row: usize, col: usize) -> Option<Complex64> {
        if row >= self.size || col >= self.size {
            return None;
        }
        Some(self.data[row * self.size + col])
    }

    /// Adds a value to one entry.
    pub fn add(&mut self, row: usize, col: usize, value: Complex64) -> Result<(), SimError> {
        if row >= self.size || col >= self.size {
            return Err(SimError::IndexOutOfBounds { row, col });
        }
        self.data[row * self.size + col] = self.data[row * self.size + col] + value;
        Ok(())
    }

    /// Solves `A * x = rhs` with partial pivoting.
    pub fn solve(&self, rhs: &[Complex64]) -> Result<Vec<Complex64>, SimError> {
        if rhs.len() != self.size {
            return Err(SimError::DimensionMismatch {
                expected: self.size,
                actual: rhs.len(),
            });
        }
        let mut matrix = self.data.clone();
        let mut result = rhs.to_vec();
        for column in 0..self.size {
            let pivot = select_pivot(&matrix, self.size, column)?;
            swap_rows(&mut matrix, self.size, column, pivot);
            result.swap(column, pivot);
            eliminate(&mut matrix, &mut result, self.size, column);
        }
        Ok(back_substitute(&matrix, &result, self.size))
    }
}

fn select_pivot(data: &[Complex64], size: usize, column: usize) -> Result<usize, SimError> {
    let mut best = column;
    let mut magnitude = 0.0;
    for row in column..size {
        let value = data[row * size + column].magnitude();
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

fn swap_rows(data: &mut [Complex64], size: usize, first: usize, second: usize) {
    if first == second {
        return;
    }
    for col in 0..size {
        data.swap(first * size + col, second * size + col);
    }
}

fn eliminate(data: &mut [Complex64], rhs: &mut [Complex64], size: usize, column: usize) {
    let pivot = data[column * size + column];
    for row in column + 1..size {
        let factor = data[row * size + column] / pivot;
        if factor.magnitude() == 0.0 {
            continue;
        }
        data[row * size + column] = Complex64::default();
        for col in column + 1..size {
            let update = factor * data[column * size + col];
            data[row * size + col] = data[row * size + col] - update;
        }
        rhs[row] = rhs[row] - factor * rhs[column];
    }
}

fn back_substitute(data: &[Complex64], rhs: &[Complex64], size: usize) -> Vec<Complex64> {
    let mut result = vec![Complex64::default(); size];
    for row in (0..size).rev() {
        let diagonal = data[row * size + row];
        let mut sum = Complex64::default();
        for col in row + 1..size {
            sum = sum + data[row * size + col] * result[col];
        }
        result[row] = (rhs[row] - sum) / diagonal;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::ComplexMatrix;
    use crate::complex::Complex64;

    #[test]
    fn solves_complex_system() {
        let mut matrix = ComplexMatrix::zeros(2).expect("matrix");
        matrix.add(0, 0, Complex64::new(1.0, 1.0)).expect("a");
        matrix.add(0, 1, Complex64::new(0.0, 0.0)).expect("b");
        matrix.add(1, 0, Complex64::new(0.0, 0.0)).expect("c");
        matrix.add(1, 1, Complex64::new(2.0, 0.0)).expect("d");
        let result = matrix
            .solve(&[Complex64::new(2.0, 2.0), Complex64::new(4.0, 0.0)])
            .expect("solve");
        assert!((result[0].re - 2.0).abs() < 1e-12);
        assert!((result[0].im).abs() < 1e-12);
        assert!((result[1].re - 2.0).abs() < 1e-12);
    }

    #[test]
    fn singular_complex_matrix_is_rejected() {
        let matrix = ComplexMatrix::zeros(2).expect("matrix");
        assert!(
            matrix
                .solve(&[Complex64::default(), Complex64::default()])
                .is_err()
        );
    }
}
