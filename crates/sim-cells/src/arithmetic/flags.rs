//! Arithmetic status flags.

/// Status flags produced by arithmetic operations.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AluFlags {
    /// Carry or borrow.
    pub carry: bool,
    /// Result is zero.
    pub zero: bool,
    /// Result bit 7 is set.
    pub negative: bool,
    /// Signed overflow.
    pub overflow: bool,
    /// Signed result sign, `negative ^ overflow`.
    pub sign: bool,
    /// Carry or borrow from bit 3.
    pub half_carry: bool,
}

/// Result value and status flags.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AluResult {
    /// Result byte.
    pub value: u8,
    /// Status flags.
    pub flags: AluFlags,
}

/// Adds two bytes with a carry input.
#[must_use]
pub fn add_with_carry(first: u8, second: u8, carry_in: bool) -> AluResult {
    let wide = u16::from(first) + u16::from(second) + u16::from(carry_in);
    let value = wide as u8;
    let half_carry = low_nibble_sum(first, second, carry_in) > 0x0F;
    let flags = addition_flags(first, second, value, wide > 0xFF, half_carry);
    AluResult { value, flags }
}

/// Subtracts two bytes with a borrow input.
#[must_use]
pub fn sub_with_borrow(first: u8, second: u8, borrow_in: bool) -> AluResult {
    let wide = i16::from(first) - i16::from(second) - i16::from(borrow_in);
    let value = wide as u8;
    let half_carry = low_nibble(first) < low_nibble(second) + u8::from(borrow_in);
    let flags = subtraction_flags(first, second, value, wide < 0, half_carry);
    AluResult { value, flags }
}

fn low_nibble(value: u8) -> u8 {
    value & 0x0F
}

fn low_nibble_sum(first: u8, second: u8, carry_in: bool) -> u8 {
    low_nibble(first) + low_nibble(second) + u8::from(carry_in)
}

fn addition_flags(first: u8, second: u8, value: u8, carry: bool, half_carry: bool) -> AluFlags {
    let overflow = sign_bit(first) == sign_bit(second) && sign_bit(value) != sign_bit(first);
    build_flags(carry, value == 0, sign_bit(value), overflow, half_carry)
}

fn subtraction_flags(
    first: u8,
    second: u8,
    value: u8,
    borrow: bool,
    half_borrow: bool,
) -> AluFlags {
    let overflow = sign_bit(first) != sign_bit(second) && sign_bit(value) != sign_bit(first);
    build_flags(borrow, value == 0, sign_bit(value), overflow, half_borrow)
}

fn build_flags(
    carry: bool,
    zero: bool,
    negative: bool,
    overflow: bool,
    half_carry: bool,
) -> AluFlags {
    AluFlags {
        carry,
        zero,
        negative,
        overflow,
        sign: negative ^ overflow,
        half_carry,
    }
}

fn sign_bit(value: u8) -> bool {
    value & 0x80 != 0
}

#[cfg(test)]
mod tests {
    use super::{add_with_carry, sub_with_borrow};

    #[test]
    fn addition_matches_wrapping_arithmetic() {
        for first in 0..=u8::MAX {
            for second in 0..=u8::MAX {
                for carry in [false, true] {
                    let result = add_with_carry(first, second, carry);
                    let wide = u16::from(first) + u16::from(second) + u16::from(carry);
                    assert_eq!(result.value, wide as u8);
                    assert_eq!(result.flags.carry, wide > 0xFF);
                }
            }
        }
    }

    #[test]
    fn subtraction_matches_wrapping_arithmetic() {
        for first in 0..=u8::MAX {
            for second in 0..=u8::MAX {
                for borrow in [false, true] {
                    let result = sub_with_borrow(first, second, borrow);
                    let wide = i16::from(first) - i16::from(second) - i16::from(borrow);
                    assert_eq!(result.value, wide as u8);
                    assert_eq!(result.flags.carry, wide < 0);
                }
            }
        }
    }

    #[test]
    fn addition_sets_half_carry_at_nibble_boundary() {
        assert!(add_with_carry(0x0F, 0x01, false).flags.half_carry);
        assert!(add_with_carry(0x0F, 0x00, true).flags.half_carry);
        assert!(!add_with_carry(0x0E, 0x01, false).flags.half_carry);
    }

    #[test]
    fn addition_overflow_follows_signed_range() {
        let positive = add_with_carry(0x7F, 0x01, false);
        assert!(positive.flags.overflow && positive.flags.negative);
        assert!(!positive.flags.sign);
        let negative = add_with_carry(0x80, 0x80, false);
        assert!(negative.flags.overflow && negative.flags.zero);
    }

    #[test]
    fn subtraction_sets_borrow_at_nibble_boundary() {
        assert!(sub_with_borrow(0x10, 0x01, false).flags.half_carry);
        assert!(sub_with_borrow(0x00, 0x00, true).flags.half_carry);
        assert!(!sub_with_borrow(0x10, 0x00, false).flags.half_carry);
    }

    #[test]
    fn subtraction_overflow_follows_signed_range() {
        let underflow = sub_with_borrow(0x80, 0x01, false);
        assert!(underflow.flags.overflow && !underflow.flags.negative);
        assert!(underflow.flags.sign);
        let positive = sub_with_borrow(0x7F, 0xFF, false);
        assert!(positive.flags.overflow && positive.flags.negative);
    }

    #[test]
    fn zero_flag_marks_zero_results() {
        assert!(add_with_carry(0xFF, 0x01, false).flags.zero);
        assert!(sub_with_borrow(0x2A, 0x2A, false).flags.zero);
        assert!(!add_with_carry(0x00, 0x01, false).flags.zero);
    }

    #[test]
    fn sign_flag_is_negative_xor_overflow() {
        for value in 0..=u8::MAX {
            let result = add_with_carry(value, value, true);
            assert_eq!(
                result.flags.sign,
                result.flags.negative ^ result.flags.overflow
            );
        }
    }
}
