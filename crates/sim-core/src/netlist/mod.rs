//! Netlist model, builder, subcircuits, and validation.

mod builder;
mod circuit;
mod device;
mod macros;
mod node;
mod subcircuit;

#[cfg(test)]
mod tests;

pub use builder::CircuitBuilder;
pub use circuit::Circuit;
pub use device::{Device, DeviceInstance, DeviceKind, Solution, StampContext};
pub use node::{Node, NodeKind};
pub use subcircuit::{Port, Subcircuit};

use crate::error::SimError;

/// Validates that a name is non-empty and has no surrounding whitespace.
pub(crate) fn ensure_name(name: &str) -> Result<(), SimError> {
    if name.trim().is_empty() {
        return Err(SimError::InvalidNetlist(
            "name must not be empty".to_string(),
        ));
    }
    if name.trim() != name {
        return Err(SimError::InvalidNetlist(
            "name must not have surrounding whitespace".to_string(),
        ));
    }
    Ok(())
}
