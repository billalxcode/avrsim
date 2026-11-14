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

/// A two-terminal switch with a fixed closed or open state.
pub struct IdealSwitchModel {
    conductance: f64,
    pins: [NodeId; 2],
}

impl IdealSwitchModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 2)?;
        let on_resistance = positive_parameter(instance.parameters(), "ron", Some(0.01))?;
        let off_resistance = positive_parameter(instance.parameters(), "roff", Some(1e12))?;
        let closed = parameter(instance.parameters(), "closed", 0.0) != 0.0;
        let conductance = if closed {
            1.0 / on_resistance
        } else {
            1.0 / off_resistance
        };
        Ok(Self {
            conductance,
            pins: [pins[0], pins[1]],
        })
    }
}

impl DeviceModel for IdealSwitchModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        _solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        stamp_conductance(ctx, self.pins[0], self.pins[1], self.conductance)
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{IdealSwitchModel, SwitchModel};
    use crate::model::DeviceModel;
    use sim_core::{DeviceId, DeviceInstance, DeviceKind, NodeId, Solution};
    use std::collections::BTreeMap;

    fn instance(kind: DeviceKind, pins: usize, params: &[(&str, f64)]) -> DeviceInstance {
        let parameters = params
            .iter()
            .map(|(name, value)| ((*name).to_string(), *value))
            .collect::<BTreeMap<_, _>>();
        DeviceInstance::new(
            DeviceId::new(0),
            "sw",
            kind,
            vec![NodeId::GROUND; pins],
            parameters,
        )
    }

    #[test]
    fn ideal_switch_closed_conducts() {
        let model = IdealSwitchModel::from_instance(&instance(
            DeviceKind::IdealSwitch,
            2,
            &[("ron", 0.01), ("closed", 1.0)],
        ))
        .expect("model");
        assert!((model.conductance - 100.0).abs() < 1e-9);
    }

    #[test]
    fn ideal_switch_open_blocks() {
        let model = IdealSwitchModel::from_instance(&instance(
            DeviceKind::IdealSwitch,
            2,
            &[("roff", 1e12), ("closed", 0.0)],
        ))
        .expect("model");
        assert!(model.conductance < 1e-9);
    }

    #[test]
    fn controlled_switch_requires_four_pins() {
        let result = SwitchModel::from_instance(&instance(DeviceKind::Switch, 2, &[]));
        assert!(result.is_err());
    }

    #[test]
    fn ideal_switch_rejects_invalid_resistance() {
        let result =
            IdealSwitchModel::from_instance(&instance(DeviceKind::IdealSwitch, 2, &[("ron", 0.0)]));
        assert!(result.is_err());
    }

    #[test]
    fn switch_models_stamp() {
        let mut model = IdealSwitchModel::from_instance(&instance(
            DeviceKind::IdealSwitch,
            2,
            &[("closed", 1.0)],
        ))
        .expect("model");
        let mut ctx =
            sim_core::MnaContext::new(2, 0, vec![Some(0), Some(1)], Vec::new()).expect("ctx");
        let solution = Solution::zero(2, 0);
        model.stamp(&mut ctx, &solution, None).expect("stamp");
    }
}
