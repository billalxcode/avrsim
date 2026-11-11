//! Shichman-Hodges Level 1 MOSFET model.

use crate::model::{
    DERIVATIVE_STEP, DeviceModel, Linearized, finite_difference, non_negative_parameter, parameter,
    pins_three_or_four, positive_parameter, stamp_capacitor, stamp_four_terminal,
};
use sim_core::{
    Device, DeviceInstance, DeviceKind, MnaContext, NOMINAL_TEMPERATURE, NodeId, SimError, Solution,
};
use std::collections::BTreeMap;

/// Level 1 MOSFET parameters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MosfetParams {
    vto: f64,
    kp: f64,
    lambda: f64,
    width: f64,
    length: f64,
    gamma: f64,
    phi: f64,
    cox: f64,
    cgso: f64,
    cgdo: f64,
}

impl MosfetParams {
    /// Creates a parameter set.
    #[must_use]
    pub fn new(vto: f64, kp: f64, lambda: f64, width: f64, length: f64) -> Self {
        Self {
            vto,
            kp,
            lambda,
            width,
            length,
            gamma: 0.0,
            phi: 0.7,
            cox: 0.0,
            cgso: 0.0,
            cgdo: 0.0,
        }
    }

    /// Builds parameters from a device parameter map.
    pub fn from_map(params: &BTreeMap<String, f64>) -> Result<Self, SimError> {
        let vto = threshold_from_map(params)?;
        let gamma = finite_parameter(params, "gamma", 0.0)?;
        let (cox, cgso, cgdo) = capacitance_from_map(params)?;
        Ok(Self {
            vto,
            kp: positive_parameter(params, "kp", Some(1e-3))?,
            lambda: parameter(params, "lambda", 0.0),
            width: positive_parameter(params, "w", Some(1e-6))?,
            length: positive_parameter(params, "l", Some(1e-6))?,
            gamma,
            phi: positive_parameter(params, "phi", Some(0.7))?,
            cox,
            cgso,
            cgdo,
        })
    }
    /// Returns the threshold voltage.
    #[must_use]
    pub const fn vto(self) -> f64 {
        self.vto
    }

    /// Returns the transconductance parameter.
    #[must_use]
    pub const fn kp(self) -> f64 {
        self.kp
    }

    /// Returns the channel-length modulation parameter.
    #[must_use]
    pub const fn lambda(self) -> f64 {
        self.lambda
    }

    /// Returns the channel width.
    #[must_use]
    pub const fn width(self) -> f64 {
        self.width
    }

    /// Returns the channel length.
    #[must_use]
    pub const fn length(self) -> f64 {
        self.length
    }

    /// Returns the body effect parameter.
    #[must_use]
    pub const fn gamma(self) -> f64 {
        self.gamma
    }

    /// Returns the surface potential.
    #[must_use]
    pub const fn phi(self) -> f64 {
        self.phi
    }
}

fn finite_parameter(
    params: &BTreeMap<String, f64>,
    name: &str,
    default: f64,
) -> Result<f64, SimError> {
    let value = parameter(params, name, default);
    if !value.is_finite() {
        return Err(SimError::InvalidNetlist(format!(
            "mosfet {name} must be finite"
        )));
    }
    Ok(value)
}

fn threshold_from_map(params: &BTreeMap<String, f64>) -> Result<f64, SimError> {
    let temperature = parameter(params, "temp", NOMINAL_TEMPERATURE);
    let tcv = parameter(params, "tcv", 0.0);
    let vto = parameter(params, "vto", 0.7) + tcv * (temperature - NOMINAL_TEMPERATURE);
    if !vto.is_finite() {
        return Err(SimError::InvalidNetlist(
            "mosfet vto must be finite".to_string(),
        ));
    }
    Ok(vto)
}

fn capacitance_from_map(params: &BTreeMap<String, f64>) -> Result<(f64, f64, f64), SimError> {
    Ok((
        non_negative_parameter(params, "cox", 0.0)?,
        non_negative_parameter(params, "cgso", 0.0)?,
        non_negative_parameter(params, "cgdo", 0.0)?,
    ))
}

/// A three-terminal or four-terminal MOSFET model.
pub struct MosfetModel {
    kind: DeviceKind,
    params: MosfetParams,
    pins: [NodeId; 4],
    vgs_prev: f64,
    vgd_prev: f64,
    vgb_prev: f64,
}

impl MosfetModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_three_or_four(instance)?;
        let params = MosfetParams::from_map(instance.parameters())?;
        Ok(Self {
            kind: instance.kind().clone(),
            params,
            pins,
            vgs_prev: 0.0,
            vgd_prev: 0.0,
            vgb_prev: 0.0,
        })
    }

    fn sign(&self) -> f64 {
        if matches!(self.kind, DeviceKind::Pmos) {
            -1.0
        } else {
            1.0
        }
    }

    fn current(&self, vg: f64, vd: f64, vs: f64, vb: f64) -> f64 {
        let vbs = self.sign() * (vb - vs);
        match self.kind {
            DeviceKind::Nmos => nmos_current_body(vg - vs, vd - vs, vbs, self.params),
            DeviceKind::Pmos => -nmos_current_body(vs - vg, vs - vd, vbs, self.params),
            _ => 0.0,
        }
    }

    fn linearization(&self, vg: f64, vd: f64, vs: f64, vb: f64) -> Linearized {
        let current = self.current(vg, vd, vs, vb);
        let gm = finite_difference(|value| self.current(value, vd, vs, vb), vg, DERIVATIVE_STEP);
        let gds = finite_difference(|value| self.current(vg, value, vs, vb), vd, DERIVATIVE_STEP);
        let gms = finite_difference(|value| self.current(vg, vd, value, vb), vs, DERIVATIVE_STEP);
        let gmb = finite_difference(|value| self.current(vg, vd, vs, value), vb, DERIVATIVE_STEP);
        Linearized {
            current,
            gm,
            gds,
            gms,
            gmb,
        }
    }

    fn voltages(&self, solution: &Solution) -> (f64, f64, f64, f64) {
        (
            solution.voltage(self.pins[0]).unwrap_or(0.0),
            solution.voltage(self.pins[1]).unwrap_or(0.0),
            solution.voltage(self.pins[2]).unwrap_or(0.0),
            solution.voltage(self.pins[3]).unwrap_or(0.0),
        )
    }

    fn capacitances(&self, vgs: f64, vds: f64, vbs: f64) -> (f64, f64, f64) {
        let overdrive = vgs - threshold(self.params, vbs);
        if overdrive <= 0.0 {
            return (self.params.cgso, self.params.cgdo, self.params.cox);
        }
        if vds < overdrive {
            return (
                self.params.cgso + 0.5 * self.params.cox,
                self.params.cgdo + 0.5 * self.params.cox,
                0.0,
            );
        }
        (
            self.params.cgso + 2.0 / 3.0 * self.params.cox,
            self.params.cgdo,
            0.0,
        )
    }

    fn stamp_caps(
        &self,
        ctx: &mut MnaContext,
        dt: Option<f64>,
        voltages: (f64, f64, f64, f64),
    ) -> Result<(), SimError> {
        let dt = match dt {
            Some(value) => value,
            None => return Ok(()),
        };
        let (vg, vd, vs, vb) = voltages;
        let sign = self.sign();
        let caps = self.capacitances(sign * (vg - vs), sign * (vd - vs), sign * (vb - vs));
        stamp_capacitor(ctx, self.pins[0], self.pins[2], caps.0, self.vgs_prev, dt)?;
        stamp_capacitor(ctx, self.pins[0], self.pins[1], caps.1, self.vgd_prev, dt)?;
        stamp_capacitor(ctx, self.pins[0], self.pins[3], caps.2, self.vgb_prev, dt)
    }
}

impl DeviceModel for MosfetModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError> {
        let voltages = self.voltages(solution);
        let (vg, vd, vs, vb) = voltages;
        let state = self.linearization(vg, vd, vs, vb);
        self.stamp_caps(ctx, dt, voltages)?;
        stamp_four_terminal(ctx, self.pins, state, voltages)
    }

    fn update(&mut self, solution: &Solution) -> Result<(), SimError> {
        let (vg, vd, vs, vb) = self.voltages(solution);
        self.vgs_prev = vg - vs;
        self.vgd_prev = vg - vd;
        self.vgb_prev = vg - vb;
        Ok(())
    }
}

fn nmos_current_body(vgs: f64, vds: f64, vbs: f64, params: MosfetParams) -> f64 {
    let overdrive = vgs - threshold(params, vbs);
    if overdrive <= 0.0 {
        return 0.0;
    }
    let gain = params.kp * (params.width / params.length);
    let lambda = 1.0 + params.lambda * vds;
    if vds < overdrive {
        gain * (overdrive * vds - 0.5 * vds * vds) * lambda
    } else {
        0.5 * gain * overdrive * overdrive * lambda
    }
}

fn threshold(params: MosfetParams, vbs: f64) -> f64 {
    let bias = (params.phi - vbs).max(0.0);
    params.vto + params.gamma * (bias.sqrt() - params.phi.sqrt())
}

#[cfg(test)]
mod tests {
    use super::{MosfetModel, MosfetParams, nmos_current_body, threshold};
    use sim_core::{DeviceKind, NodeId};

    const VTO: f64 = 0.5;

    fn params() -> MosfetParams {
        MosfetParams::new(VTO, 1e-3, 0.0, 1e-6, 1e-6)
    }

    fn body_params() -> MosfetParams {
        MosfetParams {
            vto: VTO,
            kp: 1e-3,
            lambda: 0.0,
            width: 1e-6,
            length: 1e-6,
            gamma: 0.5,
            phi: 0.7,
            cox: 0.0,
            cgso: 0.0,
            cgdo: 0.0,
        }
    }

    fn cap_params() -> MosfetParams {
        MosfetParams {
            vto: VTO,
            kp: 1e-3,
            lambda: 0.0,
            width: 1e-6,
            length: 1e-6,
            gamma: 0.0,
            phi: 0.7,
            cox: 1e-12,
            cgso: 1e-13,
            cgdo: 2e-13,
        }
    }

    fn model(kind: DeviceKind, params: MosfetParams) -> MosfetModel {
        MosfetModel {
            kind,
            params,
            pins: [
                NodeId::new(1),
                NodeId::new(2),
                NodeId::GROUND,
                NodeId::new(3),
            ],
            vgs_prev: 0.0,
            vgd_prev: 0.0,
            vgb_prev: 0.0,
        }
    }

    #[test]
    fn cutoff_current_is_zero() {
        assert_eq!(nmos_current_body(0.4, 1.0, 0.0, params()), 0.0);
        assert_eq!(nmos_current_body(0.0, 1.0, 0.0, params()), 0.0);
    }

    #[test]
    fn linear_region_matches_formula() {
        let expected = 1e-3 * (0.5 * 0.2 - 0.5 * 0.2 * 0.2);
        let actual = nmos_current_body(1.0, 0.2, 0.0, params());
        assert!((actual - expected).abs() < 1e-12);
    }

    #[test]
    fn saturation_region_matches_formula() {
        let expected = 0.5 * 1e-3 * 0.5 * 0.5;
        let actual = nmos_current_body(1.0, 1.0, 0.0, params());
        assert!((actual - expected).abs() < 1e-12);
    }

    #[test]
    fn lambda_raises_saturation_current() {
        let base = nmos_current_body(1.0, 1.0, 0.0, params());
        let modulated =
            nmos_current_body(1.0, 1.0, 0.0, MosfetParams::new(VTO, 1e-3, 0.1, 1e-6, 1e-6));
        assert!(modulated > base);
    }

    #[test]
    fn width_over_length_scales_current() {
        let base = nmos_current_body(1.0, 1.0, 0.0, params());
        let wider = nmos_current_body(1.0, 1.0, 0.0, MosfetParams::new(VTO, 1e-3, 0.0, 2e-6, 1e-6));
        assert!((wider - 2.0 * base).abs() < 1e-12);
    }

    #[test]
    fn numeric_derivative_matches_analytic() {
        let params = params();
        let step = 1e-6;
        let base = nmos_current_body(1.0, 0.2, 0.0, params);
        let gm = (nmos_current_body(1.0 + step, 0.2, 0.0, params) - base) / step;
        let gds = (nmos_current_body(1.0, 0.2 + step, 0.0, params) - base) / step;
        assert!((gm - 1e-3 * 0.2).abs() < 1e-6);
        assert!((gds - 1e-3 * 0.3).abs() < 1e-6);
    }

    #[test]
    fn body_bias_raises_threshold_and_lowers_current() {
        let base = nmos_current_body(1.0, 1.0, 0.0, body_params());
        let biased = nmos_current_body(1.0, 1.0, -1.0, body_params());
        assert!(biased < base);
        assert!(threshold(body_params(), -1.0) > threshold(body_params(), 0.0));
    }

    #[test]
    fn gmb_is_positive_for_reverse_body_bias() {
        let model = model(DeviceKind::Nmos, body_params());
        let gmb = model.linearization(1.2, 1.0, 0.0, -1.0).gmb;
        assert!(gmb > 0.0, "gmb={gmb}");
    }

    #[test]
    fn capacitance_partition_matches_region() {
        let model = model(DeviceKind::Nmos, cap_params());
        let cutoff = model.capacitances(0.4, 1.0, 0.0);
        assert!((cutoff.0 - 1e-13).abs() < 1e-20);
        assert!((cutoff.2 - 1e-12).abs() < 1e-20);
        let linear = model.capacitances(1.0, 0.2, 0.0);
        assert!((linear.0 - 6e-13).abs() < 1e-20);
        assert!((linear.1 - 7e-13).abs() < 1e-20);
        let saturation = model.capacitances(1.0, 1.0, 0.0);
        assert!((saturation.0 - (1e-13 + 2.0 / 3.0 * 1e-12)).abs() < 1e-20);
        assert!((saturation.1 - 2e-13).abs() < 1e-20);
    }
}
