//! Linear operational amplifier macromodel.

use crate::model::{DeviceModel, branch_index, parameter, pins_exact};
use sim_core::{Device, DeviceId, DeviceInstance, MnaContext, NodeId, SimError, Solution};

/// A five-pin op-amp with pins `in+, in-, V+, V-, out`.
pub struct OpAmpModel {
    id: DeviceId,
    gain: f64,
    pins: [NodeId; 5],
}

impl OpAmpModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 5)?;
        let gain = parameter(instance.parameters(), "gain", 1.0e5);
        if !gain.is_finite() || gain <= 0.0 {
            return Err(SimError::InvalidNetlist(
                "op-amp gain must be positive and finite".to_string(),
            ));
        }
        Ok(Self {
            id: instance.id(),
            gain,
            pins: [pins[0], pins[1], pins[2], pins[3], pins[4]],
        })
    }
}

impl DeviceModel for OpAmpModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        _solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        let branch = branch_index(ctx, self.id)?;
        let output = ctx.node_index(self.pins[4]);
        let reference = ctx.node_index(self.pins[3]);
        let non_inverting = ctx.node_index(self.pins[0]);
        let inverting = ctx.node_index(self.pins[1]);
        ctx.add_matrix_opt(output, Some(branch), 1.0)?;
        ctx.add_matrix_opt(reference, Some(branch), -1.0)?;
        ctx.add_matrix_opt(Some(branch), output, 1.0)?;
        ctx.add_matrix_opt(Some(branch), reference, -1.0)?;
        ctx.add_matrix_opt(Some(branch), non_inverting, -self.gain)?;
        ctx.add_matrix_opt(Some(branch), inverting, self.gain)
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::OpAmpModel;
    use sim_core::{DeviceId, DeviceInstance, DeviceKind, NodeId};

    fn instance(pins: usize) -> DeviceInstance {
        DeviceInstance::new(
            DeviceId::new(0),
            "op",
            DeviceKind::OpAmp,
            vec![NodeId::GROUND; pins],
            Default::default(),
        )
    }

    #[test]
    fn opamp_requires_five_pins() {
        assert!(OpAmpModel::from_instance(&instance(4)).is_err());
    }

    #[test]
    fn opamp_defaults_to_high_gain() {
        let model = OpAmpModel::from_instance(&instance(5)).expect("model");
        assert!((model.gain - 1.0e5).abs() < 1.0);
    }

    #[test]
    fn opamp_rejects_invalid_gain() {
        let mut instance = instance(5);
        instance.set_parameter("gain", 0.0);
        assert!(OpAmpModel::from_instance(&instance).is_err());
    }
}
