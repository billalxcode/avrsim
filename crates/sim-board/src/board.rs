//! Small Arduino-like board model.

use super::components::{Button, Led, Resistor};
use std::collections::VecDeque;

/// Host UART endpoint.
#[derive(Default)]
pub struct UartHost {
    host_to_device: VecDeque<u8>,
    device_to_host: VecDeque<u8>,
}

impl UartHost {
    /// Creates an idle host endpoint.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sends one byte from host to device.
    pub fn host_send(&mut self, value: u8) {
        self.host_to_device.push_back(value);
    }

    /// Receives one byte at the device.
    pub fn device_receive(&mut self) -> Option<u8> {
        self.host_to_device.pop_front()
    }

    /// Sends one byte from device to host.
    pub fn device_send(&mut self, value: u8) {
        self.device_to_host.push_back(value);
    }

    /// Receives one byte at the host.
    pub fn host_receive(&mut self) -> Option<u8> {
        self.device_to_host.pop_front()
    }

    /// Returns pending host-to-device count.
    #[must_use]
    pub fn pending_to_device(&self) -> usize {
        self.host_to_device.len()
    }

    /// Returns pending device-to-host count.
    #[must_use]
    pub fn pending_to_host(&self) -> usize {
        self.device_to_host.len()
    }
}

/// Basic board with PB5 LED, resistor, button, and UART host.
pub struct Board {
    led: Led,
    resistor: Resistor,
    button: Button,
    uart: UartHost,
}

impl Board {
    /// Builds an Arduino-like board.
    #[must_use]
    pub fn new() -> Self {
        Self {
            led: Led::new(2.0),
            resistor: Resistor::new(220.0),
            button: Button::new(),
            uart: UartHost::new(),
        }
    }

    /// Drives PB5 high or low.
    pub fn set_pb5(&mut self, high: bool) {
        let voltage = if high { 5.0 } else { 0.0 };
        let across = (voltage - self.led.forward_voltage).max(0.0);
        self.led.set_current(self.resistor.current(across));
    }

    /// Returns the LED.
    #[must_use]
    pub const fn led(&self) -> &Led {
        &self.led
    }

    /// Returns the button.
    #[must_use]
    pub const fn button(&self) -> &Button {
        &self.button
    }

    /// Returns the button for mutation.
    pub fn button_mut(&mut self) -> &mut Button {
        &mut self.button
    }

    /// Returns the UART host.
    #[must_use]
    pub const fn uart(&self) -> &UartHost {
        &self.uart
    }

    /// Returns the UART host for mutation.
    pub fn uart_mut(&mut self) -> &mut UartHost {
        &mut self.uart
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{Board, UartHost};

    #[test]
    fn pb5_high_turns_led_on() {
        let mut board = Board::new();
        board.set_pb5(true);
        assert!(board.led().is_on());
        board.set_pb5(false);
        assert!(!board.led().is_on());
    }

    #[test]
    fn button_state_is_shared() {
        let mut board = Board::new();
        board.button_mut().press();
        assert!(board.button().is_pressed());
    }

    #[test]
    fn uart_host_bidirectional_queues() {
        let mut uart = UartHost::new();
        uart.host_send(0x41);
        uart.device_send(0x42);
        assert_eq!(uart.device_receive(), Some(0x41));
        assert_eq!(uart.host_receive(), Some(0x42));
    }

    #[test]
    fn blink_program_drives_board_led() {
        let program = vec![
            0xE200, 0xB904, 0xB905, 0xEF1F, 0x951A, 0xF7F1, 0xE220, 0x2702, 0xCFF9,
        ];
        let mut cpu = sim_avr::Cpu::new(program).expect("cpu");
        let mut board = Board::new();
        let mut low = false;
        let mut high = false;
        for _ in 0..2000 {
            cpu.step().expect("step");
            board.set_pb5(cpu.io(0x05) & 0x20 != 0);
            high |= board.led().is_on();
            low |= !board.led().is_on();
        }
        assert!(high && low);
    }
}
