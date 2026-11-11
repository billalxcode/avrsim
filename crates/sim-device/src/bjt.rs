//! Ebers-Moll bipolar junction transistor model.

use crate::model::{
    DeviceModel, MAX_EXPONENT, MAX_FORWARD_VOLTAGE, pins_exact, positive_parameter, thermal_voltage,
};
use sim_core::{Device, DeviceInstance, DeviceKind, MnaContext, NodeId, SimError, Solution};

/// A three-terminal BJT with pins ordered collector, base, emitter.
pub struct BjtModel {
    sign: f64,
    saturation_current: f64,
    alpha_forward: f64,
    alpha_reverse: f64,
    thermal_voltage: f64,
    pins: [NodeId; 3],
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

impl BjtModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 3)?;
        let params = instance.parameters();
        let saturation_current = positive_parameter(params, "is", Some(1e-15))?;
        let forward_beta = positive_parameter(params, "bf", Some(100.0))?;
        let reverse_beta = positive_parameter(params, "br", Some(1.0))?;
        let thermal_voltage = thermal_voltage(params)?;
        let sign = match instance.kind() {
            DeviceKind::Pnp => -1.0,
            _ => 1.0,
        };
        Ok(Self {
            sign,
            saturation_current,
            alpha_forward: beta_alpha(forward_beta),
            alpha_reverse: beta_alpha(reverse_beta),
            thermal_voltage,
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

    fn linearize(&self, voltages: [f64; 3]) -> BjtState {
        let vbe = self.sign * (voltages[0] - voltages[2]);
        let vbc = self.sign * (voltages[0] - voltages[1]);
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
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        let voltages = self.terminal_voltages(solution);
        let state = self.linearize(voltages);
        stamp_state(ctx, self.pins, &state, voltages)
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }
}

fn beta_alpha(beta: f64) -> f64 {
    beta / (beta + 1.0)
}

fn junction(saturation: f64, thermal_voltage: f64, voltage: f64) -> Junction {
    let limited = voltage.min(MAX_FORWARD_VOLTAGE);
    let exponent = (limited / thermal_voltage).min(MAX_EXPONENT);
    let exponential = exponent.exp();
    Junction {
        current: saturation * (exponential - 1.0),
        conductance: saturation / thermal_voltage * exponential,
    }
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
    use super::{BjtModel, beta_alpha};
    use sim_core::{DeviceKind, NodeId};

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
        let state = model(DeviceKind::Npn, 0.02585).linearize([0.65, 5.0, 0.0]);
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
}
