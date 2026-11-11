//! Digital building blocks for registers and counters.

use super::digital::{DigitalBuilder, DigitalCircuit, NetId};
use sim_core::SimError;

/// Reduces nets with AND. An empty list yields true.
#[must_use]
pub fn and_all(builder: &mut DigitalBuilder, inputs: &[NetId]) -> NetId {
    if inputs.is_empty() {
        return builder.constant(true);
    }
    inputs[1..]
        .iter()
        .fold(inputs[0], |left, right| builder.and(left, *right))
}

/// Reduces nets with OR. An empty list yields false.
#[must_use]
pub fn or_all(builder: &mut DigitalBuilder, inputs: &[NetId]) -> NetId {
    if inputs.is_empty() {
        return builder.constant(false);
    }
    inputs[1..]
        .iter()
        .fold(inputs[0], |left, right| builder.or(left, *right))
}

/// Builds a multiplexer tree. The number of inputs must be two to the
/// power of the number of select nets.
pub fn mux_tree(
    builder: &mut DigitalBuilder,
    inputs: &[NetId],
    select: &[NetId],
) -> Result<NetId, SimError> {
    if inputs.len() != 1 << select.len() {
        return Err(SimError::InvalidNetlist(
            "mux tree shape mismatch".to_string(),
        ));
    }
    let mut level = inputs.to_vec();
    for &bit in select {
        level = level
            .chunks(2)
            .map(|pair| builder.mux(pair[0], pair[1], bit))
            .collect();
    }
    Ok(level[0])
}

/// Builds a one-hot decoder.
pub fn decoder(builder: &mut DigitalBuilder, inputs: &[NetId]) -> Result<Vec<NetId>, SimError> {
    if inputs.len() > 8 {
        return Err(SimError::InvalidNetlist(
            "decoder supports at most 8 inputs".to_string(),
        ));
    }
    let count = 1usize << inputs.len();
    Ok((0..count)
        .map(|value| decoder_line(builder, inputs, value))
        .collect())
}

fn decoder_line(builder: &mut DigitalBuilder, inputs: &[NetId], value: usize) -> NetId {
    let terms: Vec<NetId> = inputs
        .iter()
        .enumerate()
        .map(|(index, &input)| decoder_term(builder, input, value, index))
        .collect();
    and_all(builder, &terms)
}

fn decoder_term(builder: &mut DigitalBuilder, input: NetId, value: usize, index: usize) -> NetId {
    if value & (1 << index) == 0 {
        builder.not(input)
    } else {
        input
    }
}

/// Reads a little-endian bit vector from a circuit.
#[must_use]
pub fn read_bits(circuit: &DigitalCircuit, bits: &[NetId]) -> u64 {
    let mut value = 0;
    for (index, &bit) in bits.iter().enumerate() {
        if circuit.value(bit) {
            value |= 1 << index;
        }
    }
    value
}

/// Builds a ripple counter with enable.
pub fn counter(
    builder: &mut DigitalBuilder,
    width: usize,
    enable: NetId,
) -> Result<Vec<NetId>, SimError> {
    if width == 0 || width > 16 {
        return Err(SimError::InvalidNetlist(
            "counter width must be 1..=16".to_string(),
        ));
    }
    let bits: Vec<NetId> = (0..width).map(|_| builder.net()).collect();
    for index in 0..width {
        let toggle = counter_toggle(builder, &bits, index, enable);
        let next = builder.xor(bits[index], toggle);
        builder.dff_into(next, bits[index]);
    }
    Ok(bits)
}

fn counter_toggle(
    builder: &mut DigitalBuilder,
    bits: &[NetId],
    index: usize,
    enable: NetId,
) -> NetId {
    let mut terms = Vec::with_capacity(index + 1);
    terms.push(enable);
    terms.extend_from_slice(&bits[..index]);
    and_all(builder, &terms)
}

/// Builds a shift register that shifts when enable is high.
pub fn shift_register(
    builder: &mut DigitalBuilder,
    width: usize,
    input: NetId,
    enable: NetId,
) -> Result<Vec<NetId>, SimError> {
    if width == 0 || width > 64 {
        return Err(SimError::InvalidNetlist(
            "shift register width must be 1..=64".to_string(),
        ));
    }
    let bits: Vec<NetId> = (0..width).map(|_| builder.net()).collect();
    for index in 0..width {
        let source = shift_source(&bits, index, input);
        let next = builder.mux(bits[index], source, enable);
        builder.dff_into(next, bits[index]);
    }
    Ok(bits)
}

fn shift_source(bits: &[NetId], index: usize, input: NetId) -> NetId {
    if index == 0 { input } else { bits[index - 1] }
}

#[cfg(test)]
mod tests {
    use super::{and_all, counter, decoder, mux_tree, or_all, read_bits, shift_register};
    use crate::digital::{DigitalBuilder, DigitalCircuit, NetId};

    #[test]
    fn reduction_helpers_handle_empty_inputs() {
        let mut builder = DigitalBuilder::new();
        let all = and_all(&mut builder, &[]);
        let any = or_all(&mut builder, &[]);
        let mut circuit = builder.build();
        circuit.settle().expect("settle");
        assert!(circuit.value(all));
        assert!(!circuit.value(any));
    }

    #[test]
    fn decoder_selects_one_line() {
        let mut builder = DigitalBuilder::new();
        let first = builder.input("a");
        let second = builder.input("b");
        let lines = decoder(&mut builder, &[first, second]).expect("decoder");
        for (index, line) in lines.iter().enumerate() {
            builder.output(&format!("y{index}"), *line);
        }
        let mut circuit = builder.build();
        for value in 0..4 {
            circuit.set_input("a", value & 1 != 0).expect("a");
            circuit.set_input("b", value & 2 != 0).expect("b");
            circuit.settle().expect("settle");
            for index in 0..4 {
                let expected = index == value;
                assert_eq!(circuit.output(&format!("y{index}")).expect("y"), expected);
            }
        }
    }

    fn mux_tree_fixture() -> DigitalCircuit {
        let mut builder = DigitalBuilder::new();
        let mut inputs = Vec::new();
        for index in 0..4 {
            inputs.push(builder.input(&format!("i{index}")));
        }
        let first = builder.input("s0");
        let second = builder.input("s1");
        let output = mux_tree(&mut builder, &inputs, &[first, second]).expect("mux");
        builder.output("y", output);
        builder.build()
    }

    fn set_mux_inputs(circuit: &mut DigitalCircuit, value: usize) {
        for index in 0..4 {
            circuit
                .set_input(&format!("i{index}"), index == value)
                .expect("input");
        }
        circuit.set_input("s0", value & 1 != 0).expect("s0");
        circuit.set_input("s1", value & 2 != 0).expect("s1");
    }

    #[test]
    fn mux_tree_selects_every_input() {
        let mut circuit = mux_tree_fixture();
        for value in 0..4 {
            set_mux_inputs(&mut circuit, value);
            circuit.settle().expect("settle");
            assert!(circuit.output("y").expect("y"));
        }
    }

    #[test]
    fn counter_counts_and_holds() {
        let mut builder = DigitalBuilder::new();
        let enable = builder.input("enable");
        let bits = counter(&mut builder, 4, enable).expect("counter");
        for (index, bit) in bits.iter().enumerate() {
            builder.output(&format!("q{index}"), *bit);
        }
        let mut circuit = builder.build();
        circuit.set_input("enable", true).expect("enable");
        for expected in 0..16 {
            assert_eq!(read_bits(&circuit, &bits), expected);
            circuit.tick().expect("tick");
        }
        assert_eq!(read_bits(&circuit, &bits), 0);
        circuit.set_input("enable", false).expect("enable");
        circuit.tick().expect("tick");
        assert_eq!(read_bits(&circuit, &bits), 0);
    }

    fn shift_fixture() -> (DigitalCircuit, Vec<NetId>) {
        let mut builder = DigitalBuilder::new();
        let input = builder.input("din");
        let enable = builder.input("enable");
        let bits = shift_register(&mut builder, 4, input, enable).expect("shift");
        (builder.build(), bits)
    }

    #[test]
    fn shift_register_moves_bits() {
        let (mut circuit, bits) = shift_fixture();
        circuit.set_input("enable", true).expect("enable");
        circuit.set_input("din", true).expect("din");
        circuit.tick().expect("tick");
        assert_eq!(read_bits(&circuit, &bits), 0b0001);
        circuit.set_input("din", false).expect("din");
        circuit.tick().expect("tick");
        assert_eq!(read_bits(&circuit, &bits), 0b0010);
        circuit.tick().expect("tick");
        circuit.tick().expect("tick");
        assert_eq!(read_bits(&circuit, &bits), 0b1000);
        circuit.set_input("enable", false).expect("enable");
        circuit.tick().expect("tick");
        assert_eq!(read_bits(&circuit, &bits), 0b1000);
    }
}
