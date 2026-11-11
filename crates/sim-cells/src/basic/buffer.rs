//! CMOS buffer cell.

use crate::builder::CellBuilder;
use crate::cell::Cell;
use sim_core::SimError;

/// Builds a two-stage CMOS buffer.
pub fn buffer() -> Result<Cell, SimError> {
    let mut builder = CellBuilder::new();
    let input = builder.node("a");
    let middle = builder.node("mid");
    let output = builder.node("y");
    let vdd = builder.vdd();
    let ground = builder.ground();
    builder.pmos("mp1", input, middle, vdd, 2e-6);
    builder.nmos("mn1", input, middle, ground, 1e-6);
    builder.pmos("mp2", middle, output, vdd, 2e-6);
    builder.nmos("mn2", middle, output, ground, 1e-6);
    builder.build(&[("a", input), ("y", output)])
}
