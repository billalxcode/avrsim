//! Board-level netlists: MCU, LED, resistor, button, and host UART endpoint.

pub mod board;
pub mod components;
pub mod elf;
pub mod hex;
pub mod trace;

pub use board::{Board, UartHost};
pub use components::{Button, Led, Resistor};
pub use elf::{ElfImage, ElfSegment, ElfSymbol, ElfSymbolKind};
pub use hex::{HexImage, to_intel_hex};
pub use trace::{AnalogVcdWriter, CsvWriter, VcdWriter, csv_header, csv_line};
