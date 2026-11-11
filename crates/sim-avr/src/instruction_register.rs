//! AVR instruction register.

use sim_cells::{DigitalBuilder, DigitalCircuit, NetId, read_bits, register};
use sim_core::SimError;

/// A 32-bit instruction register built from D flip-flops.
pub struct InstructionRegister {
    bits: Vec<NetId>,
}

impl InstructionRegister {
    /// Builds an instruction register from 32 data nets and a load signal.
    pub fn new(
        builder: &mut DigitalBuilder,
        data: &[NetId],
        load: NetId,
    ) -> Result<Self, SimError> {
        let bits = register(builder, 32, data, load)?;
        Ok(Self { bits })
    }

    /// Returns the register bit nets.
    #[must_use]
    pub fn bits(&self) -> &[NetId] {
        &self.bits
    }

    /// Reads the register value.
    #[must_use]
    pub fn read(&self, circuit: &DigitalCircuit) -> u32 {
        read_bits(circuit, &self.bits) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::InstructionRegister;
    use crate::test_support::{named_inputs, set_bits};
    use sim_cells::{DigitalBuilder, DigitalCircuit};

    fn fixture() -> (DigitalCircuit, InstructionRegister) {
        let mut builder = DigitalBuilder::new();
        let data = named_inputs(&mut builder, "d", 32);
        let load = builder.input("load");
        let register = InstructionRegister::new(&mut builder, &data, load).expect("ir");
        (builder.build(), register)
    }

    #[test]
    fn register_loads_and_holds() {
        let (mut circuit, register) = fixture();
        set_bits(&mut circuit, "d", 0x1234_5678, 32);
        circuit.set_input("load", true).expect("load");
        circuit.tick().expect("tick");
        assert_eq!(register.read(&circuit), 0x1234_5678);
        set_bits(&mut circuit, "d", 0xFFFF_FFFF, 32);
        circuit.set_input("load", false).expect("load");
        circuit.tick().expect("tick");
        assert_eq!(register.read(&circuit), 0x1234_5678);
    }

    #[test]
    fn register_reset_clears_value() {
        let (mut circuit, register) = fixture();
        set_bits(&mut circuit, "d", 0xFFFF_FFFF, 32);
        circuit.set_input("load", true).expect("load");
        circuit.tick().expect("tick");
        circuit.reset().expect("reset");
        assert_eq!(register.read(&circuit), 0);
    }

    #[test]
    fn register_rejects_wrong_data_width() {
        let mut builder = DigitalBuilder::new();
        let data = named_inputs(&mut builder, "d", 31);
        let load = builder.input("load");
        assert!(InstructionRegister::new(&mut builder, &data, load).is_err());
    }
}
