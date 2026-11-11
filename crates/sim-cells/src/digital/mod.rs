//! Two-state digital logic and digital blocks.

pub mod alu;
pub mod blocks;
pub mod circuit;
pub mod cpu;
pub mod registers;

pub use alu::{DigitalAlu, digital_alu};
pub use blocks::{
    ProgramCounter, StackPointer, and_all, counter, decoder, decrement, increment, mux_tree,
    or_all, read_bits, register, shift_register,
};
pub use circuit::{DigitalBuilder, DigitalCircuit, NetId};
pub use cpu::{
    Cpu, CpuTrace, OP_ADD, OP_AND, OP_HLT, OP_JMP, OP_JNZ, OP_JZ, OP_LDI, OP_MOV, OP_NOP, OP_OR,
    OP_SUB, OP_XOR, encode_alu, encode_branch, encode_hlt, encode_ldi,
};
pub use registers::{RegisterFile, RegisterPorts, register_file};
