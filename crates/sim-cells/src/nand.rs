//! CMOS NAND2 cell.

use super::builder::CellBuilder;
use super::cell::Cell;
use sim_core::SimError;

/// Builds a two-input CMOS NAND gate.
pub fn nand2() -> Result<Cell, SimError> {
    let mut builder = CellBuilder::new();
    let first = builder.node("a");
    let second = builder.node("b");
    let output = builder.node("y");
    let middle = builder.node("mid");
    let vdd = builder.vdd();
    let ground = builder.ground();
    builder.pmos("mp1", first, output, vdd, 2e-6);
    builder.pmos("mp2", second, output, vdd, 2e-6);
    builder.nmos("mn1", first, output, middle, 1e-6);
    builder.nmos("mn2", second, middle, ground, 1e-6);
    builder.build(&[("a", first), ("b", second), ("y", output)])
}
