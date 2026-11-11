//! Board components: LED, resistor, and button.

/// A resistor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Resistor {
    /// Resistance in ohms.
    pub ohms: f64,
}

impl Resistor {
    /// Creates a resistor.
    #[must_use]
    pub const fn new(ohms: f64) -> Self {
        Self { ohms }
    }

    /// Returns current for a voltage across the resistor.
    #[must_use]
    pub fn current(&self, voltage: f64) -> f64 {
        if self.ohms <= 0.0 {
            return 0.0;
        }
        voltage / self.ohms
    }
}

/// A light emitting diode.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Led {
    /// Forward voltage.
    pub forward_voltage: f64,
    current: f64,
}

impl Led {
    /// Creates an LED.
    #[must_use]
    pub const fn new(forward_voltage: f64) -> Self {
        Self {
            forward_voltage,
            current: 0.0,
        }
    }

    /// Sets the LED current.
    pub fn set_current(&mut self, current: f64) {
        self.current = current.max(0.0);
    }

    /// Returns true when the LED is conducting.
    #[must_use]
    pub fn is_on(&self) -> bool {
        self.current > 0.0
    }

    /// Returns the LED current.
    #[must_use]
    pub const fn current(&self) -> f64 {
        self.current
    }
}

/// A push button.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Button {
    pressed: bool,
}

impl Button {
    /// Creates a released button.
    #[must_use]
    pub const fn new() -> Self {
        Self { pressed: false }
    }

    /// Presses the button.
    pub fn press(&mut self) {
        self.pressed = true;
    }

    /// Releases the button.
    pub fn release(&mut self) {
        self.pressed = false;
    }

    /// Returns true when pressed.
    #[must_use]
    pub const fn is_pressed(&self) -> bool {
        self.pressed
    }
}

#[cfg(test)]
mod tests {
    use super::{Button, Led, Resistor};

    #[test]
    fn resistor_current_follows_ohms_law() {
        let resistor = Resistor::new(220.0);
        assert!((resistor.current(2.2) - 0.01).abs() < 1e-9);
    }

    #[test]
    fn led_turns_on_with_current() {
        let mut led = Led::new(2.0);
        assert!(!led.is_on());
        led.set_current(0.01);
        assert!(led.is_on());
    }

    #[test]
    fn button_press_and_release() {
        let mut button = Button::new();
        assert!(!button.is_pressed());
        button.press();
        assert!(button.is_pressed());
        button.release();
        assert!(!button.is_pressed());
    }
}
