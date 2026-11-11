//! AVR arithmetic unit.

use sim_cells::{DigitalBuilder, NetId, and_all, digital_full_adder, or_all};
use sim_core::SimError;

/// Result and flags of the AVR arithmetic unit.
pub struct ArithmeticUnit {
    /// Result bits, least significant first.
    pub result: Vec<NetId>,
    /// Carry or borrow.
    pub carry: NetId,
    /// Result is zero.
    pub zero: NetId,
    /// Result bit 7.
    pub negative: NetId,
    /// Signed overflow.
    pub overflow: NetId,
    /// Signed result sign.
    pub sign: NetId,
    /// Carry or borrow from bit 3.
    pub half_carry: NetId,
}

struct SumBits {
    sum: Vec<NetId>,
    carry: NetId,
    half_carry: NetId,
}

/// Builds the 8-bit AVR arithmetic unit.
///
/// `subtract` selects subtraction. `carry_in` is the C flag for ADC or SBC.
pub fn arithmetic_unit(
    builder: &mut DigitalBuilder,
    first: &[NetId],
    second: &[NetId],
    subtract: NetId,
    carry_in: NetId,
) -> Result<ArithmeticUnit, SimError> {
    validate(first, second)?;
    let bits = add_bytes(builder, first, second, subtract, carry_in);
    let zero = zero_flag(builder, &bits.sum);
    let negative = bits.sum[7];
    let overflow = overflow_flag(builder, first, second, &bits.sum, subtract);
    let sign = builder.xor(negative, overflow);
    Ok(ArithmeticUnit {
        result: bits.sum,
        carry: carry_flag(builder, bits.carry, subtract),
        zero,
        negative,
        overflow,
        sign,
        half_carry: carry_flag(builder, bits.half_carry, subtract),
    })
}

fn validate(first: &[NetId], second: &[NetId]) -> Result<(), SimError> {
    if first.len() != 8 || second.len() != 8 {
        return Err(SimError::InvalidNetlist(
            "AVR arithmetic expects 8-bit operands".to_string(),
        ));
    }
    Ok(())
}

fn add_bytes(
    builder: &mut DigitalBuilder,
    first: &[NetId],
    second: &[NetId],
    subtract: NetId,
    carry_in: NetId,
) -> SumBits {
    let mut carry = adder_carry(builder, carry_in, subtract);
    let mut half_carry = carry;
    let mut sum = Vec::with_capacity(8);
    for index in 0..8 {
        let bit = effective_bit(builder, second[index], subtract);
        let (result, next) = digital_full_adder(builder, first[index], bit, carry);
        if index == 3 {
            half_carry = next;
        }
        sum.push(result);
        carry = next;
    }
    SumBits {
        sum,
        carry,
        half_carry,
    }
}

fn effective_bit(builder: &mut DigitalBuilder, bit: NetId, subtract: NetId) -> NetId {
    let inverted = builder.not(bit);
    builder.mux(bit, inverted, subtract)
}

fn adder_carry(builder: &mut DigitalBuilder, carry_in: NetId, subtract: NetId) -> NetId {
    let inverted = builder.not(carry_in);
    builder.mux(carry_in, inverted, subtract)
}

fn carry_flag(builder: &mut DigitalBuilder, value: NetId, subtract: NetId) -> NetId {
    let inverted = builder.not(value);
    builder.mux(value, inverted, subtract)
}

fn zero_flag(builder: &mut DigitalBuilder, result: &[NetId]) -> NetId {
    let any = or_all(builder, result);
    builder.not(any)
}

fn overflow_flag(
    builder: &mut DigitalBuilder,
    first: &[NetId],
    second: &[NetId],
    result: &[NetId],
    subtract: NetId,
) -> NetId {
    let addition = addition_overflow(builder, first[7], second[7], result[7]);
    let subtraction = subtraction_overflow(builder, first[7], second[7], result[7]);
    builder.mux(addition, subtraction, subtract)
}

fn addition_overflow(
    builder: &mut DigitalBuilder,
    first: NetId,
    second: NetId,
    result: NetId,
) -> NetId {
    let first_bar = builder.not(first);
    let second_bar = builder.not(second);
    let result_bar = builder.not(result);
    let same = and_all(builder, &[first, second, result_bar]);
    let low = and_all(builder, &[first_bar, second_bar, result]);
    or_all(builder, &[same, low])
}

fn subtraction_overflow(
    builder: &mut DigitalBuilder,
    first: NetId,
    second: NetId,
    result: NetId,
) -> NetId {
    let first_bar = builder.not(first);
    let second_bar = builder.not(second);
    let result_bar = builder.not(result);
    let negative = and_all(builder, &[first, second_bar, result_bar]);
    let positive = and_all(builder, &[first_bar, second, result]);
    or_all(builder, &[negative, positive])
}

#[cfg(test)]
mod tests {
    use super::{ArithmeticUnit, arithmetic_unit};
    use crate::test_support::{named_inputs, set_bits};
    use sim_cells::{DigitalBuilder, DigitalCircuit, add_with_carry, read_bits, sub_with_borrow};

    #[derive(Debug, PartialEq, Eq)]
    struct Outcome {
        value: u8,
        carry: bool,
        zero: bool,
        negative: bool,
        overflow: bool,
        sign: bool,
        half_carry: bool,
    }

    fn fixture() -> (DigitalCircuit, ArithmeticUnit) {
        let mut builder = DigitalBuilder::new();
        let first = named_inputs(&mut builder, "a", 8);
        let second = named_inputs(&mut builder, "b", 8);
        let subtract = builder.input("sub");
        let carry = builder.input("cin");
        let unit = arithmetic_unit(&mut builder, &first, &second, subtract, carry).expect("unit");
        (builder.build(), unit)
    }

    fn evaluate(
        circuit: &mut DigitalCircuit,
        unit: &ArithmeticUnit,
        first: u8,
        second: u8,
        subtract: bool,
        carry_in: bool,
    ) -> Outcome {
        set_bits(circuit, "a", u64::from(first), 8);
        set_bits(circuit, "b", u64::from(second), 8);
        circuit.set_input("sub", subtract).expect("sub");
        circuit.set_input("cin", carry_in).expect("cin");
        circuit.settle().expect("settle");
        Outcome {
            value: read_bits(circuit, &unit.result) as u8,
            carry: circuit.value(unit.carry),
            zero: circuit.value(unit.zero),
            negative: circuit.value(unit.negative),
            overflow: circuit.value(unit.overflow),
            sign: circuit.value(unit.sign),
            half_carry: circuit.value(unit.half_carry),
        }
    }

    fn addition(first: u8, second: u8, carry_in: bool) -> Outcome {
        let result = add_with_carry(first, second, carry_in);
        Outcome {
            value: result.value,
            carry: result.flags.carry,
            zero: result.flags.zero,
            negative: result.flags.negative,
            overflow: result.flags.overflow,
            sign: result.flags.sign,
            half_carry: result.flags.half_carry,
        }
    }

    fn subtraction(first: u8, second: u8, borrow: bool) -> Outcome {
        let result = sub_with_borrow(first, second, borrow);
        Outcome {
            value: result.value,
            carry: result.flags.carry,
            zero: result.flags.zero,
            negative: result.flags.negative,
            overflow: result.flags.overflow,
            sign: result.flags.sign,
            half_carry: result.flags.half_carry,
        }
    }

    #[test]
    fn addition_matches_reference() {
        let (mut circuit, unit) = fixture();
        let values = [0, 1, 0x0F, 0x10, 0x7F, 0x80, 0xFF];
        for first in 0..=u8::MAX {
            for second in values {
                for carry in [false, true] {
                    let actual = evaluate(&mut circuit, &unit, first, second, false, carry);
                    assert_eq!(actual, addition(first, second, carry));
                }
            }
        }
    }

    #[test]
    fn subtraction_matches_reference() {
        let (mut circuit, unit) = fixture();
        let values = [0, 1, 0x0F, 0x10, 0x7F, 0x80, 0xFF];
        for first in 0..=u8::MAX {
            for second in values {
                for borrow in [false, true] {
                    let actual = evaluate(&mut circuit, &unit, first, second, true, borrow);
                    assert_eq!(actual, subtraction(first, second, borrow));
                }
            }
        }
    }

    #[test]
    fn rejects_wrong_operand_width() {
        let mut builder = DigitalBuilder::new();
        let first = named_inputs(&mut builder, "a", 8);
        let second = named_inputs(&mut builder, "b", 7);
        let subtract = builder.input("sub");
        let carry = builder.input("cin");
        assert!(arithmetic_unit(&mut builder, &first, &second, subtract, carry).is_err());
    }
}
