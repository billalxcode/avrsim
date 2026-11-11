//! Cell model with named ports.

use sim_core::{Circuit, DeviceKind, NodeId, SimError};
use std::collections::BTreeMap;

/// A reusable cell with a circuit and named ports.
pub struct Cell {
    circuit: Circuit,
    ports: BTreeMap<String, NodeId>,
}

impl Cell {
    pub(crate) fn new(circuit: Circuit, ports: BTreeMap<String, NodeId>) -> Self {
        Self { circuit, ports }
    }

    /// Returns the cell circuit.
    #[must_use]
    pub fn circuit(&self) -> &Circuit {
        &self.circuit
    }

    /// Returns a port by name.
    #[must_use]
    pub fn port(&self, name: &str) -> Option<NodeId> {
        self.ports.get(name).copied()
    }

    /// Returns the VDD port.
    #[must_use]
    pub fn vdd(&self) -> Option<NodeId> {
        self.port("vdd")
    }

    /// Returns a powered copy of the circuit.
    pub fn powered(&self, voltage: f64) -> Result<Circuit, SimError> {
        let vdd = self
            .vdd()
            .ok_or_else(|| SimError::InvalidNetlist("cell has no vdd port".to_string()))?;
        let mut circuit = self.circuit.clone();
        let ground = circuit.ground();
        let parameters = parameter("voltage", voltage);
        circuit.add_device(
            "vdd_src",
            DeviceKind::VoltageSource,
            vec![vdd, ground],
            parameters,
        )?;
        circuit.validate()?;
        Ok(circuit)
    }
}

fn parameter(name: &str, value: f64) -> BTreeMap<String, f64> {
    let mut map = BTreeMap::new();
    map.insert(name.to_string(), value);
    map
}
