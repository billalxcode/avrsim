//! Device model API and factory.

use crate::bjt::BjtModel;
use crate::controlled::{CccsModel, CcvsModel, VccsModel, VcvsModel};
use crate::diode::DiodeModel;
use crate::jfet::JfetModel;
use crate::mosfet::MosfetModel;
use crate::passive::{CapacitorModel, InductorModel, ResistorModel};
use crate::source::{CurrentSourceModel, PulseSourceModel, VoltageSourceModel};
use crate::switch::SwitchModel;
use crate::transmission::TransmissionLineModel;
use sim_core::{
    BOLTZMANN, Device, DeviceId, DeviceInstance, DeviceKind, ELEMENTARY_CHARGE, MnaContext,
    NOMINAL_TEMPERATURE, NodeId, SimError, Solution,
};
use std::collections::BTreeMap;

/// Maximum exponent used by junction models.
pub(crate) const MAX_EXPONENT: f64 = 40.0;

/// Maximum forward junction voltage used for limiting.
pub(crate) const MAX_FORWARD_VOLTAGE: f64 = 0.9;

/// Numerical integration method for reactive devices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IntegrationMethod {
    /// Backward Euler.
    #[default]
    BackwardEuler,
    /// Trapezoidal.
    Trapezoidal,
    /// Second-order Gear.
    Gear2,
}

/// A device model that can stamp an MNA system.
pub trait DeviceModel {
    /// Stamps the device into the MNA system.
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError>;

    /// Stamps with an explicit integration method.
    fn stamp_method(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        dt: Option<f64>,
        _method: IntegrationMethod,
    ) -> Result<(), SimError> {
        self.stamp(ctx, solution, dt)
    }

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
        DeviceKind::PulseVoltageSource => Ok(Box::new(PulseSourceModel::from_instance(instance)?)),
        DeviceKind::CurrentSource => Ok(Box::new(CurrentSourceModel::from_instance(instance)?)),
        DeviceKind::Vcvs => Ok(Box::new(VcvsModel::from_instance(instance)?)),
        DeviceKind::Vccs => Ok(Box::new(VccsModel::from_instance(instance)?)),
        DeviceKind::Cccs => Ok(Box::new(CccsModel::from_instance(instance)?)),
        DeviceKind::Ccvs => Ok(Box::new(CcvsModel::from_instance(instance)?)),
        DeviceKind::Switch => Ok(Box::new(SwitchModel::from_instance(instance)?)),
        DeviceKind::Nmos | DeviceKind::Pmos => Ok(Box::new(MosfetModel::from_instance(instance)?)),
        DeviceKind::Npn | DeviceKind::Pnp => Ok(Box::new(BjtModel::from_instance(instance)?)),
        DeviceKind::JfetN | DeviceKind::JfetP => Ok(Box::new(JfetModel::from_instance(instance)?)),
        DeviceKind::TransmissionLine => {
            Ok(Box::new(TransmissionLineModel::from_instance(instance)?))
        }
        DeviceKind::Diode => Ok(Box::new(DiodeModel::from_instance(instance)?)),
        other => Err(SimError::UnsupportedDevice(other.name().to_string())),
    }
}

pub(crate) fn parameter(params: &BTreeMap<String, f64>, name: &str, default: f64) -> f64 {
    params.get(name).copied().unwrap_or(default)
}

pub(crate) fn non_negative_parameter(
    params: &BTreeMap<String, f64>,
    name: &str,
    default: f64,
) -> Result<f64, SimError> {
    let value = parameter(params, name, default);
    if !value.is_finite() || value < 0.0 {
        return Err(SimError::InvalidNetlist(format!(
            "parameter {name} must be non-negative"
        )));
    }
    Ok(value)
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

pub(crate) fn thermal_voltage(params: &BTreeMap<String, f64>) -> Result<f64, SimError> {
    let temperature = parameter(params, "temp", NOMINAL_TEMPERATURE);
    let default = BOLTZMANN * temperature / ELEMENTARY_CHARGE;
    positive_parameter(params, "vt", Some(default))
}

pub(crate) fn depletion_capacitance(zero: f64, potential: f64, grading: f64, voltage: f64) -> f64 {
    if zero <= 0.0 {
        return 0.0;
    }
    let ratio = (1.0 - voltage / potential).max(0.1);
    zero / ratio.powf(grading)
}

/// Returns the critical junction voltage used by limiting.
pub(crate) fn junction_critical_voltage(saturation: f64, thermal_voltage: f64) -> f64 {
    thermal_voltage * (thermal_voltage / (std::f64::consts::SQRT_2 * saturation)).ln()
}

pub(crate) fn terminal_conductance(junction: f64, series_resistance: f64) -> f64 {
    if series_resistance <= 0.0 {
        junction
    } else {
        junction / (1.0 + series_resistance * junction)
    }
}

/// Limits a large junction voltage step.
pub(crate) fn pnjlim(voltage: f64, previous: f64, thermal: f64, critical: f64) -> f64 {
    if voltage <= critical || (voltage - previous).abs() <= 2.0 * thermal {
        return voltage;
    }
    if previous > 0.0 {
        let argument = 1.0 + (voltage - previous) / thermal;
        if argument > 0.0 {
            return previous + thermal * argument.ln();
        }
        return critical;
    }
    thermal * (voltage / thermal).ln()
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

pub(crate) fn pins_three_or_four(instance: &DeviceInstance) -> Result<[NodeId; 4], SimError> {
    let pins = instance.pins();
    if pins.len() != 3 && pins.len() != 4 {
        return Err(SimError::InvalidNetlist(format!(
            "{} requires three or four pins",
            instance.name()
        )));
    }
    let body = if pins.len() == 4 { pins[3] } else { pins[2] };
    Ok([pins[0], pins[1], pins[2], body])
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

pub(crate) fn stamp_capacitor(
    ctx: &mut MnaContext,
    first: NodeId,
    second: NodeId,
    capacitance: f64,
    voltage_prev: f64,
    dt: f64,
) -> Result<(), SimError> {
    if capacitance <= 0.0 {
        return Ok(());
    }
    let conductance = capacitance / dt;
    stamp_conductance(ctx, first, second, conductance)?;
    let a = ctx.node_index(first);
    let b = ctx.node_index(second);
    let current = conductance * voltage_prev;
    ctx.add_rhs_opt(a, current)?;
    ctx.add_rhs_opt(b, -current)
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

pub(crate) fn stamp_inductor_branch(
    ctx: &mut MnaContext,
    id: DeviceId,
    positive: NodeId,
    negative: NodeId,
    resistance: f64,
    rhs: f64,
) -> Result<(), SimError> {
    let branch = branch_index(ctx, id)?;
    let p = ctx.node_index(positive);
    let n = ctx.node_index(negative);
    ctx.add_matrix_opt(p, Some(branch), 1.0)?;
    ctx.add_matrix_opt(n, Some(branch), -1.0)?;
    ctx.add_matrix_opt(Some(branch), p, 1.0)?;
    ctx.add_matrix_opt(Some(branch), n, -1.0)?;
    ctx.add_matrix(branch, branch, -resistance)?;
    ctx.add_rhs(branch, rhs)
}

/// Numerical derivative step for device linearization.
pub(crate) const DERIVATIVE_STEP: f64 = 1e-6;

pub(crate) fn finite_difference<F: Fn(f64) -> f64>(function: F, value: f64, step: f64) -> f64 {
    (function(value + step) - function(value)) / step
}

/// Linearized terminal current and its derivatives.
#[derive(Clone, Copy, Default)]
pub(crate) struct Linearized {
    pub(crate) current: f64,
    pub(crate) gm: f64,
    pub(crate) gds: f64,
    pub(crate) gms: f64,
    pub(crate) gmb: f64,
}

impl Linearized {
    pub(crate) fn from_three(current: f64, gm: f64, gds: f64, gms: f64) -> Self {
        Self {
            current,
            gm,
            gds,
            gms,
            gmb: 0.0,
        }
    }
}

pub(crate) fn stamp_three_terminal(
    ctx: &mut MnaContext,
    pins: [NodeId; 3],
    current: f64,
    gm: f64,
    gds: f64,
    gms: f64,
    voltages: (f64, f64, f64),
) -> Result<(), SimError> {
    let (vg, vd, vs) = voltages;
    let state = Linearized::from_three(current, gm, gds, gms);
    stamp_four_terminal(
        ctx,
        [pins[0], pins[1], pins[2], pins[2]],
        state,
        (vg, vd, vs, vs),
    )
}

pub(crate) fn stamp_four_terminal(
    ctx: &mut MnaContext,
    pins: [NodeId; 4],
    state: Linearized,
    voltages: (f64, f64, f64, f64),
) -> Result<(), SimError> {
    let gate = ctx.node_index(pins[0]);
    let drain = ctx.node_index(pins[1]);
    let source = ctx.node_index(pins[2]);
    let body = ctx.node_index(pins[3]);
    let (vg, vd, vs, vb) = voltages;
    let constant = state.gm * vg + state.gds * vd + state.gms * vs + state.gmb * vb - state.current;
    let cols = (gate, drain, source, body);
    let values = (state.gm, state.gds, state.gms, state.gmb);
    let negative = (-values.0, -values.1, -values.2, -values.3);
    stamp_row(ctx, drain, cols, values, constant)?;
    stamp_row(ctx, source, cols, negative, -constant)
}

fn stamp_row(
    ctx: &mut MnaContext,
    row: Option<usize>,
    cols: (Option<usize>, Option<usize>, Option<usize>, Option<usize>),
    values: (f64, f64, f64, f64),
    constant: f64,
) -> Result<(), SimError> {
    ctx.add_matrix_opt(row, cols.0, values.0)?;
    ctx.add_matrix_opt(row, cols.1, values.1)?;
    ctx.add_matrix_opt(row, cols.2, values.2)?;
    ctx.add_matrix_opt(row, cols.3, values.3)?;
    ctx.add_rhs_opt(row, constant)
}

#[cfg(test)]
mod tests {
    use super::{junction_critical_voltage, parameter, pnjlim, positive_parameter};
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

    #[test]
    fn pnjlim_limits_large_forward_step() {
        let critical = junction_critical_voltage(1e-12, 0.02585);
        let limited = pnjlim(5.0, 0.0, 0.02585, critical);
        assert!(limited > 0.0 && limited < 0.2, "limited={limited}");
    }

    #[test]
    fn pnjlim_keeps_small_step() {
        let critical = junction_critical_voltage(1e-12, 0.02585);
        let value = 0.62;
        assert_eq!(pnjlim(value, 0.60, 0.02585, critical), value);
    }
}
