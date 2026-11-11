//! Shichman-Hodges Level 1 MOSFET model.

use crate::model::{DeviceModel, parameter, pins_exact, positive_parameter};
use sim_core::{
    Device, DeviceInstance, DeviceKind, MnaContext, NOMINAL_TEMPERATURE, NodeId, SimError, Solution,
};
use std::collections::BTreeMap;

const DERIVATIVE_STEP: f64 = 1e-6;

/// Level 1 MOSFET parameters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MosfetParams {
    vto: f64,
    kp: f64,
    lambda: f64,
    width: f64,
    length: f64,
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
        }
    }

    /// Builds parameters from a device parameter map.
    pub fn from_map(params: &BTreeMap<String, f64>) -> Result<Self, SimError> {
        let temperature = parameter(params, "temp", NOMINAL_TEMPERATURE);
        let tcv = parameter(params, "tcv", 0.0);
        let vto = parameter(params, "vto", 0.7) + tcv * (temperature - NOMINAL_TEMPERATURE);
        if !vto.is_finite() {
            return Err(SimError::InvalidNetlist(
                "mosfet vto must be finite".to_string(),
            ));
        }
        Ok(Self {
            vto,
            kp: positive_parameter(params, "kp", Some(1e-3))?,
            lambda: parameter(params, "lambda", 0.0),
            width: positive_parameter(params, "w", Some(1e-6))?,
            length: positive_parameter(params, "l", Some(1e-6))?,
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
}

/// A three-terminal MOSFET model.
pub struct MosfetModel {
    kind: DeviceKind,
    params: MosfetParams,
    pins: [NodeId; 3],
}

impl MosfetModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 3)?;
        let params = MosfetParams::from_map(instance.parameters())?;
        Ok(Self {
            kind: instance.kind().clone(),
            params,
            pins: [pins[0], pins[1], pins[2]],
        })
    }

    fn current(&self, vg: f64, vd: f64, vs: f64) -> f64 {
        match self.kind {
            DeviceKind::Nmos => nmos_current(vg - vs, vd - vs, self.params),
            DeviceKind::Pmos => -nmos_current(vs - vg, vs - vd, self.params),
            _ => 0.0,
        }
    }

    fn linearization(&self, vg: f64, vd: f64, vs: f64) -> (f64, f64, f64, f64) {
        let current = self.current(vg, vd, vs);
        let gm = (self.current(vg + DERIVATIVE_STEP, vd, vs) - current) / DERIVATIVE_STEP;
        let gds = (self.current(vg, vd + DERIVATIVE_STEP, vs) - current) / DERIVATIVE_STEP;
        let gms = (self.current(vg, vd, vs + DERIVATIVE_STEP) - current) / DERIVATIVE_STEP;
        (current, gm, gds, gms)
    }
}

impl DeviceModel for MosfetModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        let vg = solution.voltage(self.pins[0]).unwrap_or(0.0);
        let vd = solution.voltage(self.pins[1]).unwrap_or(0.0);
        let vs = solution.voltage(self.pins[2]).unwrap_or(0.0);
        let (current, gm, gds, gms) = self.linearization(vg, vd, vs);
        stamp_linearized(ctx, self.pins, current, gm, gds, gms, (vg, vd, vs))
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }
}

fn nmos_current(vgs: f64, vds: f64, params: MosfetParams) -> f64 {
    let overdrive = vgs - params.vto;
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

fn stamp_linearized(
    ctx: &mut MnaContext,
    pins: [NodeId; 3],
    current: f64,
    gm: f64,
    gds: f64,
    gms: f64,
    voltages: (f64, f64, f64),
) -> Result<(), SimError> {
    let gate = ctx.node_index(pins[0]);
    let drain = ctx.node_index(pins[1]);
    let source = ctx.node_index(pins[2]);
    let (vg, vd, vs) = voltages;
    let constant = gm * vg + gds * vd + gms * vs - current;
    let cols = (gate, drain, source);
    stamp_row(ctx, drain, cols, (gm, gds, gms), constant)?;
    stamp_row(ctx, source, cols, (-gm, -gds, -gms), -constant)
}

fn stamp_row(
    ctx: &mut MnaContext,
    row: Option<usize>,
    cols: (Option<usize>, Option<usize>, Option<usize>),
    values: (f64, f64, f64),
    constant: f64,
) -> Result<(), SimError> {
    ctx.add_matrix_opt(row, cols.0, values.0)?;
    ctx.add_matrix_opt(row, cols.1, values.1)?;
    ctx.add_matrix_opt(row, cols.2, values.2)?;
    ctx.add_rhs_opt(row, constant)
}

#[cfg(test)]
mod tests {
    use super::{MosfetParams, nmos_current};

    const VTO: f64 = 0.5;

    fn params() -> MosfetParams {
        MosfetParams::new(VTO, 1e-3, 0.0, 1e-6, 1e-6)
    }

    #[test]
    fn cutoff_current_is_zero() {
        assert_eq!(nmos_current(0.4, 1.0, params()), 0.0);
        assert_eq!(nmos_current(0.0, 1.0, params()), 0.0);
    }

    #[test]
    fn linear_region_matches_formula() {
        let expected = 1e-3 * (0.5 * 0.2 - 0.5 * 0.2 * 0.2);
        let actual = nmos_current(1.0, 0.2, params());
        assert!((actual - expected).abs() < 1e-12);
    }

    #[test]
    fn saturation_region_matches_formula() {
        let expected = 0.5 * 1e-3 * 0.5 * 0.5;
        let actual = nmos_current(1.0, 1.0, params());
        assert!((actual - expected).abs() < 1e-12);
    }

    #[test]
    fn lambda_raises_saturation_current() {
        let base = nmos_current(1.0, 1.0, params());
        let modulated = nmos_current(1.0, 1.0, MosfetParams::new(VTO, 1e-3, 0.1, 1e-6, 1e-6));
        assert!(modulated > base);
    }

    #[test]
    fn width_over_length_scales_current() {
        let base = nmos_current(1.0, 1.0, params());
        let wider = nmos_current(1.0, 1.0, MosfetParams::new(VTO, 1e-3, 0.0, 2e-6, 1e-6));
        assert!((wider - 2.0 * base).abs() < 1e-12);
    }

    #[test]
    fn numeric_derivative_matches_analytic() {
        let params = params();
        let step = 1e-6;
        let base = nmos_current(1.0, 0.2, params);
        let gm = (nmos_current(1.0 + step, 0.2, params) - base) / step;
        let gds = (nmos_current(1.0, 0.2 + step, params) - base) / step;
        assert!((gm - 1e-3 * 0.2).abs() < 1e-6);
        assert!((gds - 1e-3 * 0.3).abs() < 1e-6);
    }
}
