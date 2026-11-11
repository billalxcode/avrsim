//! Thermal noise analysis for linear circuits.

use crate::ac::{ac_analysis, sweep_frequencies};
use sim_core::{Circuit, Device, DeviceKind, NodeId, SimError};
use std::collections::BTreeMap;

/// Boltzmann constant in joule per kelvin.
pub const BOLTZMANN: f64 = 1.380_649e-23;

/// Default temperature in kelvin (27 deg C).
pub const DEFAULT_TEMPERATURE: f64 = 300.15;

/// Noise sweep configuration.
#[derive(Clone, Copy, Debug)]
pub struct NoiseConfig {
    /// Start frequency in hertz.
    pub start: f64,
    /// Stop frequency in hertz.
    pub stop: f64,
    /// Number of frequency points.
    pub points: usize,
    /// Temperature in kelvin.
    pub temperature: f64,
}

impl Default for NoiseConfig {
    fn default() -> Self {
        Self {
            start: 1.0,
            stop: 1e6,
            points: 100,
            temperature: DEFAULT_TEMPERATURE,
        }
    }
}

/// Output noise density at one frequency.
#[derive(Clone, Copy, Debug)]
pub struct NoisePoint {
    /// Frequency in hertz.
    pub frequency: f64,
    /// Voltage noise density in volt per square root hertz.
    pub density: f64,
}

/// Output noise result.
#[derive(Clone, Debug)]
pub struct NoiseResult {
    /// Density at each frequency.
    pub points: Vec<NoisePoint>,
    /// Integrated root-mean-square noise voltage.
    pub total_rms: f64,
}

struct NoiseSource {
    pins: [NodeId; 2],
    resistance: f64,
}

/// Computes resistor thermal noise at one output node.
pub fn noise_analysis(
    circuit: &Circuit,
    output: NodeId,
    config: &NoiseConfig,
) -> Result<NoiseResult, SimError> {
    validate_analysis(circuit, output, config)?;
    let frequencies = sweep_frequencies(config.start, config.stop, config.points)?;
    let job = NoiseJob {
        circuit,
        output,
        frequencies: &frequencies,
        temperature: config.temperature,
    };
    let psd = job.accumulate_all(&collect_sources(circuit)?)?;
    Ok(finish(psd, frequencies))
}

fn validate_analysis(
    circuit: &Circuit,
    output: NodeId,
    config: &NoiseConfig,
) -> Result<(), SimError> {
    validate_config(config)?;
    validate_output(circuit, output)
}

struct NoiseJob<'a> {
    circuit: &'a Circuit,
    output: NodeId,
    frequencies: &'a [f64],
    temperature: f64,
}

impl NoiseJob<'_> {
    fn accumulate_all(&self, sources: &[NoiseSource]) -> Result<Vec<f64>, SimError> {
        let mut psd = vec![0.0; self.frequencies.len()];
        for (index, source) in sources.iter().enumerate() {
            self.accumulate(source, index, &mut psd)?;
        }
        Ok(psd)
    }

    fn accumulate(
        &self,
        source: &NoiseSource,
        index: usize,
        psd: &mut [f64],
    ) -> Result<(), SimError> {
        let probe = probe_circuit(self.circuit, source, index)?;
        let scale = 4.0 * BOLTZMANN * self.temperature / source.resistance;
        for (slot, frequency) in psd.iter_mut().zip(self.frequencies) {
            *slot += scale * transfer_power(&probe, self.output, *frequency)?;
        }
        Ok(())
    }
}

fn validate_config(config: &NoiseConfig) -> Result<(), SimError> {
    if !config.temperature.is_finite() || config.temperature <= 0.0 {
        return Err(SimError::InvalidNetlist(
            "noise temperature must be positive".to_string(),
        ));
    }
    Ok(())
}

fn validate_output(circuit: &Circuit, output: NodeId) -> Result<(), SimError> {
    if circuit.node(output).is_none() {
        return Err(SimError::InvalidNetlist(format!(
            "unknown noise output node: {}",
            output.raw()
        )));
    }
    Ok(())
}

fn collect_sources(circuit: &Circuit) -> Result<Vec<NoiseSource>, SimError> {
    let mut sources = Vec::new();
    for device in circuit.devices() {
        if !matches!(device.kind(), DeviceKind::Resistor) {
            continue;
        }
        let resistance = resistance_value(device)?;
        sources.push(NoiseSource {
            pins: [device.pins()[0], device.pins()[1]],
            resistance,
        });
    }
    Ok(sources)
}

fn resistance_value(device: &impl Device) -> Result<f64, SimError> {
    let value = device
        .parameters()
        .get("resistance")
        .copied()
        .ok_or_else(|| {
            SimError::InvalidNetlist(format!("missing resistance: {}", device.name()))
        })?;
    if !value.is_finite() || value <= 0.0 {
        return Err(SimError::InvalidNetlist(format!(
            "invalid resistance: {}",
            device.name()
        )));
    }
    Ok(value)
}

fn transfer_power(circuit: &Circuit, output: NodeId, frequency: f64) -> Result<f64, SimError> {
    let solution = ac_analysis(circuit, frequency)?;
    let voltage = solution.node_voltage(output).ok_or_else(|| {
        SimError::InvalidNetlist("noise output missing from AC solution".to_string())
    })?;
    Ok(voltage.magnitude().powi(2))
}

fn probe_circuit(
    circuit: &Circuit,
    source: &NoiseSource,
    index: usize,
) -> Result<Circuit, SimError> {
    let mut probe = circuit.snapshot();
    neutralize_sources(&mut probe)?;
    add_probe(&mut probe, source, index)?;
    Ok(probe)
}

fn neutralize_sources(circuit: &mut Circuit) -> Result<(), SimError> {
    let names: Vec<String> = circuit
        .devices()
        .iter()
        .filter(|device| is_source(device.kind()))
        .map(|device| device.name().to_string())
        .collect();
    for name in names {
        circuit.set_device_parameter(&name, "ac", 0.0)?;
    }
    Ok(())
}

fn is_source(kind: &DeviceKind) -> bool {
    matches!(kind, DeviceKind::VoltageSource | DeviceKind::CurrentSource)
}

fn add_probe(circuit: &mut Circuit, source: &NoiseSource, index: usize) -> Result<(), SimError> {
    let mut parameters = BTreeMap::new();
    parameters.insert("ac".to_string(), 1.0);
    circuit.add_device(
        format!("__noise_probe_{index}"),
        DeviceKind::CurrentSource,
        vec![source.pins[1], source.pins[0]],
        parameters,
    )?;
    Ok(())
}

fn finish(psd: Vec<f64>, frequencies: Vec<f64>) -> NoiseResult {
    let points = frequencies
        .iter()
        .zip(&psd)
        .map(|(frequency, value)| NoisePoint {
            frequency: *frequency,
            density: value.sqrt(),
        })
        .collect();
    NoiseResult {
        points,
        total_rms: integrate(&frequencies, &psd).sqrt(),
    }
}

fn integrate(frequencies: &[f64], psd: &[f64]) -> f64 {
    frequencies
        .windows(2)
        .zip(psd.windows(2))
        .map(|(frequency, value)| {
            let bandwidth = frequency[1] - frequency[0];
            0.5 * (value[0] + value[1]) * bandwidth
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::{BOLTZMANN, DEFAULT_TEMPERATURE, NoiseConfig, noise_analysis};
    use crate::test_support::{rc_circuit, resistor_to_ground, voltage_divider};

    fn config(points: usize) -> NoiseConfig {
        NoiseConfig {
            start: 1.0,
            stop: 1e6,
            points,
            temperature: DEFAULT_TEMPERATURE,
        }
    }

    #[test]
    fn resistor_noise_density_matches_analytic() {
        let (circuit, output) = resistor_to_ground();
        let result = noise_analysis(&circuit, output, &config(10)).expect("noise");
        let expected = (4.0 * BOLTZMANN * DEFAULT_TEMPERATURE * 1000.0).sqrt();
        let density = result.points[0].density;
        assert!((density / expected - 1.0).abs() < 1e-9, "density={density}");
    }

    #[test]
    fn divider_noise_uses_thevenin_resistance() {
        let (circuit, output) = voltage_divider();
        let result = noise_analysis(&circuit, output, &config(10)).expect("noise");
        let expected = (4.0 * BOLTZMANN * DEFAULT_TEMPERATURE * 500.0).sqrt();
        let density = result.points[0].density;
        assert!((density / expected - 1.0).abs() < 1e-9, "density={density}");
    }

    #[test]
    fn rc_integrated_noise_matches_thermal_limit() {
        let (circuit, output) = rc_circuit();
        let result = noise_analysis(&circuit, output, &config(2000)).expect("noise");
        let expected = (BOLTZMANN * DEFAULT_TEMPERATURE / 1e-6).sqrt();
        assert!(
            (result.total_rms / expected - 1.0).abs() < 0.01,
            "rms={}",
            result.total_rms
        );
    }

    #[test]
    fn noise_rejects_invalid_config_and_output() {
        let (circuit, output) = resistor_to_ground();
        let mut invalid = config(10);
        invalid.temperature = 0.0;
        assert!(noise_analysis(&circuit, output, &invalid).is_err());
        assert!(noise_analysis(&circuit, sim_core::NodeId::new(99), &config(10)).is_err());
    }
}
