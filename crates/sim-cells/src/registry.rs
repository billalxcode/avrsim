//! Cell registry.

use super::buffer::buffer;
use super::cell::Cell;
use super::inverter::inverter;
use super::logic::{and2, mux2, or2, xor2};
use super::memory::sram6t;
use super::nand::nand2;
use super::nor::nor2;
use super::oscillator::ring_oscillator;
use super::sequential::{d_flip_flop, d_latch};
use super::transmission::transmission_gate;
use super::tristate::tri_state_inverter;
use sim_core::SimError;

/// Available cell kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellKind {
    /// CMOS inverter.
    Inverter,
    /// Two-input NAND.
    Nand2,
    /// Two-input NOR.
    Nor2,
    /// Two-input AND.
    And2,
    /// Two-input OR.
    Or2,
    /// Two-input XOR.
    Xor2,
    /// Two-input multiplexer.
    Mux2,
    /// Two-stage buffer.
    Buffer,
    /// Transmission gate.
    TransmissionGate,
    /// Tri-state inverter.
    TriStateInverter,
    /// D latch.
    DLatch,
    /// D flip-flop.
    DFlipFlop,
    /// Six-transistor SRAM cell.
    Sram6t,
    /// Three-stage ring oscillator.
    RingOscillator,
}

impl CellKind {
    /// All registered cell kinds.
    pub const ALL: [Self; 14] = [
        Self::Inverter,
        Self::Nand2,
        Self::Nor2,
        Self::And2,
        Self::Or2,
        Self::Xor2,
        Self::Mux2,
        Self::Buffer,
        Self::TransmissionGate,
        Self::TriStateInverter,
        Self::DLatch,
        Self::DFlipFlop,
        Self::Sram6t,
        Self::RingOscillator,
    ];

    /// Returns the cell name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Inverter => "inverter",
            Self::Nand2 => "nand2",
            Self::Nor2 => "nor2",
            Self::And2 => "and2",
            Self::Or2 => "or2",
            Self::Xor2 => "xor2",
            Self::Mux2 => "mux2",
            Self::Buffer => "buffer",
            Self::TransmissionGate => "transmission_gate",
            Self::TriStateInverter => "tri_state_inverter",
            Self::DLatch => "d_latch",
            Self::DFlipFlop => "d_flip_flop",
            Self::Sram6t => "sram6t",
            Self::RingOscillator => "ring_oscillator",
        }
    }

    /// Builds the cell.
    pub fn build(self) -> Result<Cell, SimError> {
        match self {
            Self::Inverter => inverter(),
            Self::Nand2 => nand2(),
            Self::Nor2 => nor2(),
            Self::And2 => and2(),
            Self::Or2 => or2(),
            Self::Xor2 => xor2(),
            Self::Mux2 => mux2(),
            Self::Buffer => buffer(),
            Self::TransmissionGate => transmission_gate(),
            Self::TriStateInverter => tri_state_inverter(),
            Self::DLatch => d_latch(),
            Self::DFlipFlop => d_flip_flop(),
            Self::Sram6t => sram6t(),
            Self::RingOscillator => ring_oscillator(),
        }
    }
}
