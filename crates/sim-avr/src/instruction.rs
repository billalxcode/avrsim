//! AVR instruction representation.

/// AVR instruction mnemonic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mnemonic {
    Nop,
    Sleep,
    Wdr,
    Break,
    Add,
    Adc,
    Adiw,
    Sub,
    Subi,
    Sbc,
    Sbci,
    Sbiw,
    And,
    Andi,
    Or,
    Ori,
    Eor,
    Com,
    Neg,
    Inc,
    Dec,
    Swap,
    Asr,
    Lsr,
    Ror,
    Mul,
    Muls,
    Mulsu,
    Fmul,
    Fmuls,
    Fmulsu,
    Cp,
    Cpc,
    Cpi,
    Cpse,
    Mov,
    Movw,
    Ldi,
    LdX,
    LdXPost,
    LdXPre,
    LdY,
    LdYPost,
    LdYPre,
    LdZ,
    LdZPost,
    LdZPre,
    LddY,
    LddZ,
    StX,
    StXPost,
    StXPre,
    StY,
    StYPost,
    StYPre,
    StZ,
    StZPost,
    StZPre,
    StdY,
    StdZ,
    Lds,
    Sts,
    LpmZ,
    LpmZPost,
    ElpmZ,
    ElpmZPost,
    Spm,
    Push,
    Pop,
    In,
    Out,
    Sbi,
    Cbi,
    Sbic,
    Sbis,
    Rjmp,
    Rcall,
    Ijmp,
    Eijmp,
    Jmp,
    Icall,
    Eicall,
    Call,
    Ret,
    Reti,
    Brbs,
    Brbc,
    Sbrc,
    Sbrs,
    Bst,
    Bld,
    Bset,
    Bclr,
}

/// Instruction operand.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operand {
    /// No operand.
    None,
    /// Register index.
    Register(u8),
    /// Immediate byte.
    Immediate(u8),
    /// Displacement for LDD/STD.
    Displacement(u8),
    /// Data space address.
    Address(u16),
    /// Program memory address.
    LongAddress(u32),
    /// Signed word offset.
    Offset(i16),
    /// Bit index.
    Bit(u8),
    /// I/O address.
    Io(u8),
    /// Status flag index.
    Flag(u8),
}

/// A decoded AVR instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Instruction {
    /// Instruction mnemonic.
    pub mnemonic: Mnemonic,
    /// First operand.
    pub first: Operand,
    /// Second operand.
    pub second: Operand,
    /// Number of 16-bit words.
    pub words: u8,
}

impl Instruction {
    /// Creates an instruction.
    #[must_use]
    pub const fn new(mnemonic: Mnemonic, first: Operand, second: Operand, words: u8) -> Self {
        Self {
            mnemonic,
            first,
            second,
            words,
        }
    }
}
