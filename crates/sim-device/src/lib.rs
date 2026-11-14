//! Device models for the simulation core.

#![forbid(unsafe_code)]

mod bjt;
mod catalog;
mod comparator;
mod controlled;
mod diode;
mod jfet;
mod logic;
mod model;
mod mosfet;
mod opamp;
mod passive;
mod regulator;
mod relay;
mod source;
mod switch;
mod transmission;

pub use bjt::{BjtModel, bjt_capacitances};
pub use catalog::{
    CatalogError, ComponentRegistry, ComponentRequest, ComponentSpec, Family, ModelClass, catalog,
};
pub use comparator::ComparatorModel;
pub use controlled::{CccsModel, CcvsModel, VccsModel, VcvsModel};
pub use diode::{DiodeModel, diode_capacitance};
pub use jfet::{JfetModel, jfet_capacitances};
pub use logic::LogicGateModel;
pub use model::{DeviceModel, IntegrationMethod, model_from_instance};
pub use mosfet::{MosfetModel, MosfetParams, mosfet_capacitances};
pub use opamp::OpAmpModel;
pub use passive::{CapacitorModel, InductorModel, ResistorModel};
pub use regulator::RegulatorModel;
pub use relay::RelayModel;
pub use source::{CurrentSourceModel, PulseSourceModel, VoltageSourceModel};
pub use switch::{IdealSwitchModel, SwitchModel};
pub use transmission::TransmissionLineModel;
