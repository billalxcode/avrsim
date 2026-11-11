//! Eight-bit digital ALU.

use super::digital::{DigitalBuilder, NetId};
use super::digital_blocks::{decoder, mux_tree, or_all};
use sim_core::SimError;

/// Result and flags of the digital ALU.
pub struct DigitalAlu {
    /// Result bits, least significant first.
    pub result: Vec<NetId>,
    /// Result is zero.
    pub zero: NetId,
    /// Adder carry out.
    pub carry: NetId,
    /// Result bit 7.
    pub negative: NetId,
    /// Adder carry out of bit 3.
    pub half_carry: NetId,
}

struct LogicBytes {
    and: Vec<NetId>,
    or: Vec<NetId>,
    xor: Vec<NetId>,
}

/// Builds an eight-bit ALU.
///
/// Operation select: 0 ADD, 1 SUB, 2 AND, 3 OR, 4 XOR, 5 pass B.
pub fn digital_alu(
    builder: &mut DigitalBuilder,
    first: &[NetId],
    second: &[NetId],
    op: &[NetId],
) -> Result<DigitalAlu, SimError> {
    validate_alu(first, second, op)?;
    let lines = decoder(builder, op)?;
    let subtract = lines[1];
    let (result, carry, half_carry) = alu_result(builder, first, second, op, subtract)?;
    Ok(finish_alu(builder, result, carry, half_carry))
}

fn alu_result(
    builder: &mut DigitalBuilder,
    first: &[NetId],
    second: &[NetId],
    op: &[NetId],
    subtract: NetId,
) -> Result<(Vec<NetId>, NetId, NetId), SimError> {
    let (sum, raw_carry, raw_half) = add_bytes(builder, first, second, subtract);
    let logic = logic_bytes(builder, first, second);
    let result = select_result(builder, &sum, &logic, second, op)?;
    let carry = carry_flag(builder, raw_carry, subtract);
    let half_carry = carry_flag(builder, raw_half, subtract);
    Ok((result, carry, half_carry))
}

fn finish_alu(
    builder: &mut DigitalBuilder,
    result: Vec<NetId>,
    carry: NetId,
    half_carry: NetId,
) -> DigitalAlu {
    let any = or_all(builder, &result);
    DigitalAlu {
        zero: builder.not(any),
        negative: result[7],
        result,
        carry,
        half_carry,
    }
}

fn carry_flag(builder: &mut DigitalBuilder, value: NetId, subtract: NetId) -> NetId {
    let inverted = builder.not(value);
    builder.mux(value, inverted, subtract)
}

fn validate_alu(first: &[NetId], second: &[NetId], op: &[NetId]) -> Result<(), SimError> {
    if first.len() != 8 || second.len() != 8 || op.len() != 3 {
        return Err(SimError::InvalidNetlist(
            "ALU expects 8-bit operands and 3-bit opcode".to_string(),
        ));
    }
    Ok(())
}

fn add_bytes(
    builder: &mut DigitalBuilder,
    first: &[NetId],
    second: &[NetId],
    subtract: NetId,
) -> (Vec<NetId>, NetId, NetId) {
    let mut carry = subtract;
    let mut half_carry = subtract;
    let mut sum = Vec::with_capacity(8);
    for index in 0..8 {
        let (result, next) = add_bit(builder, first[index], second[index], carry, subtract);
        if index == 3 {
            half_carry = next;
        }
        sum.push(result);
        carry = next;
    }
    (sum, carry, half_carry)
}

fn add_bit(
    builder: &mut DigitalBuilder,
    first: NetId,
    second: NetId,
    carry_in: NetId,
    subtract: NetId,
) -> (NetId, NetId) {
    let inverted = builder.not(second);
    let bit = builder.mux(second, inverted, subtract);
    full_adder(builder, first, bit, carry_in)
}

fn full_adder(
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

fn logic_bytes(builder: &mut DigitalBuilder, first: &[NetId], second: &[NetId]) -> LogicBytes {
    let mut and = Vec::with_capacity(8);
    let mut or = Vec::with_capacity(8);
    let mut xor = Vec::with_capacity(8);
    for index in 0..8 {
        and.push(builder.and(first[index], second[index]));
        or.push(builder.or(first[index], second[index]));
        xor.push(builder.xor(first[index], second[index]));
    }
    LogicBytes { and, or, xor }
}

fn select_result(
    builder: &mut DigitalBuilder,
    sum: &[NetId],
    logic: &LogicBytes,
    pass: &[NetId],
    op: &[NetId],
) -> Result<Vec<NetId>, SimError> {
    let zero = builder.constant(false);
    (0..8)
        .map(|bit| select_bit(builder, bit, sum, logic, pass, op, zero))
        .collect()
}

fn select_bit(
    builder: &mut DigitalBuilder,
    bit: usize,
    sum: &[NetId],
    logic: &LogicBytes,
    pass: &[NetId],
    op: &[NetId],
    zero: NetId,
) -> Result<NetId, SimError> {
    let options = select_options(bit, sum, logic, pass, zero);
    mux_tree(builder, &options, op)
}

fn select_options(
    bit: usize,
    sum: &[NetId],
    logic: &LogicBytes,
    pass: &[NetId],
    zero: NetId,
) -> [NetId; 8] {
    [
        sum[bit],
        sum[bit],
        logic.and[bit],
        logic.or[bit],
        logic.xor[bit],
        pass[bit],
        zero,
        zero,
    ]
}

#[cfg(test)]
mod tests {
    use super::{DigitalAlu, digital_alu};
    use crate::digital::{DigitalBuilder, DigitalCircuit};
    use crate::test_support::{named_inputs, set_bits};

    fn fixture() -> (DigitalCircuit, DigitalAlu) {
        let mut builder = DigitalBuilder::new();
        let first = named_inputs(&mut builder, "a", 8);
        let second = named_inputs(&mut builder, "b", 8);
        let op = named_inputs(&mut builder, "op", 3);
        let alu = digital_alu(&mut builder, &first, &second, &op).expect("alu");
        (builder.build(), alu)
    }

    fn evaluate(
        circuit: &mut DigitalCircuit,
        alu: &DigitalAlu,
        first: u64,
        second: u64,
        op: u64,
    ) -> (u64, bool, bool, bool, bool) {
        set_bits(circuit, "a", first, 8);
        set_bits(circuit, "b", second, 8);
        set_bits(circuit, "op", op, 3);
        circuit.settle().expect("settle");
        let result = crate::read_bits(circuit, &alu.result);
        (
            result,
            circuit.value(alu.zero),
            circuit.value(alu.carry),
            circuit.value(alu.negative),
            circuit.value(alu.half_carry),
        )
    }

    #[test]
    fn addition_and_subtraction() {
        let (mut circuit, alu) = fixture();
        assert_eq!(evaluate(&mut circuit, &alu, 5, 3, 0).0, 8);
        let (result, zero, carry, _, _) = evaluate(&mut circuit, &alu, 0xFF, 1, 0);
        assert_eq!((result, zero, carry), (0, true, true));
        assert_eq!(evaluate(&mut circuit, &alu, 9, 4, 1).0, 5);
        let (result, _, carry, negative, _) = evaluate(&mut circuit, &alu, 0, 1, 1);
        assert_eq!((result, carry, negative), (0xFF, true, true));
    }

    #[test]
    fn half_carry_tracks_bit_three() {
        let (mut circuit, alu) = fixture();
        let (_, _, _, _, half_carry) = evaluate(&mut circuit, &alu, 0x0F, 1, 0);
        assert!(half_carry);
        let (_, _, _, _, half_carry) = evaluate(&mut circuit, &alu, 0x0E, 1, 0);
        assert!(!half_carry);
    }

    #[test]
    fn logic_and_pass_operations() {
        let (mut circuit, alu) = fixture();
        assert_eq!(evaluate(&mut circuit, &alu, 0xAA, 0xCC, 2).0, 0x88);
        assert_eq!(evaluate(&mut circuit, &alu, 0xAA, 0xCC, 3).0, 0xEE);
        assert_eq!(evaluate(&mut circuit, &alu, 0xAA, 0xCC, 4).0, 0x66);
        assert_eq!(evaluate(&mut circuit, &alu, 0xAA, 0xCC, 5).0, 0xCC);
    }

    #[test]
    fn zero_and_negative_flags() {
        let (mut circuit, alu) = fixture();
        let (_, zero, _, negative, _) = evaluate(&mut circuit, &alu, 0xFF, 0xFF, 4);
        assert!(zero && !negative);
        let (_, zero, _, negative, _) = evaluate(&mut circuit, &alu, 0, 0x80, 5);
        assert!(!zero && negative);
    }
}
