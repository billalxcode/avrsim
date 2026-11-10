//! Numeric tolerance helpers.

/// Default relative tolerance for numerical comparisons.
pub const DEFAULT_REL_TOL: f64 = 1e-9;

/// Default absolute tolerance for numerical comparisons.
pub const DEFAULT_ABS_TOL: f64 = 1e-12;

/// Compares two floats with relative and absolute tolerance.
#[must_use]
pub fn approx_eq(a: f64, b: f64, rel_tol: f64, abs_tol: f64) -> bool {
    if !a.is_finite() || !b.is_finite() {
        return false;
    }
    if a == b {
        return true;
    }
    let difference = (a - b).abs();
    let scale = a.abs().max(b.abs());
    difference <= abs_tol.max(rel_tol * scale)
}

/// Returns the largest absolute value in a slice.
#[must_use]
pub fn max_abs(values: &[f64]) -> f64 {
    values
        .iter()
        .fold(0.0_f64, |largest, value| largest.max(value.abs()))
}

/// Returns the Euclidean norm of a slice.
#[must_use]
pub fn l2_norm(values: &[f64]) -> f64 {
    values.iter().map(|value| value * value).sum::<f64>().sqrt()
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_ABS_TOL, DEFAULT_REL_TOL, approx_eq, l2_norm, max_abs};

    fn close(a: f64, b: f64) -> bool {
        approx_eq(a, b, DEFAULT_REL_TOL, DEFAULT_ABS_TOL)
    }

    #[test]
    fn equal_values_are_close() {
        assert!(close(1.0, 1.0));
        assert!(close(0.0, 0.0));
    }

    #[test]
    fn small_differences_are_close() {
        assert!(close(1.0, 1.0 + 1e-12));
        assert!(!close(1.0, 1.0 + 1e-3));
    }

    #[test]
    fn non_finite_values_are_not_close() {
        assert!(!close(f64::NAN, f64::NAN));
        assert!(!close(f64::INFINITY, f64::INFINITY));
    }

    #[test]
    fn max_abs_uses_magnitude() {
        assert_eq!(max_abs(&[1.0, -5.0, 3.0]), 5.0);
        assert_eq!(max_abs(&[]), 0.0);
    }

    #[test]
    fn l2_norm_is_euclidean() {
        assert_eq!(l2_norm(&[3.0, 4.0]), 5.0);
        assert_eq!(l2_norm(&[]), 0.0);
    }
}
