//! CMOS transmission gate.

use crate::builder::CellBuilder;
use crate::cell::Cell;
use sim_core::SimError;

/// Builds a transmission gate.
pub fn transmission_gate() -> Result<Cell, SimError> {
    let mut builder = CellBuilder::new();
    let first = builder.node("a");
    let second = builder.node("b");
    let enable = builder.node("en");
    let enable_bar = builder.node("enb");
    builder.transmission_gate("tg", first, second, enable, enable_bar);
    builder.build(&[
        ("a", first),
        ("b", second),
        ("en", enable),
        ("enb", enable_bar),
    ])
}
