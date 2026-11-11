//! Structural AVR CPU and peripherals, targeting the ATmega328P.

pub mod alu;
pub mod decode;
pub mod fetch;
pub mod instruction;
pub mod instruction_register;
pub mod register_bank;

#[cfg(test)]
mod test_support;

pub use alu::{ArithmeticUnit, arithmetic_unit};
pub use decode::{decode, uses_second_word};
pub use fetch::{Fetched, ProgramMemory};
pub use instruction::{Instruction, Mnemonic, Operand};
pub use instruction_register::InstructionRegister;
pub use register_bank::RegisterBank;
