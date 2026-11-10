//! Subcircuit templates and port definitions.

use super::circuit::Circuit;
use super::ensure_name;
use crate::error::SimError;
use crate::ids::NodeId;
use std::collections::BTreeSet;

/// A named port on a subcircuit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Port {
    name: String,
    node: NodeId,
}

impl Port {
    /// Creates a port.
    #[must_use]
    pub fn new(name: impl Into<String>, node: NodeId) -> Self {
        Self {
            name: name.into(),
            node,
        }
    }

    /// Returns the port name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the local node bound to the port.
    #[must_use]
    pub const fn node(&self) -> NodeId {
        self.node
    }
}

/// A reusable circuit template with named ports.
#[derive(Clone, Debug)]
pub struct Subcircuit {
    name: String,
    template: Circuit,
    ports: Vec<Port>,
}

impl Subcircuit {
    /// Creates a subcircuit from a validated template and port list.
    pub fn new(
        name: impl Into<String>,
        template: Circuit,
        ports: Vec<(String, NodeId)>,
    ) -> Result<Self, SimError> {
        let name = name.into();
        ensure_name(&name)?;
        template.validate()?;
        let ports = validate_ports(&template, ports)?;
        Ok(Self {
            name,
            template,
            ports,
        })
    }

    /// Returns the subcircuit name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the template circuit.
    #[must_use]
    pub fn template(&self) -> &Circuit {
        &self.template
    }

    /// Returns the declared ports.
    #[must_use]
    pub fn ports(&self) -> &[Port] {
        &self.ports
    }

    /// Returns true when a port with this name exists.
    #[must_use]
    pub fn has_port(&self, name: &str) -> bool {
        self.ports.iter().any(|port| port.name() == name)
    }

    /// Returns the port name bound to a node, if any.
    #[must_use]
    pub fn port_name_for(&self, node: NodeId) -> Option<&str> {
        self.ports
            .iter()
            .find(|port| port.node() == node)
            .map(Port::name)
    }
}

fn validate_ports(template: &Circuit, ports: Vec<(String, NodeId)>) -> Result<Vec<Port>, SimError> {
    let mut names = BTreeSet::new();
    let mut nodes = BTreeSet::new();
    let mut result = Vec::with_capacity(ports.len());
    for (name, node) in ports {
        ensure_name(&name)?;
        validate_port_node(template, node, &mut nodes)?;
        if !names.insert(name.clone()) {
            return Err(SimError::InvalidNetlist(format!(
                "duplicate port name: {name}"
            )));
        }
        result.push(Port::new(name, node));
    }
    Ok(result)
}

fn validate_port_node(
    template: &Circuit,
    node: NodeId,
    nodes: &mut BTreeSet<NodeId>,
) -> Result<(), SimError> {
    ensure_not_ground(node)?;
    ensure_template_node(template, node)?;
    ensure_unique_port_node(nodes, node)
}

fn ensure_not_ground(node: NodeId) -> Result<(), SimError> {
    if node == NodeId::GROUND {
        return Err(SimError::InvalidNetlist(
            "ground cannot be a subcircuit port".to_string(),
        ));
    }
    Ok(())
}

fn ensure_template_node(template: &Circuit, node: NodeId) -> Result<(), SimError> {
    if template.node(node).is_none() {
        return Err(SimError::InvalidNetlist(format!(
            "unknown port node: {}",
            node.raw()
        )));
    }
    Ok(())
}

fn ensure_unique_port_node(nodes: &mut BTreeSet<NodeId>, node: NodeId) -> Result<(), SimError> {
    if !nodes.insert(node) {
        return Err(SimError::InvalidNetlist(format!(
            "duplicate port node: {}",
            node.raw()
        )));
    }
    Ok(())
}
