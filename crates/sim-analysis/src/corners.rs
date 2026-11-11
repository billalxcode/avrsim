//! Temperature and process corner helpers.

use sim_core::{Circuit, Device, DeviceKind, SimError};

/// Process corner factors applied to device parameters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProcessCorner {
    /// Resistor resistance scale.
    pub resistor: f64,
    /// Capacitor capacitance scale.
    pub capacitor: f64,
    /// MOSFET threshold voltage scale.
    pub vto: f64,
    /// MOSFET transconductance scale.
    pub kp: f64,
}

impl ProcessCorner {
    /// Returns a corner with no scaling.
    #[must_use]
    pub const fn typical() -> Self {
        Self {
            resistor: 1.0,
            capacitor: 1.0,
            vto: 1.0,
            kp: 1.0,
        }
    }
}

/// Returns a copy of the circuit with a device temperature.
pub fn with_temperature(circuit: &Circuit, temperature: f64) -> Result<Circuit, SimError> {
    validate_temperature(temperature)?;
    let mut adjusted = circuit.snapshot();
    let names = temperature_device_names(&adjusted);
    for name in names {
        adjusted.set_device_parameter(&name, "temp", temperature)?;
    }
    Ok(adjusted)
}

/// Returns a copy of the circuit with corner factors applied.
pub fn with_process_corner(circuit: &Circuit, corner: &ProcessCorner) -> Result<Circuit, SimError> {
    validate_corner(corner)?;
    let mut adjusted = circuit.snapshot();
    let changes = corner_changes(&adjusted, corner);
    for (device, parameter, value) in changes {
        adjusted.set_device_parameter(&device, &parameter, value)?;
    }
    Ok(adjusted)
}

fn validate_temperature(temperature: f64) -> Result<(), SimError> {
    if !temperature.is_finite() || temperature <= 0.0 {
        return Err(SimError::InvalidNetlist(
            "temperature must be positive".to_string(),
        ));
    }
    Ok(())
}

fn validate_corner(corner: &ProcessCorner) -> Result<(), SimError> {
    let factors = [corner.resistor, corner.capacitor, corner.vto, corner.kp];
    if factors
        .iter()
        .any(|value| !value.is_finite() || *value <= 0.0)
    {
        return Err(SimError::InvalidNetlist(
            "process corner factors must be positive".to_string(),
        ));
    }
    Ok(())
}

fn temperature_device_names(circuit: &Circuit) -> Vec<String> {
    circuit
        .devices()
        .iter()
        .filter(|device| is_temperature_dependent(device.kind()))
        .map(|device| device.name().to_string())
        .collect()
}

fn is_temperature_dependent(kind: &DeviceKind) -> bool {
    matches!(
        kind,
        DeviceKind::Resistor | DeviceKind::Diode | DeviceKind::Nmos | DeviceKind::Pmos
    )
}

fn corner_changes(circuit: &Circuit, corner: &ProcessCorner) -> Vec<(String, String, f64)> {
    let mut changes = Vec::new();
    for device in circuit.devices() {
        collect_corner_changes(&mut changes, device, corner);
    }
    changes
}

fn collect_corner_changes(
    changes: &mut Vec<(String, String, f64)>,
    device: &impl Device,
    corner: &ProcessCorner,
) {
    match device.kind() {
        DeviceKind::Resistor => push_scale(changes, device, "resistance", corner.resistor),
        DeviceKind::Capacitor => push_scale(changes, device, "capacitance", corner.capacitor),
        DeviceKind::Nmos | DeviceKind::Pmos => {
            push_scale(changes, device, "vto", corner.vto);
            push_scale(changes, device, "kp", corner.kp);
        }
        _ => {}
    }
}

fn push_scale(
    changes: &mut Vec<(String, String, f64)>,
    device: &impl Device,
    parameter: &str,
    scale: f64,
) {
    if let Some(value) = device.parameters().get(parameter).copied() {
        changes.push((
            device.name().to_string(),
            parameter.to_string(),
            value * scale,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::{ProcessCorner, with_process_corner, with_temperature};
    use crate::ac::ac_analysis;
    use crate::dc::{DcConfig, dc_operating_point};
    use crate::test_support::{
        TestCircuit, current_source_circuit, diode_clamp, rc_circuit, voltage_divider,
    };
    use sim_core::{Circuit, DeviceKind, NodeId};

    fn dc_voltage(circuit: &Circuit, output: NodeId) -> f64 {
        let result = dc_operating_point(circuit, &DcConfig::default()).expect("dc");
        result.solution.voltage(output).expect("output")
    }

    fn solve_temperature(circuit: &Circuit, output: NodeId, temperature: f64) -> f64 {
        let adjusted = with_temperature(circuit, temperature).expect("temperature");
        dc_voltage(&adjusted, output)
    }

    fn nmos_load() -> (Circuit, NodeId) {
        let mut circuit = TestCircuit::new();
        let vdd = circuit.node("vdd");
        let drain = circuit.node("drain");
        let ground = circuit.ground();
        circuit.voltage("vdd_src", vdd, ground, 5.0);
        circuit.resistor("r1", vdd, drain, 1000.0);
        circuit.mosfet("m1", DeviceKind::Nmos, drain, drain, ground, 10e-6);
        (circuit.build(), drain)
    }

    #[test]
    fn temperature_applies_resistor_coefficient() {
        let (mut circuit, output) = voltage_divider();
        circuit
            .set_device_parameter("r1", "tc1", 0.01)
            .expect("tc1");
        let hot = with_temperature(&circuit, 400.15).expect("temperature");
        let voltage = dc_voltage(&hot, output);
        assert!(
            (voltage - 10.0 * 1000.0 / 3000.0).abs() < 1e-6,
            "v={voltage}"
        );
    }

    #[test]
    fn temperature_raises_diode_clamp_voltage() {
        let (circuit, output) = diode_clamp();
        let cold = solve_temperature(&circuit, output, 300.15);
        let hot = solve_temperature(&circuit, output, 400.15);
        assert!(hot > cold + 0.01, "cold={cold} hot={hot}");
    }

    #[test]
    fn process_corner_scales_resistance() {
        let (circuit, output) = current_source_circuit();
        let corner = ProcessCorner {
            resistor: 2.0,
            ..ProcessCorner::typical()
        };
        let adjusted = with_process_corner(&circuit, &corner).expect("corner");
        let voltage = dc_voltage(&adjusted, output);
        assert!((voltage - 2.0).abs() < 1e-6, "v={voltage}");
    }

    #[test]
    fn process_corner_scales_capacitance() {
        let (mut circuit, output) = rc_circuit();
        circuit.set_device_parameter("v1", "ac", 1.0).expect("ac");
        let cutoff = 1.0 / (2.0 * std::f64::consts::PI * 1000.0 * 1e-6);
        let corner = ProcessCorner {
            capacitor: 2.0,
            ..ProcessCorner::typical()
        };
        let adjusted = with_process_corner(&circuit, &corner).expect("corner");
        let result = ac_analysis(&adjusted, cutoff).expect("ac");
        let magnitude = result.magnitude(output).expect("magnitude");
        assert!(
            (magnitude - 1.0 / 5.0_f64.sqrt()).abs() < 1e-6,
            "m={magnitude}"
        );
    }

    #[test]
    fn process_corner_lowers_threshold_voltage() {
        let (circuit, output) = nmos_load();
        let base = dc_voltage(&circuit, output);
        let corner = ProcessCorner {
            vto: 0.5,
            ..ProcessCorner::typical()
        };
        let adjusted = with_process_corner(&circuit, &corner).expect("corner");
        let lowered = dc_voltage(&adjusted, output);
        assert!(lowered < base - 0.01, "base={base} lowered={lowered}");
    }

    #[test]
    fn corners_reject_invalid_factors() {
        let (circuit, _) = voltage_divider();
        assert!(with_temperature(&circuit, 0.0).is_err());
        let invalid = ProcessCorner {
            resistor: 0.0,
            ..ProcessCorner::typical()
        };
        assert!(with_process_corner(&circuit, &invalid).is_err());
    }
}
