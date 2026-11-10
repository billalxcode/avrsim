//! Physical quantity newtypes.

macro_rules! define_unit {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
        pub struct $name(f64);

        impl $name {
            /// Creates a quantity from its base unit.
            #[must_use]
            pub const fn new(value: f64) -> Self {
                Self(value)
            }

            /// Returns the value in its base unit.
            #[must_use]
            pub const fn value(self) -> f64 {
                self.0
            }

            /// Returns true when the value is finite.
            #[must_use]
            pub fn is_finite(self) -> bool {
                self.0.is_finite()
            }
        }
    };
}

define_unit!(Voltage, "Electrical potential in volts.");
define_unit!(Current, "Electrical current in amperes.");
define_unit!(Resistance, "Electrical resistance in ohms.");
define_unit!(Capacitance, "Electrical capacitance in farads.");

#[cfg(test)]
mod tests {
    use super::{Capacitance, Current, Resistance, Voltage};

    #[test]
    fn base_units_round_trip() {
        assert_eq!(Voltage::new(3.3).value(), 3.3);
        assert_eq!(Current::new(0.001).value(), 0.001);
        assert_eq!(Resistance::new(1_000.0).value(), 1_000.0);
        assert_eq!(Capacitance::new(1e-6).value(), 1e-6);
    }

    #[test]
    fn default_is_zero() {
        assert_eq!(Voltage::default().value(), 0.0);
        assert_eq!(Current::default().value(), 0.0);
    }

    #[test]
    fn finite_check_rejects_invalid_values() {
        assert!(Voltage::new(5.0).is_finite());
        assert!(!Voltage::new(f64::NAN).is_finite());
        assert!(!Resistance::new(f64::INFINITY).is_finite());
    }
}
