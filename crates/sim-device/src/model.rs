//! Device model API and factory.

use crate::diode::DiodeModel;
use crate::mosfet::MosfetModel;
use crate::passive::{CapacitorModel, InductorModel, ResistorModel};
use crate::source::{CurrentSourceModel, VoltageSourceModel};
use sim_core::{
    Device, DeviceId, DeviceInstance, DeviceKind, MnaContext, NodeId, SimError, Solution,
};
use std::collections::BTreeMap;

/// A device model that can stamp an MNA system.
pub trait DeviceModel {
    /// Stamps the device into the MNA system.
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError>;

    /// Updates internal state after a converged solution.
    fn update(&mut self, solution: &Solution) -> Result<(), SimError>;
}

/// Builds a model from a structural device instance.
pub fn model_from_instance(instance: &DeviceInstance) -> Result<Box<dyn DeviceModel>, SimError> {
    match instance.kind() {
        DeviceKind::Resistor => Ok(Box::new(ResistorModel::from_instance(instance)?)),
        DeviceKind::Capacitor => Ok(Box::new(CapacitorModel::from_instance(instance)?)),
        DeviceKind::Inductor => Ok(Box::new(InductorModel::from_instance(instance)?)),
        DeviceKind::VoltageSource => Ok(Box::new(VoltageSourceModel::from_instance(instance)?)),
        DeviceKind::CurrentSource => Ok(Box::new(CurrentSourceModel::from_instance(instance)?)),
        DeviceKind::Nmos | DeviceKind::Pmos => Ok(Box::new(MosfetModel::from_instance(instance)?)),
        DeviceKind::Diode => Ok(Box::new(DiodeModel::from_instance(instance)?)),
        other => Err(SimError::UnsupportedDevice(other.name().to_string())),
    }
}

pub(crate) fn parameter(params: &BTreeMap<String, f64>, name: &str, default: f64) -> f64 {
    params.get(name).copied().unwrap_or(default)
}

pub(crate) fn positive_parameter(
    params: &BTreeMap<String, f64>,
    name: &str,
    default: Option<f64>,
) -> Result<f64, SimError> {
    let value = match default {
        Some(default) => parameter(params, name, default),
        None => params
            .get(name)
            .copied()
            .ok_or_else(|| SimError::InvalidNetlist(format!("missing parameter: {name}")))?,
    };
    if !value.is_finite() || value <= 0.0 {
        return Err(SimError::InvalidNetlist(format!(
            "parameter {name} must be positive"
        )));
    }
    Ok(value)
}

pub(crate) fn pins_exact(instance: &DeviceInstance, count: usize) -> Result<Vec<NodeId>, SimError> {
    let pins = instance.pins();
    if pins.len() != count {
        return Err(SimError::InvalidNetlist(format!(
            "{} requires {count} pins",
            instance.name()
        )));
    }
    Ok(pins.to_vec())
}

pub(crate) fn branch_index(ctx: &MnaContext, id: DeviceId) -> Result<usize, SimError> {
    ctx.branch_index(id)
        .ok_or_else(|| SimError::Internal(format!("missing branch for device {}", id.raw())))
}

pub(crate) fn stamp_conductance(
    ctx: &mut MnaContext,
    first: NodeId,
    second: NodeId,
    conductance: f64,
) -> Result<(), SimError> {
    let a = ctx.node_index(first);
    let b = ctx.node_index(second);
    ctx.add_matrix_opt(a, a, conductance)?;
    ctx.add_matrix_opt(a, b, -conductance)?;
    ctx.add_matrix_opt(b, a, -conductance)?;
    ctx.add_matrix_opt(b, b, conductance)?;
    Ok(())
}

pub(crate) fn stamp_current_source(
    ctx: &mut MnaContext,
    from: NodeId,
    to: NodeId,
    current: f64,
) -> Result<(), SimError> {
    let from_index = ctx.node_index(from);
    let to_index = ctx.node_index(to);
    ctx.add_rhs_opt(from_index, -current)?;
    ctx.add_rhs_opt(to_index, current)?;
    Ok(())
}

pub(crate) fn stamp_voltage_branch(
    ctx: &mut MnaContext,
    id: DeviceId,
    positive: NodeId,
    negative: NodeId,
    voltage: f64,
) -> Result<(), SimError> {
    let branch = branch_index(ctx, id)?;
    let p = ctx.node_index(positive);
    let n = ctx.node_index(negative);
    ctx.add_matrix_opt(p, Some(branch), 1.0)?;
    ctx.add_matrix_opt(n, Some(branch), -1.0)?;
    ctx.add_matrix_opt(Some(branch), p, 1.0)?;
    ctx.add_matrix_opt(Some(branch), n, -1.0)?;
    ctx.add_rhs(branch, voltage)
}

pub(crate) fn stamp_inductor_transient(
    ctx: &mut MnaContext,
    id: DeviceId,
    positive: NodeId,
    negative: NodeId,
    inductance: f64,
    dt: f64,
    current_prev: f64,
) -> Result<(), SimError> {
    let branch = branch_index(ctx, id)?;
    let p = ctx.node_index(positive);
    let n = ctx.node_index(negative);
    let resistance = inductance / dt;
    ctx.add_matrix_opt(p, Some(branch), 1.0)?;
    ctx.add_matrix_opt(n, Some(branch), -1.0)?;
    ctx.add_matrix_opt(Some(branch), p, 1.0)?;
    ctx.add_matrix_opt(Some(branch), n, -1.0)?;
    ctx.add_matrix(branch, branch, -resistance)?;
    ctx.add_rhs(branch, -resistance * current_prev)
}

#[cfg(test)]
mod tests {
    use super::{parameter, positive_parameter};
    use std::collections::BTreeMap;

    fn map(pairs: &[(&str, f64)]) -> BTreeMap<String, f64> {
        pairs
            .iter()
            .map(|(name, value)| ((*name).to_string(), *value))
            .collect()
    }

    #[test]
    fn parameter_uses_value_when_present() {
        assert_eq!(parameter(&map(&[("x", 2.0)]), "x", 1.0), 2.0);
    }

    #[test]
    fn parameter_uses_default_when_missing() {
        assert_eq!(parameter(&map(&[]), "x", 1.0), 1.0);
    }

    #[test]
    fn positive_parameter_accepts_positive() {
        assert_eq!(
            positive_parameter(&map(&[("x", 2.0)]), "x", None).unwrap(),
            2.0
        );
    }

    #[test]
    fn positive_parameter_rejects_zero() {
        assert!(positive_parameter(&map(&[("x", 0.0)]), "x", None).is_err());
    }

    #[test]
    fn positive_parameter_rejects_negative() {
        assert!(positive_parameter(&map(&[("x", -1.0)]), "x", None).is_err());
    }

    #[test]
    fn positive_parameter_rejects_missing() {
        assert!(positive_parameter(&map(&[]), "x", None).is_err());
    }

    #[test]
    fn positive_parameter_rejects_nan() {
        assert!(positive_parameter(&map(&[("x", f64::NAN)]), "x", None).is_err());
    }
}
