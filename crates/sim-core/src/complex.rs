//! Complex numbers for AC analysis.

use std::ops::{Add, Div, Mul, Neg, Sub};

/// A complex number with `f64` components.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Complex64 {
    /// Real component.
    pub re: f64,
    /// Imaginary component.
    pub im: f64,
}

impl Complex64 {
    /// Creates a complex number.
    #[must_use]
    pub const fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    /// Creates a real complex number.
    #[must_use]
    pub const fn from_real(re: f64) -> Self {
        Self { re, im: 0.0 }
    }

    /// Returns the magnitude.
    #[must_use]
    pub fn magnitude(self) -> f64 {
        self.re.hypot(self.im)
    }

    /// Returns the phase in radians.
    #[must_use]
    pub fn phase(self) -> f64 {
        self.im.atan2(self.re)
    }

    /// Returns the complex conjugate.
    #[must_use]
    pub const fn conjugate(self) -> Self {
        Self {
            re: self.re,
            im: -self.im,
        }
    }

    /// Returns true when both components are finite.
    #[must_use]
    pub fn is_finite(self) -> bool {
        self.re.is_finite() && self.im.is_finite()
    }
}

impl Add for Complex64 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.re + rhs.re, self.im + rhs.im)
    }
}

impl Sub for Complex64 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.re - rhs.re, self.im - rhs.im)
    }
}

impl Mul for Complex64 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let re = self.re * rhs.re - self.im * rhs.im;
        let im = self.re * rhs.im + self.im * rhs.re;
        Self::new(re, im)
    }
}

impl Div for Complex64 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let denominator = rhs.re * rhs.re + rhs.im * rhs.im;
        if denominator == 0.0 {
            return Self::new(f64::NAN, f64::NAN);
        }
        let re = (self.re * rhs.re + self.im * rhs.im) / denominator;
        let im = (self.im * rhs.re - self.re * rhs.im) / denominator;
        Self::new(re, im)
    }
}

impl Neg for Complex64 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.re, -self.im)
    }
}

#[cfg(test)]
mod tests {
    use super::Complex64;

    fn assert_close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 1e-9, "{actual} != {expected}");
    }

    #[test]
    fn add_and_sub_are_componentwise() {
        let a = Complex64::new(1.0, 2.0);
        let b = Complex64::new(3.0, -1.0);
        assert_eq!(a + b, Complex64::new(4.0, 1.0));
        assert_eq!(a - b, Complex64::new(-2.0, 3.0));
    }

    #[test]
    fn multiplication_uses_complex_rule() {
        let a = Complex64::new(1.0, 2.0);
        let b = Complex64::new(3.0, 4.0);
        assert_eq!(a * b, Complex64::new(-5.0, 10.0));
    }

    #[test]
    fn division_inverts_multiplication() {
        let a = Complex64::new(1.0, 2.0);
        let b = Complex64::new(3.0, 4.0);
        let result = a / b;
        assert_close(result.re, 0.44);
        assert_close(result.im, 0.08);
    }

    #[test]
    fn magnitude_and_phase_are_correct() {
        let value = Complex64::new(3.0, 4.0);
        assert_close(value.magnitude(), 5.0);
        assert_close(value.phase(), 4.0_f64.atan2(3.0));
    }

    #[test]
    fn conjugate_flips_imaginary_part() {
        let value = Complex64::new(2.0, -3.0);
        assert_eq!(value.conjugate(), Complex64::new(2.0, 3.0));
    }

    #[test]
    fn division_by_zero_is_nan() {
        let result = Complex64::new(1.0, 0.0) / Complex64::new(0.0, 0.0);
        assert!(result.re.is_nan());
        assert!(result.im.is_nan());
    }
}
