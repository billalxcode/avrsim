//! Digital register file with two read ports.

use super::digital::{DigitalBuilder, NetId};
use super::digital_blocks::{decoder, mux_tree, register};
use sim_core::SimError;

/// Wiring for a register file.
pub struct RegisterPorts {
    /// Write address nets, least significant bit first.
    pub write_address: Vec<NetId>,
    /// Write data nets, least significant bit first.
    pub write_data: Vec<NetId>,
    /// Write enable.
    pub write_enable: NetId,
    /// Read address A nets.
    pub read_address_a: Vec<NetId>,
    /// Read address B nets.
    pub read_address_b: Vec<NetId>,
}

/// A two-read-port register file built from D flip-flops.
pub struct RegisterFile {
    bits: Vec<Vec<NetId>>,
    read_a: Vec<NetId>,
    read_b: Vec<NetId>,
}

impl RegisterFile {
    /// Returns storage bit nets indexed by register and bit.
    #[must_use]
    pub fn bits(&self) -> &[Vec<NetId>] {
        &self.bits
    }

    /// Returns read port A nets.
    #[must_use]
    pub fn read_a(&self) -> &[NetId] {
        &self.read_a
    }

    /// Returns read port B nets.
    #[must_use]
    pub fn read_b(&self) -> &[NetId] {
        &self.read_b
    }
}

/// Builds a register file.
pub fn register_file(
    builder: &mut DigitalBuilder,
    count: usize,
    width: usize,
    ports: &RegisterPorts,
) -> Result<RegisterFile, SimError> {
    validate_shape(count, width, ports)?;
    let bits = build_storage(builder, count, width, ports)?;
    let read_a = build_read_port(builder, &bits, width, &ports.read_address_a)?;
    let read_b = build_read_port(builder, &bits, width, &ports.read_address_b)?;
    Ok(RegisterFile {
        bits,
        read_a,
        read_b,
    })
}

fn validate_shape(count: usize, width: usize, ports: &RegisterPorts) -> Result<(), SimError> {
    validate_count(count)?;
    validate_width(width)?;
    validate_address(count, &ports.write_address)?;
    validate_address(count, &ports.read_address_a)?;
    validate_address(count, &ports.read_address_b)?;
    validate_data(width, &ports.write_data)
}

fn validate_count(count: usize) -> Result<(), SimError> {
    if !count.is_power_of_two() || !(2..=32).contains(&count) {
        return Err(SimError::InvalidNetlist(
            "register file count must be a power of two in 2..=32".to_string(),
        ));
    }
    Ok(())
}

fn validate_width(width: usize) -> Result<(), SimError> {
    if !(1..=64).contains(&width) {
        return Err(SimError::InvalidNetlist(
            "register width must be 1..=64".to_string(),
        ));
    }
    Ok(())
}

fn validate_address(count: usize, address: &[NetId]) -> Result<(), SimError> {
    if address.len() != count.trailing_zeros() as usize {
        return Err(SimError::InvalidNetlist(
            "register address width mismatch".to_string(),
        ));
    }
    Ok(())
}

fn validate_data(width: usize, data: &[NetId]) -> Result<(), SimError> {
    if data.len() != width {
        return Err(SimError::InvalidNetlist(
            "register write data width mismatch".to_string(),
        ));
    }
    Ok(())
}

fn build_storage(
    builder: &mut DigitalBuilder,
    count: usize,
    width: usize,
    ports: &RegisterPorts,
) -> Result<Vec<Vec<NetId>>, SimError> {
    let lines = decoder(builder, &ports.write_address)?;
    let mut bits = Vec::with_capacity(count);
    for line in lines {
        let enable = builder.and(line, ports.write_enable);
        bits.push(register(builder, width, &ports.write_data, enable)?);
    }
    Ok(bits)
}

fn build_read_port(
    builder: &mut DigitalBuilder,
    bits: &[Vec<NetId>],
    width: usize,
    address: &[NetId],
) -> Result<Vec<NetId>, SimError> {
    (0..width)
        .map(|bit| {
            let column: Vec<NetId> = bits.iter().map(|register| register[bit]).collect();
            mux_tree(builder, &column, address)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{RegisterPorts, register_file};
    use crate::digital::{DigitalBuilder, DigitalCircuit, NetId};

    fn named_inputs(builder: &mut DigitalBuilder, prefix: &str, width: usize) -> Vec<NetId> {
        (0..width)
            .map(|index| builder.input(&format!("{prefix}{index}")))
            .collect()
    }

    fn expose(builder: &mut DigitalBuilder, prefix: &str, nets: &[NetId]) {
        for (index, &net) in nets.iter().enumerate() {
            builder.output(&format!("{prefix}{index}"), net);
        }
    }

    fn fixture() -> DigitalCircuit {
        let mut builder = DigitalBuilder::new();
        let ports = fixture_ports(&mut builder);
        let file = register_file(&mut builder, 32, 8, &ports).expect("file");
        expose(&mut builder, "a", file.read_a());
        expose(&mut builder, "b", file.read_b());
        builder.build()
    }

    fn fixture_ports(builder: &mut DigitalBuilder) -> RegisterPorts {
        RegisterPorts {
            write_address: named_inputs(builder, "wa", 5),
            write_data: named_inputs(builder, "wd", 8),
            write_enable: builder.input("we"),
            read_address_a: named_inputs(builder, "ra", 5),
            read_address_b: named_inputs(builder, "rb", 5),
        }
    }

    fn set_bits(circuit: &mut DigitalCircuit, prefix: &str, value: u64, width: usize) {
        for index in 0..width {
            let bit = value & (1 << index) != 0;
            circuit
                .set_input(&format!("{prefix}{index}"), bit)
                .expect("input");
        }
    }

    fn write_register(circuit: &mut DigitalCircuit, address: usize, value: u64) {
        set_bits(circuit, "wa", address as u64, 5);
        set_bits(circuit, "wd", value, 8);
        circuit.set_input("we", true).expect("we");
        circuit.tick().expect("tick");
    }

    fn read_port(
        circuit: &mut DigitalCircuit,
        input_prefix: &str,
        output_prefix: &str,
        address: usize,
    ) -> u64 {
        set_bits(circuit, input_prefix, address as u64, 5);
        circuit.settle().expect("settle");
        read_named(circuit, output_prefix, 8)
    }

    fn read_named(circuit: &DigitalCircuit, prefix: &str, width: usize) -> u64 {
        let mut value = 0;
        for index in 0..width {
            if circuit.output(&format!("{prefix}{index}")).expect("output") {
                value |= 1 << index;
            }
        }
        value
    }

    fn sample_value(address: usize) -> u64 {
        (address as u64 * 7 + 3) & 0xFF
    }

    #[test]
    fn register_file_writes_and_reads_all_registers() {
        let mut circuit = fixture();
        for address in 0..32 {
            write_register(&mut circuit, address, sample_value(address));
        }
        for address in 0..32 {
            let expected = sample_value(address);
            assert_eq!(read_port(&mut circuit, "ra", "a", address), expected);
        }
    }

    #[test]
    fn register_file_holds_without_write() {
        let mut circuit = fixture();
        write_register(&mut circuit, 1, 0x55);
        set_bits(&mut circuit, "wa", 1, 5);
        set_bits(&mut circuit, "wd", 0x30, 8);
        circuit.set_input("we", false).expect("we");
        circuit.tick().expect("tick");
        assert_eq!(read_port(&mut circuit, "ra", "a", 1), 0x55);
    }

    #[test]
    fn register_file_ports_are_independent() {
        let mut circuit = fixture();
        write_register(&mut circuit, 0, 0x3);
        write_register(&mut circuit, 31, 0xC);
        assert_eq!(read_port(&mut circuit, "ra", "a", 0), 0x3);
        assert_eq!(read_port(&mut circuit, "rb", "b", 31), 0xC);
    }

    #[test]
    fn register_file_reset_clears_storage() {
        let mut circuit = fixture();
        write_register(&mut circuit, 2, 0xF);
        circuit.reset().expect("reset");
        assert_eq!(read_port(&mut circuit, "ra", "a", 2), 0);
    }
}
