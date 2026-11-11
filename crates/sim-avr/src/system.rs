//! Behavioral system peripherals.

/// 1 KB EEPROM model.
pub struct Eeprom {
    data: Vec<u8>,
}

impl Eeprom {
    /// Creates an erased EEPROM.
    #[must_use]
    pub fn new() -> Self {
        Self {
            data: vec![0xFF; 1024],
        }
    }

    /// Returns the EEPROM size.
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true when the EEPROM has no bytes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Reads one byte.
    #[must_use]
    pub fn read(&self, address: u16) -> u8 {
        self.data[usize::from(address) % self.data.len()]
    }

    /// Writes one byte.
    pub fn write(&mut self, address: u16, value: u8) {
        let index = usize::from(address) % self.data.len();
        self.data[index] = value;
    }

    /// Erases all bytes to 0xFF.
    pub fn erase(&mut self) {
        self.data.fill(0xFF);
    }
}

impl Default for Eeprom {
    fn default() -> Self {
        Self::new()
    }
}

/// Watchdog timeout model.
pub struct Watchdog {
    enabled: bool,
    timeout: u32,
    counter: u32,
}

impl Watchdog {
    /// Creates a disabled watchdog.
    #[must_use]
    pub fn new() -> Self {
        Self {
            enabled: false,
            timeout: 2048,
            counter: 0,
        }
    }

    /// Enables the watchdog with a timeout in cycles.
    pub fn enable(&mut self, timeout: u32) {
        self.enabled = true;
        self.timeout = timeout.max(1);
        self.counter = 0;
    }

    /// Disables the watchdog.
    pub fn disable(&mut self) {
        self.enabled = false;
        self.counter = 0;
    }

    /// Feeds the watchdog.
    pub fn kick(&mut self) {
        self.counter = 0;
    }

    /// Advances the watchdog. Returns true on timeout.
    pub fn tick(&mut self, cycles: u32) -> bool {
        if !self.enabled {
            return false;
        }
        self.counter = self.counter.saturating_add(cycles);
        if self.counter < self.timeout {
            return false;
        }
        self.counter = 0;
        true
    }
}

impl Default for Watchdog {
    fn default() -> Self {
        Self::new()
    }
}

/// AVR fuse bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fuses {
    /// Low fuse byte.
    pub low: u8,
    /// High fuse byte.
    pub high: u8,
    /// Extended fuse byte.
    pub extended: u8,
}

impl Fuses {
    /// Arduino Uno profile.
    #[must_use]
    pub const fn uno_profile() -> Self {
        Self {
            low: 0xFF,
            high: 0xDE,
            extended: 0xFD,
        }
    }
}

/// Clock prescaler model.
pub struct ClockPrescaler {
    divide8: bool,
}

impl ClockPrescaler {
    /// Creates a prescaler with CKDIV8 disabled.
    #[must_use]
    pub const fn new() -> Self {
        Self { divide8: false }
    }

    /// Enables or disables CKDIV8.
    pub fn set_divide8(&mut self, enabled: bool) {
        self.divide8 = enabled;
    }

    /// Returns the clock divider.
    #[must_use]
    pub const fn divider(&self) -> u16 {
        if self.divide8 { 8 } else { 1 }
    }

    /// Divides CPU cycles by the prescaler.
    #[must_use]
    pub fn apply(&self, cycles: u32) -> u32 {
        cycles / u32::from(self.divider())
    }
}

impl Default for ClockPrescaler {
    fn default() -> Self {
        Self::new()
    }
}

/// Analog comparator model.
pub struct AnalogComparator {
    enabled: bool,
    positive: u16,
    negative: u16,
}

impl AnalogComparator {
    /// Creates a disabled comparator.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            enabled: false,
            positive: 0,
            negative: 0,
        }
    }

    /// Enables or disables the comparator.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Sets both input voltages.
    pub fn set_inputs(&mut self, positive: u16, negative: u16) {
        self.positive = positive;
        self.negative = negative;
    }

    /// Returns the comparator output.
    #[must_use]
    pub const fn output(&self) -> bool {
        self.enabled && self.positive > self.negative
    }
}

impl Default for AnalogComparator {
    fn default() -> Self {
        Self::new()
    }
}

/// Power reset and brown-out state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PowerState {
    /// Power-on reset active.
    pub por: bool,
    /// Brown-out reset active.
    pub brown_out: bool,
}

impl PowerState {
    /// Creates an inactive power state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            por: false,
            brown_out: false,
        }
    }

    /// Raises power-on reset.
    pub fn trigger_por(&mut self) {
        self.por = true;
    }

    /// Raises brown-out reset.
    pub fn trigger_brown_out(&mut self) {
        self.brown_out = true;
    }

    /// Clears both reset sources.
    pub fn clear(&mut self) {
        self.por = false;
        self.brown_out = false;
    }

    /// Returns true when reset is required.
    #[must_use]
    pub const fn reset_required(&self) -> bool {
        self.por || self.brown_out
    }
}

#[cfg(test)]
mod tests {
    use super::{AnalogComparator, ClockPrescaler, Eeprom, Fuses, PowerState, Watchdog};

    #[test]
    fn eeprom_reads_writes_and_erases() {
        let mut eeprom = Eeprom::new();
        assert_eq!(eeprom.len(), 1024);
        eeprom.write(0x0100, 0x5A);
        assert_eq!(eeprom.read(0x0100), 0x5A);
        eeprom.erase();
        assert_eq!(eeprom.read(0x0100), 0xFF);
    }

    #[test]
    fn watchdog_times_out_and_kick_resets() {
        let mut watchdog = Watchdog::new();
        assert!(!watchdog.tick(10));
        watchdog.enable(10);
        assert!(!watchdog.tick(9));
        assert!(watchdog.tick(1));
        watchdog.kick();
        assert!(!watchdog.tick(9));
    }

    #[test]
    fn watchdog_disable_stops_timeout() {
        let mut watchdog = Watchdog::new();
        watchdog.enable(5);
        watchdog.disable();
        assert!(!watchdog.tick(100));
    }

    #[test]
    fn uno_fuse_profile_matches_expected_bytes() {
        let fuses = Fuses::uno_profile();
        assert_eq!((fuses.low, fuses.high, fuses.extended), (0xFF, 0xDE, 0xFD));
    }

    #[test]
    fn clock_prescaler_divides_cycles() {
        let mut prescaler = ClockPrescaler::new();
        assert_eq!(prescaler.divider(), 1);
        assert_eq!(prescaler.apply(80), 80);
        prescaler.set_divide8(true);
        assert_eq!(prescaler.divider(), 8);
        assert_eq!(prescaler.apply(80), 10);
    }

    #[test]
    fn comparator_requires_enable_and_positive_input() {
        let mut comparator = AnalogComparator::new();
        comparator.set_inputs(500, 100);
        assert!(!comparator.output());
        comparator.set_enabled(true);
        assert!(comparator.output());
        comparator.set_inputs(100, 500);
        assert!(!comparator.output());
    }

    #[test]
    fn power_state_tracks_reset_sources() {
        let mut power = PowerState::new();
        assert!(!power.reset_required());
        power.trigger_por();
        assert!(power.reset_required());
        power.clear();
        power.trigger_brown_out();
        assert!(power.reset_required());
    }
}
