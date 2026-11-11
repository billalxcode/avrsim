//! Timer1 16-bit counter, prescaler, compare, capture, and PWM.

use sim_cells::{DigitalBuilder, DigitalCircuit, NetId, increment, read_bits};
use sim_core::SimError;

/// Timer1 core with a 16-bit structural counter.
pub struct Timer1 {
    circuit: DigitalCircuit,
    bits: Vec<NetId>,
    prescaler: u32,
    divider: u32,
    compare_a: u16,
    compare_b: u16,
    captured: u16,
    ctc: bool,
    pwm: bool,
    inverting: bool,
    pwm_output: bool,
    overflow: bool,
    compare_a_flag: bool,
    compare_b_flag: bool,
    capture_flag: bool,
}

impl Timer1 {
    /// Builds Timer1 with prescaler 1.
    pub fn new() -> Result<Self, SimError> {
        let (circuit, bits) = build_counter();
        Ok(Self {
            circuit,
            bits,
            prescaler: 0,
            divider: 1,
            compare_a: 0,
            compare_b: 0,
            captured: 0,
            ctc: false,
            pwm: false,
            inverting: false,
            pwm_output: false,
            overflow: false,
            compare_a_flag: false,
            compare_b_flag: false,
            capture_flag: false,
        })
    }

    /// Sets the prescaler select value.
    pub fn set_prescaler(&mut self, select: u8) -> Result<(), SimError> {
        self.divider = match select {
            0 => 1,
            1 => 8,
            2 => 64,
            3 => 256,
            4 => 1024,
            _ => {
                return Err(SimError::InvalidNetlist(
                    "invalid Timer1 prescaler".to_string(),
                ));
            }
        };
        self.prescaler = 0;
        Ok(())
    }

    /// Sets compare A.
    pub fn set_compare_a(&mut self, value: u16) {
        self.compare_a = value;
    }

    /// Sets compare B.
    pub fn set_compare_b(&mut self, value: u16) {
        self.compare_b = value;
    }

    /// Sets the captured input value.
    pub fn capture(&mut self, value: u16) {
        self.captured = value;
        self.capture_flag = true;
    }

    /// Returns the captured value.
    #[must_use]
    pub const fn captured(&self) -> u16 {
        self.captured
    }

    /// Enables or disables CTC mode using compare A.
    pub fn set_ctc(&mut self, enabled: bool) {
        self.ctc = enabled;
        self.pwm = false;
    }

    /// Enables fast PWM on compare A.
    pub fn set_pwm(&mut self, inverting: bool) {
        self.pwm = true;
        self.inverting = inverting;
        self.ctc = false;
    }

    /// Advances the timer by CPU cycles.
    pub fn tick(&mut self, cycles: u32) -> Result<(), SimError> {
        for _ in 0..cycles {
            self.tick_once()?;
        }
        Ok(())
    }

    /// Returns the counter value.
    #[must_use]
    pub fn counter(&self) -> u16 {
        read_bits(&self.circuit, &self.bits) as u16
    }

    /// Returns the PWM output compare A signal.
    #[must_use]
    pub const fn pwm_output(&self) -> bool {
        self.pwm_output
    }

    /// Takes and clears the overflow flag.
    pub fn take_overflow(&mut self) -> bool {
        take(&mut self.overflow)
    }

    /// Takes and clears compare A.
    pub fn take_compare_a(&mut self) -> bool {
        take(&mut self.compare_a_flag)
    }

    /// Takes and clears compare B.
    pub fn take_compare_b(&mut self) -> bool {
        take(&mut self.compare_b_flag)
    }

    /// Takes and clears capture.
    pub fn take_capture(&mut self) -> bool {
        take(&mut self.capture_flag)
    }

    /// Returns overflow without clearing it.
    #[must_use]
    pub const fn overflow_flag(&self) -> bool {
        self.overflow
    }

    /// Returns compare A without clearing it.
    #[must_use]
    pub const fn compare_a_flag(&self) -> bool {
        self.compare_a_flag
    }

    /// Returns compare B without clearing it.
    #[must_use]
    pub const fn compare_b_flag(&self) -> bool {
        self.compare_b_flag
    }

    /// Returns capture without clearing it.
    #[must_use]
    pub const fn capture_flag(&self) -> bool {
        self.capture_flag
    }

    /// Resets counter, prescaler, and flags.
    pub fn reset(&mut self) -> Result<(), SimError> {
        self.clear_counter()?;
        self.prescaler = 0;
        self.overflow = false;
        self.compare_a_flag = false;
        self.compare_b_flag = false;
        self.capture_flag = false;
        Ok(())
    }

    fn tick_once(&mut self) -> Result<(), SimError> {
        self.prescaler += 1;
        if self.prescaler < self.divider {
            return Ok(());
        }
        self.prescaler = 0;
        let previous = self.counter();
        self.pulse()?;
        let current = self.counter();
        self.update_flags(previous, current);
        if current == self.compare_a && self.ctc {
            self.clear_counter()?;
        }
        Ok(())
    }

    fn update_flags(&mut self, previous: u16, current: u16) {
        self.overflow |= previous == u16::MAX;
        self.compare_a_flag |= current == self.compare_a;
        self.compare_b_flag |= current == self.compare_b;
        self.update_pwm(current);
    }

    fn update_pwm(&mut self, value: u16) {
        if !self.pwm {
            self.pwm_output = false;
            return;
        }
        self.pwm_output = if self.inverting {
            value >= self.compare_a
        } else {
            value < self.compare_a
        };
    }

    fn pulse(&mut self) -> Result<(), SimError> {
        self.set_bool("enable", true)?;
        self.circuit.tick()?;
        self.set_bool("enable", false)?;
        self.circuit.settle()
    }

    fn clear_counter(&mut self) -> Result<(), SimError> {
        self.set_bool("reset", true)?;
        self.circuit.tick()?;
        self.set_bool("reset", false)?;
        self.circuit.settle()
    }

    fn set_bool(&mut self, name: &str, value: bool) -> Result<(), SimError> {
        self.circuit.set_input(name, value)
    }
}

fn take(value: &mut bool) -> bool {
    let result = *value;
    *value = false;
    result
}

fn build_counter() -> (DigitalCircuit, Vec<NetId>) {
    let mut builder = DigitalBuilder::new();
    let enable = builder.input("enable");
    let reset = builder.input("reset");
    let bits: Vec<NetId> = (0..16).map(|_| builder.net()).collect();
    let incremented = increment(&mut builder, &bits);
    let zero = builder.constant(false);
    for index in 0..16 {
        let stepped = builder.mux(bits[index], incremented[index], enable);
        let next = builder.mux(stepped, zero, reset);
        builder.dff_into(next, bits[index]);
    }
    (builder.build(), bits)
}

#[cfg(test)]
mod tests {
    use super::Timer1;

    fn timer() -> Timer1 {
        Timer1::new().expect("timer")
    }

    #[test]
    fn prescaler_divides_clock() {
        let mut timer = timer();
        timer.set_prescaler(1).expect("prescaler");
        timer.tick(7).expect("tick");
        assert_eq!(timer.counter(), 0);
        timer.tick(1).expect("tick");
        assert_eq!(timer.counter(), 1);
    }

    #[test]
    fn compare_a_and_b_set_flags() {
        let mut timer = timer();
        timer.set_compare_a(2);
        timer.set_compare_b(4);
        timer.tick(5).expect("tick");
        assert!(timer.take_compare_a());
        assert!(timer.take_compare_b());
    }

    #[test]
    fn capture_stores_value() {
        let mut timer = timer();
        timer.capture(0x1234);
        assert_eq!(timer.captured(), 0x1234);
        assert!(timer.take_capture());
    }

    #[test]
    fn ctc_resets_counter() {
        let mut timer = timer();
        timer.set_compare_a(3);
        timer.set_ctc(true);
        timer.tick(3).expect("tick");
        assert_eq!(timer.counter(), 0);
        assert!(timer.take_compare_a());
    }

    #[test]
    fn pwm_tracks_compare_a() {
        let mut timer = timer();
        timer.set_compare_a(2);
        timer.set_pwm(false);
        timer.tick(1).expect("tick");
        assert!(timer.pwm_output());
        timer.tick(1).expect("tick");
        assert!(!timer.pwm_output());
    }

    #[test]
    fn invalid_prescaler_is_rejected() {
        let mut timer = timer();
        assert!(timer.set_prescaler(5).is_err());
    }
}
