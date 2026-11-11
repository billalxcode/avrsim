//! CMOS sequential cells.

use super::builder::CellBuilder;
use super::cell::Cell;
use sim_core::{NodeId, SimError};

/// Builds a level-sensitive D latch.
pub fn d_latch() -> Result<Cell, SimError> {
    let mut builder = CellBuilder::new();
    let data = builder.node("d");
    let output = builder.node("q");
    let enable = builder.node("en");
    let enable_bar = builder.node("enb");
    let feedback = builder.node("n1");
    builder.transmission_gate("tg1", data, output, enable, enable_bar);
    builder.inverter("inv1", output, feedback);
    builder.inverter("inv2", feedback, output);
    builder.build(&[
        ("d", data),
        ("q", output),
        ("en", enable),
        ("enb", enable_bar),
    ])
}

/// Builds a positive-edge D flip-flop.
pub fn d_flip_flop() -> Result<Cell, SimError> {
    let mut builder = CellBuilder::new();
    let data = builder.node("d");
    let clock = builder.node("clk");
    let clock_bar = builder.node("clkb");
    let output = builder.node("q");
    builder.inverter("clk_inv", clock, clock_bar);
    let master = add_master_latch(&mut builder, data, clock, clock_bar);
    add_slave_latch(&mut builder, master, clock, clock_bar, output);
    builder.build(&[("d", data), ("clk", clock), ("q", output)])
}

fn add_master_latch(
    builder: &mut CellBuilder,
    data: NodeId,
    clock: NodeId,
    clock_bar: NodeId,
) -> NodeId {
    let first = builder.node("m1");
    let second = builder.node("m2");
    let third = builder.node("m3");
    builder.transmission_gate("tg1", data, first, clock_bar, clock);
    builder.inverter("inv1", first, second);
    builder.inverter("inv2", second, third);
    builder.transmission_gate("tg2", third, first, clock, clock_bar);
    second
}

fn add_slave_latch(
    builder: &mut CellBuilder,
    data: NodeId,
    clock: NodeId,
    clock_bar: NodeId,
    output: NodeId,
) {
    let first = builder.node("s1");
    let second = builder.node("s2");
    builder.transmission_gate("tg3", data, first, clock, clock_bar);
    builder.inverter("inv3", first, output);
    builder.inverter("inv4", output, second);
    builder.transmission_gate("tg4", second, first, clock_bar, clock);
}
