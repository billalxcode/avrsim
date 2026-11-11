//! AVR disassembler.

use super::instruction::{Instruction, Mnemonic, Operand};

/// Disassembles one AVR instruction.
#[must_use]
pub fn disassemble(instruction: Instruction) -> String {
    let name = mnemonic_name(instruction.mnemonic);
    let operands = operands(instruction);
    if operands.is_empty() {
        name.to_string()
    } else {
        format!("{name} {operands}")
    }
}

fn mnemonic_name(mnemonic: Mnemonic) -> &'static str {
    arithmetic_name(mnemonic)
        .or_else(|| logic_name(mnemonic))
        .or_else(|| flow_name(mnemonic))
        .or_else(|| load_store_name(mnemonic))
        .or_else(|| transfer_name(mnemonic))
        .or_else(|| misc_name(mnemonic))
        .unwrap_or("???")
}

fn arithmetic_name(mnemonic: Mnemonic) -> Option<&'static str> {
    arithmetic_name_1(mnemonic).or_else(|| arithmetic_name_2(mnemonic))
}

fn arithmetic_name_1(mnemonic: Mnemonic) -> Option<&'static str> {
    Some(match mnemonic {
        Mnemonic::Add => "add",
        Mnemonic::Adc => "adc",
        Mnemonic::Adiw => "adiw",
        Mnemonic::Sub => "sub",
        Mnemonic::Subi => "subi",
        Mnemonic::Sbc => "sbc",
        Mnemonic::Sbci => "sbci",
        Mnemonic::Sbiw => "sbiw",
        Mnemonic::Mul => "mul",
        _ => return None,
    })
}

fn arithmetic_name_2(mnemonic: Mnemonic) -> Option<&'static str> {
    Some(match mnemonic {
        Mnemonic::Muls => "muls",
        Mnemonic::Mulsu => "mulsu",
        Mnemonic::Fmul => "fmul",
        Mnemonic::Fmuls => "fmuls",
        Mnemonic::Fmulsu => "fmulsu",
        Mnemonic::Neg => "neg",
        Mnemonic::Inc => "inc",
        Mnemonic::Dec => "dec",
        _ => return None,
    })
}

fn logic_name(mnemonic: Mnemonic) -> Option<&'static str> {
    Some(match mnemonic {
        Mnemonic::And => "and",
        Mnemonic::Andi => "andi",
        Mnemonic::Or => "or",
        Mnemonic::Ori => "ori",
        Mnemonic::Eor => "eor",
        Mnemonic::Com => "com",
        Mnemonic::Cp => "cp",
        Mnemonic::Cpc => "cpc",
        Mnemonic::Cpi => "cpi",
        Mnemonic::Cpse => "cpse",
        _ => return None,
    })
}

fn flow_name(mnemonic: Mnemonic) -> Option<&'static str> {
    flow_control_name(mnemonic).or_else(|| branch_bit_name(mnemonic))
}

fn flow_control_name(mnemonic: Mnemonic) -> Option<&'static str> {
    Some(match mnemonic {
        Mnemonic::Rjmp => "rjmp",
        Mnemonic::Rcall => "rcall",
        Mnemonic::Ijmp => "ijmp",
        Mnemonic::Eijmp => "eijmp",
        Mnemonic::Jmp => "jmp",
        Mnemonic::Icall => "icall",
        Mnemonic::Eicall => "eicall",
        Mnemonic::Call => "call",
        Mnemonic::Ret => "ret",
        Mnemonic::Reti => "reti",
        _ => return None,
    })
}

fn branch_bit_name(mnemonic: Mnemonic) -> Option<&'static str> {
    Some(match mnemonic {
        Mnemonic::Brbs => "brbs",
        Mnemonic::Brbc => "brbc",
        Mnemonic::Sbrc => "sbrc",
        Mnemonic::Sbrs => "sbrs",
        Mnemonic::Bst => "bst",
        Mnemonic::Bld => "bld",
        Mnemonic::Bset => "bset",
        Mnemonic::Bclr => "bclr",
        _ => return None,
    })
}

fn load_store_name(mnemonic: Mnemonic) -> Option<&'static str> {
    Some(match mnemonic {
        Mnemonic::LdX | Mnemonic::LdXPost | Mnemonic::LdXPre => "ld",
        Mnemonic::LdY | Mnemonic::LdYPost | Mnemonic::LdYPre => "ld",
        Mnemonic::LdZ | Mnemonic::LdZPost | Mnemonic::LdZPre => "ld",
        Mnemonic::LddY | Mnemonic::LddZ => "ldd",
        Mnemonic::Lds => "lds",
        Mnemonic::StX | Mnemonic::StXPost | Mnemonic::StXPre => "st",
        Mnemonic::StY | Mnemonic::StYPost | Mnemonic::StYPre => "st",
        Mnemonic::StZ | Mnemonic::StZPost | Mnemonic::StZPre => "st",
        Mnemonic::StdY | Mnemonic::StdZ => "std",
        Mnemonic::Sts => "sts",
        _ => return None,
    })
}

fn transfer_name(mnemonic: Mnemonic) -> Option<&'static str> {
    Some(match mnemonic {
        Mnemonic::Mov => "mov",
        Mnemonic::Movw => "movw",
        Mnemonic::Ldi => "ldi",
        Mnemonic::LpmZ | Mnemonic::LpmZPost => "lpm",
        Mnemonic::ElpmZ | Mnemonic::ElpmZPost => "elpm",
        Mnemonic::Spm => "spm",
        Mnemonic::Push => "push",
        Mnemonic::Pop => "pop",
        Mnemonic::In => "in",
        Mnemonic::Out => "out",
        Mnemonic::Sbi => "sbi",
        Mnemonic::Cbi => "cbi",
        Mnemonic::Sbic => "sbic",
        Mnemonic::Sbis => "sbis",
        _ => return None,
    })
}

fn misc_name(mnemonic: Mnemonic) -> Option<&'static str> {
    Some(match mnemonic {
        Mnemonic::Nop => "nop",
        Mnemonic::Sleep => "sleep",
        Mnemonic::Wdr => "wdr",
        Mnemonic::Break => "break",
        _ => return None,
    })
}

fn operands(instruction: Instruction) -> String {
    if let Some(text) = load_pointer_operands(instruction) {
        return text;
    }
    if let Some(text) = store_pointer_operands(instruction) {
        return text;
    }
    if let Some(text) = lpm_operands(instruction) {
        return text;
    }
    generic_operands(instruction)
}

fn load_pointer_operands(instruction: Instruction) -> Option<String> {
    pointer_load(instruction).or_else(|| displacement_load(instruction))
}

fn pointer_load(instruction: Instruction) -> Option<String> {
    let register = register_value(instruction.first);
    Some(match instruction.mnemonic {
        Mnemonic::LdX => format!("r{register}, X"),
        Mnemonic::LdXPost => format!("r{register}, X+"),
        Mnemonic::LdXPre => format!("r{register}, -X"),
        Mnemonic::LdY => format!("r{register}, Y"),
        Mnemonic::LdYPost => format!("r{register}, Y+"),
        Mnemonic::LdYPre => format!("r{register}, -Y"),
        Mnemonic::LdZ => format!("r{register}, Z"),
        Mnemonic::LdZPost => format!("r{register}, Z+"),
        Mnemonic::LdZPre => format!("r{register}, -Z"),
        _ => return None,
    })
}

fn displacement_load(instruction: Instruction) -> Option<String> {
    let register = register_value(instruction.first);
    let displacement = displacement_value(instruction.second);
    Some(match instruction.mnemonic {
        Mnemonic::LddY => format!("r{register}, Y+{displacement}"),
        Mnemonic::LddZ => format!("r{register}, Z+{displacement}"),
        _ => return None,
    })
}

fn store_pointer_operands(instruction: Instruction) -> Option<String> {
    pointer_store(instruction).or_else(|| displacement_store(instruction))
}

fn pointer_store(instruction: Instruction) -> Option<String> {
    let register = register_value(instruction.first);
    Some(match instruction.mnemonic {
        Mnemonic::StX => format!("X, r{register}"),
        Mnemonic::StXPost => format!("X+, r{register}"),
        Mnemonic::StXPre => format!("-X, r{register}"),
        Mnemonic::StY => format!("Y, r{register}"),
        Mnemonic::StYPost => format!("Y+, r{register}"),
        Mnemonic::StYPre => format!("-Y, r{register}"),
        Mnemonic::StZ => format!("Z, r{register}"),
        Mnemonic::StZPost => format!("Z+, r{register}"),
        Mnemonic::StZPre => format!("-Z, r{register}"),
        _ => return None,
    })
}

fn displacement_store(instruction: Instruction) -> Option<String> {
    let register = register_value(instruction.first);
    let displacement = displacement_value(instruction.first);
    Some(match instruction.mnemonic {
        Mnemonic::StdY => format!("Y+{displacement}, r{register}"),
        Mnemonic::StdZ => format!("Z+{displacement}, r{register}"),
        _ => return None,
    })
}

fn lpm_operands(instruction: Instruction) -> Option<String> {
    let register = register_value(instruction.first);
    let text = match instruction.mnemonic {
        Mnemonic::LpmZ => format!("r{register}, Z"),
        Mnemonic::LpmZPost => format!("r{register}, Z+"),
        Mnemonic::ElpmZ => format!("r{register}, Z"),
        Mnemonic::ElpmZPost => format!("r{register}, Z+"),
        _ => return None,
    };
    Some(text)
}

fn generic_operands(instruction: Instruction) -> String {
    if instruction.second == Operand::None {
        return format_operand(instruction.first);
    }
    format!(
        "{}, {}",
        format_operand(instruction.first),
        format_operand(instruction.second)
    )
}

fn format_operand(operand: Operand) -> String {
    match operand {
        Operand::None => String::new(),
        Operand::Register(value) => format!("r{value}"),
        Operand::Immediate(value) => format!("0x{value:02X}"),
        Operand::Displacement(value) => format!("{value}"),
        Operand::Address(value) => format!("0x{value:04X}"),
        Operand::LongAddress(value) => format!("0x{value:06X}"),
        Operand::Offset(value) => format!("{value}"),
        Operand::Bit(value) => format!("{value}"),
        Operand::Io(value) => format!("0x{value:02X}"),
        Operand::Flag(value) => flag_name(value).to_string(),
    }
}

fn register_value(operand: Operand) -> u8 {
    match operand {
        Operand::Register(value) => value,
        _ => 0,
    }
}

fn displacement_value(operand: Operand) -> u8 {
    match operand {
        Operand::Displacement(value) => value,
        _ => 0,
    }
}

fn flag_name(flag: u8) -> &'static str {
    match flag {
        0 => "C",
        1 => "Z",
        2 => "N",
        3 => "V",
        4 => "S",
        5 => "H",
        6 => "T",
        _ => "I",
    }
}

#[cfg(test)]
mod tests {
    use super::disassemble;
    use crate::decode::decode;

    fn text(first: u16, second: Option<u16>) -> String {
        disassemble(decode(first, second).expect("decode"))
    }

    #[test]
    fn disassembles_arithmetic_and_logic() {
        assert_eq!(text(0xE005, None), "ldi r16, 0x05");
        assert_eq!(text(0x0C12, None), "add r1, r2");
        assert_eq!(text(0x2000 | (5 << 4) | 3, None), "and r5, r3");
        assert_eq!(text(0x9608, None), "adiw r24, 0x08");
    }

    #[test]
    fn disassembles_memory_forms() {
        assert_eq!(text(0x900D, None), "ld r0, X+");
        assert_eq!(text(0x8008 | (7 << 4) | 3, None), "ldd r7, Y+3");
        assert_eq!(text(0x920E, None), "st -X, r0");
        assert_eq!(text(0x9000 | (9 << 4), Some(0x1234)), "lds r9, 0x1234");
        assert_eq!(text(0x9104, None), "lpm r16, Z");
    }

    #[test]
    fn disassembles_control_flow() {
        assert_eq!(text(0xC001, None), "rjmp 1");
        assert_eq!(text(0xF009, None), "brbs Z, 1");
        assert_eq!(text(0x940C, Some(0x1234)), "jmp 0x001234");
        assert_eq!(text(0x9508, None), "ret");
    }

    #[test]
    fn disassembles_io_and_bits() {
        assert_eq!(text(0x9A66, None), "sbi 0x0C, 6");
        assert_eq!(text(0xB453, None), "in r5, 0x23");
        assert_eq!(text(0xBC53, None), "out 0x23, r5");
        assert_eq!(text(0x9478, None), "bset I");
        assert_eq!(text(0xFA62, None), "bst r6, 2");
    }
}
