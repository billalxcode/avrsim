//! Arithmetic cells and flags.

pub mod adder;
pub mod alu;
pub mod compose;
pub mod flags;

pub use adder::{adder8, ripple_carry_adder};
pub use alu::alu_slice;
pub use compose::{full_adder, half_adder};
pub use flags::{AluFlags, AluResult, add_with_carry, sub_with_borrow};
