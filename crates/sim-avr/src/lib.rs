//! Structural AVR CPU and peripherals, targeting the ATmega328P.

pub mod adc;
pub mod alu;
pub mod cpu;
pub mod decode;
pub mod disassemble;
pub mod fetch;
pub mod gpio;
pub mod instruction;
pub mod instruction_register;
pub mod multiply;
pub mod pin;
pub mod register_bank;
pub mod shift;
pub mod spi;
pub mod system;
pub mod timer0;
pub mod timer1;
pub mod timer2;
pub mod twi;
pub mod uart;

#[cfg(test)]
mod test_support;

pub use adc::Adc;
pub use alu::{
    ArithmeticUnit, CompareUnit, LogicUnit, arithmetic_unit, compare_unit, decrement_unit,
    increment_unit, logic_unit, negate_unit, select_operand,
};
pub use cpu::{AvrTrace, Cpu, Sreg};
pub use decode::{decode, uses_second_word};
pub use disassemble::disassemble;
pub use fetch::{Fetched, ProgramMemory};
pub use gpio::GpioPort;
pub use instruction::{Instruction, Mnemonic, Operand};
pub use instruction_register::InstructionRegister;
pub use multiply::{
    MultiplyUnit, multiply_signed_unit, multiply_signed_unsigned_unit, multiply_unit,
};
pub use pin::{ExternalPin, PinElectrical, PinElectricalConfig, PinMode, measure_pin};
pub use register_bank::RegisterBank;
pub use shift::{ShiftUnit, shift_unit};
pub use spi::Spi;
pub use system::{AnalogComparator, ClockPrescaler, Eeprom, Fuses, PowerState, Watchdog};
pub use timer0::Timer0;
pub use timer1::Timer1;
pub use timer2::Timer2;
pub use twi::Twi;
pub use uart::Uart;
