//! AVR opcode decoder.

use super::instruction::{Instruction, Mnemonic, Operand};
use sim_core::SimError;

/// Returns true when the instruction occupies two words.
#[must_use]
pub fn uses_second_word(first: u16) -> bool {
    matches!(first & 0xFE0F, 0x9000 | 0x9200) || matches!(first & 0xFE0E, 0x940C | 0x940E)
}

/// Decodes one AVR instruction.
pub fn decode(first: u16, second: Option<u16>) -> Result<Instruction, SimError> {
    match first & 0xF000 {
        0x0000 => decode_group_0(first),
        0x1000 => decode_group_1(first),
        0x2000 => decode_group_2(first),
        0x3000 => Ok(immediate(first, Mnemonic::Cpi)),
        0x4000 => Ok(immediate(first, Mnemonic::Sbci)),
        0x5000 => Ok(immediate(first, Mnemonic::Subi)),
        0x6000 => Ok(immediate(first, Mnemonic::Ori)),
        0x7000 => Ok(immediate(first, Mnemonic::Andi)),
        0x8000 | 0xA000 => decode_ldd(first),
        0x9000 => decode_group_9(first, second),
        0xB000 => decode_in_out(first),
        0xC000 => Ok(relative(first, Mnemonic::Rjmp)),
        0xD000 => Ok(relative(first, Mnemonic::Rcall)),
        0xE000 => Ok(immediate(first, Mnemonic::Ldi)),
        0xF000 => decode_group_f(first),
        _ => Err(illegal(first)),
    }
}

fn decode_group_0(first: u16) -> Result<Instruction, SimError> {
    if first == 0 {
        return Ok(fixed(Mnemonic::Nop));
    }
    match first & 0xFC00 {
        0x0400 => Ok(two_reg(first, Mnemonic::Cpc)),
        0x0800 => Ok(two_reg(first, Mnemonic::Sbc)),
        0x0C00 => Ok(two_reg(first, Mnemonic::Add)),
        0x0000 => decode_group_0_misc(first),
        _ => Err(illegal(first)),
    }
}

fn decode_group_0_misc(first: u16) -> Result<Instruction, SimError> {
    match first & 0xFF00 {
        0x0100 => Ok(movw(first)),
        0x0200 => Ok(muls(first)),
        0x0300 => Ok(mul_family(first)),
        _ => Err(illegal(first)),
    }
}

fn decode_group_1(first: u16) -> Result<Instruction, SimError> {
    match first & 0xFC00 {
        0x1000 => Ok(two_reg(first, Mnemonic::Cpse)),
        0x1400 => Ok(two_reg(first, Mnemonic::Cp)),
        0x1800 => Ok(two_reg(first, Mnemonic::Sub)),
        0x1C00 => Ok(two_reg(first, Mnemonic::Adc)),
        _ => Err(illegal(first)),
    }
}

fn decode_group_2(first: u16) -> Result<Instruction, SimError> {
    match first & 0xFC00 {
        0x2000 => Ok(two_reg(first, Mnemonic::And)),
        0x2400 => Ok(two_reg(first, Mnemonic::Eor)),
        0x2800 => Ok(two_reg(first, Mnemonic::Or)),
        0x2C00 => Ok(two_reg(first, Mnemonic::Mov)),
        _ => Err(illegal(first)),
    }
}

fn decode_ldd(first: u16) -> Result<Instruction, SimError> {
    let store = first & 0x0200 != 0;
    let y = first & 0x0008 != 0;
    let mnemonic = match (store, y) {
        (false, false) => Mnemonic::LddZ,
        (false, true) => Mnemonic::LddY,
        (true, false) => Mnemonic::StdZ,
        (true, true) => Mnemonic::StdY,
    };
    let d = register_index(first);
    let q = (((first & 0x2000) >> 8) | ((first & 0x0C00) >> 7) | (first & 0x7)) as u8;
    Ok(if store {
        Instruction::new(mnemonic, Operand::Displacement(q), Operand::Register(d), 1)
    } else {
        Instruction::new(mnemonic, Operand::Register(d), Operand::Displacement(q), 1)
    })
}

fn decode_group_9(first: u16, second: Option<u16>) -> Result<Instruction, SimError> {
    if first & 0xFF0F == 0x9408 {
        return Ok(sreg_bit(first));
    }
    if let Some(instruction) = fixed_9(first) {
        return Ok(instruction);
    }
    match first & 0xFE0F {
        0x9000 => lds(first, second),
        0x9200 => sts(first, second),
        _ => decode_9_masked(first, second),
    }
}

fn fixed_9(first: u16) -> Option<Instruction> {
    let mnemonic = match first {
        0x9409 => Mnemonic::Ijmp,
        0x9419 => Mnemonic::Eijmp,
        0x9509 => Mnemonic::Icall,
        0x9519 => Mnemonic::Eicall,
        0x9508 => Mnemonic::Ret,
        0x9518 => Mnemonic::Reti,
        0x9588 => Mnemonic::Sleep,
        0x9598 => Mnemonic::Break,
        0x95A8 => Mnemonic::Wdr,
        0x95E8 => Mnemonic::Spm,
        0x95C8 => return Some(lpm_fixed(Mnemonic::LpmZ)),
        0x95D8 => return Some(lpm_fixed(Mnemonic::ElpmZ)),
        _ => return None,
    };
    Some(fixed(mnemonic))
}

fn decode_9_masked(first: u16, second: Option<u16>) -> Result<Instruction, SimError> {
    if let Some(instruction) = decode_9_memory(first) {
        return Ok(instruction);
    }
    if let Some(instruction) = decode_9_alu(first, second)? {
        return Ok(instruction);
    }
    decode_9_io(first)
}

fn decode_9_memory(first: u16) -> Option<Instruction> {
    if let Some(instruction) = load_memory(first) {
        return Some(instruction);
    }
    store_memory(first)
}

fn load_memory(first: u16) -> Option<Instruction> {
    let mnemonic = match first & 0xFE0F {
        0x9001 => Mnemonic::LdZPost,
        0x9002 => Mnemonic::LdZPre,
        0x9004 => Mnemonic::LpmZ,
        0x9005 => Mnemonic::LpmZPost,
        0x9006 => Mnemonic::ElpmZ,
        0x9007 => Mnemonic::ElpmZPost,
        0x9009 => Mnemonic::LdYPost,
        0x900A => Mnemonic::LdYPre,
        0x900C => Mnemonic::LdX,
        0x900D => Mnemonic::LdXPost,
        0x900E => Mnemonic::LdXPre,
        0x900F => Mnemonic::Pop,
        _ => return None,
    };
    Some(register_instruction(first, mnemonic))
}

fn store_memory(first: u16) -> Option<Instruction> {
    let mnemonic = match first & 0xFE0F {
        0x9201 => Mnemonic::StZPost,
        0x9202 => Mnemonic::StZPre,
        0x9209 => Mnemonic::StYPost,
        0x920A => Mnemonic::StYPre,
        0x920C => Mnemonic::StX,
        0x920D => Mnemonic::StXPost,
        0x920E => Mnemonic::StXPre,
        0x920F => Mnemonic::Push,
        _ => return None,
    };
    Some(register_instruction(first, mnemonic))
}

fn decode_9_alu(first: u16, second: Option<u16>) -> Result<Option<Instruction>, SimError> {
    let mnemonic = match first & 0xFE0F {
        0x9400 => Mnemonic::Com,
        0x9401 => Mnemonic::Neg,
        0x9402 => Mnemonic::Swap,
        0x9403 => Mnemonic::Inc,
        0x9405 => Mnemonic::Asr,
        0x9406 => Mnemonic::Lsr,
        0x9407 => Mnemonic::Ror,
        0x940A => Mnemonic::Dec,
        _ => return decode_9_long(first, second),
    };
    Ok(Some(register_instruction(first, mnemonic)))
}

fn decode_9_long(first: u16, second: Option<u16>) -> Result<Option<Instruction>, SimError> {
    let mnemonic = match first & 0xFE0E {
        0x940C => Mnemonic::Jmp,
        0x940E => Mnemonic::Call,
        _ => return Ok(None),
    };
    let next = second.ok_or_else(|| truncated(first))?;
    let high = (((first & 0x01F0) >> 3) | (first & 1)) as u32;
    let address = (high << 16) | u32::from(next);
    Ok(Some(Instruction::new(
        mnemonic,
        Operand::LongAddress(address),
        Operand::None,
        2,
    )))
}

fn decode_9_io(first: u16) -> Result<Instruction, SimError> {
    if first & 0xFC00 == 0x9C00 {
        return Ok(two_reg(first, Mnemonic::Mul));
    }
    match first & 0xFF00 {
        0x9600 => Ok(word_immediate(first, Mnemonic::Adiw)),
        0x9700 => Ok(word_immediate(first, Mnemonic::Sbiw)),
        0x9800 => Ok(io_bit(first, Mnemonic::Cbi)),
        0x9900 => Ok(io_bit(first, Mnemonic::Sbic)),
        0x9A00 => Ok(io_bit(first, Mnemonic::Sbi)),
        0x9B00 => Ok(io_bit(first, Mnemonic::Sbis)),
        _ => Err(illegal(first)),
    }
}

fn decode_group_f(first: u16) -> Result<Instruction, SimError> {
    match first & 0xFE00 {
        0xF000 | 0xF200 | 0xF400 | 0xF600 => Ok(branch(first)),
        0xF800 | 0xF900 => Ok(bit_register(first, Mnemonic::Bld)),
        0xFA00 | 0xFB00 => Ok(bit_register(first, Mnemonic::Bst)),
        0xFC00 | 0xFE00 => Ok(skip_bit(first)),
        _ => Err(illegal(first)),
    }
}

fn decode_in_out(first: u16) -> Result<Instruction, SimError> {
    let io = ((((first >> 9) & 0x3) << 4) | (first & 0xF)) as u8;
    let d = register_index(first);
    match first & 0xF800 {
        0xB000 => Ok(Instruction::new(
            Mnemonic::In,
            Operand::Register(d),
            Operand::Io(io),
            1,
        )),
        0xB800 => Ok(Instruction::new(
            Mnemonic::Out,
            Operand::Io(io),
            Operand::Register(d),
            1,
        )),
        _ => Err(illegal(first)),
    }
}

fn two_reg(first: u16, mnemonic: Mnemonic) -> Instruction {
    let d = register_index(first);
    let r = (((first >> 5) & 0x10) | (first & 0xF)) as u8;
    Instruction::new(mnemonic, Operand::Register(d), Operand::Register(r), 1)
}

fn immediate(first: u16, mnemonic: Mnemonic) -> Instruction {
    let d = 16 + ((first >> 4) & 0xF) as u8;
    let k = (((first & 0x0F00) >> 4) | (first & 0xF)) as u8;
    Instruction::new(mnemonic, Operand::Register(d), Operand::Immediate(k), 1)
}

fn movw(first: u16) -> Instruction {
    let d = (((first >> 4) & 0xF) << 1) as u8;
    let r = ((first & 0xF) << 1) as u8;
    Instruction::new(
        Mnemonic::Movw,
        Operand::Register(d),
        Operand::Register(r),
        1,
    )
}

fn muls(first: u16) -> Instruction {
    let d = 16 + ((first >> 4) & 0xF) as u8;
    let r = 16 + (first & 0xF) as u8;
    Instruction::new(
        Mnemonic::Muls,
        Operand::Register(d),
        Operand::Register(r),
        1,
    )
}

fn mul_family(first: u16) -> Instruction {
    let mnemonic = match first & 0x0088 {
        0x0000 => Mnemonic::Mulsu,
        0x0008 => Mnemonic::Fmul,
        0x0080 => Mnemonic::Fmuls,
        _ => Mnemonic::Fmulsu,
    };
    let d = 16 + ((first >> 4) & 0x7) as u8;
    let r = 16 + (first & 0x7) as u8;
    Instruction::new(mnemonic, Operand::Register(d), Operand::Register(r), 1)
}

fn word_immediate(first: u16, mnemonic: Mnemonic) -> Instruction {
    let d = 24 + ((first >> 3) & 0x6) as u8;
    let k = (((first & 0x00C0) >> 2) | (first & 0xF)) as u8;
    Instruction::new(mnemonic, Operand::Register(d), Operand::Immediate(k), 1)
}

fn io_bit(first: u16, mnemonic: Mnemonic) -> Instruction {
    let io = ((first >> 3) & 0x1F) as u8;
    let bit = (first & 0x7) as u8;
    Instruction::new(mnemonic, Operand::Io(io), Operand::Bit(bit), 1)
}

fn branch(first: u16) -> Instruction {
    let mnemonic = if first & 0x0400 == 0 {
        Mnemonic::Brbs
    } else {
        Mnemonic::Brbc
    };
    let flag = (first & 0x7) as u8;
    let offset = sign_extend((first >> 3) & 0x7F, 7);
    Instruction::new(mnemonic, Operand::Flag(flag), Operand::Offset(offset), 1)
}

fn bit_register(first: u16, mnemonic: Mnemonic) -> Instruction {
    Instruction::new(
        mnemonic,
        Operand::Register(register_index(first)),
        Operand::Bit((first & 0x7) as u8),
        1,
    )
}

fn skip_bit(first: u16) -> Instruction {
    let mnemonic = if first & 0x0200 != 0 {
        Mnemonic::Sbrs
    } else {
        Mnemonic::Sbrc
    };
    bit_register(first, mnemonic)
}

fn sreg_bit(first: u16) -> Instruction {
    let mnemonic = if first & 0x0080 == 0 {
        Mnemonic::Bset
    } else {
        Mnemonic::Bclr
    };
    Instruction::new(
        mnemonic,
        Operand::Flag(((first >> 4) & 0x7) as u8),
        Operand::None,
        1,
    )
}

fn relative(first: u16, mnemonic: Mnemonic) -> Instruction {
    let offset = sign_extend(first & 0x0FFF, 12);
    Instruction::new(mnemonic, Operand::Offset(offset), Operand::None, 1)
}

fn lds(first: u16, second: Option<u16>) -> Result<Instruction, SimError> {
    let address = second.ok_or_else(|| truncated(first))?;
    Ok(Instruction::new(
        Mnemonic::Lds,
        Operand::Register(register_index(first)),
        Operand::Address(address),
        2,
    ))
}

fn sts(first: u16, second: Option<u16>) -> Result<Instruction, SimError> {
    let address = second.ok_or_else(|| truncated(first))?;
    Ok(Instruction::new(
        Mnemonic::Sts,
        Operand::Register(register_index(first)),
        Operand::Address(address),
        2,
    ))
}

fn register_instruction(first: u16, mnemonic: Mnemonic) -> Instruction {
    Instruction::new(
        mnemonic,
        Operand::Register(register_index(first)),
        Operand::None,
        1,
    )
}

fn lpm_fixed(mnemonic: Mnemonic) -> Instruction {
    Instruction::new(mnemonic, Operand::Register(0), Operand::None, 1)
}

fn fixed(mnemonic: Mnemonic) -> Instruction {
    Instruction::new(mnemonic, Operand::None, Operand::None, 1)
}

fn register_index(first: u16) -> u8 {
    ((first >> 4) & 0x1F) as u8
}

fn sign_extend(value: u16, bits: u16) -> i16 {
    let shift = 16 - bits;
    ((value << shift) as i16) >> shift
}

fn illegal(first: u16) -> SimError {
    SimError::Parse(format!("illegal AVR opcode 0x{first:04X}"))
}

fn truncated(first: u16) -> SimError {
    SimError::Parse(format!("missing second word for 0x{first:04X}"))
}

#[cfg(test)]
mod tests {
    use super::{decode, uses_second_word};
    use crate::instruction::{Instruction, Mnemonic, Operand};

    fn two_reg(base: u16, d: u8, r: u8) -> u16 {
        base | ((u16::from(r & 0x10)) << 5) | (u16::from(d) << 4) | u16::from(r & 0xF)
    }

    fn immediate(base: u16, d: u8, k: u8) -> u16 {
        base | (u16::from(k & 0xF0) << 4) | (u16::from(d - 16) << 4) | u16::from(k & 0xF)
    }

    fn branch(base: u16, flag: u8, offset: i8) -> u16 {
        base | ((u16::from(offset as u8) & 0x7F) << 3) | u16::from(flag)
    }

    fn relative(base: u16, offset: i16) -> u16 {
        base | (offset as u16 & 0x0FFF)
    }

    fn reg(mnemonic: Mnemonic, d: u8, r: u8) -> Instruction {
        Instruction::new(mnemonic, Operand::Register(d), Operand::Register(r), 1)
    }

    fn one(mnemonic: Mnemonic, d: u8) -> Instruction {
        Instruction::new(mnemonic, Operand::Register(d), Operand::None, 1)
    }

    #[test]
    fn decodes_arithmetic_register_ops() {
        let cases = [
            (0x0C00, Mnemonic::Add),
            (0x1C00, Mnemonic::Adc),
            (0x1800, Mnemonic::Sub),
            (0x0800, Mnemonic::Sbc),
            (0x2000, Mnemonic::And),
            (0x2800, Mnemonic::Or),
            (0x2400, Mnemonic::Eor),
            (0x1400, Mnemonic::Cp),
            (0x0400, Mnemonic::Cpc),
            (0x1000, Mnemonic::Cpse),
            (0x2C00, Mnemonic::Mov),
            (0x9C00, Mnemonic::Mul),
        ];
        for (base, mnemonic) in cases {
            let word = two_reg(base, 17, 22);
            assert_eq!(decode(word, None).expect("decode"), reg(mnemonic, 17, 22));
        }
    }

    #[test]
    fn decodes_immediate_ops() {
        let cases = [
            (0xE000, Mnemonic::Ldi),
            (0x3000, Mnemonic::Cpi),
            (0x5000, Mnemonic::Subi),
            (0x4000, Mnemonic::Sbci),
            (0x7000, Mnemonic::Andi),
            (0x6000, Mnemonic::Ori),
        ];
        for (base, mnemonic) in cases {
            let word = immediate(base, 21, 0xA5);
            let expected =
                Instruction::new(mnemonic, Operand::Register(21), Operand::Immediate(0xA5), 1);
            assert_eq!(decode(word, None).expect("decode"), expected);
        }
    }

    #[test]
    fn decodes_single_register_ops() {
        let cases = [
            (0x9400, Mnemonic::Com),
            (0x9401, Mnemonic::Neg),
            (0x9402, Mnemonic::Swap),
            (0x9403, Mnemonic::Inc),
            (0x9405, Mnemonic::Asr),
            (0x9406, Mnemonic::Lsr),
            (0x9407, Mnemonic::Ror),
            (0x940A, Mnemonic::Dec),
        ];
        for (base, mnemonic) in cases {
            let word = base | (u16::from(13u8) << 4);
            assert_eq!(decode(word, None).expect("decode"), one(mnemonic, 13));
        }
    }

    #[test]
    fn decodes_branches_and_relative_jumps() {
        let word = branch(0xF000, 1, -5);
        let expected = Instruction::new(Mnemonic::Brbs, Operand::Flag(1), Operand::Offset(-5), 1);
        assert_eq!(decode(word, None).expect("brbs"), expected);
        let word = branch(0xF400, 2, 9);
        let expected = Instruction::new(Mnemonic::Brbc, Operand::Flag(2), Operand::Offset(9), 1);
        assert_eq!(decode(word, None).expect("brbc"), expected);
        let word = relative(0xC000, -2);
        assert_eq!(
            decode(word, None).expect("rjmp"),
            Instruction::new(Mnemonic::Rjmp, Operand::Offset(-2), Operand::None, 1)
        );
        let word = relative(0xD000, 7);
        assert_eq!(
            decode(word, None).expect("rcall"),
            Instruction::new(Mnemonic::Rcall, Operand::Offset(7), Operand::None, 1)
        );
    }

    #[test]
    fn decodes_ldd_and_std() {
        let word = 0x8000 | (u16::from(7u8) << 4) | 0x0003;
        let expected = Instruction::new(
            Mnemonic::LddZ,
            Operand::Register(7),
            Operand::Displacement(3),
            1,
        );
        assert_eq!(decode(word, None).expect("ldd"), expected);
        let word = 0x8208 | (u16::from(4u8) << 4) | 0x0005;
        let expected = Instruction::new(
            Mnemonic::StdY,
            Operand::Displacement(5),
            Operand::Register(4),
            1,
        );
        assert_eq!(decode(word, None).expect("std"), expected);
    }

    #[test]
    fn decodes_lds_and_sts() {
        let word = 0x9000 | (u16::from(9u8) << 4);
        let expected = Instruction::new(
            Mnemonic::Lds,
            Operand::Register(9),
            Operand::Address(0x1234),
            2,
        );
        assert_eq!(decode(word, Some(0x1234)).expect("lds"), expected);
        let word = 0x9200 | (u16::from(9u8) << 4);
        let expected = Instruction::new(
            Mnemonic::Sts,
            Operand::Register(9),
            Operand::Address(0x4321),
            2,
        );
        assert_eq!(decode(word, Some(0x4321)).expect("sts"), expected);
    }

    #[test]
    fn decodes_indirect_loads() {
        let cases = [
            (0x900C, Mnemonic::LdX),
            (0x900D, Mnemonic::LdXPost),
            (0x900E, Mnemonic::LdXPre),
            (0x9009, Mnemonic::LdYPost),
            (0x900A, Mnemonic::LdYPre),
            (0x9001, Mnemonic::LdZPost),
            (0x9002, Mnemonic::LdZPre),
            (0x900F, Mnemonic::Pop),
        ];
        for (base, mnemonic) in cases {
            let word = base | (u16::from(11u8) << 4);
            assert_eq!(decode(word, None).expect("decode"), one(mnemonic, 11));
        }
    }

    #[test]
    fn decodes_indirect_stores() {
        let cases = [
            (0x920C, Mnemonic::StX),
            (0x920D, Mnemonic::StXPost),
            (0x920E, Mnemonic::StXPre),
            (0x9209, Mnemonic::StYPost),
            (0x920A, Mnemonic::StYPre),
            (0x9201, Mnemonic::StZPost),
            (0x9202, Mnemonic::StZPre),
            (0x920F, Mnemonic::Push),
        ];
        for (base, mnemonic) in cases {
            let word = base | (u16::from(11u8) << 4);
            assert_eq!(decode(word, None).expect("decode"), one(mnemonic, 11));
        }
    }

    #[test]
    fn decodes_io_and_bit_ops() {
        let word = 0xB400 | (u16::from(5u8) << 4) | 0x0003;
        let expected = Instruction::new(Mnemonic::In, Operand::Register(5), Operand::Io(0x23), 1);
        assert_eq!(decode(word, None).expect("in"), expected);
        let word = 0xBC00 | (u16::from(5u8) << 4) | 0x0003;
        let expected = Instruction::new(Mnemonic::Out, Operand::Io(0x23), Operand::Register(5), 1);
        assert_eq!(decode(word, None).expect("out"), expected);
        let word = 0x9A00 | (u16::from(12u8) << 3) | 0x0006;
        let expected = Instruction::new(Mnemonic::Sbi, Operand::Io(12), Operand::Bit(6), 1);
        assert_eq!(decode(word, None).expect("sbi"), expected);
    }

    #[test]
    fn decodes_long_jump_and_call() {
        let expected = Instruction::new(
            Mnemonic::Jmp,
            Operand::LongAddress(0x001234),
            Operand::None,
            2,
        );
        assert_eq!(decode(0x940C, Some(0x1234)).expect("jmp"), expected);
        let expected = Instruction::new(
            Mnemonic::Call,
            Operand::LongAddress(0x011234),
            Operand::None,
            2,
        );
        assert_eq!(decode(0x940F, Some(0x1234)).expect("call"), expected);
    }

    #[test]
    fn decodes_fixed_ops() {
        let cases = [
            (0x0000, Mnemonic::Nop),
            (0x9588, Mnemonic::Sleep),
            (0x9598, Mnemonic::Break),
            (0x95A8, Mnemonic::Wdr),
            (0x95E8, Mnemonic::Spm),
            (0x9508, Mnemonic::Ret),
            (0x9518, Mnemonic::Reti),
            (0x9409, Mnemonic::Ijmp),
            (0x9419, Mnemonic::Eijmp),
            (0x9509, Mnemonic::Icall),
            (0x9519, Mnemonic::Eicall),
        ];
        for (word, mnemonic) in cases {
            let expected = Instruction::new(mnemonic, Operand::None, Operand::None, 1);
            assert_eq!(decode(word, None).expect("decode"), expected);
        }
    }

    #[test]
    fn decodes_flag_and_bit_ops() {
        let expected = Instruction::new(Mnemonic::Bset, Operand::Flag(3), Operand::None, 1);
        assert_eq!(decode(0x9438, None).expect("bset"), expected);
        let expected = Instruction::new(Mnemonic::Bclr, Operand::Flag(3), Operand::None, 1);
        assert_eq!(decode(0x94B8, None).expect("bclr"), expected);
        let word = 0xF800 | (u16::from(6u8) << 4) | 0x0002;
        let expected = Instruction::new(Mnemonic::Bld, Operand::Register(6), Operand::Bit(2), 1);
        assert_eq!(decode(word, None).expect("bld"), expected);
        let word = 0xFC00 | (u16::from(6u8) << 4) | 0x0002;
        let expected = Instruction::new(Mnemonic::Sbrc, Operand::Register(6), Operand::Bit(2), 1);
        assert_eq!(decode(word, None).expect("sbrc"), expected);
    }

    #[test]
    fn detects_second_word_and_rejects_truncated() {
        assert!(uses_second_word(0x9000));
        assert!(uses_second_word(0x940C));
        assert!(!uses_second_word(0x0C00));
        assert!(decode(0x9000, None).is_err());
        assert!(decode(0x940C, None).is_err());
    }

    #[test]
    fn every_word_decodes_without_panic() {
        for word in 0..=u16::MAX {
            let _ = decode(word, Some(0));
        }
    }

    #[test]
    fn decoded_instructions_have_valid_ranges() {
        for word in 0..=u16::MAX {
            let Ok(instruction) = decode(word, Some(0)) else {
                continue;
            };
            assert_eq!(instruction.words == 2, uses_second_word(word));
            for operand in [instruction.first, instruction.second] {
                match operand {
                    Operand::Register(value) => assert!(value < 32),
                    Operand::Bit(value) | Operand::Flag(value) => assert!(value < 8),
                    Operand::Io(value) => assert!(value < 64),
                    Operand::Displacement(value) => assert!(value < 64),
                    _ => {}
                }
            }
        }
    }
}
