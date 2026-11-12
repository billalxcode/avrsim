//! Shichman-Hodges junction field-effect transistor model.

use crate::model::{
    DERIVATIVE_STEP, DeviceModel, finite_difference, non_negative_parameter, parameter, pins_exact,
    positive_parameter, stamp_capacitor, stamp_three_terminal,
};
use sim_core::{Device, DeviceInstance, DeviceKind, MnaContext, NodeId, SimError, Solution};
use std::collections::BTreeMap;

/// A three-terminal JFET with pins ordered drain, gate, source.
pub struct JfetModel {
    sign: f64,
    vto: f64,
    beta: f64,
    lambda: f64,
    cgs: f64,
    cgd: f64,
    vgs_prev: f64,
    vgd_prev: f64,
    pins: [NodeId; 3],
}

impl JfetModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 3)?;
        let sign = jfet_sign(instance.kind());
        let params = instance.parameters();
        let vto = parameter(params, "vto", -2.0 * sign);
        let beta = positive_parameter(params, "beta", Some(1e-3))?;
        let lambda = parameter(params, "lambda", 0.0);
        let (cgs, cgd) = jfet_capacitances(params)?;
        Ok(Self {
            sign,
            vto,
            beta,
            lambda,
            cgs,
            cgd,
            vgs_prev: 0.0,
            vgd_prev: 0.0,
            pins: [pins[0], pins[1], pins[2]],
        })
    }

    fn current(&self, drain: f64, gate: f64, source: f64) -> f64 {
        let vgs = self.sign * (gate - source);
        let vds = self.sign * (drain - source);
        self.sign * self.channel_current(vgs, vds)
    }

    fn channel_current(&self, vgs: f64, vds: f64) -> f64 {
        let overdrive = vgs - self.sign * self.vto;
        if overdrive <= 0.0 {
            return 0.0;
        }
        let lambda = 1.0 + self.lambda * vds;
        if vds < overdrive {
            self.beta * (2.0 * overdrive * vds - vds * vds) * lambda
        } else {
            self.beta * overdrive * overdrive * lambda
        }
    }

    fn linearization(&self, drain: f64, gate: f64, source: f64) -> (f64, f64, f64, f64) {
        let current = self.current(drain, gate, source);
        let gm = finite_difference(
            |value| self.current(drain, value, source),
            gate,
            DERIVATIVE_STEP,
        );
        let gds = finite_difference(
            |value| self.current(value, gate, source),
            drain,
            DERIVATIVE_STEP,
        );
        let gms = finite_difference(
            |value| self.current(drain, gate, value),
            source,
            DERIVATIVE_STEP,
        );
        (current, gm, gds, gms)
    }

    fn stamp_pins(&self) -> [NodeId; 3] {
        [self.pins[1], self.pins[0], self.pins[2]]
    }

    fn voltages(&self, solution: &Solution) -> (f64, f64, f64) {
        (
            solution.voltage(self.pins[1]).unwrap_or(0.0),
            solution.voltage(self.pins[0]).unwrap_or(0.0),
            solution.voltage(self.pins[2]).unwrap_or(0.0),
        )
    }

    fn stamp_capacitances(&self, ctx: &mut MnaContext, dt: Option<f64>) -> Result<(), SimError> {
        let dt = match dt {
            Some(value) => value,
            None => return Ok(()),
        };
        stamp_capacitor(ctx, self.pins[1], self.pins[2], self.cgs, self.vgs_prev, dt)?;
        stamp_capacitor(ctx, self.pins[1], self.pins[0], self.cgd, self.vgd_prev, dt)
    }
}

impl DeviceModel for JfetModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError> {
        let (gate, drain, source) = self.voltages(solution);
        let (current, gm, gds, gms) = self.linearization(drain, gate, source);
        stamp_three_terminal(
            ctx,
            self.stamp_pins(),
            current,
            gm,
            gds,
            gms,
            (gate, drain, source),
        )?;
        self.stamp_capacitances(ctx, dt)
    }

    fn update(&mut self, solution: &Solution) -> Result<(), SimError> {
        let (gate, drain, source) = self.voltages(solution);
        self.vgs_prev = gate - source;
        self.vgd_prev = gate - drain;
        Ok(())
    }
}

fn jfet_sign(kind: &DeviceKind) -> f64 {
    if matches!(kind, DeviceKind::JfetP) {
        -1.0
    } else {
        1.0
    }
}

/// Returns constant small-signal JFET capacitances.
pub fn jfet_capacitances(params: &BTreeMap<String, f64>) -> Result<(f64, f64), SimError> {
    Ok((
        non_negative_parameter(params, "cgs", 0.0)?,
        non_negative_parameter(params, "cgd", 0.0)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::{JfetModel, jfet_capacitances};
    use sim_core::NodeId;
    use std::collections::BTreeMap;

    fn model(sign: f64, vto: f64) -> JfetModel {
        JfetModel {
            sign,
            vto,
            beta: 1e-3,
            lambda: 0.0,
            cgs: 0.0,
            cgd: 0.0,
            vgs_prev: 0.0,
            vgd_prev: 0.0,
            pins: [NodeId::new(1), NodeId::new(2), NodeId::GROUND],
        }
    }

    #[test]
    fn n_jfet_saturation_current_matches_formula() {
        let current = model(1.0, -2.0).current(5.0, 0.0, 0.0);
        assert!((current - 4e-3).abs() < 1e-12, "current={current}");
    }

    #[test]
    fn n_jfet_is_off_below_threshold() {
        let current = model(1.0, -2.0).current(5.0, -3.0, 0.0);
        assert_eq!(current, 0.0);
    }

    #[test]
    fn p_jfet_current_is_negative() {
        let current = model(-1.0, 2.0).current(0.0, 5.0, 5.0);
        assert!((current + 4e-3).abs() < 1e-12, "current={current}");
    }

    #[test]
    fn gate_bias_reduces_n_jfet_current() {
        let base = model(1.0, -2.0).current(5.0, 0.0, 0.0);
        let biased = model(1.0, -2.0).current(5.0, -1.0, 0.0);
        assert!(biased < base, "base={base} biased={biased}");
    }

    #[test]
    fn public_capacitances_follow_parameters() {
        let mut params = BTreeMap::new();
        params.insert("cgs".to_string(), 1e-12);
        params.insert("cgd".to_string(), 2e-12);
        let (cgs, cgd) = jfet_capacitances(&params).expect("capacitance");
        assert!((cgs - 1e-12).abs() < 1e-20);
        assert!((cgd - 2e-12).abs() < 1e-20);
    }

    #[test]
    fn capacitances_default_to_zero() {
        let (cgs, cgd) = jfet_capacitances(&BTreeMap::new()).expect("capacitance");
        assert_eq!(cgs, 0.0);
        assert_eq!(cgd, 0.0);
    }
}
