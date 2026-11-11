//! Combinational logic gates composed from NAND cells.

use super::builder::CellBuilder;
use super::cell::Cell;
use super::compose::add_nand;
use sim_core::SimError;

/// Builds an AND gate from a NAND gate and an inverter.
pub fn and2() -> Result<Cell, SimError> {
    let mut builder = CellBuilder::new();
    let first = builder.node("a");
    let second = builder.node("b");
    let output = builder.node("y");
    let inverted = builder.node("n1");
    add_nand(&mut builder, "g1", first, second, inverted)?;
    builder.inverter("inv", inverted, output);
    builder.build(&[("a", first), ("b", second), ("y", output)])
}

/// Builds an OR gate from three NAND gates.
pub fn or2() -> Result<Cell, SimError> {
    let mut builder = CellBuilder::new();
    let first = builder.node("a");
    let second = builder.node("b");
    let output = builder.node("y");
    let first_bar = builder.node("n1");
    let second_bar = builder.node("n2");
    add_nand(&mut builder, "g1", first, first, first_bar)?;
    add_nand(&mut builder, "g2", second, second, second_bar)?;
    add_nand(&mut builder, "g3", first_bar, second_bar, output)?;
    builder.build(&[("a", first), ("b", second), ("y", output)])
}

/// Builds an XOR gate from four NAND gates.
pub fn xor2() -> Result<Cell, SimError> {
    let mut builder = CellBuilder::new();
    let first = builder.node("a");
    let second = builder.node("b");
    let output = builder.node("y");
    let shared = builder.node("n1");
    let left = builder.node("n2");
    let right = builder.node("n3");
    add_nand(&mut builder, "g1", first, second, shared)?;
    add_nand(&mut builder, "g2", first, shared, left)?;
    add_nand(&mut builder, "g3", second, shared, right)?;
    add_nand(&mut builder, "g4", left, right, output)?;
    builder.build(&[("a", first), ("b", second), ("y", output)])
}

/// Builds a two-input multiplexer from three NAND gates.
pub fn mux2() -> Result<Cell, SimError> {
    let mut builder = CellBuilder::new();
    let first = builder.node("a");
    let second = builder.node("b");
    let select = builder.node("sel");
    let output = builder.node("y");
    let select_bar = builder.node("n1");
    let left = builder.node("n2");
    let right = builder.node("n3");
    add_nand(&mut builder, "g1", select, select, select_bar)?;
    add_nand(&mut builder, "g2", first, select_bar, left)?;
    add_nand(&mut builder, "g3", second, select, right)?;
    add_nand(&mut builder, "g4", left, right, output)?;
    builder.build(&[("a", first), ("b", second), ("sel", select), ("y", output)])
}
