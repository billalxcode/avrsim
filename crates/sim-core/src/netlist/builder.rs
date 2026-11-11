//! Circuit builder and subcircuit instantiation.

use super::circuit::Circuit;
use super::device::{Device, DeviceInstance, DeviceKind};
use super::ensure_name;
use super::node::{Node, NodeKind};
use super::subcircuit::Subcircuit;
use crate::error::SimError;
use crate::ids::{DeviceId, NodeId};
use std::collections::{BTreeMap, BTreeSet};

/// Builds a flat circuit from nodes, devices, and subcircuit instances.
#[derive(Clone, Debug)]
pub struct CircuitBuilder {
    nodes: Vec<Node>,
    devices: Vec<DeviceInstance>,
    node_names: BTreeMap<String, NodeId>,
    device_names: BTreeSet<String>,
    instance_names: BTreeSet<String>,
    next_device: u32,
}

impl CircuitBuilder {
    /// Creates a builder with a ground node at index zero.
    #[must_use]
    pub fn new() -> Self {
        let ground = Node::new(NodeId::GROUND, "0".to_string(), NodeKind::Ground);
        let mut node_names = BTreeMap::new();
        node_names.insert("0".to_string(), NodeId::GROUND);
        Self {
            nodes: vec![ground],
            devices: Vec::new(),
            node_names,
            device_names: BTreeSet::new(),
            instance_names: BTreeSet::new(),
            next_device: 0,
        }
    }

    /// Returns the ground node.
    #[must_use]
    pub const fn ground(&self) -> NodeId {
        NodeId::GROUND
    }

    /// Adds a signal node.
    pub fn node(&mut self, name: impl Into<String>) -> Result<NodeId, SimError> {
        self.node_with_kind(name, NodeKind::Signal)
    }

    /// Adds a supply node.
    pub fn supply(&mut self, name: impl Into<String>) -> Result<NodeId, SimError> {
        self.node_with_kind(name, NodeKind::Supply)
    }

    /// Adds a device with a name, kind, pins, and parameters.
    pub fn device(
        &mut self,
        name: impl Into<String>,
        kind: DeviceKind,
        pins: Vec<NodeId>,
        parameters: BTreeMap<String, f64>,
    ) -> Result<DeviceId, SimError> {
        let name = name.into();
        Self::ensure_new_name(&name, self.device_names.contains(&name))?;
        self.ensure_pins_valid(&pins)?;
        let id = self.allocate_device_id()?;
        self.device_names.insert(name.clone());
        self.devices
            .push(DeviceInstance::new(id, name, kind, pins, parameters));
        Ok(id)
    }

    /// Builds and validates the circuit.
    pub fn build(self) -> Result<Circuit, SimError> {
        let circuit = Circuit::new(self.nodes, self.devices);
        circuit.validate()?;
        Ok(circuit)
    }

    /// Builds a circuit with topology validation only.
    pub fn build_topology(self) -> Result<Circuit, SimError> {
        let circuit = Circuit::new(self.nodes, self.devices);
        circuit.validate_topology()?;
        Ok(circuit)
    }

    /// Instantiates a subcircuit and flattens it into this builder.
    pub fn instantiate(
        &mut self,
        instance_name: &str,
        subcircuit: &Subcircuit,
        connections: &[(&str, NodeId)],
    ) -> Result<(), SimError> {
        let mut staged = self.clone();
        staged.instantiate_in_place(instance_name, subcircuit, connections)?;
        *self = staged;
        Ok(())
    }

    fn node_with_kind(
        &mut self,
        name: impl Into<String>,
        kind: NodeKind,
    ) -> Result<NodeId, SimError> {
        let name = name.into();
        Self::ensure_new_name(&name, self.node_names.contains_key(&name))?;
        let id = self.allocate_node_id()?;
        self.node_names.insert(name.clone(), id);
        self.nodes.push(Node::new(id, name, kind));
        Ok(id)
    }

    fn ensure_pins_valid(&self, pins: &[NodeId]) -> Result<(), SimError> {
        if pins.len() < 2 {
            return Err(SimError::InvalidNetlist(
                "device must have at least two pins".to_string(),
            ));
        }
        for &pin in pins {
            if pin.raw() as usize >= self.nodes.len() {
                return Err(SimError::InvalidNetlist(format!(
                    "unknown node: {}",
                    pin.raw()
                )));
            }
        }
        Ok(())
    }

    fn allocate_node_id(&self) -> Result<NodeId, SimError> {
        let raw = u32::try_from(self.nodes.len())
            .map_err(|_| SimError::Internal("node limit exceeded".to_string()))?;
        Ok(NodeId::new(raw))
    }

    fn allocate_device_id(&mut self) -> Result<DeviceId, SimError> {
        let id = DeviceId::new(self.next_device);
        self.next_device = self
            .next_device
            .checked_add(1)
            .ok_or_else(|| SimError::Internal("device limit exceeded".to_string()))?;
        Ok(id)
    }

    fn instantiate_in_place(
        &mut self,
        instance_name: &str,
        subcircuit: &Subcircuit,
        connections: &[(&str, NodeId)],
    ) -> Result<(), SimError> {
        Self::ensure_new_name(instance_name, self.instance_names.contains(instance_name))?;
        let connection_map = validate_connections(subcircuit, connections)?;
        let mapping = self.build_node_mapping(instance_name, subcircuit, &connection_map)?;
        self.copy_devices(instance_name, subcircuit, &mapping)?;
        self.instance_names.insert(instance_name.to_string());
        Ok(())
    }

    fn build_node_mapping(
        &mut self,
        instance_name: &str,
        subcircuit: &Subcircuit,
        connections: &BTreeMap<String, NodeId>,
    ) -> Result<Vec<NodeId>, SimError> {
        let mut mapping = Vec::with_capacity(subcircuit.template().nodes().len());
        for node in subcircuit.template().nodes() {
            mapping.push(self.map_sub_node(instance_name, node, subcircuit, connections)?);
        }
        Ok(mapping)
    }

    fn map_sub_node(
        &mut self,
        instance_name: &str,
        node: &Node,
        subcircuit: &Subcircuit,
        connections: &BTreeMap<String, NodeId>,
    ) -> Result<NodeId, SimError> {
        if node.id() == NodeId::GROUND {
            return Ok(self.ground());
        }
        if let Some(port) = subcircuit.port_name_for(node.id()) {
            return connections.get(port).copied().ok_or_else(|| {
                SimError::InvalidNetlist(format!("missing connection for port: {port}"))
            });
        }
        self.node(format!("{instance_name}.{}", node.name()))
    }

    fn copy_devices(
        &mut self,
        instance_name: &str,
        subcircuit: &Subcircuit,
        mapping: &[NodeId],
    ) -> Result<(), SimError> {
        for device in subcircuit.template().devices() {
            let pins = self.map_pins(device.pins(), mapping)?;
            let name = format!("{instance_name}.{}", device.name());
            self.device(
                name,
                device.kind().clone(),
                pins,
                device.parameters().clone(),
            )?;
        }
        Ok(())
    }

    fn map_pins(&self, pins: &[NodeId], mapping: &[NodeId]) -> Result<Vec<NodeId>, SimError> {
        pins.iter()
            .map(|pin| {
                mapping
                    .get(pin.raw() as usize)
                    .copied()
                    .ok_or_else(|| SimError::Internal("subcircuit pin mapping missing".to_string()))
            })
            .collect()
    }

    fn ensure_new_name(name: &str, exists: bool) -> Result<(), SimError> {
        ensure_name(name)?;
        if exists {
            return Err(SimError::InvalidNetlist(format!(
                "name already exists: {name}"
            )));
        }
        Ok(())
    }
}

impl Default for CircuitBuilder {
    fn default() -> Self {
        Self::new()
    }
}

fn validate_connections(
    subcircuit: &Subcircuit,
    connections: &[(&str, NodeId)],
) -> Result<BTreeMap<String, NodeId>, SimError> {
    let map = collect_connections(subcircuit, connections)?;
    ensure_all_ports_connected(subcircuit, &map)?;
    Ok(map)
}

fn collect_connections(
    subcircuit: &Subcircuit,
    connections: &[(&str, NodeId)],
) -> Result<BTreeMap<String, NodeId>, SimError> {
    let mut map = BTreeMap::new();
    for (name, node) in connections {
        if !subcircuit.has_port(name) {
            return Err(SimError::InvalidNetlist(format!("unknown port: {name}")));
        }
        if map.insert((*name).to_string(), *node).is_some() {
            return Err(SimError::InvalidNetlist(format!(
                "duplicate connection: {name}"
            )));
        }
    }
    Ok(map)
}

fn ensure_all_ports_connected(
    subcircuit: &Subcircuit,
    connections: &BTreeMap<String, NodeId>,
) -> Result<(), SimError> {
    for port in subcircuit.ports() {
        if !connections.contains_key(port.name()) {
            return Err(SimError::InvalidNetlist(format!(
                "missing port connection: {}",
                port.name()
            )));
        }
    }
    Ok(())
}
