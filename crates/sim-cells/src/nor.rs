//! CMOS NOR2 cell.

use super::builder::CellBuilder;
use super::cell::Cell;
use sim_core::SimError;

/// Builds a two-input CMOS NOR gate.
pub fn nor2() -> Result<Cell, SimError> {
    let mut builder = CellBuilder::new();
    let first = builder.node("a");
    let second = builder.node("b");
    let output = builder.node("y");
    let middle = builder.node("mid");
    let vdd = builder.vdd();
    let ground = builder.ground();
    builder.nmos("mn1", first, output, ground, 1e-6);
    builder.nmos("mn2", second, output, ground, 1e-6);
    builder.pmos("mp1", first, middle, vdd, 2e-6);
    builder.pmos("mp2", second, output, middle, 2e-6);
    builder.build(&[("a", first), ("b", second), ("y", output)])
}
