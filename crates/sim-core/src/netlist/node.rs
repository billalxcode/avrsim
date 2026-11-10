//! Netlist nodes.

use crate::ids::NodeId;

/// The role of a node in a circuit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeKind {
    /// The reference node. It is always node zero.
    Ground,
    /// A normal signal node.
    Signal,
    /// A power or supply node.
    Supply,
}

impl NodeKind {
    /// Returns true for the ground node kind.
    #[must_use]
    pub const fn is_ground(self) -> bool {
        matches!(self, Self::Ground)
    }
}

/// A named node in a flat netlist.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    id: NodeId,
    name: String,
    kind: NodeKind,
}

impl Node {
    /// Creates a node.
    #[must_use]
    pub fn new(id: NodeId, name: impl Into<String>, kind: NodeKind) -> Self {
        Self {
            id,
            name: name.into(),
            kind,
        }
    }

    /// Returns the node identifier.
    #[must_use]
    pub const fn id(&self) -> NodeId {
        self.id
    }

    /// Returns the node name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the node kind.
    #[must_use]
    pub const fn kind(&self) -> NodeKind {
        self.kind
    }
}
