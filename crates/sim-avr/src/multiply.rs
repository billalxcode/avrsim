//! AVR multiply units.

use sim_cells::{DigitalBuilder, NetId, digital_full_adder, or_all};
use sim_core::SimError;

/// Result and flags of an AVR multiply unit.
pub struct MultiplyUnit {
    /// 16-bit result, least significant first.
    pub result: Vec<NetId>,
    /// Result is zero.
    pub zero: NetId,
    /// Result bit 15.
    pub carry: NetId,
}

/// Builds an unsigned 8 x 8 multiplier.
pub fn multiply_unit(
    builder: &mut DigitalBuilder,
    first: &[NetId],
    second: &[NetId],
) -> Result<MultiplyUnit, SimError> {
    validate(first, second)?;
    let product = unsigned_product(builder, first, second);
    Ok(finish_product(builder, product))
}

/// Builds a signed 8 x 8 multiplier.
pub fn multiply_signed_unit(
    builder: &mut DigitalBuilder,
    first: &[NetId],
    second: &[NetId],
) -> Result<MultiplyUnit, SimError> {
    validate(first, second)?;
    let first_abs = absolute(builder, first);
    let second_abs = absolute(builder, second);
    let product = unsigned_product(builder, &first_abs, &second_abs);
    let negative = builder.xor(first[7], second[7]);
    let product = conditional_negate(builder, &product, negative);
    Ok(finish_product(builder, product))
}

/// Builds a signed by unsigned 8 x 8 multiplier.
pub fn multiply_signed_unsigned_unit(
    builder: &mut DigitalBuilder,
    first: &[NetId],
    second: &[NetId],
) -> Result<MultiplyUnit, SimError> {
    validate(first, second)?;
    let first_abs = absolute(builder, first);
    let product = unsigned_product(builder, &first_abs, second);
    let product = conditional_negate(builder, &product, first[7]);
    Ok(finish_product(builder, product))
}

fn validate(first: &[NetId], second: &[NetId]) -> Result<(), SimError> {
    if first.len() != 8 || second.len() != 8 {
        return Err(SimError::InvalidNetlist(
            "AVR multiply expects 8-bit operands".to_string(),
        ));
    }
    Ok(())
}

fn unsigned_product(builder: &mut DigitalBuilder, first: &[NetId], second: &[NetId]) -> Vec<NetId> {
    let zero = builder.constant(false);
    let mut product = vec![zero; 16];
    for (shift, &bit) in second.iter().enumerate() {
        let row = partial_row(builder, first, bit, shift, zero);
        product = add_word(builder, &product, &row);
    }
    product
}

fn partial_row(
    builder: &mut DigitalBuilder,
    first: &[NetId],
    bit: NetId,
    shift: usize,
    zero: NetId,
) -> Vec<NetId> {
    (0..16)
        .map(|index| {
            if index < shift || index >= shift + 8 {
                zero
            } else {
                builder.and(first[index - shift], bit)
            }
        })
        .collect()
}

fn add_word(builder: &mut DigitalBuilder, first: &[NetId], second: &[NetId]) -> Vec<NetId> {
    let mut carry = builder.constant(false);
    let mut sum = Vec::with_capacity(16);
    for index in 0..16 {
        let (result, next) = digital_full_adder(builder, first[index], second[index], carry);
        sum.push(result);
        carry = next;
    }
    sum
}

fn absolute(builder: &mut DigitalBuilder, bits: &[NetId]) -> Vec<NetId> {
    let negated = negate_word(builder, bits);
    bits.iter()
        .zip(negated)
        .map(|(&value, negative)| builder.mux(value, negative, bits[7]))
        .collect()
}

fn negate_word(builder: &mut DigitalBuilder, bits: &[NetId]) -> Vec<NetId> {
    let mut carry = builder.constant(true);
    let mut output = Vec::with_capacity(bits.len());
    for &bit in bits {
        let inverted = builder.not(bit);
        output.push(builder.xor(inverted, carry));
        carry = builder.and(inverted, carry);
    }
    output
}

fn conditional_negate(
    builder: &mut DigitalBuilder,
    bits: &[NetId],
    condition: NetId,
) -> Vec<NetId> {
    let negated = negate_word(builder, bits);
    bits.iter()
        .zip(negated)
        .map(|(&value, negative)| builder.mux(value, negative, condition))
        .collect()
}

fn finish_product(builder: &mut DigitalBuilder, product: Vec<NetId>) -> MultiplyUnit {
    let any = or_all(builder, &product);
    let zero = builder.not(any);
    let carry = product[15];
    MultiplyUnit {
        result: product,
        zero,
        carry,
    }
}

#[cfg(test)]
mod tests {
    use super::{MultiplyUnit, multiply_signed_unit, multiply_signed_unsigned_unit, multiply_unit};
    use crate::test_support::{named_inputs, set_bits};
    use sim_cells::{DigitalBuilder, DigitalCircuit, NetId, read_bits};
    use sim_core::SimError;

    type UnitBuilder =
        fn(&mut DigitalBuilder, &[NetId], &[NetId]) -> Result<MultiplyUnit, SimError>;

    fn fixture(builder: UnitBuilder) -> (DigitalCircuit, MultiplyUnit) {
        let mut digital = DigitalBuilder::new();
        let first = named_inputs(&mut digital, "a", 8);
        let second = named_inputs(&mut digital, "b", 8);
        let unit = builder(&mut digital, &first, &second).expect("unit");
        (digital.build(), unit)
    }

    fn evaluate(
        circuit: &mut DigitalCircuit,
        unit: &MultiplyUnit,
        first: u8,
        second: u8,
    ) -> (u16, bool, bool) {
        set_bits(circuit, "a", u64::from(first), 8);
        set_bits(circuit, "b", u64::from(second), 8);
        circuit.settle().expect("settle");
        (
            read_bits(circuit, &unit.result) as u16,
            circuit.value(unit.zero),
            circuit.value(unit.carry),
        )
    }

    fn check_unsigned(circuit: &mut DigitalCircuit, unit: &MultiplyUnit) {
        let values = [0, 1, 2, 0x55, 0x7F, 0x80, 0xFF];
        for first in 0..=u8::MAX {
            for second in values {
                let expected = u16::from(first) * u16::from(second);
                let (value, zero, carry) = evaluate(circuit, unit, first, second);
                assert_eq!(value, expected);
                assert_eq!(zero, expected == 0);
                assert_eq!(carry, expected & 0x8000 != 0);
            }
        }
    }

    #[test]
    fn mul_matches_reference() {
        let (mut circuit, unit) = fixture(multiply_unit);
        check_unsigned(&mut circuit, &unit);
    }

    #[test]
    fn muls_matches_reference() {
        let (mut circuit, unit) = fixture(multiply_signed_unit);
        let values = [0, 1, 2, 0x55, 0x7F, 0x80, 0xFF];
        for first in 0..=u8::MAX {
            for second in values {
                let expected = i16::from(first as i8) * i16::from(second as i8);
                let expected = expected as u16;
                let (value, zero, carry) = evaluate(&mut circuit, &unit, first, second);
                assert_eq!(value, expected);
                assert_eq!(zero, expected == 0);
                assert_eq!(carry, expected & 0x8000 != 0);
            }
        }
    }

    #[test]
    fn mulsu_matches_reference() {
        let (mut circuit, unit) = fixture(multiply_signed_unsigned_unit);
        let values = [0, 1, 2, 0x55, 0x7F, 0x80, 0xFF];
        for first in 0..=u8::MAX {
            for second in values {
                let expected = i16::from(first as i8) * i16::from(second);
                let expected = expected as u16;
                let (value, zero, carry) = evaluate(&mut circuit, &unit, first, second);
                assert_eq!(value, expected);
                assert_eq!(zero, expected == 0);
                assert_eq!(carry, expected & 0x8000 != 0);
            }
        }
    }

    #[test]
    fn multiply_rejects_wrong_width() {
        let mut builder = DigitalBuilder::new();
        let first = named_inputs(&mut builder, "a", 8);
        let second = named_inputs(&mut builder, "b", 7);
        assert!(multiply_unit(&mut builder, &first, &second).is_err());
    }
}
