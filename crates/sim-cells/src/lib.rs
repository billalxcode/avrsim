//! CMOS standard cells.

#![forbid(unsafe_code)]

mod adder;
mod alu;
mod buffer;
mod builder;
mod cell;
mod characterize;
mod compose;
mod digital;
mod digital_blocks;
mod flags;
mod inverter;
mod logic;
mod memory;
mod nand;
mod nor;
mod oscillator;
mod register_file;
mod registry;
mod sequential;
mod transmission;
mod tristate;

pub use adder::{adder8, ripple_carry_adder};
pub use alu::alu_slice;
pub use buffer::buffer;
pub use builder::CellBuilder;
pub use cell::Cell;
pub use characterize::{VtcMetrics, noise_margins};
pub use compose::{full_adder, half_adder};
pub use digital::{DigitalBuilder, DigitalCircuit, NetId};
pub use digital_blocks::{
    ProgramCounter, StackPointer, and_all, counter, decoder, decrement, increment, mux_tree,
    or_all, read_bits, register, shift_register,
};
pub use flags::{AluFlags, AluResult, add_with_carry, sub_with_borrow};
pub use inverter::inverter;
pub use logic::{and2, mux2, or2, xor2};
pub use memory::sram6t;
pub use nand::nand2;
pub use nor::nor2;
pub use oscillator::ring_oscillator;
pub use register_file::{RegisterFile, RegisterPorts, register_file};
pub use registry::CellKind;
pub use sequential::{d_flip_flop, d_latch};
pub use transmission::transmission_gate;
pub use tristate::tri_state_inverter;
