//! Timer2 wrapper around the 8-bit timer core.

use super::timer0::Timer0;
use sim_core::SimError;

/// Timer2 with an async mode flag.
pub struct Timer2 {
    timer: Timer0,
    async_mode: bool,
}

impl Timer2 {
    /// Builds Timer2.
    pub fn new() -> Result<Self, SimError> {
        Ok(Self {
            timer: Timer0::new()?,
            async_mode: false,
        })
    }

    /// Sets the prescaler select value.
    pub fn set_prescaler(&mut self, select: u8) -> Result<(), SimError> {
        self.timer.set_prescaler(select)
    }

    /// Sets compare A.
    pub fn set_compare(&mut self, value: u8) {
        self.timer.set_compare(value);
    }

    /// Enables or disables CTC mode.
    pub fn set_ctc(&mut self, enabled: bool) {
        self.timer.set_ctc(enabled);
    }

    /// Enables fast PWM.
    pub fn set_pwm(&mut self, inverting: bool) {
        self.timer.set_pwm(inverting);
    }

    /// Enables asynchronous clock mode.
    pub fn set_async(&mut self, enabled: bool) {
        self.async_mode = enabled;
    }

    /// Returns true when asynchronous mode is enabled.
    #[must_use]
    pub const fn is_async(&self) -> bool {
        self.async_mode
    }

    /// Advances Timer2.
    pub fn tick(&mut self, cycles: u32) -> Result<(), SimError> {
        self.timer.tick(cycles)
    }

    /// Returns the counter value.
    #[must_use]
    pub fn counter(&self) -> u8 {
        self.timer.counter()
    }

    /// Takes and clears overflow.
    pub fn take_overflow(&mut self) -> bool {
        self.timer.take_overflow()
    }

    /// Takes and clears compare.
    pub fn take_compare(&mut self) -> bool {
        self.timer.take_compare()
    }

    /// Returns overflow without clearing it.
    #[must_use]
    pub const fn overflow_flag(&self) -> bool {
        self.timer.overflow_flag()
    }

    /// Returns compare without clearing it.
    #[must_use]
    pub const fn compare_flag(&self) -> bool {
        self.timer.compare_flag()
    }

    /// Returns the PWM output.
    #[must_use]
    pub const fn pwm_output(&self) -> bool {
        self.timer.pwm_output()
    }

    /// Resets Timer2.
    pub fn reset(&mut self) -> Result<(), SimError> {
        self.timer.reset()
    }
}

#[cfg(test)]
mod tests {
    use super::Timer2;

    #[test]
    fn timer2_counts_and_wraps() {
        let mut timer = Timer2::new().expect("timer");
        timer.tick(255).expect("tick");
        assert_eq!(timer.counter(), 255);
        timer.tick(1).expect("tick");
        assert_eq!(timer.counter(), 0);
        assert!(timer.take_overflow());
    }

    #[test]
    fn timer2_async_flag_is_stored() {
        let mut timer = Timer2::new().expect("timer");
        assert!(!timer.is_async());
        timer.set_async(true);
        assert!(timer.is_async());
    }

    #[test]
    fn timer2_pwm_tracks_compare() {
        let mut timer = Timer2::new().expect("timer");
        timer.set_compare(2);
        timer.set_pwm(false);
        timer.tick(1).expect("tick");
        assert!(timer.pwm_output());
        timer.tick(1).expect("tick");
        assert!(!timer.pwm_output());
    }
}
