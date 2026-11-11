//! Shockley diode model.

use crate::model::{
    DeviceModel, MAX_EXPONENT, MAX_FORWARD_VOLTAGE, pins_exact, positive_parameter,
    stamp_conductance, thermal_voltage,
};
use sim_core::{Device, DeviceInstance, MnaContext, NodeId, SimError, Solution};

/// A two-terminal Shockley diode.
pub struct DiodeModel {
    saturation_current: f64,
    emission: f64,
    thermal_voltage: f64,
    pins: [NodeId; 2],
}

impl DiodeModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 2)?;
        let saturation_current = positive_parameter(instance.parameters(), "is", Some(1e-12))?;
        let emission = positive_parameter(instance.parameters(), "n", Some(1.0))?;
        let thermal_voltage = thermal_voltage(instance.parameters())?;
        Ok(Self {
            saturation_current,
            emission,
            thermal_voltage,
            pins: [pins[0], pins[1]],
        })
    }

    fn current(&self, voltage: f64) -> f64 {
        let limited = voltage.min(MAX_FORWARD_VOLTAGE);
        let exponent = (limited / (self.emission * self.thermal_voltage)).min(MAX_EXPONENT);
        self.saturation_current * (exponent.exp() - 1.0)
    }

    fn conductance(&self, voltage: f64) -> f64 {
        let limited = voltage.min(MAX_FORWARD_VOLTAGE);
        let exponent = (limited / (self.emission * self.thermal_voltage)).min(MAX_EXPONENT);
        self.saturation_current / (self.emission * self.thermal_voltage) * exponent.exp()
    }
}

impl DeviceModel for DiodeModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        let first = solution.voltage(self.pins[0]).unwrap_or(0.0);
        let second = solution.voltage(self.pins[1]).unwrap_or(0.0);
        let voltage = first - second;
        let current = self.current(voltage);
        let conductance = self.conductance(voltage);
        stamp_conductance(ctx, self.pins[0], self.pins[1], conductance)?;
        let constant = conductance * voltage.min(MAX_FORWARD_VOLTAGE) - current;
        let a = ctx.node_index(self.pins[0]);
        let b = ctx.node_index(self.pins[1]);
        ctx.add_rhs_opt(a, constant)?;
        ctx.add_rhs_opt(b, -constant)
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{DiodeModel, MAX_FORWARD_VOLTAGE};
    use sim_core::NodeId;

    fn model() -> DiodeModel {
        DiodeModel {
            saturation_current: 1e-12,
            emission: 1.0,
            thermal_voltage: 0.02585,
            pins: [NodeId::new(1), NodeId::GROUND],
        }
    }

    #[test]
    fn zero_voltage_has_zero_current() {
        assert_eq!(model().current(0.0), 0.0);
    }

    #[test]
    fn forward_bias_increases_current() {
        let model = model();
        assert!(model.current(0.6) > model.current(0.5));
    }

    #[test]
    fn reverse_bias_is_small() {
        let model = model();
        assert!(model.current(-1.0).abs() < 1e-9);
    }

    #[test]
    fn conductance_is_positive() {
        let model = model();
        assert!(model.conductance(0.0) > 0.0);
        assert!(model.conductance(0.5) > model.conductance(0.4));
    }

    #[test]
    fn forward_voltage_is_limited() {
        let model = model();
        let at_limit = model.current(MAX_FORWARD_VOLTAGE);
        let above_limit = model.current(MAX_FORWARD_VOLTAGE + 1.0);
        assert!((at_limit - above_limit).abs() < 1e-9);
    }
}
