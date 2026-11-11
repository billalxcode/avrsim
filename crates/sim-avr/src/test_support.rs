//! Shared helpers for AVR tests.

use sim_cells::{DigitalBuilder, NetId};

/// Creates named input nets.
pub fn named_inputs(builder: &mut DigitalBuilder, prefix: &str, width: usize) -> Vec<NetId> {
    (0..width)
        .map(|index| builder.input(&format!("{prefix}{index}")))
        .collect()
}

/// Drives named input nets from a value.
pub fn set_bits(circuit: &mut sim_cells::DigitalCircuit, prefix: &str, value: u64, width: usize) {
    for index in 0..width {
        let bit = value & (1 << index) != 0;
        circuit
            .set_input(&format!("{prefix}{index}"), bit)
            .expect("input");
    }
}
