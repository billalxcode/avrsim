//! Voltage-controlled switch.

use crate::model::{DeviceModel, parameter, pins_exact, positive_parameter, stamp_conductance};
use sim_core::{Device, DeviceInstance, MnaContext, NodeId, SimError, Solution};

/// A voltage-controlled switch with pins `out+, out-, ctrl+, ctrl-`.
pub struct SwitchModel {
    on_resistance: f64,
    off_resistance: f64,
    threshold: f64,
    width: f64,
    pins: [NodeId; 4],
}

impl SwitchModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 4)?;
        let on_resistance = positive_parameter(instance.parameters(), "ron", Some(1.0))?;
        let off_resistance = positive_parameter(instance.parameters(), "roff", Some(1e9))?;
        let threshold = parameter(instance.parameters(), "vth", 2.5);
        let width = positive_parameter(instance.parameters(), "width", Some(0.1))?;
        if !threshold.is_finite() {
            return Err(SimError::InvalidNetlist(
                "switch threshold must be finite".to_string(),
            ));
        }
        Ok(Self {
            on_resistance,
            off_resistance,
            threshold,
            width,
            pins: [pins[0], pins[1], pins[2], pins[3]],
        })
    }

    fn conductance(&self, control: f64) -> f64 {
        let argument = ((control - self.threshold) / self.width).clamp(-40.0, 40.0);
        let fraction = 1.0 / (1.0 + (-argument).exp());
        1.0 / self.off_resistance
            + (1.0 / self.on_resistance - 1.0 / self.off_resistance) * fraction
    }
}

impl DeviceModel for SwitchModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        let control = solution.voltage(self.pins[2]).unwrap_or(0.0)
            - solution.voltage(self.pins[3]).unwrap_or(0.0);
        let conductance = self.conductance(control);
        stamp_conductance(ctx, self.pins[0], self.pins[1], conductance)
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }
}
