//! Quasi-static distortion analysis for memoryless nonlinear circuits.

use crate::dc::{DcConfig, dc_operating_point};
use sim_core::{Circuit, Device, DeviceKind, NodeId, SimError};

/// Distortion analysis configuration.
#[derive(Clone, Debug)]
pub struct DistortionConfig {
    /// Sinusoidal input amplitude.
    pub amplitude: f64,
    /// Number of samples over one period.
    pub samples: usize,
    /// Highest harmonic order, including the fundamental.
    pub harmonics: usize,
    /// DC solver configuration.
    pub dc: DcConfig,
}

impl Default for DistortionConfig {
    fn default() -> Self {
        Self {
            amplitude: 0.1,
            samples: 256,
            harmonics: 5,
            dc: DcConfig::default(),
        }
    }
}

/// Harmonic distortion result.
#[derive(Clone, Debug)]
pub struct DistortionResult {
    /// Fundamental magnitude.
    pub fundamental: f64,
    /// Magnitudes for orders 2 up to the configured maximum.
    pub harmonics: Vec<f64>,
    /// Total harmonic distortion relative to the fundamental.
    pub thd: f64,
}

struct DistortionSpec<'a> {
    circuit: &'a Circuit,
    device: &'a str,
    parameter: &'a str,
    output: NodeId,
    dc: &'a DcConfig,
}

/// Computes harmonic distortion of one output node.
pub fn distortion_analysis(
    circuit: &Circuit,
    device: &str,
    output: NodeId,
    config: &DistortionConfig,
) -> Result<DistortionResult, SimError> {
    validate(circuit, device, output, config)?;
    let parameter = source_parameter(circuit, device)?;
    let bias = source_bias(circuit, device, parameter)?;
    let spec = DistortionSpec {
        circuit,
        device,
        parameter,
        output,
        dc: &config.dc,
    };
    let outputs = sample_outputs(&spec, bias, config)?;
    build_result(harmonic_amplitudes(&outputs, config.harmonics))
}

fn validate(
    circuit: &Circuit,
    device: &str,
    output: NodeId,
    config: &DistortionConfig,
) -> Result<(), SimError> {
    validate_config(config)?;
    validate_output(circuit, output)?;
    source_parameter(circuit, device).map(|_| ())
}

fn validate_config(config: &DistortionConfig) -> Result<(), SimError> {
    if !config.amplitude.is_finite() || config.amplitude <= 0.0 {
        return Err(SimError::InvalidNetlist(
            "distortion amplitude must be positive".to_string(),
        ));
    }
    if config.samples < 8 || config.samples > 1_000_000 {
        return Err(SimError::InvalidNetlist(
            "distortion samples must be between 8 and 1000000".to_string(),
        ));
    }
    if config.harmonics < 2 || config.harmonics * 2 >= config.samples {
        return Err(SimError::InvalidNetlist(
            "distortion harmonics exceed the Nyquist limit".to_string(),
        ));
    }
    Ok(())
}

fn validate_output(circuit: &Circuit, output: NodeId) -> Result<(), SimError> {
    if circuit.node(output).is_none() {
        return Err(SimError::InvalidNetlist(format!(
            "unknown distortion output node: {}",
            output.raw()
        )));
    }
    Ok(())
}

fn source_parameter(circuit: &Circuit, device: &str) -> Result<&'static str, SimError> {
    let instance = circuit
        .device_by_name(device)
        .ok_or_else(|| SimError::InvalidNetlist(format!("unknown device: {device}")))?;
    match instance.kind() {
        DeviceKind::VoltageSource => Ok("voltage"),
        DeviceKind::CurrentSource => Ok("current"),
        _ => Err(SimError::InvalidNetlist(format!(
            "distortion source must be independent: {device}"
        ))),
    }
}

fn source_bias(circuit: &Circuit, device: &str, parameter: &str) -> Result<f64, SimError> {
    let instance = circuit
        .device_by_name(device)
        .ok_or_else(|| SimError::InvalidNetlist(format!("unknown device: {device}")))?;
    instance
        .parameters()
        .get(parameter)
        .copied()
        .ok_or_else(|| SimError::InvalidNetlist(format!("missing parameter: {device}.{parameter}")))
}

fn sample_outputs(
    spec: &DistortionSpec<'_>,
    bias: f64,
    config: &DistortionConfig,
) -> Result<Vec<f64>, SimError> {
    let mut values = Vec::with_capacity(config.samples);
    for index in 0..config.samples {
        let angle = std::f64::consts::TAU * index as f64 / config.samples as f64;
        let input = bias + config.amplitude * angle.sin();
        values.push(spec.solve(input)?);
    }
    Ok(values)
}

impl DistortionSpec<'_> {
    fn solve(&self, input: f64) -> Result<f64, SimError> {
        let mut perturbed = self.circuit.snapshot();
        perturbed.set_device_parameter(self.device, self.parameter, input)?;
        let solution = dc_operating_point(&perturbed, self.dc)?.solution;
        solution.voltage(self.output).ok_or_else(|| {
            SimError::InvalidNetlist("distortion output missing from solution".to_string())
        })
    }
}

fn harmonic_amplitudes(outputs: &[f64], harmonics: usize) -> Vec<f64> {
    let count = outputs.len() as f64;
    (1..=harmonics)
        .map(|order| harmonic_amplitude(outputs, count, order))
        .collect()
}

fn harmonic_amplitude(outputs: &[f64], count: f64, order: usize) -> f64 {
    let mut real = 0.0;
    let mut imaginary = 0.0;
    for (index, value) in outputs.iter().enumerate() {
        let angle = std::f64::consts::TAU * order as f64 * index as f64 / count;
        real += value * angle.cos();
        imaginary += value * angle.sin();
    }
    let scale = 2.0 / count;
    (real * scale).hypot(imaginary * scale)
}

fn build_result(amplitudes: Vec<f64>) -> Result<DistortionResult, SimError> {
    let fundamental = amplitudes.first().copied().unwrap_or(0.0);
    if fundamental <= 1e-12 {
        return Err(SimError::InvalidNetlist(
            "distortion fundamental is zero".to_string(),
        ));
    }
    let energy: f64 = amplitudes.iter().skip(1).map(|value| value * value).sum();
    Ok(DistortionResult {
        fundamental,
        harmonics: amplitudes[1..].to_vec(),
        thd: energy.sqrt() / fundamental,
    })
}

#[cfg(test)]
mod tests {
    use super::{DistortionConfig, distortion_analysis};
    use crate::DcConfig;
    use crate::test_support::{diode_clamp, voltage_divider};

    fn config(amplitude: f64) -> DistortionConfig {
        DistortionConfig {
            amplitude,
            samples: 256,
            harmonics: 5,
            dc: DcConfig::default(),
        }
    }

    #[test]
    fn linear_divider_has_zero_thd() {
        let (circuit, output) = voltage_divider();
        let result = distortion_analysis(&circuit, "v1", output, &config(0.5)).expect("distortion");
        assert!(
            (result.fundamental - 0.25).abs() < 1e-9,
            "f={}",
            result.fundamental
        );
        assert!(result.thd < 1e-9, "thd={}", result.thd);
    }

    #[test]
    fn diode_clamp_has_measurable_distortion() {
        let (circuit, output) = diode_clamp();
        let mut settings = config(1.0);
        settings.samples = 512;
        let result = distortion_analysis(&circuit, "v1", output, &settings).expect("distortion");
        assert!(result.thd > 0.001, "thd={}", result.thd);
    }

    #[test]
    fn distortion_rejects_invalid_inputs() {
        let (circuit, output) = voltage_divider();
        assert!(distortion_analysis(&circuit, "v1", output, &config(0.0)).is_err());
        assert!(distortion_analysis(&circuit, "r1", output, &config(0.1)).is_err());
        assert!(distortion_analysis(&circuit, "missing", output, &config(0.1)).is_err());
        let mut invalid = config(0.1);
        invalid.samples = 8;
        invalid.harmonics = 4;
        assert!(distortion_analysis(&circuit, "v1", output, &invalid).is_err());
    }
}
