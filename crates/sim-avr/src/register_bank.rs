//! AVR register bank.

use sim_cells::{
    DigitalBuilder, DigitalCircuit, NetId, RegisterFile, RegisterPorts, read_bits, register_file,
};
use sim_core::SimError;

/// AVR register bank with two read ports and one write port.
pub struct RegisterBank {
    file: RegisterFile,
}

impl RegisterBank {
    /// Builds the 32 x 8 register bank.
    pub fn new(builder: &mut DigitalBuilder, ports: &RegisterPorts) -> Result<Self, SimError> {
        let file = register_file(builder, 32, 8, ports)?;
        Ok(Self { file })
    }

    /// Returns read port A nets.
    #[must_use]
    pub fn read_a(&self) -> &[NetId] {
        self.file.read_a()
    }

    /// Returns read port B nets.
    #[must_use]
    pub fn read_b(&self) -> &[NetId] {
        self.file.read_b()
    }

    /// Returns storage bits indexed by register.
    #[must_use]
    pub fn bits(&self) -> &[Vec<NetId>] {
        self.file.bits()
    }

    /// Reads one register.
    #[must_use]
    pub fn read_byte(&self, circuit: &DigitalCircuit, index: usize) -> Option<u8> {
        let bits = self.file.bits().get(index)?;
        Some(read_bits(circuit, bits) as u8)
    }

    /// Reads a little-endian register pair.
    #[must_use]
    pub fn read_word(&self, circuit: &DigitalCircuit, low: usize) -> Option<u16> {
        let low_value = u16::from(self.read_byte(circuit, low)?);
        let high_value = u16::from(self.read_byte(circuit, low + 1)?);
        Some(low_value | (high_value << 8))
    }
}

#[cfg(test)]
mod tests {
    use super::RegisterBank;
    use crate::test_support::{named_inputs, set_bits};
    use sim_cells::{DigitalBuilder, DigitalCircuit, RegisterPorts};

    fn fixture() -> (DigitalCircuit, RegisterBank) {
        let mut builder = DigitalBuilder::new();
        let ports = RegisterPorts {
            write_address: named_inputs(&mut builder, "wa", 5),
            write_data: named_inputs(&mut builder, "wd", 8),
            write_enable: builder.input("we"),
            read_address_a: named_inputs(&mut builder, "ra", 5),
            read_address_b: named_inputs(&mut builder, "rb", 5),
        };
        let bank = RegisterBank::new(&mut builder, &ports).expect("bank");
        (builder.build(), bank)
    }

    fn write_register(circuit: &mut DigitalCircuit, address: usize, value: u64) {
        set_bits(circuit, "wa", address as u64, 5);
        set_bits(circuit, "wd", value, 8);
        circuit.set_input("we", true).expect("we");
        circuit.tick().expect("tick");
    }

    fn read_port(circuit: &mut DigitalCircuit, prefix: &str, address: usize) {
        set_bits(circuit, prefix, address as u64, 5);
        circuit.settle().expect("settle");
    }

    #[test]
    fn writes_and_reads_through_ports() {
        let (mut circuit, bank) = fixture();
        write_register(&mut circuit, 0, 0x12);
        write_register(&mut circuit, 31, 0x34);
        read_port(&mut circuit, "ra", 0);
        assert_eq!(bank.read_byte(&circuit, 0), Some(0x12));
        read_port(&mut circuit, "rb", 31);
        assert_eq!(bank.read_byte(&circuit, 31), Some(0x34));
    }

    #[test]
    fn reads_little_endian_pairs() {
        let (mut circuit, bank) = fixture();
        write_register(&mut circuit, 26, 0x34);
        write_register(&mut circuit, 27, 0x12);
        assert_eq!(bank.read_word(&circuit, 26), Some(0x1234));
        write_register(&mut circuit, 30, 0x78);
        write_register(&mut circuit, 31, 0x56);
        assert_eq!(bank.read_word(&circuit, 30), Some(0x5678));
    }

    #[test]
    fn write_enable_low_holds_value() {
        let (mut circuit, bank) = fixture();
        write_register(&mut circuit, 5, 0x55);
        set_bits(&mut circuit, "wa", 5, 5);
        set_bits(&mut circuit, "wd", 0xAA, 8);
        circuit.set_input("we", false).expect("we");
        circuit.tick().expect("tick");
        assert_eq!(bank.read_byte(&circuit, 5), Some(0x55));
    }

    #[test]
    fn reset_clears_register_bank() {
        let (mut circuit, bank) = fixture();
        write_register(&mut circuit, 7, 0x77);
        circuit.reset().expect("reset");
        assert_eq!(bank.read_byte(&circuit, 7), Some(0));
    }

    #[test]
    fn invalid_register_index_returns_none() {
        let (circuit, bank) = fixture();
        assert_eq!(bank.read_byte(&circuit, 32), None);
        assert_eq!(bank.read_word(&circuit, 31), None);
    }
}
