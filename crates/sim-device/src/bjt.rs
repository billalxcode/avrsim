//! Ebers-Moll bipolar junction transistor model.

use crate::model::{
    DeviceModel, MAX_EXPONENT, MAX_FORWARD_VOLTAGE, depletion_capacitance,
    junction_critical_voltage, non_negative_parameter, pins_exact, pnjlim, positive_parameter,
    stamp_capacitor, thermal_voltage,
};
use sim_core::{Device, DeviceInstance, DeviceKind, MnaContext, NodeId, SimError, Solution};
use std::collections::BTreeMap;

/// A three-terminal BJT with pins ordered collector, base, emitter.
pub struct BjtModel {
    sign: f64,
    saturation_current: f64,
    alpha_forward: f64,
    alpha_reverse: f64,
    thermal_voltage: f64,
    critical_voltage: f64,
    vbe_prev: f64,
    vbc_prev: f64,
    cap: BjtCapacitance,
    cap_vbe_prev: f64,
    cap_vbc_prev: f64,
    pins: [NodeId; 3],
}

#[derive(Clone, Copy)]
struct BjtCapacitance {
    cje: f64,
    vje: f64,
    mje: f64,
    cjc: f64,
    vjc: f64,
    mjc: f64,
    tf: f64,
    tr: f64,
}

struct BjtParameters {
    sign: f64,
    saturation_current: f64,
    alpha_forward: f64,
    alpha_reverse: f64,
    thermal_voltage: f64,
    critical_voltage: f64,
}

fn bjt_parameters(
    params: &BTreeMap<String, f64>,
    kind: &DeviceKind,
) -> Result<BjtParameters, SimError> {
    let saturation_current = positive_parameter(params, "is", Some(1e-15))?;
    let forward_beta = positive_parameter(params, "bf", Some(100.0))?;
    let reverse_beta = positive_parameter(params, "br", Some(1.0))?;
    let thermal = thermal_voltage(params)?;
    Ok(BjtParameters {
        sign: bjt_sign(kind),
        saturation_current,
        alpha_forward: beta_alpha(forward_beta),
        alpha_reverse: beta_alpha(reverse_beta),
        thermal_voltage: thermal,
        critical_voltage: junction_critical_voltage(saturation_current, thermal),
    })
}

struct Junction {
    current: f64,
    conductance: f64,
}

struct BjtDerivatives {
    ic: [f64; 3],
    ib: [f64; 3],
    ie: [f64; 3],
}

struct BjtState {
    ic: f64,
    ib: f64,
    ie: f64,
    dic: [f64; 3],
    dib: [f64; 3],
    die: [f64; 3],
}

impl BjtCapacitance {
    fn from_params(params: &BTreeMap<String, f64>) -> Result<Self, SimError> {
        Ok(Self {
            cje: non_negative_parameter(params, "cje", 0.0)?,
            vje: positive_parameter(params, "vje", Some(0.75))?,
            mje: non_negative_parameter(params, "mje", 0.33)?,
            cjc: non_negative_parameter(params, "cjc", 0.0)?,
            vjc: positive_parameter(params, "vjc", Some(0.75))?,
            mjc: non_negative_parameter(params, "mjc", 0.33)?,
            tf: non_negative_parameter(params, "tf", 0.0)?,
            tr: non_negative_parameter(params, "tr", 0.0)?,
        })
    }

    fn values(&self, vbe: f64, vbc: f64, saturation: f64, thermal: f64) -> (f64, f64) {
        let forward = junction_conductance(saturation, thermal, vbe);
        let reverse = junction_conductance(saturation, thermal, vbc);
        let cbe = depletion_capacitance(self.cje, self.vje, self.mje, vbe) + self.tf * forward;
        let cbc = depletion_capacitance(self.cjc, self.vjc, self.mjc, vbc) + self.tr * reverse;
        (cbe, cbc)
    }
}

/// Returns small-signal BJT capacitances at the given terminal bias.
pub fn bjt_capacitances(
    kind: &DeviceKind,
    params: &BTreeMap<String, f64>,
    vb: f64,
    vc: f64,
    ve: f64,
) -> Result<(f64, f64), SimError> {
    let cap = BjtCapacitance::from_params(params)?;
    let saturation = positive_parameter(params, "is", Some(1e-15))?;
    let thermal = thermal_voltage(params)?;
    let sign = bjt_sign(kind);
    Ok(cap.values(sign * (vb - ve), sign * (vb - vc), saturation, thermal))
}

fn bjt_sign(kind: &DeviceKind) -> f64 {
    if matches!(kind, DeviceKind::Pnp) {
        -1.0
    } else {
        1.0
    }
}

impl BjtModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 3)?;
        let params = instance.parameters();
        let base = bjt_parameters(params, instance.kind())?;
        let cap = BjtCapacitance::from_params(params)?;
        Ok(Self {
            sign: base.sign,
            saturation_current: base.saturation_current,
            alpha_forward: base.alpha_forward,
            alpha_reverse: base.alpha_reverse,
            thermal_voltage: base.thermal_voltage,
            critical_voltage: base.critical_voltage,
            vbe_prev: 0.0,
            vbc_prev: 0.0,
            cap,
            cap_vbe_prev: 0.0,
            cap_vbc_prev: 0.0,
            pins: [pins[0], pins[1], pins[2]],
        })
    }

    fn terminal_voltages(&self, solution: &Solution) -> [f64; 3] {
        [
            solution.voltage(self.pins[1]).unwrap_or(0.0),
            solution.voltage(self.pins[0]).unwrap_or(0.0),
            solution.voltage(self.pins[2]).unwrap_or(0.0),
        ]
    }

    fn junction_voltages(&self, voltages: [f64; 3]) -> (f64, f64) {
        let vbe = self.sign * (voltages[0] - voltages[2]);
        let vbc = self.sign * (voltages[0] - voltages[1]);
        (vbe, vbc)
    }

    fn stamp_capacitances(
        &self,
        ctx: &mut MnaContext,
        dt: Option<f64>,
        voltages: [f64; 3],
    ) -> Result<(), SimError> {
        let dt = match dt {
            Some(value) => value,
            None => return Ok(()),
        };
        let (vbe, vbc) = self.junction_voltages(voltages);
        let (cbe, cbc) = self
            .cap
            .values(vbe, vbc, self.saturation_current, self.thermal_voltage);
        stamp_capacitor(ctx, self.pins[1], self.pins[2], cbe, self.cap_vbe_prev, dt)?;
        stamp_capacitor(ctx, self.pins[1], self.pins[0], cbc, self.cap_vbc_prev, dt)
    }

    fn linearize(&mut self, voltages: [f64; 3]) -> BjtState {
        let (vbe, vbc) = self.limited_voltages(voltages);
        let forward = junction(self.saturation_current, self.thermal_voltage, vbe);
        let reverse = junction(self.saturation_current, self.thermal_voltage, vbc);
        let derivatives = self.derivatives(&forward, &reverse);
        BjtState {
            ic: self.sign * (self.alpha_forward * forward.current - reverse.current),
            ib: self.sign
                * (self.base_forward() * forward.current + self.base_reverse() * reverse.current),
            ie: self.sign * (-forward.current + self.alpha_reverse * reverse.current),
            dic: derivatives.ic,
            dib: derivatives.ib,
            die: derivatives.ie,
        }
    }

    fn limited_voltages(&mut self, voltages: [f64; 3]) -> (f64, f64) {
        let vbe = self.sign * (voltages[0] - voltages[2]);
        let vbc = self.sign * (voltages[0] - voltages[1]);
        let limited_vbe = pnjlim(
            vbe,
            self.vbe_prev,
            self.thermal_voltage,
            self.critical_voltage,
        );
        let limited_vbc = pnjlim(
            vbc,
            self.vbc_prev,
            self.thermal_voltage,
            self.critical_voltage,
        );
        self.vbe_prev = limited_vbe;
        self.vbc_prev = limited_vbc;
        (limited_vbe, limited_vbc)
    }

    fn derivatives(&self, forward: &Junction, reverse: &Junction) -> BjtDerivatives {
        let dic_dvbe = self.alpha_forward * forward.conductance;
        let dic_dvbc = -reverse.conductance;
        let dib_dvbe = self.base_forward() * forward.conductance;
        let dib_dvbc = self.base_reverse() * reverse.conductance;
        let die_dvbe = -forward.conductance;
        let die_dvbc = self.alpha_reverse * reverse.conductance;
        BjtDerivatives {
            ic: pair_derivatives(dic_dvbe, dic_dvbc),
            ib: pair_derivatives(dib_dvbe, dib_dvbc),
            ie: pair_derivatives(die_dvbe, die_dvbc),
        }
    }

    fn base_forward(&self) -> f64 {
        1.0 - self.alpha_forward
    }

    fn base_reverse(&self) -> f64 {
        1.0 - self.alpha_reverse
    }
}

impl DeviceModel for BjtModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError> {
        let voltages = self.terminal_voltages(solution);
        let state = self.linearize(voltages);
        stamp_state(ctx, self.pins, &state, voltages)?;
        self.stamp_capacitances(ctx, dt, voltages)
    }

    fn update(&mut self, solution: &Solution) -> Result<(), SimError> {
        let first = solution.voltage(self.pins[0]).unwrap_or(0.0);
        let base = solution.voltage(self.pins[1]).unwrap_or(0.0);
        let emitter = solution.voltage(self.pins[2]).unwrap_or(0.0);
        self.vbe_prev = self.sign * (base - emitter);
        self.vbc_prev = self.sign * (base - first);
        self.cap_vbe_prev = base - emitter;
        self.cap_vbc_prev = base - first;
        Ok(())
    }
}

fn beta_alpha(beta: f64) -> f64 {
    beta / (beta + 1.0)
}

fn junction(saturation: f64, thermal_voltage: f64, voltage: f64) -> Junction {
    let exponential = junction_exponential(voltage, thermal_voltage);
    Junction {
        current: saturation * (exponential - 1.0),
        conductance: junction_conductance(saturation, thermal_voltage, voltage),
    }
}

fn junction_conductance(saturation: f64, thermal_voltage: f64, voltage: f64) -> f64 {
    saturation / thermal_voltage * junction_exponential(voltage, thermal_voltage)
}

fn junction_exponential(voltage: f64, thermal_voltage: f64) -> f64 {
    let limited = voltage.min(MAX_FORWARD_VOLTAGE);
    (limited / thermal_voltage).min(MAX_EXPONENT).exp()
}

fn pair_derivatives(dvbe: f64, dvbc: f64) -> [f64; 3] {
    [dvbe + dvbc, -dvbc, -dvbe]
}

fn stamp_state(
    ctx: &mut MnaContext,
    pins: [NodeId; 3],
    state: &BjtState,
    voltages: [f64; 3],
) -> Result<(), SimError> {
    stamp_terminal(ctx, pins[0], pins, state.ic, state.dic, voltages)?;
    stamp_terminal(ctx, pins[1], pins, state.ib, state.dib, voltages)?;
    stamp_terminal(ctx, pins[2], pins, state.ie, state.die, voltages)
}

fn stamp_terminal(
    ctx: &mut MnaContext,
    row: NodeId,
    pins: [NodeId; 3],
    current: f64,
    derivatives: [f64; 3],
    voltages: [f64; 3],
) -> Result<(), SimError> {
    let base = ctx.node_index(pins[1]);
    let collector = ctx.node_index(pins[0]);
    let emitter = ctx.node_index(pins[2]);
    let constant = dot(derivatives, voltages) - current;
    let row = ctx.node_index(row);
    ctx.add_matrix_opt(row, base, derivatives[0])?;
    ctx.add_matrix_opt(row, collector, derivatives[1])?;
    ctx.add_matrix_opt(row, emitter, derivatives[2])?;
    ctx.add_rhs_opt(row, constant)
}

fn dot(values: [f64; 3], voltages: [f64; 3]) -> f64 {
    values[0] * voltages[0] + values[1] * voltages[1] + values[2] * voltages[2]
}

#[cfg(test)]
mod tests {
    use super::{BjtCapacitance, BjtModel, beta_alpha, bjt_capacitances};
    use crate::model::DeviceModel;
    use sim_core::{DeviceKind, NodeId, Solution};
    use std::collections::BTreeMap;

    fn default_cap() -> BjtCapacitance {
        BjtCapacitance::from_params(&BTreeMap::new()).expect("cap")
    }

    fn model(kind: DeviceKind, thermal_voltage: f64) -> BjtModel {
        BjtModel {
            sign: if matches!(kind, DeviceKind::Pnp) {
                -1.0
            } else {
                1.0
            },
            saturation_current: 1e-15,
            alpha_forward: beta_alpha(100.0),
            alpha_reverse: beta_alpha(1.0),
            thermal_voltage,
            critical_voltage: 0.611,
            vbe_prev: 0.0,
            vbc_prev: 0.0,
            cap: default_cap(),
            cap_vbe_prev: 0.0,
            cap_vbc_prev: 0.0,
            pins: [NodeId::new(1), NodeId::new(2), NodeId::GROUND],
        }
    }

    #[test]
    fn terminal_currents_balance() {
        let state = model(DeviceKind::Npn, 0.02585).linearize([0.7, 0.2, 0.0]);
        let sum = state.ic + state.ib + state.ie;
        assert!(sum.abs() < 1e-12, "sum={sum}");
    }

    #[test]
    fn forward_active_gain_matches_beta() {
        let mut model = model(DeviceKind::Npn, 0.02585);
        model.vbe_prev = 0.65;
        let state = model.linearize([0.65, 5.0, 0.0]);
        let ratio = state.ic / state.ib;
        assert!((ratio - 100.0).abs() < 1e-6, "ratio={ratio}");
    }

    #[test]
    fn pnp_collector_current_is_negative() {
        let state = model(DeviceKind::Pnp, 0.02585).linearize([4.3, 0.0, 5.0]);
        assert!(state.ic < 0.0, "ic={}", state.ic);
        assert!(state.ie > 0.0, "ie={}", state.ie);
    }

    #[test]
    fn higher_temperature_lowers_collector_current() {
        let cold = model(DeviceKind::Npn, 0.02585).linearize([0.65, 5.0, 0.0]);
        let hot = model(DeviceKind::Npn, 0.03447).linearize([0.65, 5.0, 0.0]);
        assert!(hot.ic < cold.ic, "cold={} hot={}", cold.ic, hot.ic);
    }

    #[test]
    fn linearization_limits_large_junction_step() {
        let mut model = model(DeviceKind::Npn, 0.02585);
        model.linearize([5.0, 5.0, 0.0]);
        assert!(model.vbe_prev > 0.0 && model.vbe_prev < 0.2);
    }

    #[test]
    fn public_capacitances_match_bias_and_kind() {
        let params = cap_params();
        let npn = bjt_capacitances(&DeviceKind::Npn, &params, 0.5, 2.0, 0.0).expect("npn");
        let pnp = bjt_capacitances(&DeviceKind::Pnp, &params, -0.5, -2.0, 0.0).expect("pnp");
        assert!((npn.0 - 1.7320508e-12).abs() < 1e-19, "cbe={}", npn.0);
        assert!((npn.1 - 1.1547005e-12).abs() < 1e-19, "cbc={}", npn.1);
        assert_eq!(npn, pnp);
    }

    #[test]
    fn diffusion_capacitance_tracks_forward_current() {
        let mut params = BTreeMap::new();
        params.insert("tf".to_string(), 1e-6);
        let with = bjt_capacitances(&DeviceKind::Npn, &params, 0.6, 5.0, 0.0).expect("with");
        let without =
            bjt_capacitances(&DeviceKind::Npn, &BTreeMap::new(), 0.6, 5.0, 0.0).expect("without");
        assert!(
            with.0 > without.0 + 1e-10,
            "with={} without={}",
            with.0,
            without.0
        );
    }

    #[test]
    fn capacitance_state_uses_actual_terminal_voltage() {
        let mut model = model(DeviceKind::Pnp, 0.02585);
        let solution = Solution {
            node_voltages: vec![0.0, -5.0, -0.7],
            branch_currents: Vec::new(),
        };
        model.update(&solution).expect("update");
        assert!((model.cap_vbe_prev + 0.7).abs() < 1e-12);
        assert!((model.cap_vbc_prev - 4.3).abs() < 1e-12);
    }

    fn cap_params() -> BTreeMap<String, f64> {
        let mut params = BTreeMap::new();
        params.insert("cje".to_string(), 1e-12);
        params.insert("cjc".to_string(), 2e-12);
        params.insert("vje".to_string(), 0.75);
        params.insert("vjc".to_string(), 0.75);
        params.insert("mje".to_string(), 0.5);
        params.insert("mjc".to_string(), 0.5);
        params
    }
}
