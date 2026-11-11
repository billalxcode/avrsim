//! DC sensitivity analysis by numerical differentiation.

use crate::dc::{DcConfig, dc_operating_point};
use sim_core::{Circuit, Device, NodeId, SimError};

/// Relative step used by the central difference.
pub const DEFAULT_RELATIVE_STEP: f64 = 1e-4;

/// Sensitivity of one node voltage to one device parameter.
#[derive(Clone, Debug)]
pub struct DcSensitivity {
    /// Device name.
    pub device: String,
    /// Parameter name.
    pub parameter: String,
    /// Parameter value at the operating point.
    pub value: f64,
    /// Derivative `dV(node) / d(parameter)`.
    pub derivative: f64,
}

/// Computes `dV(node) / d(parameter)` with a central difference.
pub fn dc_sensitivity(
    circuit: &Circuit,
    device: &str,
    parameter: &str,
    node: NodeId,
    config: &DcConfig,
) -> Result<DcSensitivity, SimError> {
    let value = parameter_value(circuit, device, parameter)?;
    validate_node(circuit, node)?;
    let step = sensitivity_step(value);
    let plus = perturbed_voltage(circuit, device, parameter, value + step, node, config)?;
    let minus = perturbed_voltage(circuit, device, parameter, value - step, node, config)?;
    Ok(DcSensitivity {
        device: device.to_string(),
        parameter: parameter.to_string(),
        value,
        derivative: (plus - minus) / (2.0 * step),
    })
}

fn parameter_value(circuit: &Circuit, device: &str, parameter: &str) -> Result<f64, SimError> {
    let instance = circuit
        .device_by_name(device)
        .ok_or_else(|| SimError::InvalidNetlist(format!("unknown device: {device}")))?;
    instance
        .parameters()
        .get(parameter)
        .copied()
        .ok_or_else(|| SimError::InvalidNetlist(format!("unknown parameter: {device}.{parameter}")))
}

fn validate_node(circuit: &Circuit, node: NodeId) -> Result<(), SimError> {
    if circuit.node(node).is_none() {
        return Err(SimError::InvalidNetlist(format!(
            "unknown node: {}",
            node.raw()
        )));
    }
    Ok(())
}

fn sensitivity_step(value: f64) -> f64 {
    value.abs().max(1.0) * DEFAULT_RELATIVE_STEP
}

fn perturbed_voltage(
    circuit: &Circuit,
    device: &str,
    parameter: &str,
    value: f64,
    node: NodeId,
    config: &DcConfig,
) -> Result<f64, SimError> {
    let mut perturbed = circuit.snapshot();
    perturbed.set_device_parameter(device, parameter, value)?;
    let solution = dc_operating_point(&perturbed, config)?.solution;
    solution
        .voltage(node)
        .ok_or_else(|| SimError::InvalidNetlist("node missing from solution".to_string()))
}

#[cfg(test)]
mod tests {
    use super::dc_sensitivity;
    use crate::DcConfig;
    use crate::test_support::voltage_divider;

    #[test]
    fn voltage_divider_sensitivity_matches_analytic() {
        let (circuit, output) = voltage_divider();
        let config = DcConfig::default();
        let first = dc_sensitivity(&circuit, "r1", "resistance", output, &config).expect("r1");
        let second = dc_sensitivity(&circuit, "r2", "resistance", output, &config).expect("r2");
        assert!(
            (first.derivative + 0.0025).abs() < 1e-6,
            "d1={}",
            first.derivative
        );
        assert!(
            (second.derivative - 0.0025).abs() < 1e-6,
            "d2={}",
            second.derivative
        );
    }

    #[test]
    fn source_sensitivity_matches_analytic() {
        let (circuit, output) = voltage_divider();
        let result = dc_sensitivity(&circuit, "v1", "voltage", output, &DcConfig::default())
            .expect("sensitivity");
        assert!(
            (result.derivative - 0.5).abs() < 1e-9,
            "d={}",
            result.derivative
        );
    }

    #[test]
    fn sensitivity_rejects_unknown_device_or_parameter() {
        let (circuit, output) = voltage_divider();
        let config = DcConfig::default();
        assert!(dc_sensitivity(&circuit, "missing", "resistance", output, &config).is_err());
        assert!(dc_sensitivity(&circuit, "r1", "missing", output, &config).is_err());
    }

    #[test]
    fn sensitivity_rejects_unknown_node() {
        let (circuit, _) = voltage_divider();
        let node = sim_core::NodeId::new(99);
        assert!(dc_sensitivity(&circuit, "r1", "resistance", node, &DcConfig::default()).is_err());
    }
}
