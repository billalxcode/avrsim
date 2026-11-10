//! Simulation time in integer picoseconds.

use std::fmt;

const PS_PER_NS: u64 = 1_000;
const PS_PER_US: u64 = 1_000_000;
const PS_PER_MS: u64 = 1_000_000_000;
const PS_PER_SECOND: u64 = 1_000_000_000_000;

/// A point or duration in simulation time, stored as picoseconds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Time(u64);

impl Time {
    /// Zero simulation time.
    pub const ZERO: Self = Self(0);

    /// Creates a time from picoseconds.
    #[must_use]
    pub const fn from_ps(ps: u64) -> Self {
        Self(ps)
    }

    /// Returns the time in picoseconds.
    #[must_use]
    pub const fn as_ps(self) -> u64 {
        self.0
    }

    /// Creates a time from nanoseconds.
    #[must_use]
    pub const fn from_ns(ns: u64) -> Option<Self> {
        match ns.checked_mul(PS_PER_NS) {
            Some(ps) => Some(Self(ps)),
            None => None,
        }
    }

    /// Creates a time from microseconds.
    #[must_use]
    pub const fn from_us(us: u64) -> Option<Self> {
        match us.checked_mul(PS_PER_US) {
            Some(ps) => Some(Self(ps)),
            None => None,
        }
    }

    /// Creates a time from milliseconds.
    #[must_use]
    pub const fn from_ms(ms: u64) -> Option<Self> {
        match ms.checked_mul(PS_PER_MS) {
            Some(ps) => Some(Self(ps)),
            None => None,
        }
    }

    /// Creates a time from whole seconds.
    #[must_use]
    pub const fn from_secs(seconds: u64) -> Option<Self> {
        match seconds.checked_mul(PS_PER_SECOND) {
            Some(ps) => Some(Self(ps)),
            None => None,
        }
    }

    /// Creates a time from fractional seconds.
    #[must_use]
    pub fn from_secs_f64(seconds: f64) -> Option<Self> {
        if !seconds.is_finite() || seconds < 0.0 {
            return None;
        }
        let ps = (seconds * PS_PER_SECOND as f64).round();
        if ps > u64::MAX as f64 {
            return None;
        }
        Some(Self(ps as u64))
    }

    /// Returns the time in nanoseconds as a float.
    #[must_use]
    pub fn as_ns_f64(self) -> f64 {
        self.0 as f64 / PS_PER_NS as f64
    }

    /// Returns the time in microseconds as a float.
    #[must_use]
    pub fn as_us_f64(self) -> f64 {
        self.0 as f64 / PS_PER_US as f64
    }

    /// Returns the time in milliseconds as a float.
    #[must_use]
    pub fn as_ms_f64(self) -> f64 {
        self.0 as f64 / PS_PER_MS as f64
    }

    /// Returns the time in seconds as a float.
    #[must_use]
    pub fn as_secs_f64(self) -> f64 {
        self.0 as f64 / PS_PER_SECOND as f64
    }

    /// Adds two times, returning `None` on overflow.
    #[must_use]
    pub const fn checked_add(self, other: Self) -> Option<Self> {
        match self.0.checked_add(other.0) {
            Some(ps) => Some(Self(ps)),
            None => None,
        }
    }

    /// Subtracts two times, returning `None` on underflow.
    #[must_use]
    pub const fn checked_sub(self, other: Self) -> Option<Self> {
        match self.0.checked_sub(other.0) {
            Some(ps) => Some(Self(ps)),
            None => None,
        }
    }

    /// Adds two times, saturating at the maximum.
    #[must_use]
    pub const fn saturating_add(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }

    /// Subtracts two times, saturating at zero.
    #[must_use]
    pub const fn saturating_sub(self, other: Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ps", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::Time;

    #[test]
    fn conversions_are_exact() {
        assert_eq!(Time::from_ns(1), Some(Time::from_ps(1_000)));
        assert_eq!(Time::from_us(1), Some(Time::from_ps(1_000_000)));
        assert_eq!(Time::from_ms(1), Some(Time::from_ps(1_000_000_000)));
        assert_eq!(Time::from_secs(1), Some(Time::from_ps(1_000_000_000_000)));
    }

    #[test]
    fn float_conversions_round_trip() {
        let time = Time::from_secs_f64(0.000_001).expect("time");
        assert!((time.as_us_f64() - 1.0).abs() < 1e-9);
        assert!((time.as_ms_f64() - 0.001).abs() < 1e-12);
        assert!((time.as_secs_f64() - 0.000_001).abs() < 1e-15);
    }

    #[test]
    fn invalid_float_input_is_rejected() {
        assert_eq!(Time::from_secs_f64(-1.0), None);
        assert_eq!(Time::from_secs_f64(f64::NAN), None);
        assert_eq!(Time::from_secs_f64(f64::INFINITY), None);
    }

    #[test]
    fn checked_arithmetic_detects_limits() {
        let max = Time::from_ps(u64::MAX);
        assert_eq!(max.checked_add(Time::from_ps(1)), None);
        assert_eq!(Time::ZERO.checked_sub(Time::from_ps(1)), None);
        assert_eq!(max.saturating_add(Time::from_ps(1)), max);
        assert_eq!(Time::ZERO.saturating_sub(Time::from_ps(1)), Time::ZERO);
    }

    #[test]
    fn ordering_uses_picoseconds() {
        assert!(Time::from_ps(1) < Time::from_ps(2));
        assert_eq!(Time::from_ns(1), Some(Time::from_ps(1_000)));
    }
}
