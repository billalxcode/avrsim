//! Saturating comparator macromodel.

use crate::model::{DeviceModel, branch_index, parameter, pins_exact};
use sim_core::{Device, DeviceId, DeviceInstance, MnaContext, NodeId, SimError, Solution};

/// A five-pin comparator with pins `in+, in-, V+, V-, out`.
pub struct ComparatorModel {
    id: DeviceId,
    gain: f64,
    pins: [NodeId; 5],
}

impl ComparatorModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 5)?;
        let gain = parameter(instance.parameters(), "gain", 1.0e4);
        if !gain.is_finite() || gain <= 0.0 {
            return Err(SimError::InvalidNetlist(
                "comparator gain must be positive and finite".to_string(),
            ));
        }
        Ok(Self {
            id: instance.id(),
            gain,
            pins: [pins[0], pins[1], pins[2], pins[3], pins[4]],
        })
    }

    fn output_level(&self, solution: &Solution) -> f64 {
        let high = solution.voltage(self.pins[2]).unwrap_or(0.0);
        let low = solution.voltage(self.pins[3]).unwrap_or(0.0);
        let input = solution.voltage(self.pins[0]).unwrap_or(0.0);
        let reference = solution.voltage(self.pins[1]).unwrap_or(0.0);
        let span = (high - low).max(0.0);
        let argument = (-self.gain * (input - reference)).clamp(-40.0, 40.0);
        span / (1.0 + argument.exp())
    }
}

impl DeviceModel for ComparatorModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        let branch = branch_index(ctx, self.id)?;
        let output = ctx.node_index(self.pins[4]);
        let reference = ctx.node_index(self.pins[3]);
        let level = self.output_level(solution);
        ctx.add_matrix_opt(output, Some(branch), 1.0)?;
        ctx.add_matrix_opt(reference, Some(branch), -1.0)?;
        ctx.add_matrix_opt(Some(branch), output, 1.0)?;
        ctx.add_matrix_opt(Some(branch), reference, -1.0)?;
        ctx.add_rhs_opt(Some(branch), level)
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ComparatorModel;
    use sim_core::{DeviceId, DeviceInstance, DeviceKind, NodeId, Solution};

    fn model() -> ComparatorModel {
        let instance = DeviceInstance::new(
            DeviceId::new(0),
            "cmp",
            DeviceKind::Comparator,
            (1..=5).map(NodeId::new).collect(),
            Default::default(),
        );
        ComparatorModel::from_instance(&instance).expect("model")
    }

    fn solution(input: f64, reference: f64) -> Solution {
        let mut solution = Solution::zero(6, 0);
        solution.node_voltages[1] = input;
        solution.node_voltages[2] = reference;
        solution.node_voltages[3] = 5.0;
        solution.node_voltages[4] = 0.0;
        solution
    }

    #[test]
    fn comparator_requires_five_pins() {
        let instance = DeviceInstance::new(
            DeviceId::new(0),
            "cmp",
            DeviceKind::Comparator,
            vec![NodeId::GROUND; 4],
            Default::default(),
        );
        assert!(ComparatorModel::from_instance(&instance).is_err());
    }

    #[test]
    fn comparator_saturates_high_and_low() {
        let model = model();
        assert!(model.output_level(&solution(3.0, 2.5)) > 4.9);
        assert!(model.output_level(&solution(2.0, 2.5)) < 0.1);
    }

    #[test]
    fn comparator_rejects_invalid_gain() {
        let mut instance = DeviceInstance::new(
            DeviceId::new(0),
            "cmp",
            DeviceKind::Comparator,
            vec![NodeId::GROUND; 5],
            Default::default(),
        );
        instance.set_parameter("gain", 0.0);
        assert!(ComparatorModel::from_instance(&instance).is_err());
    }
}
