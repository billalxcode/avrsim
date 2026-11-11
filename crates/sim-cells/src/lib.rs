//! CMOS standard cells.

#![forbid(unsafe_code)]

mod buffer;
mod builder;
mod cell;
mod characterize;
mod inverter;
mod memory;
mod nand;
mod nor;
mod oscillator;
mod registry;
mod sequential;
mod transmission;
mod tristate;

pub use buffer::buffer;
pub use builder::CellBuilder;
pub use cell::Cell;
pub use characterize::{VtcMetrics, noise_margins};
pub use inverter::inverter;
pub use memory::sram6t;
pub use nand::nand2;
pub use nor::nor2;
pub use oscillator::ring_oscillator;
pub use registry::CellKind;
pub use sequential::{d_flip_flop, d_latch};
pub use transmission::transmission_gate;
pub use tristate::tri_state_inverter;
