//! Behavioral AVR USART transmitter and receiver model.

const TX_ENABLE: u8 = 1 << 3;
const FRAME_BITS: u32 = 10;
const NORMAL_MODE_FACTOR: u32 = 16;

/// Behavioral model of the AVR USART.
pub struct Uart {
    ucsr_a: u8,
    ucsr_b: u8,
    ucsr_c: u8,
    ubrr: u16,
    tx_data: u8,
    tx_bit_cycles: u32,
    tx_elapsed: u32,
    tx_active: bool,
    udre: bool,
    txc: bool,
    rx_data: u8,
    rxc: bool,
    rx_active: bool,
    rx_shift: u8,
    rx_bits: u8,
}

impl Uart {
    /// Builds a USART in its reset state with an empty transmitter.
    #[must_use]
    pub fn new() -> Self {
        Self {
            ucsr_a: 0,
            ucsr_b: 0,
            ucsr_c: 0,
            ubrr: 0,
            tx_data: 0,
            tx_bit_cycles: 0,
            tx_elapsed: 0,
            tx_active: false,
            udre: true,
            txc: false,
            rx_data: 0,
            rxc: false,
            rx_active: false,
            rx_shift: 0,
            rx_bits: 0,
        }
    }

    /// Writes a control register: 0 = UCSR0A, 1 = UCSR0B, 2 = UCSR0C.
    pub fn write_control(&mut self, register: u8, value: u8) {
        match register {
            0 => self.ucsr_a = value,
            1 => self.ucsr_b = value,
            2 => self.ucsr_c = value,
            _ => {}
        }
    }

    /// Stores the 12-bit UBRR baud divider.
    pub fn write_baud(&mut self, low: u8, high: u8) {
        self.ubrr = u16::from(low) | ((u16::from(high) & 0x0F) << 8);
    }

    /// Loads one byte when the transmitter is enabled and idle.
    pub fn write_tx(&mut self, value: u8) {
        if self.ucsr_b & TX_ENABLE == 0 {
            return;
        }
        if self.tx_active {
            return;
        }
        self.tx_data = value;
        self.udre = false;
        self.txc = false;
        self.tx_elapsed = 0;
        self.tx_bit_cycles = self.cycles_per_bit();
        self.tx_active = true;
    }

    /// Returns the received byte and clears RXC.
    pub fn read_rx(&mut self) -> u8 {
        let value = self.rx_data;
        self.rxc = false;
        value
    }

    /// Records an externally received byte and sets RXC.
    pub fn receive(&mut self, value: u8) {
        self.rx_data = value;
        self.rxc = true;
    }

    /// Shifts one received serial bit into the receiver.
    pub fn receive_bit(&mut self, level: bool) {
        if !self.rx_active {
            self.start_rx(level);
            return;
        }
        if self.rx_bits >= 8 {
            return;
        }
        self.shift_rx(level);
    }

    fn start_rx(&mut self, level: bool) {
        if level {
            return;
        }
        self.rx_active = true;
        self.rx_bits = 0;
        self.rx_shift = 0;
    }

    fn shift_rx(&mut self, level: bool) {
        if level {
            self.rx_shift |= 1 << self.rx_bits;
        }
        self.rx_bits += 1;
        if self.rx_bits == 8 {
            self.rx_data = self.rx_shift;
            self.rxc = true;
            self.rx_active = false;
        }
    }

    /// Advances the transmitter shift timing by CPU cycles.
    pub fn tick(&mut self, cycles: u32) {
        if cycles == 0 {
            return;
        }
        self.advance_tx(cycles);
    }

    /// Returns true when the TX data register is empty (UDRE).
    #[must_use]
    pub fn tx_ready(&self) -> bool {
        self.udre
    }

    /// Returns true when the transmitter is complete (TXC).
    #[must_use]
    pub fn tx_complete(&self) -> bool {
        self.txc
    }

    /// Returns true when a received byte is available (RXC).
    #[must_use]
    pub fn rx_ready(&self) -> bool {
        self.rxc
    }

    /// Takes and clears TXC.
    pub fn take_tx_complete(&mut self) -> bool {
        let value = self.txc;
        self.txc = false;
        value
    }

    /// Takes and clears RXC.
    pub fn take_rx_ready(&mut self) -> bool {
        let value = self.rxc;
        self.rxc = false;
        value
    }

    /// Returns the TX line level; the idle level is high.
    #[must_use]
    pub fn tx_line(&self) -> bool {
        if !self.tx_active {
            return true;
        }
        let position = self.tx_elapsed / self.tx_bit_cycles;
        frame_level(position, self.tx_data)
    }

    /// Returns true while a frame is being shifted out.
    #[must_use]
    pub fn tx_activity(&self) -> bool {
        self.tx_active
    }

    fn advance_tx(&mut self, cycles: u32) {
        if !self.tx_active {
            return;
        }
        let elapsed = self.tx_elapsed.saturating_add(cycles);
        let frame_cycles = self.tx_bit_cycles.saturating_mul(FRAME_BITS);
        if elapsed < frame_cycles {
            self.tx_elapsed = elapsed;
            return;
        }
        self.finish_tx();
    }

    fn finish_tx(&mut self) {
        self.tx_active = false;
        self.tx_elapsed = 0;
        self.tx_bit_cycles = 0;
        self.udre = true;
        self.txc = true;
    }

    fn cycles_per_bit(&self) -> u32 {
        (u32::from(self.ubrr) + 1) * NORMAL_MODE_FACTOR
    }
}

impl Default for Uart {
    fn default() -> Self {
        Self::new()
    }
}

fn frame_level(position: u32, data: u8) -> bool {
    if position == 0 {
        return false;
    }
    if position >= 9 {
        return true;
    }
    (data >> (position - 1)) & 1 != 0
}

#[cfg(test)]
mod tests {
    use super::{FRAME_BITS, NORMAL_MODE_FACTOR, Uart};

    fn enabled_uart() -> Uart {
        let mut uart = Uart::new();
        uart.write_control(1, 0x18);
        uart
    }

    #[test]
    fn disabled_transmitter_ignores_write() {
        let mut uart = Uart::new();
        uart.write_tx(0x55);
        assert!(uart.tx_ready());
        assert!(!uart.tx_activity());
        uart.tick(NORMAL_MODE_FACTOR * FRAME_BITS);
        assert!(uart.tx_line());
        assert!(!uart.tx_complete());
    }

    #[test]
    fn control_register_b_enables_transmitter() {
        let mut uart = Uart::new();
        uart.write_control(1, 0x08);
        uart.write_tx(0x55);
        assert!(uart.tx_activity());
        assert!(!uart.tx_ready());
    }

    #[test]
    fn control_register_c_does_not_enable_transmitter() {
        let mut uart = Uart::new();
        uart.write_control(2, 0x08);
        uart.write_tx(0x55);
        assert!(!uart.tx_activity());
    }

    #[test]
    fn transmitter_takes_ten_bit_times() {
        let mut uart = enabled_uart();
        uart.write_tx(0xA5);
        assert!(!uart.tx_ready());
        uart.tick(NORMAL_MODE_FACTOR * FRAME_BITS - 1);
        assert!(uart.tx_activity());
        assert!(!uart.tx_complete());
        uart.tick(1);
        assert!(!uart.tx_activity());
        assert!(uart.tx_complete());
        assert!(uart.tx_ready());
    }

    #[test]
    fn tx_complete_flag_can_be_taken() {
        let mut uart = enabled_uart();
        uart.write_tx(0x00);
        uart.tick(NORMAL_MODE_FACTOR * FRAME_BITS);
        assert!(uart.take_tx_complete());
        assert!(!uart.tx_complete());
        assert!(!uart.take_tx_complete());
    }

    #[test]
    fn baud_divider_changes_frame_duration() {
        let mut uart = enabled_uart();
        uart.write_baud(1, 0);
        uart.write_tx(0x00);
        uart.tick(NORMAL_MODE_FACTOR * FRAME_BITS);
        assert!(uart.tx_activity());
        uart.tick(NORMAL_MODE_FACTOR * FRAME_BITS);
        assert!(uart.tx_complete());
    }

    #[test]
    fn baud_high_register_contributes_high_bits() {
        let mut uart = enabled_uart();
        uart.write_baud(0, 1);
        uart.write_tx(0x00);
        uart.tick(NORMAL_MODE_FACTOR * FRAME_BITS);
        assert!(uart.tx_activity());
    }

    #[test]
    fn tx_line_follows_frame_bits() {
        let mut uart = enabled_uart();
        assert!(uart.tx_line());
        uart.write_tx(0x01);
        assert!(!uart.tx_line());
        uart.tick(NORMAL_MODE_FACTOR);
        assert!(uart.tx_line());
        uart.tick(NORMAL_MODE_FACTOR);
        assert!(!uart.tx_line());
        uart.tick(NORMAL_MODE_FACTOR * 7);
        assert!(uart.tx_line());
        assert!(uart.tx_activity());
        uart.tick(NORMAL_MODE_FACTOR);
        assert!(!uart.tx_activity());
        assert!(uart.tx_line());
    }

    #[test]
    fn receive_sets_rxc_and_read_clears_it() {
        let mut uart = Uart::new();
        assert!(!uart.rx_ready());
        uart.receive(0x5A);
        assert!(uart.rx_ready());
        assert_eq!(uart.read_rx(), 0x5A);
        assert!(!uart.rx_ready());
    }

    #[test]
    fn take_rx_ready_clears_rxc_and_keeps_data() {
        let mut uart = Uart::new();
        uart.receive(0x33);
        assert!(uart.take_rx_ready());
        assert!(!uart.rx_ready());
        assert_eq!(uart.read_rx(), 0x33);
    }

    #[test]
    fn rx_shift_collects_eight_data_bits() {
        let mut uart = Uart::new();
        uart.receive_bit(false);
        for bit in 0..8 {
            uart.receive_bit((0xA5 >> bit) & 1 != 0);
        }
        assert!(uart.rx_ready());
        assert_eq!(uart.read_rx(), 0xA5);
    }

    #[test]
    fn source_is_ascii() {
        assert!(include_str!("uart.rs").is_ascii());
    }
}
