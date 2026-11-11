//! CMOS ring oscillator.

use super::builder::CellBuilder;
use super::cell::Cell;
use sim_core::SimError;

/// Builds a three-stage ring oscillator.
pub fn ring_oscillator() -> Result<Cell, SimError> {
    let mut builder = CellBuilder::new();
    let first = builder.node("n1");
    let second = builder.node("n2");
    let third = builder.node("n3");
    builder.inverter("inv1", first, second);
    builder.inverter("inv2", second, third);
    builder.inverter("inv3", third, first);
    builder.build(&[("y", first)])
}
