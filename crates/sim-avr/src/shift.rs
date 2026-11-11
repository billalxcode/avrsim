//! AVR shift and rotate unit.

use sim_cells::{DigitalBuilder, NetId, mux_tree, or_all};
use sim_core::SimError;

/// Result and flags of the AVR shift unit.
pub struct ShiftUnit {
    /// Result bits, least significant first.
    pub result: Vec<NetId>,
    /// Carry out.
    pub carry: NetId,
    /// Result is zero.
    pub zero: NetId,
    /// Result bit 7.
    pub negative: NetId,
    /// Signed overflow, `N xor C`.
    pub overflow: NetId,
    /// Signed result sign.
    pub sign: NetId,
}

/// Builds the 8-bit AVR shift unit.
///
/// `op` selects LSR, ASR, ROR, and SWAP in that order.
pub fn shift_unit(
    builder: &mut DigitalBuilder,
    input: &[NetId],
    carry_in: NetId,
    op: &[NetId],
) -> Result<ShiftUnit, SimError> {
    validate_shift(input, op)?;
    let zero = builder.constant(false);
    let result = (0..8)
        .map(|bit| shift_bit(builder, bit, input, carry_in, op, zero))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(finish_shift(builder, input, result))
}

fn validate_shift(input: &[NetId], op: &[NetId]) -> Result<(), SimError> {
    if input.len() != 8 || op.len() != 2 {
        return Err(SimError::InvalidNetlist(
            "AVR shift expects 8-bit input and 2-bit opcode".to_string(),
        ));
    }
    Ok(())
}

fn finish_shift(builder: &mut DigitalBuilder, input: &[NetId], result: Vec<NetId>) -> ShiftUnit {
    let any = or_all(builder, &result);
    let zero = builder.not(any);
    let negative = result[7];
    let carry = input[0];
    let overflow = builder.xor(negative, carry);
    ShiftUnit {
        result,
        carry,
        zero,
        negative,
        overflow,
        sign: builder.xor(negative, overflow),
    }
}

fn shift_bit(
    builder: &mut DigitalBuilder,
    bit: usize,
    input: &[NetId],
    carry_in: NetId,
    op: &[NetId],
    zero: NetId,
) -> Result<NetId, SimError> {
    let options = [
        lsr_source(input, bit, zero),
        asr_source(input, bit),
        ror_source(input, bit, carry_in),
        swap_source(input, bit),
    ];
    mux_tree(builder, &options, op)
}

fn lsr_source(input: &[NetId], bit: usize, zero: NetId) -> NetId {
    if bit == 7 { zero } else { input[bit + 1] }
}

fn asr_source(input: &[NetId], bit: usize) -> NetId {
    if bit == 7 { input[7] } else { input[bit + 1] }
}

fn ror_source(input: &[NetId], bit: usize, carry_in: NetId) -> NetId {
    if bit == 7 { carry_in } else { input[bit + 1] }
}

fn swap_source(input: &[NetId], bit: usize) -> NetId {
    input[(bit + 4) % 8]
}

#[cfg(test)]
mod tests {
    use super::{ShiftUnit, shift_unit};
    use crate::test_support::{named_inputs, set_bits};
    use sim_cells::{DigitalBuilder, DigitalCircuit, read_bits};

    #[derive(Debug, PartialEq, Eq)]
    struct Outcome {
        value: u8,
        carry: bool,
        zero: bool,
        negative: bool,
        overflow: bool,
        sign: bool,
    }

    fn fixture() -> (DigitalCircuit, ShiftUnit) {
        let mut builder = DigitalBuilder::new();
        let input = named_inputs(&mut builder, "a", 8);
        let carry = builder.input("cin");
        let op = named_inputs(&mut builder, "op", 2);
        let unit = shift_unit(&mut builder, &input, carry, &op).expect("shift unit");
        (builder.build(), unit)
    }

    fn evaluate(
        circuit: &mut DigitalCircuit,
        unit: &ShiftUnit,
        value: u8,
        carry: bool,
        op: u8,
    ) -> Outcome {
        set_bits(circuit, "a", u64::from(value), 8);
        set_bits(circuit, "op", u64::from(op), 2);
        circuit.set_input("cin", carry).expect("cin");
        circuit.settle().expect("settle");
        Outcome {
            value: read_bits(circuit, &unit.result) as u8,
            carry: circuit.value(unit.carry),
            zero: circuit.value(unit.zero),
            negative: circuit.value(unit.negative),
            overflow: circuit.value(unit.overflow),
            sign: circuit.value(unit.sign),
        }
    }

    fn reference(value: u8, carry: bool, op: u8) -> Outcome {
        match op {
            0 => lsr(value),
            1 => asr(value),
            2 => ror(value, carry),
            _ => swap(value),
        }
    }

    fn lsr(value: u8) -> Outcome {
        let result = value >> 1;
        let carry = value & 1 != 0;
        outcome(result, carry, false, carry)
    }

    fn asr(value: u8) -> Outcome {
        let result = (value >> 1) | (value & 0x80);
        let carry = value & 1 != 0;
        outcome(result, carry, value & 0x80 != 0, carry)
    }

    fn ror(value: u8, carry_in: bool) -> Outcome {
        let result = (value >> 1) | if carry_in { 0x80 } else { 0 };
        outcome(result, value & 1 != 0, carry_in, value & 1 != 0)
    }

    fn swap(value: u8) -> Outcome {
        Outcome {
            value: value.rotate_left(4),
            carry: false,
            zero: false,
            negative: false,
            overflow: false,
            sign: false,
        }
    }

    fn outcome(value: u8, carry: bool, negative: bool, old_carry: bool) -> Outcome {
        let overflow = negative ^ old_carry;
        Outcome {
            value,
            carry,
            zero: value == 0,
            negative,
            overflow,
            sign: negative ^ overflow,
        }
    }

    #[test]
    fn shift_operations_match_reference() {
        let (mut circuit, unit) = fixture();
        for value in 0..=u8::MAX {
            for carry in [false, true] {
                for op in 0..3 {
                    let actual = evaluate(&mut circuit, &unit, value, carry, op);
                    assert_eq!(actual, reference(value, carry, op));
                }
            }
        }
    }

    #[test]
    fn swap_matches_reference() {
        let (mut circuit, unit) = fixture();
        for value in 0..=u8::MAX {
            let actual = evaluate(&mut circuit, &unit, value, false, 3);
            assert_eq!(actual.value, reference(value, false, 3).value);
        }
    }

    #[test]
    fn rejects_wrong_width() {
        let mut builder = DigitalBuilder::new();
        let input = named_inputs(&mut builder, "a", 7);
        let carry = builder.input("cin");
        let op = named_inputs(&mut builder, "op", 2);
        assert!(shift_unit(&mut builder, &input, carry, &op).is_err());
    }
}
