//! Structural AVR CPU and peripherals, targeting the ATmega328P.

pub mod alu;
pub mod decode;
pub mod fetch;
pub mod instruction;
pub mod instruction_register;
pub mod multiply;
pub mod register_bank;
pub mod shift;

#[cfg(test)]
mod test_support;

pub use alu::{
    ArithmeticUnit, CompareUnit, LogicUnit, arithmetic_unit, compare_unit, decrement_unit,
    increment_unit, logic_unit, negate_unit, select_operand,
};
pub use decode::{decode, uses_second_word};
pub use fetch::{Fetched, ProgramMemory};
pub use instruction::{Instruction, Mnemonic, Operand};
pub use instruction_register::InstructionRegister;
pub use multiply::{
    MultiplyUnit, multiply_signed_unit, multiply_signed_unsigned_unit, multiply_unit,
};
pub use register_bank::RegisterBank;
pub use shift::{ShiftUnit, shift_unit};
