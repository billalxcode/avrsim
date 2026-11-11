//! Structural AVR GPIO port.

use sim_cells::{DigitalBuilder, DigitalCircuit, NetId, and_all, read_bits, register};
use sim_core::SimError;

/// One AVR GPIO port with DDR, PORT, PIN, and pull-up signals.
pub struct GpioPort {
    prefix: String,
    ddr: Vec<NetId>,
    port: Vec<NetId>,
    pin_output: Vec<NetId>,
    pull_up: Vec<NetId>,
}

impl GpioPort {
    /// Builds one 8-bit GPIO port.
    pub fn new(builder: &mut DigitalBuilder, prefix: &str) -> Result<Self, SimError> {
        let ddr_data = named_inputs(builder, &format!("{prefix}ddr"), 8);
        let ddr_load = builder.input(&format!("{prefix}ddr_load"));
        let ddr = register(builder, 8, &ddr_data, ddr_load)?;
        let port_data = named_inputs(builder, &format!("{prefix}port"), 8);
        let port_load = builder.input(&format!("{prefix}port_load"));
        let port = register(builder, 8, &port_data, port_load)?;
        let pin_input = named_inputs(builder, &format!("{prefix}pin"), 8);
        let pin_output = build_pin_output(builder, &ddr, &port, &pin_input);
        let pull_up = build_pull_up(builder, &ddr, &port);
        Ok(Self {
            prefix: prefix.to_string(),
            ddr,
            port,
            pin_output,
            pull_up,
        })
    }

    /// Writes the DDR register.
    pub fn set_ddr(&mut self, circuit: &mut DigitalCircuit, value: u8) -> Result<(), SimError> {
        self.drive(circuit, "ddr", value)?;
        Ok(())
    }

    /// Writes the PORT register.
    pub fn set_port(&mut self, circuit: &mut DigitalCircuit, value: u8) -> Result<(), SimError> {
        self.drive(circuit, "port", value)?;
        Ok(())
    }

    /// Drives external pin inputs.
    pub fn set_pin_input(
        &mut self,
        circuit: &mut DigitalCircuit,
        value: u8,
    ) -> Result<(), SimError> {
        set_bits(circuit, &format!("{}pin", self.prefix), value, 8)?;
        circuit.settle()
    }

    /// Reads the DDR register.
    #[must_use]
    pub fn read_ddr(&self, circuit: &DigitalCircuit) -> u8 {
        read_bits(circuit, &self.ddr) as u8
    }

    /// Reads the PORT register.
    #[must_use]
    pub fn read_port(&self, circuit: &DigitalCircuit) -> u8 {
        read_bits(circuit, &self.port) as u8
    }

    /// Reads the PIN value.
    #[must_use]
    pub fn read_pin(&self, circuit: &DigitalCircuit) -> u8 {
        read_bits(circuit, &self.pin_output) as u8
    }

    /// Reads active pull-up bits.
    #[must_use]
    pub fn read_pull_up(&self, circuit: &DigitalCircuit) -> u8 {
        read_bits(circuit, &self.pull_up) as u8
    }

    /// Returns pin output nets.
    #[must_use]
    pub fn pin_output(&self) -> &[NetId] {
        &self.pin_output
    }

    fn drive(
        &mut self,
        circuit: &mut DigitalCircuit,
        kind: &str,
        value: u8,
    ) -> Result<(), SimError> {
        let prefix = self.prefix.as_str();
        set_bits(circuit, &format!("{prefix}{kind}"), value, 8)?;
        circuit.set_input(&format!("{prefix}{kind}_load"), true)?;
        circuit.tick()?;
        circuit.set_input(&format!("{prefix}{kind}_load"), false)?;
        circuit.settle()
    }
}

fn build_pin_output(
    builder: &mut DigitalBuilder,
    ddr: &[NetId],
    port: &[NetId],
    pin_input: &[NetId],
) -> Vec<NetId> {
    (0..8)
        .map(|bit| builder.mux(pin_input[bit], port[bit], ddr[bit]))
        .collect()
}

fn build_pull_up(builder: &mut DigitalBuilder, ddr: &[NetId], port: &[NetId]) -> Vec<NetId> {
    (0..8)
        .map(|bit| {
            let enabled = builder.not(ddr[bit]);
            and_all(builder, &[enabled, port[bit]])
        })
        .collect()
}

fn named_inputs(builder: &mut DigitalBuilder, prefix: &str, width: usize) -> Vec<NetId> {
    (0..width)
        .map(|index| builder.input(&format!("{prefix}{index}")))
        .collect()
}

fn set_bits(
    circuit: &mut DigitalCircuit,
    prefix: &str,
    value: u8,
    width: usize,
) -> Result<(), SimError> {
    for index in 0..width {
        let bit = value & (1 << index) != 0;
        circuit.set_input(&format!("{prefix}{index}"), bit)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::GpioPort;
    use sim_cells::{DigitalBuilder, DigitalCircuit};

    fn fixture() -> (DigitalCircuit, GpioPort) {
        let mut builder = DigitalBuilder::new();
        let port = GpioPort::new(&mut builder, "pb").expect("gpio");
        (builder.build(), port)
    }

    #[test]
    fn output_follows_ddr_and_port() {
        let (mut circuit, mut gpio) = fixture();
        gpio.set_ddr(&mut circuit, 0xFF).expect("ddr");
        gpio.set_port(&mut circuit, 0x55).expect("port");
        assert_eq!(gpio.read_ddr(&circuit), 0xFF);
        assert_eq!(gpio.read_port(&circuit), 0x55);
        assert_eq!(gpio.read_pin(&circuit), 0x55);
    }

    #[test]
    fn input_follows_pin_when_ddr_is_low() {
        let (mut circuit, mut gpio) = fixture();
        gpio.set_ddr(&mut circuit, 0x00).expect("ddr");
        gpio.set_pin_input(&mut circuit, 0xAA).expect("pin");
        assert_eq!(gpio.read_pin(&circuit), 0xAA);
    }

    #[test]
    fn mixed_directions_merge_output_and_input() {
        let (mut circuit, mut gpio) = fixture();
        gpio.set_ddr(&mut circuit, 0xF0).expect("ddr");
        gpio.set_port(&mut circuit, 0xAA).expect("port");
        gpio.set_pin_input(&mut circuit, 0x0F).expect("pin");
        assert_eq!(gpio.read_pin(&circuit), 0xAF);
    }

    #[test]
    fn pull_up_tracks_port_when_ddr_is_low() {
        let (mut circuit, mut gpio) = fixture();
        gpio.set_ddr(&mut circuit, 0x00).expect("ddr");
        gpio.set_port(&mut circuit, 0xFF).expect("port");
        assert_eq!(gpio.read_pull_up(&circuit), 0xFF);
        gpio.set_ddr(&mut circuit, 0xFF).expect("ddr");
        assert_eq!(gpio.read_pull_up(&circuit), 0x00);
    }

    #[test]
    fn reset_clears_gpio_registers() {
        let (mut circuit, mut gpio) = fixture();
        gpio.set_ddr(&mut circuit, 0xFF).expect("ddr");
        gpio.set_port(&mut circuit, 0x55).expect("port");
        circuit.reset().expect("reset");
        assert_eq!(gpio.read_ddr(&circuit), 0);
        assert_eq!(gpio.read_port(&circuit), 0);
    }
}
