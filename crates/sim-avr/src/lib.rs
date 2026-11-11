//! Structural AVR CPU and peripherals, targeting the ATmega328P.

pub mod decode;
pub mod fetch;
pub mod instruction;

pub use decode::{decode, uses_second_word};
pub use fetch::{Fetched, ProgramMemory};
pub use instruction::{Instruction, Mnemonic, Operand};
