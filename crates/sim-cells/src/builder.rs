//! Builder helpers for CMOS cells.

use super::cell::Cell;
use sim_core::{CircuitBuilder, DeviceKind, NodeId, SimError};
use std::collections::BTreeMap;

/// Builds a cell with a VDD supply node and ground.
pub struct CellBuilder {
    builder: CircuitBuilder,
    vdd: NodeId,
    ground: NodeId,
}

impl CellBuilder {
    /// Creates a cell builder.
    #[must_use]
    pub fn new() -> Self {
        let mut builder = CircuitBuilder::new();
        let vdd = builder.supply("vdd").expect("vdd");
        let ground = builder.ground();
        Self {
            builder,
            vdd,
            ground,
        }
    }

    /// Adds a signal node.
    pub fn node(&mut self, name: &str) -> NodeId {
        self.builder.node(name).expect(name)
    }

    /// Returns the VDD node.
    #[must_use]
    pub const fn vdd(&self) -> NodeId {
        self.vdd
    }

    /// Returns the ground node.
    #[must_use]
    pub const fn ground(&self) -> NodeId {
        self.ground
    }

    /// Adds an NMOS transistor with pins in gate, drain, source order.
    pub fn nmos(&mut self, name: &str, gate: NodeId, drain: NodeId, source: NodeId, width: f64) {
        self.transistor(name, DeviceKind::Nmos, gate, drain, source, width);
    }

    /// Adds a PMOS transistor with pins in gate, drain, source order.
    pub fn pmos(&mut self, name: &str, gate: NodeId, drain: NodeId, source: NodeId, width: f64) {
        self.transistor(name, DeviceKind::Pmos, gate, drain, source, width);
    }

    /// Instantiates a cell and connects its signal ports.
    pub fn instantiate(
        &mut self,
        name: &str,
        cell: &Cell,
        connections: &[(&str, NodeId)],
    ) -> Result<(), SimError> {
        let subcircuit = cell.to_subcircuit(name)?;
        let mut all = connections.to_vec();
        all.push(("vdd", self.vdd));
        self.builder.instantiate(name, &subcircuit, &all)
    }

    /// Builds a cell with the provided port list.
    pub fn build(self, ports: &[(&str, NodeId)]) -> Result<Cell, SimError> {
        let circuit = self.builder.build_topology()?;
        let mut port_map = BTreeMap::new();
        port_map.insert("vdd".to_string(), self.vdd);
        for (name, node) in ports {
            port_map.insert((*name).to_string(), *node);
        }
        Ok(Cell::new(circuit, port_map))
    }

    /// Adds a CMOS inverter.
    pub fn inverter(&mut self, name: &str, input: NodeId, output: NodeId) {
        let pmos_name = format!("{name}_p");
        let nmos_name = format!("{name}_n");
        self.pmos(&pmos_name, input, output, self.vdd, 2e-6);
        self.nmos(&nmos_name, input, output, self.ground, 1e-6);
    }

    /// Adds a CMOS transmission gate.
    pub fn transmission_gate(
        &mut self,
        name: &str,
        first: NodeId,
        second: NodeId,
        enable: NodeId,
        enable_bar: NodeId,
    ) {
        let nmos_name = format!("{name}_n");
        let pmos_name = format!("{name}_p");
        self.nmos(&nmos_name, enable, second, first, 1e-6);
        self.pmos(&pmos_name, enable_bar, second, first, 2e-6);
    }

    fn transistor(
        &mut self,
        name: &str,
        kind: DeviceKind,
        gate: NodeId,
        drain: NodeId,
        source: NodeId,
        width: f64,
    ) {
        let parameters = mos_params(width);
        self.builder
            .device(name, kind, vec![gate, drain, source], parameters)
            .expect(name);
    }
}

impl Default for CellBuilder {
    fn default() -> Self {
        Self::new()
    }
}

fn mos_params(width: f64) -> BTreeMap<String, f64> {
    let mut map = BTreeMap::new();
    map.insert("vto".to_string(), 0.7);
    map.insert("kp".to_string(), 1e-3);
    map.insert("lambda".to_string(), 0.02);
    map.insert("w".to_string(), width);
    map.insert("l".to_string(), 1e-6);
    map
}
