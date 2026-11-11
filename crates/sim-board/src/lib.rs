//! Board-level netlists: MCU, LED, resistor, button, and host UART endpoint.

pub mod board;
pub mod components;
pub mod hex;
pub mod trace;

pub use board::{Board, UartHost};
pub use components::{Button, Led, Resistor};
pub use hex::HexImage;
pub use trace::{VcdWriter, csv_header, csv_line};
