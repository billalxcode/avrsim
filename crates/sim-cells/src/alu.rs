//! One-bit ALU slice composed from digital cells.

use super::builder::CellBuilder;
use super::cell::Cell;
use super::compose::full_adder;
use super::logic::{and2, mux2, or2, xor2};
use sim_core::{NodeId, SimError};

/// Builds a one-bit ALU slice.
///
/// Operation select `op1:op0` chooses AND, OR, XOR, and ADD.
pub fn alu_slice() -> Result<Cell, SimError> {
    let mut builder = CellBuilder::new();
    let inputs = add_inputs(&mut builder);
    let arithmetic = add_arithmetic(&mut builder, inputs)?;
    let logic = add_logic(&mut builder, inputs)?;
    let output = add_output_mux(&mut builder, inputs, logic, arithmetic)?;
    builder.build(&slice_ports(inputs, arithmetic, output))
}

#[derive(Clone, Copy)]
struct Inputs {
    first: NodeId,
    second: NodeId,
    carry_in: NodeId,
    op_low: NodeId,
    op_high: NodeId,
}

#[derive(Clone, Copy)]
struct Arithmetic {
    sum: NodeId,
    carry: NodeId,
}

#[derive(Clone, Copy)]
struct Logic {
    and_out: NodeId,
    or_out: NodeId,
    xor_out: NodeId,
}

fn add_inputs(builder: &mut CellBuilder) -> Inputs {
    Inputs {
        first: builder.node("a"),
        second: builder.node("b"),
        carry_in: builder.node("cin"),
        op_low: builder.node("op0"),
        op_high: builder.node("op1"),
    }
}

fn add_arithmetic(builder: &mut CellBuilder, inputs: Inputs) -> Result<Arithmetic, SimError> {
    let sum = builder.node("sum");
    let carry = builder.node("carry");
    let cell = full_adder()?;
    let connections = [
        ("a", inputs.first),
        ("b", inputs.second),
        ("cin", inputs.carry_in),
        ("sum", sum),
        ("carry", carry),
    ];
    builder.instantiate("adder", &cell, &connections)?;
    Ok(Arithmetic { sum, carry })
}

fn add_logic(builder: &mut CellBuilder, inputs: Inputs) -> Result<Logic, SimError> {
    let and_out = builder.node("and_y");
    let or_out = builder.node("or_y");
    let xor_out = builder.node("xor_y");
    add_gate(builder, "and", and2()?, inputs, and_out)?;
    add_gate(builder, "or", or2()?, inputs, or_out)?;
    add_gate(builder, "xor", xor2()?, inputs, xor_out)?;
    Ok(Logic {
        and_out,
        or_out,
        xor_out,
    })
}

fn add_gate(
    builder: &mut CellBuilder,
    name: &str,
    cell: Cell,
    inputs: Inputs,
    output: NodeId,
) -> Result<(), SimError> {
    builder.instantiate(
        name,
        &cell,
        &[("a", inputs.first), ("b", inputs.second), ("y", output)],
    )
}

fn add_output_mux(
    builder: &mut CellBuilder,
    inputs: Inputs,
    logic: Logic,
    arithmetic: Arithmetic,
) -> Result<NodeId, SimError> {
    let (low, high) = add_low_muxes(builder, inputs, logic, arithmetic)?;
    let output = builder.node("y");
    add_mux(builder, "m2", low, high, inputs.op_high, output)?;
    Ok(output)
}

fn add_low_muxes(
    builder: &mut CellBuilder,
    inputs: Inputs,
    logic: Logic,
    arithmetic: Arithmetic,
) -> Result<(NodeId, NodeId), SimError> {
    let low = add_first_mux(builder, inputs, logic)?;
    let high = add_second_mux(builder, inputs, logic, arithmetic)?;
    Ok((low, high))
}

fn add_first_mux(
    builder: &mut CellBuilder,
    inputs: Inputs,
    logic: Logic,
) -> Result<NodeId, SimError> {
    let output = builder.node("m0");
    add_mux(
        builder,
        "m0",
        logic.and_out,
        logic.or_out,
        inputs.op_low,
        output,
    )?;
    Ok(output)
}

fn add_second_mux(
    builder: &mut CellBuilder,
    inputs: Inputs,
    logic: Logic,
    arithmetic: Arithmetic,
) -> Result<NodeId, SimError> {
    let output = builder.node("m1");
    add_mux(
        builder,
        "m1",
        logic.xor_out,
        arithmetic.sum,
        inputs.op_low,
        output,
    )?;
    Ok(output)
}

fn add_mux(
    builder: &mut CellBuilder,
    name: &str,
    first: NodeId,
    second: NodeId,
    select: NodeId,
    output: NodeId,
) -> Result<(), SimError> {
    let cell = mux2()?;
    let connections = [("a", first), ("b", second), ("sel", select), ("y", output)];
    builder.instantiate(name, &cell, &connections)
}

fn slice_ports(
    inputs: Inputs,
    arithmetic: Arithmetic,
    output: NodeId,
) -> [(&'static str, NodeId); 7] {
    [
        ("a", inputs.first),
        ("b", inputs.second),
        ("cin", inputs.carry_in),
        ("op0", inputs.op_low),
        ("op1", inputs.op_high),
        ("y", output),
        ("carry", arithmetic.carry),
    ]
}
