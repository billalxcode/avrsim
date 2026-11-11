//! CMOS standard cells.

#![forbid(unsafe_code)]

mod arithmetic;
mod basic;
mod builder;
mod cell;
mod characterize;
mod digital;
mod registry;
mod sequential;

#[cfg(test)]
mod test_support;

pub use arithmetic::{
    AluFlags, AluResult, add_with_carry, adder8, alu_slice, full_adder, half_adder,
    ripple_carry_adder, sub_with_borrow,
};
pub use basic::{
    and2, buffer, inverter, mux2, nand2, nor2, or2, transmission_gate, tri_state_inverter, xor2,
};
pub use builder::CellBuilder;
pub use cell::Cell;
pub use characterize::{VtcMetrics, noise_margins};
pub use digital::{
    Cpu, CpuTrace, DigitalAlu, DigitalBuilder, DigitalCircuit, NetId, OP_ADD, OP_AND, OP_HLT,
    OP_JMP, OP_JNZ, OP_JZ, OP_LDI, OP_MOV, OP_NOP, OP_OR, OP_SUB, OP_XOR, ProgramCounter,
    RegisterFile, RegisterPorts, StackPointer, and_all, counter, decoder, decrement, digital_alu,
    encode_alu, encode_branch, encode_hlt, encode_ldi, increment, mux_tree, or_all, read_bits,
    register, register_file, shift_register,
};
pub use registry::CellKind;
pub use sequential::{d_flip_flop, d_latch, ring_oscillator, sram6t};
