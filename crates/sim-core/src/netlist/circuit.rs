//! Flat circuit representation and validation.

use super::device::{Device, DeviceInstance};
use super::node::{Node, NodeKind};
use crate::error::SimError;
use crate::ids::{DeviceId, NodeId};
use std::collections::BTreeSet;

/// A validated, flat circuit netlist.
#[derive(Clone, Debug, PartialEq)]
pub struct Circuit {
    nodes: Vec<Node>,
    devices: Vec<DeviceInstance>,
    ground: NodeId,
}

impl Circuit {
    pub(crate) fn new(nodes: Vec<Node>, devices: Vec<DeviceInstance>) -> Self {
        Self {
            nodes,
            devices,
            ground: NodeId::GROUND,
        }
    }

    /// Returns the ground node.
    #[must_use]
    pub const fn ground(&self) -> NodeId {
        self.ground
    }

    /// Returns all nodes.
    #[must_use]
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    /// Returns all devices.
    #[must_use]
    pub fn devices(&self) -> &[DeviceInstance] {
        &self.devices
    }

    /// Returns a node by identifier.
    #[must_use]
    pub fn node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id.raw() as usize)
    }

    /// Returns a device by identifier.
    #[must_use]
    pub fn device(&self, id: DeviceId) -> Option<&DeviceInstance> {
        self.devices.get(id.raw() as usize)
    }

    /// Returns a node by name.
    #[must_use]
    pub fn node_by_name(&self, name: &str) -> Option<&Node> {
        self.nodes.iter().find(|node| node.name() == name)
    }

    /// Returns a device by name.
    #[must_use]
    pub fn device_by_name(&self, name: &str) -> Option<&DeviceInstance> {
        self.devices.iter().find(|device| device.name() == name)
    }

    /// Returns an independent copy of the netlist.
    #[must_use]
    pub fn snapshot(&self) -> Self {
        self.clone()
    }

    /// Validates ground, names, node usage, and DC connectivity.
    pub fn validate(&self) -> Result<(), SimError> {
        self.validate_ground()?;
        self.validate_unique_names()?;
        self.validate_node_usage()?;
        self.validate_dc_paths()
    }

    fn validate_ground(&self) -> Result<(), SimError> {
        let ground = self
            .node(self.ground)
            .ok_or_else(|| SimError::InvalidNetlist("ground node is missing".to_string()))?;
        if ground.kind() != NodeKind::Ground {
            return Err(SimError::InvalidNetlist(
                "node zero is not ground".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_unique_names(&self) -> Result<(), SimError> {
        validate_node_names(&self.nodes)?;
        validate_device_names(&self.devices)
    }

    fn validate_node_usage(&self) -> Result<(), SimError> {
        let degrees = self.node_degrees()?;
        self.ensure_nodes_used(&degrees)
    }

    fn node_degrees(&self) -> Result<Vec<usize>, SimError> {
        let mut degrees = vec![0usize; self.nodes.len()];
        for device in &self.devices {
            for &pin in device.pins() {
                let index = pin.raw() as usize;
                if index >= degrees.len() {
                    return Err(SimError::InvalidNetlist(format!(
                        "unknown node in device {}",
                        device.name()
                    )));
                }
                degrees[index] += 1;
            }
        }
        Ok(degrees)
    }

    fn ensure_nodes_used(&self, degrees: &[usize]) -> Result<(), SimError> {
        for node in &self.nodes {
            let index = node.id().raw() as usize;
            if !node.kind().is_ground() && degrees[index] == 0 {
                return Err(SimError::FloatingNode(node.id()));
            }
        }
        Ok(())
    }

    fn validate_dc_paths(&self) -> Result<(), SimError> {
        let adjacency = self.build_dc_adjacency();
        let visited = mark_reachable(&adjacency, self.ground);
        for node in &self.nodes {
            let index = node.id().raw() as usize;
            if !node.kind().is_ground() && !visited[index] {
                return Err(SimError::NoDcPath { node: node.id() });
            }
        }
        Ok(())
    }

    fn build_dc_adjacency(&self) -> Vec<Vec<NodeId>> {
        let mut adjacency = vec![Vec::new(); self.nodes.len()];
        for device in &self.devices {
            if device.kind().conducts_dc() {
                connect_pins(device.pins(), &mut adjacency);
            }
        }
        adjacency
    }
}

fn connect_pins(pins: &[NodeId], adjacency: &mut [Vec<NodeId>]) {
    for (index, &first) in pins.iter().enumerate() {
        for &second in &pins[index + 1..] {
            adjacency[first.raw() as usize].push(second);
            adjacency[second.raw() as usize].push(first);
        }
    }
}

fn validate_node_names(nodes: &[Node]) -> Result<(), SimError> {
    let mut names = BTreeSet::new();
    for node in nodes {
        if !names.insert(node.name()) {
            return Err(SimError::InvalidNetlist(format!(
                "duplicate node name: {}",
                node.name()
            )));
        }
    }
    Ok(())
}

fn validate_device_names(devices: &[DeviceInstance]) -> Result<(), SimError> {
    let mut names = BTreeSet::new();
    for device in devices {
        if !names.insert(device.name()) {
            return Err(SimError::InvalidNetlist(format!(
                "duplicate device name: {}",
                device.name()
            )));
        }
    }
    Ok(())
}

fn mark_reachable(adjacency: &[Vec<NodeId>], start: NodeId) -> Vec<bool> {
    let mut visited = vec![false; adjacency.len()];
    let mut stack = vec![start];
    while let Some(node) = stack.pop() {
        let index = node.raw() as usize;
        if index >= visited.len() || visited[index] {
            continue;
        }
        visited[index] = true;
        stack.extend(adjacency[index].iter().copied());
    }
    visited
}
