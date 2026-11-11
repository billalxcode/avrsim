//! CMOS tri-state inverter.

use crate::builder::CellBuilder;
use crate::cell::Cell;
use sim_core::SimError;

/// Builds a tri-state inverter with an active-high enable.
pub fn tri_state_inverter() -> Result<Cell, SimError> {
    let mut builder = CellBuilder::new();
    let input = builder.node("a");
    let output = builder.node("y");
    let middle = builder.node("mid");
    let enable = builder.node("en");
    let enable_bar = builder.node("enb");
    builder.inverter("inv", input, middle);
    builder.transmission_gate("tg", middle, output, enable, enable_bar);
    builder.build(&[
        ("a", input),
        ("y", output),
        ("en", enable),
        ("enb", enable_bar),
    ])
}
