//! Behavioral AVR serial peripheral interface.

const ENABLE: u8 = 1 << 6;
const MASTER: u8 = 1 << 4;
const CPOL: u8 = 1 << 3;
const CLOCK_SELECT_MASK: u8 = 0x03;
const DOUBLE_SPEED: u8 = 1 << 0;
const BITS_PER_TRANSFER: u8 = 8;
const MAX_CYCLES_PER_BIT: u32 = 64;

/// Behavioral model of the AVR SPI peripheral.
pub struct Spi {
    control: u8,
    status: u8,
    tx_data: u8,
    rx_data: u8,
    rx_shift: u8,
    received: u8,
    bits_done: u8,
    bit_elapsed: u32,
    cycles_per_bit: u32,
    active: bool,
    spif: bool,
}

impl Spi {
    /// Builds an SPI peripheral in its reset state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            control: 0,
            status: 0,
            tx_data: 0,
            rx_data: 0,
            rx_shift: 0,
            received: 0,
            bits_done: 0,
            bit_elapsed: 0,
            cycles_per_bit: 0,
            active: false,
            spif: false,
        }
    }

    /// Stores the SPCR control register.
    pub fn write_control(&mut self, value: u8) {
        self.control = value;
    }

    /// Stores the SPSR status register.
    pub fn write_status(&mut self, value: u8) {
        self.status = value;
    }

    /// Loads a byte and starts a transfer when the peripheral is enabled.
    pub fn write_data(&mut self, value: u8) {
        if !self.enabled() {
            return;
        }
        if self.active {
            return;
        }
        self.tx_data = value;
        self.start_transfer();
    }

    /// Returns the received byte and clears SPIF.
    pub fn read_data(&mut self) -> u8 {
        let value = self.rx_data;
        self.spif = false;
        value
    }

    /// Advances an active transfer by CPU cycles.
    pub fn tick(&mut self, cycles: u32) {
        if !self.active {
            return;
        }
        if cycles == 0 {
            return;
        }
        let mut remaining = cycles;
        while remaining > 0 && self.active {
            remaining = self.advance_bit(remaining);
        }
    }

    /// Sets the external input byte for the next completed transfer.
    pub fn set_received(&mut self, value: u8) {
        self.received = value;
    }

    /// Returns true while a transfer is in progress.
    #[must_use]
    pub fn busy(&self) -> bool {
        self.active
    }

    /// Returns true when a transfer has completed (SPIF).
    #[must_use]
    pub fn spif(&self) -> bool {
        self.spif
    }

    /// Takes and clears SPIF.
    pub fn take_spif(&mut self) -> bool {
        let value = self.spif;
        self.spif = false;
        value
    }

    /// Returns the SCK line level.
    #[must_use]
    pub fn sck(&self) -> bool {
        if !self.active {
            return self.cpol();
        }
        let half_bit = self.cycles_per_bit / 2;
        self.cpol() ^ (self.bit_elapsed >= half_bit)
    }

    /// Returns the MOSI line level.
    #[must_use]
    pub fn mosi(&self) -> bool {
        if self.master() {
            return bit_at(self.tx_data, self.bits_done);
        }
        bit_at(self.received, self.bits_done)
    }

    /// Returns the MISO line level.
    #[must_use]
    pub fn miso(&self) -> bool {
        if self.master() {
            return bit_at(self.received, self.bits_done);
        }
        bit_at(self.tx_data, self.bits_done)
    }

    fn start_transfer(&mut self) {
        self.rx_shift = 0;
        self.bits_done = 0;
        self.bit_elapsed = 0;
        self.spif = false;
        self.cycles_per_bit = select_cycles_per_bit(self.control, self.status);
        self.active = true;
    }

    fn advance_bit(&mut self, cycles: u32) -> u32 {
        let to_boundary = self.cycles_per_bit.saturating_sub(self.bit_elapsed);
        if cycles < to_boundary {
            self.bit_elapsed += cycles;
            return 0;
        }
        self.complete_bit();
        cycles - to_boundary
    }

    fn complete_bit(&mut self) {
        self.sample_input();
        self.bits_done += 1;
        self.bit_elapsed = 0;
        if self.bits_done < BITS_PER_TRANSFER {
            return;
        }
        self.finish_transfer();
    }

    fn sample_input(&mut self) {
        if bit_at(self.received, self.bits_done) {
            self.rx_shift |= 1 << self.bits_done;
        }
    }

    fn finish_transfer(&mut self) {
        self.active = false;
        self.rx_data = self.rx_shift;
        self.spif = true;
        self.bits_done = 0;
        self.bit_elapsed = 0;
        self.cycles_per_bit = 0;
    }

    fn enabled(&self) -> bool {
        self.control & ENABLE != 0
    }

    fn master(&self) -> bool {
        self.control & MASTER != 0
    }

    fn cpol(&self) -> bool {
        self.control & CPOL != 0
    }
}

impl Default for Spi {
    fn default() -> Self {
        Self::new()
    }
}

/// Maps the clock select bits and SPI2X to cycles per bit, capped at 64.
///
/// Clock select 0/1/2/3 uses 4/16/64/64 cycles; SPI2X halves each value.
fn select_cycles_per_bit(control: u8, status: u8) -> u32 {
    let base = match control & CLOCK_SELECT_MASK {
        0 => 4,
        1 => 16,
        _ => MAX_CYCLES_PER_BIT,
    };
    if status & DOUBLE_SPEED == 0 {
        return base;
    }
    base / 2
}

fn bit_at(value: u8, index: u8) -> bool {
    if index >= BITS_PER_TRANSFER {
        return false;
    }
    (value >> index) & 1 != 0
}

#[cfg(test)]
mod tests {
    use super::Spi;

    fn enabled_master() -> Spi {
        let mut spi = Spi::new();
        spi.write_control(0x50);
        spi
    }

    #[test]
    fn disabled_spi_ignores_write() {
        let mut spi = Spi::new();
        spi.write_data(0xA5);
        spi.tick(64);
        assert!(!spi.busy());
        assert!(!spi.spif());
        assert_eq!(spi.read_data(), 0);
    }

    #[test]
    fn enabled_master_starts_busy() {
        let mut spi = enabled_master();
        spi.write_data(0x81);
        assert!(spi.busy());
        assert!(!spi.spif());
    }

    #[test]
    fn transfer_completes_after_eight_bit_times() {
        let mut spi = enabled_master();
        spi.set_received(0x3C);
        spi.write_data(0x55);
        spi.tick(31);
        assert!(spi.busy());
        assert!(!spi.spif());
        spi.tick(1);
        assert!(!spi.busy());
        assert!(spi.spif());
    }

    #[test]
    fn clock_select_bits_change_duration() {
        let mut spi = enabled_master();
        spi.write_control(0x51);
        spi.write_data(0x00);
        spi.tick(127);
        assert!(spi.busy());
        spi.tick(1);
        assert!(!spi.busy());
    }

    #[test]
    fn read_data_returns_received_byte_and_clears_spif() {
        let mut spi = enabled_master();
        spi.set_received(0xC3);
        spi.write_data(0x00);
        spi.tick(32);
        assert_eq!(spi.read_data(), 0xC3);
        assert!(!spi.spif());
        assert_eq!(spi.read_data(), 0xC3);
    }

    #[test]
    fn spi2x_halves_transfer_time() {
        let mut spi = enabled_master();
        spi.write_status(0x01);
        spi.write_data(0x00);
        spi.tick(15);
        assert!(spi.busy());
        spi.tick(1);
        assert!(!spi.busy());
        assert!(spi.spif());
    }

    #[test]
    fn sck_idle_level_follows_cpol() {
        let mut spi = Spi::new();
        assert!(!spi.sck());
        spi.write_control(0x48);
        assert!(spi.sck());
    }

    #[test]
    fn sck_toggles_during_transfer() {
        let mut spi = enabled_master();
        spi.write_data(0x00);
        assert!(!spi.sck());
        spi.tick(2);
        assert!(spi.sck());
        spi.tick(2);
        assert!(!spi.sck());
    }

    #[test]
    fn take_spif_reports_and_clears_flag() {
        let mut spi = enabled_master();
        spi.write_data(0x00);
        spi.tick(32);
        assert!(spi.take_spif());
        assert!(!spi.spif());
        assert!(!spi.take_spif());
    }

    #[test]
    fn mosi_sends_least_significant_bit_first() {
        let mut spi = enabled_master();
        spi.write_data(0x01);
        assert!(spi.mosi());
        spi.tick(4);
        assert!(!spi.mosi());
    }

    #[test]
    fn miso_reads_external_bit_in_master_mode() {
        let mut spi = enabled_master();
        spi.set_received(0x01);
        spi.write_data(0x00);
        assert!(spi.miso());
        spi.tick(4);
        assert!(!spi.miso());
    }

    #[test]
    fn source_is_ascii() {
        assert!(include_str!("spi.rs").is_ascii());
    }
}
