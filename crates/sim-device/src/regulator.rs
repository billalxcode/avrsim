//! Three-terminal ideal linear regulator macromodel.

use crate::model::{DeviceModel, parameter, pins_exact, stamp_voltage_branch};
use sim_core::{Device, DeviceId, DeviceInstance, MnaContext, NodeId, SimError, Solution};

/// A three-pin regulator with pins `in, out, ground`.
pub struct RegulatorModel {
    id: DeviceId,
    output_voltage: f64,
    output: NodeId,
    ground: NodeId,
}

impl RegulatorModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 3)?;
        let output_voltage = parameter(instance.parameters(), "output_voltage", 5.0);
        if !output_voltage.is_finite() || output_voltage < 0.0 {
            return Err(SimError::InvalidNetlist(
                "regulator output voltage must be non-negative and finite".to_string(),
            ));
        }
        Ok(Self {
            id: instance.id(),
            output_voltage,
            output: pins[1],
            ground: pins[2],
        })
    }
}

impl DeviceModel for RegulatorModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        _solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        stamp_voltage_branch(ctx, self.id, self.output, self.ground, self.output_voltage)
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::RegulatorModel;
    use sim_core::{DeviceId, DeviceInstance, DeviceKind, NodeId};

    fn instance(pins: usize) -> DeviceInstance {
        DeviceInstance::new(
            DeviceId::new(0),
            "reg",
            DeviceKind::Regulator,
            vec![NodeId::GROUND; pins],
            Default::default(),
        )
    }

    #[test]
    fn regulator_requires_three_pins() {
        assert!(RegulatorModel::from_instance(&instance(2)).is_err());
    }

    #[test]
    fn regulator_defaults_to_five_volts() {
        let model = RegulatorModel::from_instance(&instance(3)).expect("model");
        assert!((model.output_voltage - 5.0).abs() < 1e-9);
    }

    #[test]
    fn regulator_rejects_negative_output() {
        let mut instance = instance(3);
        instance.set_parameter("output_voltage", -1.0);
        assert!(RegulatorModel::from_instance(&instance).is_err());
    }
}
