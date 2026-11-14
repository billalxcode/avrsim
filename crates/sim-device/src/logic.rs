//! Five-pin behavioral logic gate.

use crate::model::{DeviceModel, branch_index, parameter, pins_exact, positive_parameter};
use sim_core::{Device, DeviceId, DeviceInstance, MnaContext, NodeId, SimError, Solution};

const MAX_GATE: u8 = 8;

/// A five-pin gate with pins `in1, in2, V+, V-, out`.
pub struct LogicGateModel {
    id: DeviceId,
    gate: u8,
    threshold: f64,
    width: f64,
    pins: [NodeId; 5],
}

impl LogicGateModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 5)?;
        let gate = gate_code(instance)?;
        let threshold = gate_threshold(instance)?;
        let width = positive_parameter(instance.parameters(), "width", Some(0.1))?;
        Ok(Self {
            id: instance.id(),
            gate,
            threshold,
            width,
            pins: [pins[0], pins[1], pins[2], pins[3], pins[4]],
        })
    }

    fn output_level(&self, solution: &Solution) -> f64 {
        let high = solution.voltage(self.pins[2]).unwrap_or(0.0);
        let low = solution.voltage(self.pins[3]).unwrap_or(0.0);
        let first = sigmoid(self.input(solution, self.pins[0]));
        let second = sigmoid(self.input(solution, self.pins[1]));
        (high - low).max(0.0) * gate_fraction(self.gate, first, second)
    }

    fn input(&self, solution: &Solution, pin: NodeId) -> f64 {
        (solution.voltage(pin).unwrap_or(0.0) - self.threshold) / self.width
    }
}

fn gate_code(instance: &DeviceInstance) -> Result<u8, SimError> {
    let gate = parameter(instance.parameters(), "gate", 0.0);
    if !gate.is_finite() || gate < 0.0 || gate > f64::from(MAX_GATE) || gate.fract() != 0.0 {
        return Err(SimError::InvalidNetlist(
            "logic gate code must be an integer from 0 to 8".to_string(),
        ));
    }
    Ok(gate as u8)
}

fn gate_threshold(instance: &DeviceInstance) -> Result<f64, SimError> {
    let threshold = parameter(instance.parameters(), "vth", 2.5);
    if !threshold.is_finite() {
        return Err(SimError::InvalidNetlist(
            "logic gate threshold must be finite".to_string(),
        ));
    }
    Ok(threshold)
}

fn sigmoid(value: f64) -> f64 {
    1.0 / (1.0 + (-value.clamp(-40.0, 40.0)).exp())
}

fn gate_fraction(gate: u8, first: f64, second: f64) -> f64 {
    let or = first + second - first * second;
    match gate {
        0 => first * second,
        1 => or,
        2 => 1.0 - first,
        3 => 1.0 - first * second,
        4 => 1.0 - or,
        5 => first + second - 2.0 * first * second,
        6 => 1.0 - (first + second - 2.0 * first * second),
        7 | 8 => first,
        _ => 0.0,
    }
}

impl DeviceModel for LogicGateModel {
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
    use super::{LogicGateModel, gate_fraction};
    use sim_core::{DeviceId, DeviceInstance, DeviceKind, NodeId, Solution};

    fn model(gate: f64) -> LogicGateModel {
        let mut instance = DeviceInstance::new(
            DeviceId::new(0),
            "gate",
            DeviceKind::LogicGate,
            (1..=5).map(|raw| NodeId::new(raw as u32)).collect(),
            Default::default(),
        );
        instance.set_parameter("gate", gate);
        LogicGateModel::from_instance(&instance).expect("model")
    }

    fn solution(first: f64, second: f64) -> Solution {
        let mut solution = Solution::zero(6, 0);
        solution.node_voltages[1] = first;
        solution.node_voltages[2] = second;
        solution.node_voltages[3] = 5.0;
        solution.node_voltages[4] = 0.0;
        solution
    }

    #[test]
    fn gate_fraction_matches_truth_tables() {
        assert_eq!(gate_fraction(0, 1.0, 1.0), 1.0);
        assert_eq!(gate_fraction(1, 1.0, 0.0), 1.0);
        assert_eq!(gate_fraction(2, 1.0, 0.0), 0.0);
        assert_eq!(gate_fraction(5, 1.0, 0.0), 1.0);
        assert_eq!(gate_fraction(6, 1.0, 0.0), 0.0);
    }

    #[test]
    fn and_gate_saturates_high_and_low() {
        let gate = model(0.0);
        assert!(gate.output_level(&solution(5.0, 5.0)) > 4.9);
        assert!(gate.output_level(&solution(5.0, 0.0)) < 0.1);
    }

    #[test]
    fn logic_gate_rejects_invalid_code() {
        let instance = DeviceInstance::new(
            DeviceId::new(0),
            "gate",
            DeviceKind::LogicGate,
            vec![NodeId::GROUND; 5],
            Default::default(),
        );
        assert!(LogicGateModel::from_instance(&instance).is_ok());
        for code in [-1.0, 9.0, 1.5] {
            let mut instance = instance.clone();
            instance.set_parameter("gate", code);
            assert!(LogicGateModel::from_instance(&instance).is_err());
        }
    }
}
