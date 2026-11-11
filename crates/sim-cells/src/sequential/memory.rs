//! CMOS SRAM cell.

use crate::builder::CellBuilder;
use crate::cell::Cell;
use sim_core::SimError;

/// Builds a six-transistor SRAM cell.
pub fn sram6t() -> Result<Cell, SimError> {
    let mut builder = CellBuilder::new();
    let output = builder.node("q");
    let output_bar = builder.node("qb");
    let word_line = builder.node("wl");
    let bit_line = builder.node("bl");
    let bit_line_bar = builder.node("blb");
    builder.inverter("inv_q", output, output_bar);
    builder.inverter("inv_qb", output_bar, output);
    builder.nmos("access_q", word_line, bit_line, output, 1e-6);
    builder.nmos("access_qb", word_line, bit_line_bar, output_bar, 1e-6);
    builder.build(&[
        ("q", output),
        ("qb", output_bar),
        ("wl", word_line),
        ("bl", bit_line),
        ("blb", bit_line_bar),
    ])
}
