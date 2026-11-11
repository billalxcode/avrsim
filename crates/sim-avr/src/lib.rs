//! Structural AVR CPU and peripherals, targeting the ATmega328P.

pub mod decode;
pub mod instruction;

pub use decode::{decode, uses_second_word};
pub use instruction::{Instruction, Mnemonic, Operand};
