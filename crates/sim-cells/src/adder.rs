//! Ripple-carry adder composition.

use super::builder::CellBuilder;
use super::cell::Cell;
use super::compose::full_adder;
use sim_core::{NodeId, SimError};

/// Builds an 8-bit ripple-carry adder.
pub fn adder8() -> Result<Cell, SimError> {
    ripple_carry_adder(8)
}

/// Builds a ripple-carry adder with the requested bit width.
pub fn ripple_carry_adder(bits: usize) -> Result<Cell, SimError> {
    if bits == 0 {
        return Err(SimError::InvalidNetlist(
            "adder needs at least one bit".to_string(),
        ));
    }
    let mut builder = CellBuilder::new();
    let carry_in = builder.node("cin");
    let (mut ports, carry_out) = add_bits(&mut builder, bits, carry_in)?;
    ports.push(("cout".to_string(), carry_out));
    build_adder_cell(builder, &ports)
}

fn add_bits(
    builder: &mut CellBuilder,
    bits: usize,
    carry_in: NodeId,
) -> Result<(Vec<(String, NodeId)>, NodeId), SimError> {
    let mut ports = vec![("cin".to_string(), carry_in)];
    let mut carry = carry_in;
    for bit in 0..bits {
        let first = builder.node(&format!("a{bit}"));
        let second = builder.node(&format!("b{bit}"));
        let (sum, carry_out) = add_bit(builder, bit, bits, first, second, carry)?;
        ports.push((format!("a{bit}"), first));
        ports.push((format!("b{bit}"), second));
        ports.push((format!("sum{bit}"), sum));
        carry = carry_out;
    }
    Ok((ports, carry))
}

fn build_adder_cell(builder: CellBuilder, ports: &[(String, NodeId)]) -> Result<Cell, SimError> {
    let references: Vec<(&str, NodeId)> = ports
        .iter()
        .map(|(name, node)| (name.as_str(), *node))
        .collect();
    builder.build(&references)
}

fn add_bit(
    builder: &mut CellBuilder,
    bit: usize,
    bits: usize,
    first: NodeId,
    second: NodeId,
    carry_in: NodeId,
) -> Result<(NodeId, NodeId), SimError> {
    let sum = builder.node(&format!("sum{bit}"));
    let carry_out = carry_node(builder, bit, bits);
    instantiate_full_adder(builder, bit, first, second, carry_in, sum, carry_out)?;
    Ok((sum, carry_out))
}

fn carry_node(builder: &mut CellBuilder, bit: usize, bits: usize) -> NodeId {
    if bit + 1 == bits {
        builder.node("cout")
    } else {
        builder.node(&format!("c{bit}"))
    }
}

fn instantiate_full_adder(
    builder: &mut CellBuilder,
    bit: usize,
    first: NodeId,
    second: NodeId,
    carry_in: NodeId,
    sum: NodeId,
    carry_out: NodeId,
) -> Result<(), SimError> {
    let cell = full_adder()?;
    let connections = full_adder_connections(first, second, carry_in, sum, carry_out);
    builder.instantiate(&format!("fa{bit}"), &cell, &connections)
}

fn full_adder_connections(
    first: NodeId,
    second: NodeId,
    carry_in: NodeId,
    sum: NodeId,
    carry_out: NodeId,
) -> [(&'static str, NodeId); 5] {
    [
        ("a", first),
        ("b", second),
        ("cin", carry_in),
        ("sum", sum),
        ("carry", carry_out),
    ]
}
