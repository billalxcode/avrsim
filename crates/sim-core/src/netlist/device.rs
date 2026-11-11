//! Structural device model and solver-facing types.

use crate::ids::{DeviceId, NodeId};
use crate::matrix::DenseMatrix;
use std::collections::BTreeMap;

/// The kind of a device in a structural netlist.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeviceKind {
    /// An ideal resistor.
    Resistor,
    /// An ideal capacitor.
    Capacitor,
    /// An ideal inductor.
    Inductor,
    /// An independent voltage source.
    VoltageSource,
    /// An independent pulse voltage source.
    PulseVoltageSource,
    /// An independent current source.
    CurrentSource,
    /// A voltage-controlled voltage source.
    Vcvs,
    /// A voltage-controlled current source.
    Vccs,
    /// An N-channel MOSFET.
    Nmos,
    /// A P-channel MOSFET.
    Pmos,
    /// An NPN bipolar junction transistor.
    Npn,
    /// A PNP bipolar junction transistor.
    Pnp,
    /// An N-channel JFET.
    JfetN,
    /// A P-channel JFET.
    JfetP,
    /// A lossless transmission line.
    TransmissionLine,
    /// A diode.
    Diode,
    /// A model supplied by another crate or a future device library.
    Custom(String),
}

impl DeviceKind {
    /// Returns a stable name for the device kind.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Custom(name) => name,
            other => other.static_name(),
        }
    }

    fn static_name(&self) -> &'static str {
        match self {
            Self::Resistor => "resistor",
            Self::Capacitor => "capacitor",
            Self::Inductor => "inductor",
            Self::VoltageSource => "voltage_source",
            Self::PulseVoltageSource => "pulse_voltage_source",
            Self::CurrentSource => "current_source",
            Self::Vcvs => "vcvs",
            Self::Vccs => "vccs",
            other => other.analog_name(),
        }
    }

    fn analog_name(&self) -> &'static str {
        match self {
            Self::Nmos => "nmos",
            Self::Pmos => "pmos",
            Self::Npn => "npn",
            Self::Pnp => "pnp",
            Self::JfetN => "jfet_n",
            Self::JfetP => "jfet_p",
            Self::TransmissionLine => "transmission_line",
            Self::Diode => "diode",
            _ => "custom",
        }
    }

    /// Returns true when the device can conduct direct current.
    #[must_use]
    pub fn conducts_dc(&self) -> bool {
        !matches!(self, Self::Capacitor)
    }
}

/// A device instance in a structural netlist.
#[derive(Clone, Debug, PartialEq)]
pub struct DeviceInstance {
    id: DeviceId,
    name: String,
    kind: DeviceKind,
    pins: Vec<NodeId>,
    parameters: BTreeMap<String, f64>,
}

impl DeviceInstance {
    /// Creates a device instance.
    #[must_use]
    pub fn new(
        id: DeviceId,
        name: impl Into<String>,
        kind: DeviceKind,
        pins: Vec<NodeId>,
        parameters: BTreeMap<String, f64>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            kind,
            pins,
            parameters,
        }
    }

    /// Returns the device identifier.
    #[must_use]
    pub const fn id(&self) -> DeviceId {
        self.id
    }

    /// Sets or replaces a device parameter.
    pub fn set_parameter(&mut self, name: impl Into<String>, value: f64) {
        self.parameters.insert(name.into(), value);
    }
}

/// Structural view of a device.
pub trait Device {
    /// Returns the device name.
    fn name(&self) -> &str;
    /// Returns the device kind.
    fn kind(&self) -> &DeviceKind;
    /// Returns the device pins.
    fn pins(&self) -> &[NodeId];
    /// Returns the device parameters.
    fn parameters(&self) -> &BTreeMap<String, f64>;
}

impl Device for DeviceInstance {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> &DeviceKind {
        &self.kind
    }

    fn pins(&self) -> &[NodeId] {
        &self.pins
    }

    fn parameters(&self) -> &BTreeMap<String, f64> {
        &self.parameters
    }
}

/// Context for device stamping in a future analysis pass.
#[derive(Debug)]
pub struct StampContext<'a> {
    /// The MNA matrix being assembled.
    pub matrix: &'a mut DenseMatrix,
    /// The right-hand side vector.
    pub rhs: &'a mut Vec<f64>,
    /// Mapping from node identifiers to matrix rows.
    pub node_map: &'a [Option<usize>],
}

/// A solved node-voltage and branch-current vector.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Solution {
    /// Node voltages indexed by node identifier.
    pub node_voltages: Vec<f64>,
    /// Branch currents indexed by MNA matrix index.
    pub branch_currents: Vec<f64>,
}

impl Solution {
    /// Creates a zero solution with the requested node slots and matrix size.
    #[must_use]
    pub fn zero(node_slots: usize, matrix_size: usize) -> Self {
        Self {
            node_voltages: vec![0.0; node_slots],
            branch_currents: vec![0.0; matrix_size],
        }
    }

    /// Returns the voltage for a node, or `None` when it is not present.
    #[must_use]
    pub fn voltage(&self, node: NodeId) -> Option<f64> {
        self.node_voltages.get(node.raw() as usize).copied()
    }

    /// Returns a branch current, or `None` when it is not present.
    #[must_use]
    pub fn branch_current(&self, index: usize) -> Option<f64> {
        self.branch_currents.get(index).copied()
    }
}
