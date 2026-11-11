//! CMOS inverter cell.

use crate::builder::CellBuilder;
use crate::cell::Cell;
use sim_core::SimError;

/// Builds a CMOS inverter.
pub fn inverter() -> Result<Cell, SimError> {
    let mut builder = CellBuilder::new();
    let input = builder.node("a");
    let output = builder.node("y");
    let vdd = builder.vdd();
    let ground = builder.ground();
    builder.pmos("mp", input, output, vdd, 2e-6);
    builder.nmos("mn", input, output, ground, 1e-6);
    builder.build(&[("a", input), ("y", output)])
}
