//! Digital building blocks for registers and counters.

use super::circuit::{DigitalBuilder, DigitalCircuit, NetId};
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

/// Builds a loadable register.
pub fn register(
    builder: &mut DigitalBuilder,
    width: usize,
    data: &[NetId],
    load: NetId,
) -> Result<Vec<NetId>, SimError> {
    if width == 0 || width > 64 || data.len() != width {
        return Err(SimError::InvalidNetlist(
            "register width mismatch".to_string(),
        ));
    }
    Ok((0..width)
        .map(|bit| register_bit(builder, data[bit], load))
        .collect())
}

fn register_bit(builder: &mut DigitalBuilder, data: NetId, load: NetId) -> NetId {
    let output = builder.net();
    let next = builder.mux(output, data, load);
    builder.dff_into(next, output);
    output
}

/// Builds a combinational incrementer.
#[must_use]
pub fn increment(builder: &mut DigitalBuilder, bits: &[NetId]) -> Vec<NetId> {
    let mut carry = builder.constant(true);
    let mut output = Vec::with_capacity(bits.len());
    for &bit in bits {
        output.push(builder.xor(bit, carry));
        carry = builder.and(bit, carry);
    }
    output
}

/// Builds a combinational decrementer.
#[must_use]
pub fn decrement(builder: &mut DigitalBuilder, bits: &[NetId]) -> Vec<NetId> {
    let mut borrow = builder.constant(true);
    let mut output = Vec::with_capacity(bits.len());
    for &bit in bits {
        let inverted = builder.not(bit);
        output.push(builder.xor(bit, borrow));
        borrow = builder.and(inverted, borrow);
    }
    output
}

/// Builds a combinational full adder.
#[must_use]
pub fn full_adder(
    builder: &mut DigitalBuilder,
    first: NetId,
    second: NetId,
    carry_in: NetId,
) -> (NetId, NetId) {
    let partial = builder.xor(first, second);
    let sum = builder.xor(partial, carry_in);
    let lower = builder.and(first, second);
    let upper = builder.and(partial, carry_in);
    (sum, builder.or(lower, upper))
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

/// A loadable program counter with increment.
pub struct ProgramCounter {
    bits: Vec<NetId>,
}

impl ProgramCounter {
    /// Builds a program counter.
    pub fn new(
        builder: &mut DigitalBuilder,
        width: usize,
        load: NetId,
        target: &[NetId],
        increment_enable: NetId,
    ) -> Result<Self, SimError> {
        validate_word_width(width, target, 16)?;
        Ok(Self {
            bits: build_pc(builder, width, load, target, increment_enable),
        })
    }

    /// Returns counter bit nets.
    #[must_use]
    pub fn bits(&self) -> &[NetId] {
        &self.bits
    }

    /// Reads the counter value.
    #[must_use]
    pub fn read(&self, circuit: &DigitalCircuit) -> u64 {
        read_bits(circuit, &self.bits)
    }
}

/// A stack pointer with load, increment, and decrement.
pub struct StackPointer {
    bits: Vec<NetId>,
}

impl StackPointer {
    /// Builds a stack pointer.
    pub fn new(
        builder: &mut DigitalBuilder,
        width: usize,
        load: NetId,
        target: &[NetId],
        increment_enable: NetId,
        decrement_enable: NetId,
    ) -> Result<Self, SimError> {
        validate_word_width(width, target, 16)?;
        let controls = SpControl {
            load,
            increment: increment_enable,
            decrement: decrement_enable,
        };
        Ok(Self {
            bits: build_sp(builder, width, target, controls),
        })
    }

    /// Returns pointer bit nets.
    #[must_use]
    pub fn bits(&self) -> &[NetId] {
        &self.bits
    }

    /// Reads the pointer value.
    #[must_use]
    pub fn read(&self, circuit: &DigitalCircuit) -> u64 {
        read_bits(circuit, &self.bits)
    }
}

#[derive(Clone, Copy)]
struct SpControl {
    load: NetId,
    increment: NetId,
    decrement: NetId,
}

fn validate_word_width(width: usize, target: &[NetId], max: usize) -> Result<(), SimError> {
    if width == 0 || width > max || target.len() != width {
        return Err(SimError::InvalidNetlist("word width mismatch".to_string()));
    }
    Ok(())
}

fn build_pc(
    builder: &mut DigitalBuilder,
    width: usize,
    load: NetId,
    target: &[NetId],
    increment_enable: NetId,
) -> Vec<NetId> {
    let bits: Vec<NetId> = (0..width).map(|_| builder.net()).collect();
    let incremented = increment(builder, &bits);
    for index in 0..width {
        let held = builder.mux(bits[index], incremented[index], increment_enable);
        let next = builder.mux(held, target[index], load);
        builder.dff_into(next, bits[index]);
    }
    bits
}

fn build_sp(
    builder: &mut DigitalBuilder,
    width: usize,
    target: &[NetId],
    controls: SpControl,
) -> Vec<NetId> {
    let bits: Vec<NetId> = (0..width).map(|_| builder.net()).collect();
    wire_sp(builder, &bits, target, controls);
    bits
}

fn wire_sp(builder: &mut DigitalBuilder, bits: &[NetId], target: &[NetId], controls: SpControl) {
    let incremented = increment(builder, bits);
    let decremented = decrement(builder, bits);
    for index in 0..bits.len() {
        let next = sp_next(
            builder,
            bits,
            &incremented,
            &decremented,
            target,
            index,
            controls,
        );
        builder.dff_into(next, bits[index]);
    }
}

fn sp_next(
    builder: &mut DigitalBuilder,
    bits: &[NetId],
    incremented: &[NetId],
    decremented: &[NetId],
    target: &[NetId],
    index: usize,
    controls: SpControl,
) -> NetId {
    let held = builder.mux(bits[index], decremented[index], controls.decrement);
    let stepped = builder.mux(held, incremented[index], controls.increment);
    builder.mux(stepped, target[index], controls.load)
}

#[cfg(test)]
mod tests {
    use super::{
        ProgramCounter, StackPointer, and_all, counter, decoder, full_adder, mux_tree, or_all,
        read_bits, register, shift_register,
    };
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

    fn named_inputs(builder: &mut DigitalBuilder, prefix: &str, width: usize) -> Vec<NetId> {
        (0..width)
            .map(|index| builder.input(&format!("{prefix}{index}")))
            .collect()
    }

    fn set_bits(circuit: &mut DigitalCircuit, prefix: &str, value: u64, width: usize) {
        for index in 0..width {
            let bit = value & (1 << index) != 0;
            circuit
                .set_input(&format!("{prefix}{index}"), bit)
                .expect("input");
        }
    }

    fn pc_fixture() -> (DigitalCircuit, ProgramCounter) {
        let mut builder = DigitalBuilder::new();
        let load = builder.input("load");
        let increment = builder.input("inc");
        let target = named_inputs(&mut builder, "t", 16);
        let pc = ProgramCounter::new(&mut builder, 16, load, &target, increment).expect("pc");
        (builder.build(), pc)
    }

    fn sp_fixture() -> (DigitalCircuit, StackPointer) {
        let mut builder = DigitalBuilder::new();
        let load = builder.input("load");
        let increment = builder.input("inc");
        let decrement = builder.input("dec");
        let target = named_inputs(&mut builder, "t", 16);
        let sp =
            StackPointer::new(&mut builder, 16, load, &target, increment, decrement).expect("sp");
        (builder.build(), sp)
    }

    #[test]
    fn program_counter_counts_and_loads() {
        let (mut circuit, pc) = pc_fixture();
        circuit.set_input("inc", true).expect("inc");
        circuit.set_input("load", false).expect("load");
        circuit.tick().expect("tick");
        assert_eq!(pc.read(&circuit), 1);
        set_bits(&mut circuit, "t", 0x1234, 16);
        circuit.set_input("inc", false).expect("inc");
        circuit.set_input("load", true).expect("load");
        circuit.tick().expect("tick");
        assert_eq!(pc.read(&circuit), 0x1234);
        circuit.reset().expect("reset");
        assert_eq!(pc.read(&circuit), 0);
    }

    #[test]
    fn program_counter_holds_without_increment() {
        let (mut circuit, pc) = pc_fixture();
        circuit.set_input("inc", false).expect("inc");
        circuit.set_input("load", false).expect("load");
        circuit.tick().expect("tick");
        assert_eq!(pc.read(&circuit), 0);
    }

    #[test]
    fn stack_pointer_steps_and_loads() {
        let (mut circuit, sp) = sp_fixture();
        set_bits(&mut circuit, "t", 0x08FF, 16);
        circuit.set_input("load", true).expect("load");
        circuit.tick().expect("tick");
        assert_eq!(sp.read(&circuit), 0x08FF);
        circuit.set_input("load", false).expect("load");
        circuit.set_input("inc", true).expect("inc");
        circuit.tick().expect("tick");
        assert_eq!(sp.read(&circuit), 0x0900);
        circuit.set_input("inc", false).expect("inc");
        circuit.set_input("dec", true).expect("dec");
        circuit.tick().expect("tick");
        assert_eq!(sp.read(&circuit), 0x08FF);
        circuit.set_input("dec", false).expect("dec");
        circuit.tick().expect("tick");
        assert_eq!(sp.read(&circuit), 0x08FF);
    }

    #[test]
    fn instruction_register_loads_and_holds() {
        let mut builder = DigitalBuilder::new();
        let load = builder.input("load");
        let data = named_inputs(&mut builder, "d", 16);
        let ir = register(&mut builder, 16, &data, load).expect("ir");
        let mut circuit = builder.build();
        set_bits(&mut circuit, "d", 0xABCD, 16);
        circuit.set_input("load", true).expect("load");
        circuit.tick().expect("tick");
        assert_eq!(read_bits(&circuit, &ir), 0xABCD);
        set_bits(&mut circuit, "d", 0x0000, 16);
        circuit.set_input("load", false).expect("load");
        circuit.tick().expect("tick");
        assert_eq!(read_bits(&circuit, &ir), 0xABCD);
    }

    fn set_adder_inputs(circuit: &mut DigitalCircuit, value: u8) {
        circuit.set_input("a", value & 1 != 0).expect("a");
        circuit.set_input("b", value & 2 != 0).expect("b");
        circuit.set_input("c", value & 4 != 0).expect("c");
        circuit.settle().expect("settle");
    }

    #[test]
    fn digital_full_adder_truth_table() {
        let mut builder = DigitalBuilder::new();
        let first = builder.input("a");
        let second = builder.input("b");
        let carry_in = builder.input("c");
        let (sum, carry_out) = full_adder(&mut builder, first, second, carry_in);
        builder.output("sum", sum);
        builder.output("carry", carry_out);
        let mut circuit = builder.build();
        for value in 0..8 {
            set_adder_inputs(&mut circuit, value);
            let total = u8::count_ones(value) as u8;
            assert_eq!(circuit.output("sum").expect("sum"), total & 1 != 0);
            assert_eq!(circuit.output("carry").expect("carry"), total >= 2);
        }
    }
}
