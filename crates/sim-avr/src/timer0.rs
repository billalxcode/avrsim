//! Timer0 counter, prescaler, normal mode, and CTC.

use sim_cells::{DigitalBuilder, DigitalCircuit, NetId, increment, read_bits};
use sim_core::SimError;

/// Timer0 core with an 8-bit structural counter.
pub struct Timer0 {
    circuit: DigitalCircuit,
    bits: Vec<NetId>,
    prescaler: u16,
    divider: u16,
    compare: u8,
    ctc: bool,
    overflow: bool,
    compare_match: bool,
}

impl Timer0 {
    /// Builds Timer0 in normal mode with prescaler 1.
    pub fn new() -> Result<Self, SimError> {
        let (circuit, bits) = build_counter();
        Ok(Self {
            circuit,
            bits,
            prescaler: 0,
            divider: 1,
            compare: 0,
            ctc: false,
            overflow: false,
            compare_match: false,
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
                    "invalid Timer0 prescaler".to_string(),
                ));
            }
        };
        self.prescaler = 0;
        Ok(())
    }

    /// Sets the compare value.
    pub fn set_compare(&mut self, value: u8) {
        self.compare = value;
    }

    /// Enables or disables CTC mode.
    pub fn set_ctc(&mut self, enabled: bool) {
        self.ctc = enabled;
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
    pub fn counter(&self) -> u8 {
        read_bits(&self.circuit, &self.bits) as u8
    }

    /// Takes and clears the overflow flag.
    pub fn take_overflow(&mut self) -> bool {
        let value = self.overflow;
        self.overflow = false;
        value
    }

    /// Takes and clears the compare flag.
    pub fn take_compare(&mut self) -> bool {
        let value = self.compare_match;
        self.compare_match = false;
        value
    }

    /// Resets the counter, prescaler, and flags.
    pub fn reset(&mut self) -> Result<(), SimError> {
        self.clear_counter()?;
        self.prescaler = 0;
        self.overflow = false;
        self.compare_match = false;
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
        self.overflow |= previous == 0xFF;
        if current != self.compare {
            return Ok(());
        }
        self.compare_match = true;
        if self.ctc {
            self.clear_counter()?;
        }
        Ok(())
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

fn build_counter() -> (DigitalCircuit, Vec<NetId>) {
    let mut builder = DigitalBuilder::new();
    let enable = builder.input("enable");
    let reset = builder.input("reset");
    let bits: Vec<NetId> = (0..8).map(|_| builder.net()).collect();
    let incremented = increment(&mut builder, &bits);
    let zero = builder.constant(false);
    for index in 0..8 {
        let stepped = builder.mux(bits[index], incremented[index], enable);
        let next = builder.mux(stepped, zero, reset);
        builder.dff_into(next, bits[index]);
    }
    (builder.build(), bits)
}

#[cfg(test)]
mod tests {
    use super::Timer0;

    fn timer() -> Timer0 {
        Timer0::new().expect("timer")
    }

    #[test]
    fn prescaler_divides_clock() {
        let mut timer = timer();
        timer.set_prescaler(1).expect("prescaler");
        timer.tick(7).expect("tick");
        assert_eq!(timer.counter(), 0);
        timer.tick(1).expect("tick");
        assert_eq!(timer.counter(), 1);
        timer.tick(8).expect("tick");
        assert_eq!(timer.counter(), 2);
    }

    #[test]
    fn counter_wraps_and_sets_overflow() {
        let mut timer = timer();
        timer.tick(256).expect("tick");
        assert_eq!(timer.counter(), 0);
        assert!(timer.take_overflow());
    }

    #[test]
    fn compare_sets_flag_in_normal_mode() {
        let mut timer = timer();
        timer.set_compare(3);
        timer.tick(4).expect("tick");
        assert_eq!(timer.counter(), 4);
        assert!(timer.take_compare());
    }

    #[test]
    fn ctc_resets_counter_on_compare() {
        let mut timer = timer();
        timer.set_compare(3);
        timer.set_ctc(true);
        timer.tick(3).expect("tick");
        assert_eq!(timer.counter(), 0);
        assert!(timer.take_compare());
    }

    #[test]
    fn reset_clears_counter_and_flags() {
        let mut timer = timer();
        timer.set_compare(3);
        timer.tick(4).expect("tick");
        timer.reset().expect("reset");
        assert_eq!(timer.counter(), 0);
        assert!(!timer.take_compare());
    }

    #[test]
    fn invalid_prescaler_is_rejected() {
        let mut timer = timer();
        assert!(timer.set_prescaler(5).is_err());
    }
}
