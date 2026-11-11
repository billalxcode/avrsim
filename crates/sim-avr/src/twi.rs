//! Behavioral AVR TWI/I2C master model.

const TWEN: u8 = 1 << 2;
const PRESCALER_MASK: u8 = 0x03;

const STATUS_IDLE: u8 = 0xF8;
const STATUS_START: u8 = 0x08;
const STATUS_MT_SLA_ACK: u8 = 0x18;
const STATUS_MT_DATA_ACK: u8 = 0x28;
const STATUS_MR_DATA_NACK: u8 = 0x58;

const PRESCALER_FACTORS: [u32; 4] = [1, 4, 16, 64];

/// Behavioral model of the AVR TWI peripheral in master mode.
pub struct Twi {
    control: u8,
    prescaler: u8,
    bitrate: u8,
    data: u8,
    received: u8,
    status: u8,
    twint: bool,
    busy: bool,
    pending: bool,
    elapsed: u32,
    duration: u32,
}

impl Twi {
    /// Builds a disabled TWI in its idle state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            control: 0,
            prescaler: 0,
            bitrate: 0,
            data: 0,
            received: 0,
            status: STATUS_IDLE,
            twint: false,
            busy: false,
            pending: false,
            elapsed: 0,
            duration: 0,
        }
    }

    /// Stores TWCR: TWEN bit 2, TWSTA bit 5, TWSTO bit 4, TWINT bit 7.
    pub fn write_control(&mut self, value: u8) {
        self.control = value;
    }

    /// Stores the TWSR prescaler bits, TWPS1 and TWPS0.
    pub fn write_status(&mut self, value: u8) {
        self.prescaler = value & PRESCALER_MASK;
    }

    /// Stores the TWBR bit rate divider.
    pub fn write_bitrate(&mut self, value: u8) {
        self.bitrate = value;
    }

    /// Stores the TWDR data byte.
    pub fn write_data(&mut self, value: u8) {
        self.data = value;
    }

    /// Returns the TWDR data byte.
    #[must_use]
    pub fn read_data(&self) -> u8 {
        self.data
    }

    /// Generates a start condition when TWI is enabled.
    ///
    /// The start condition completes after one bit period of `tick`.
    pub fn start(&mut self) {
        if self.control & TWEN == 0 {
            return;
        }
        self.busy = true;
        self.status = STATUS_START;
        self.twint = false;
        self.begin_transaction();
    }

    /// Generates a stop condition and returns the bus to idle.
    pub fn stop(&mut self) {
        self.busy = false;
        self.pending = false;
        self.twint = false;
        self.elapsed = 0;
        self.status = STATUS_IDLE;
    }

    /// Performs a master write transaction with a callback-free slave.
    ///
    /// The slave address selects a device on real hardware. This model only
    /// tracks the resulting status and data phases.
    pub fn write_byte(&mut self, _address: u8, value: u8) {
        self.data = value;
        // The address phase is acknowledged before the data phase.
        self.status = STATUS_MT_SLA_ACK;
        self.status = STATUS_MT_DATA_ACK;
        self.twint = false;
        self.busy = true;
        self.begin_transaction();
    }

    /// Performs a master read transaction and returns the slave byte.
    ///
    /// The slave address selects a device on real hardware. This model only
    /// returns the byte supplied through `set_received`.
    pub fn read_byte(&mut self, _address: u8) -> u8 {
        let value = self.received;
        self.data = value;
        self.status = STATUS_MR_DATA_NACK;
        self.twint = false;
        self.busy = true;
        self.begin_transaction();
        value
    }

    /// Records a byte supplied by the external slave model.
    pub fn set_received(&mut self, value: u8) {
        self.received = value;
    }

    /// Advances the transaction timer and sets TWINT when it completes.
    pub fn tick(&mut self, cycles: u32) {
        if cycles == 0 {
            return;
        }
        if !self.pending {
            return;
        }
        self.elapsed = self.elapsed.saturating_add(cycles);
        if self.elapsed < self.duration {
            return;
        }
        self.finish_transaction();
    }

    /// Returns true when the TWI interrupt flag is set.
    #[must_use]
    pub fn twint(&self) -> bool {
        self.twint
    }

    /// Takes and clears the TWI interrupt flag.
    pub fn take_twint(&mut self) -> bool {
        let value = self.twint;
        self.twint = false;
        value
    }

    /// Returns TWSR: status in bits 7..3 and prescaler in bits 1..0.
    #[must_use]
    pub fn status(&self) -> u8 {
        self.status | self.prescaler
    }

    /// Returns true while a start or transaction keeps the bus active.
    #[must_use]
    pub fn busy(&self) -> bool {
        self.busy
    }

    fn begin_transaction(&mut self) {
        self.elapsed = 0;
        self.duration = self.transaction_cycles();
        self.pending = true;
    }

    fn finish_transaction(&mut self) {
        self.pending = false;
        self.elapsed = 0;
        self.twint = true;
    }

    fn transaction_cycles(&self) -> u32 {
        let factor = PRESCALER_FACTORS[usize::from(self.prescaler)];
        (u32::from(self.bitrate) + 1).saturating_mul(factor)
    }
}

impl Default for Twi {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PRESCALER_MASK, STATUS_IDLE, STATUS_MR_DATA_NACK, STATUS_MT_DATA_ACK, STATUS_START, TWEN,
        Twi,
    };

    fn enabled_twi() -> Twi {
        let mut twi = Twi::new();
        twi.write_control(TWEN);
        twi
    }

    #[test]
    fn disabled_start_is_ignored() {
        let mut twi = Twi::new();
        twi.start();
        twi.tick(1);
        assert!(!twi.busy());
        assert!(!twi.twint());
        assert_eq!(twi.status(), STATUS_IDLE);
    }

    #[test]
    fn start_sets_status_and_twint() {
        let mut twi = enabled_twi();
        twi.start();
        assert!(twi.busy());
        assert_eq!(twi.status(), STATUS_START);
        assert!(!twi.twint());
        twi.tick(1);
        assert!(twi.twint());
    }

    #[test]
    fn restart_clears_twint() {
        let mut twi = enabled_twi();
        twi.start();
        twi.tick(1);
        assert!(twi.twint());
        twi.start();
        assert!(!twi.twint());
        assert_eq!(twi.status(), STATUS_START);
    }

    #[test]
    fn write_byte_sets_data_status_and_twint() {
        let mut twi = enabled_twi();
        twi.write_byte(0x50, 0xA5);
        assert_eq!(twi.read_data(), 0xA5);
        assert_eq!(twi.status(), STATUS_MT_DATA_ACK);
        assert!(twi.busy());
        assert!(!twi.twint());
        twi.tick(1);
        assert!(twi.twint());
    }

    #[test]
    fn read_byte_returns_received_value() {
        let mut twi = enabled_twi();
        twi.set_received(0x3C);
        assert_eq!(twi.read_byte(0x50), 0x3C);
        assert_eq!(twi.read_data(), 0x3C);
        assert_eq!(twi.status(), STATUS_MR_DATA_NACK);
        twi.tick(1);
        assert!(twi.twint());
    }

    #[test]
    fn stop_clears_busy_and_twint() {
        let mut twi = enabled_twi();
        twi.start();
        twi.tick(1);
        assert!(twi.twint());
        twi.stop();
        assert!(!twi.busy());
        assert!(!twi.twint());
        assert_eq!(twi.status(), STATUS_IDLE);
        twi.tick(1);
        assert!(!twi.twint());
    }

    #[test]
    fn prescaler_and_bitrate_are_stored() {
        let mut twi = Twi::new();
        twi.write_status(0x02);
        twi.write_bitrate(3);
        assert_eq!(twi.status() & PRESCALER_MASK, 0x02);
        assert_eq!(twi.bitrate, 3);
    }

    #[test]
    fn prescaler_and_bitrate_scale_transaction_time() {
        let mut twi = enabled_twi();
        twi.write_status(0x02);
        twi.write_bitrate(1);
        twi.start();
        twi.tick(31);
        assert!(!twi.twint());
        twi.tick(1);
        assert!(twi.twint());
    }

    #[test]
    fn take_twint_clears_flag() {
        let mut twi = enabled_twi();
        twi.start();
        twi.tick(1);
        assert!(twi.take_twint());
        assert!(!twi.twint());
        assert!(!twi.take_twint());
    }

    #[test]
    fn data_register_round_trips() {
        let mut twi = Twi::new();
        twi.write_data(0x5A);
        assert_eq!(twi.read_data(), 0x5A);
    }

    #[test]
    fn default_matches_new() {
        let twi = Twi::default();
        assert_eq!(twi.status(), STATUS_IDLE);
        assert!(!twi.busy());
        assert!(!twi.twint());
    }

    #[test]
    fn source_is_ascii() {
        assert!(include_str!("twi.rs").is_ascii());
    }
}
