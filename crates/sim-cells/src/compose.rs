//! Digital composition of cells.

use super::builder::CellBuilder;
use super::cell::Cell;
use super::nand::nand2;
use sim_core::{NodeId, SimError};

/// Builds a half adder from NAND cells.
pub fn half_adder() -> Result<Cell, SimError> {
    let mut builder = CellBuilder::new();
    let first = builder.node("a");
    let second = builder.node("b");
    let sum = builder.node("sum");
    let carry = builder.node("carry");
    add_half_adder(&mut builder, first, second, sum, carry)?;
    builder.build(&[("a", first), ("b", second), ("sum", sum), ("carry", carry)])
}

fn add_half_adder(
    builder: &mut CellBuilder,
    first: NodeId,
    second: NodeId,
    sum: NodeId,
    carry: NodeId,
) -> Result<(), SimError> {
    let shared = builder.node("n1");
    let left = builder.node("n2");
    let right = builder.node("n3");
    add_nand(builder, "g1", first, second, shared)?;
    add_nand(builder, "g2", first, shared, left)?;
    add_nand(builder, "g3", second, shared, right)?;
    add_nand(builder, "g4", left, right, sum)?;
    add_nand(builder, "g5", shared, shared, carry)
}

/// Builds a full adder from two half adders and NAND cells.
pub fn full_adder() -> Result<Cell, SimError> {
    let mut builder = CellBuilder::new();
    let first = builder.node("a");
    let second = builder.node("b");
    let carry_in = builder.node("cin");
    let sum = builder.node("sum");
    let partial = builder.node("s1");
    let first_carry = builder.node("c1");
    let second_carry = builder.node("c2");
    add_first_half(&mut builder, first, second, partial, first_carry)?;
    add_second_half(&mut builder, partial, carry_in, sum, second_carry)?;
    let carry = add_or(&mut builder, first_carry, second_carry)?;
    builder.build(&[
        ("a", first),
        ("b", second),
        ("cin", carry_in),
        ("sum", sum),
        ("carry", carry),
    ])
}

fn add_first_half(
    builder: &mut CellBuilder,
    first: NodeId,
    second: NodeId,
    sum: NodeId,
    carry: NodeId,
) -> Result<(), SimError> {
    let cell = half_adder()?;
    builder.instantiate(
        "h1",
        &cell,
        &[("a", first), ("b", second), ("sum", sum), ("carry", carry)],
    )
}

fn add_second_half(
    builder: &mut CellBuilder,
    first: NodeId,
    second: NodeId,
    sum: NodeId,
    carry: NodeId,
) -> Result<(), SimError> {
    let cell = half_adder()?;
    builder.instantiate(
        "h2",
        &cell,
        &[("a", first), ("b", second), ("sum", sum), ("carry", carry)],
    )
}

fn add_or(builder: &mut CellBuilder, first: NodeId, second: NodeId) -> Result<NodeId, SimError> {
    let first_bar = builder.node("or_n1");
    let second_bar = builder.node("or_n2");
    let output = builder.node("carry");
    add_nand(builder, "or_g1", first, first, first_bar)?;
    add_nand(builder, "or_g2", second, second, second_bar)?;
    add_nand(builder, "or_g3", first_bar, second_bar, output)?;
    Ok(output)
}

pub(crate) fn add_nand(
    builder: &mut CellBuilder,
    name: &str,
    first: NodeId,
    second: NodeId,
    output: NodeId,
) -> Result<(), SimError> {
    let cell = nand2()?;
    builder.instantiate(name, &cell, &[("a", first), ("b", second), ("y", output)])
}
