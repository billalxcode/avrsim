//! Linear AC small-signal analysis.

use sim_core::{Circuit, Complex64, ComplexMatrix, Device, DeviceKind, NodeId, SimError};

/// AC solution at one frequency.
#[derive(Clone, Debug)]
pub struct AcSolution {
    frequency: f64,
    node_map: Vec<Option<usize>>,
    branch_map: Vec<Option<usize>>,
    node_voltages: Vec<Complex64>,
    branch_currents: Vec<Complex64>,
}

impl AcSolution {
    /// Returns the analysis frequency.
    #[must_use]
    pub const fn frequency(&self) -> f64 {
        self.frequency
    }

    /// Returns a node voltage.
    #[must_use]
    pub fn node_voltage(&self, node: NodeId) -> Option<Complex64> {
        let index = *self.node_map.get(node.raw() as usize)?.as_ref()?;
        self.node_voltages.get(index).copied()
    }

    /// Returns a node magnitude.
    #[must_use]
    pub fn magnitude(&self, node: NodeId) -> Option<f64> {
        self.node_voltage(node).map(Complex64::magnitude)
    }

    /// Returns a node phase in degrees.
    #[must_use]
    pub fn phase_degrees(&self, node: NodeId) -> Option<f64> {
        self.node_voltage(node)
            .map(|value| value.phase().to_degrees())
    }

    /// Returns a branch current.
    #[must_use]
    pub fn branch_current(&self, device: usize) -> Option<Complex64> {
        let index = *self.branch_map.get(device)?.as_ref()?;
        self.branch_currents.get(index).copied()
    }
}

/// Runs AC analysis at one frequency.
pub fn ac_analysis(circuit: &Circuit, frequency: f64) -> Result<AcSolution, SimError> {
    validate_frequency(frequency)?;
    let (node_map, node_count) = build_node_map(circuit);
    let (branch_map, branch_count) = build_branch_map(circuit, node_count);
    let size = node_count + branch_count;
    let mut matrix = ComplexMatrix::zeros(size)?;
    let mut rhs = vec![Complex64::default(); size];
    stamp_circuit(
        circuit,
        &node_map,
        &branch_map,
        &mut matrix,
        &mut rhs,
        frequency,
    )?;
    let values = matrix.solve(&rhs)?;
    Ok(build_solution(
        frequency, node_map, branch_map, values, node_count,
    ))
}

fn build_solution(
    frequency: f64,
    node_map: Vec<Option<usize>>,
    branch_map: Vec<Option<usize>>,
    values: Vec<Complex64>,
    node_count: usize,
) -> AcSolution {
    AcSolution {
        frequency,
        node_map,
        branch_map,
        node_voltages: values[..node_count].to_vec(),
        branch_currents: values[node_count..].to_vec(),
    }
}

fn stamp_circuit(
    circuit: &Circuit,
    node_map: &[Option<usize>],
    branch_map: &[Option<usize>],
    matrix: &mut ComplexMatrix,
    rhs: &mut [Complex64],
    frequency: f64,
) -> Result<(), SimError> {
    for (index, device) in circuit.devices().iter().enumerate() {
        stamp_device(device, index, node_map, branch_map, matrix, rhs, frequency)?;
    }
    Ok(())
}

/// Sweeps frequency logarithmically.
pub fn ac_sweep(
    circuit: &Circuit,
    start: f64,
    stop: f64,
    points: usize,
) -> Result<Vec<AcSolution>, SimError> {
    let frequencies = sweep_frequencies(start, stop, points)?;
    frequencies
        .into_iter()
        .map(|frequency| ac_analysis(circuit, frequency))
        .collect()
}

pub(crate) fn sweep_frequencies(
    start: f64,
    stop: f64,
    points: usize,
) -> Result<Vec<f64>, SimError> {
    validate_sweep(start, stop, points)?;
    let ratio = (stop / start).powf(1.0 / (points - 1) as f64);
    let mut frequencies = Vec::with_capacity(points);
    let mut frequency = start;
    for _ in 0..points {
        frequencies.push(frequency);
        frequency *= ratio;
    }
    Ok(frequencies)
}

fn validate_frequency(frequency: f64) -> Result<(), SimError> {
    if !frequency.is_finite() || frequency <= 0.0 {
        return Err(SimError::InvalidNetlist(
            "AC frequency must be positive".to_string(),
        ));
    }
    Ok(())
}

fn validate_sweep(start: f64, stop: f64, points: usize) -> Result<(), SimError> {
    validate_frequency(start)?;
    if !stop.is_finite() || stop < start {
        return Err(SimError::InvalidNetlist(
            "AC sweep range is invalid".to_string(),
        ));
    }
    if points < 2 {
        return Err(SimError::InvalidNetlist(
            "AC sweep needs at least two points".to_string(),
        ));
    }
    Ok(())
}

fn build_node_map(circuit: &Circuit) -> (Vec<Option<usize>>, usize) {
    let mut map = Vec::with_capacity(circuit.nodes().len());
    let mut index = 0;
    for node in circuit.nodes() {
        if node.kind().is_ground() {
            map.push(None);
        } else {
            map.push(Some(index));
            index += 1;
        }
    }
    (map, index)
}

fn build_branch_map(circuit: &Circuit, node_count: usize) -> (Vec<Option<usize>>, usize) {
    let mut map = Vec::with_capacity(circuit.devices().len());
    let mut index = node_count;
    let mut count = 0;
    for device in circuit.devices() {
        match device.kind() {
            DeviceKind::VoltageSource | DeviceKind::Inductor => {
                map.push(Some(index));
                index += 1;
                count += 1;
            }
            _ => map.push(None),
        }
    }
    (map, count)
}

fn stamp_device(
    device: &impl Device,
    index: usize,
    node_map: &[Option<usize>],
    branch_map: &[Option<usize>],
    matrix: &mut ComplexMatrix,
    rhs: &mut [Complex64],
    frequency: f64,
) -> Result<(), SimError> {
    match device.kind() {
        DeviceKind::Resistor | DeviceKind::Capacitor | DeviceKind::Inductor => {
            stamp_passive(device, index, node_map, branch_map, matrix, frequency)
        }
        DeviceKind::VoltageSource | DeviceKind::CurrentSource => {
            stamp_source(device, index, node_map, branch_map, matrix, rhs)
        }
        other => unsupported(other),
    }
}

fn unsupported(kind: &DeviceKind) -> Result<(), SimError> {
    Err(SimError::UnsupportedDevice(format!(
        "AC supports linear devices only: {}",
        kind.name()
    )))
}

fn stamp_passive(
    device: &impl Device,
    index: usize,
    node_map: &[Option<usize>],
    branch_map: &[Option<usize>],
    matrix: &mut ComplexMatrix,
    frequency: f64,
) -> Result<(), SimError> {
    match device.kind() {
        DeviceKind::Resistor => stamp_resistor(device, node_map, matrix),
        DeviceKind::Capacitor => stamp_capacitor(device, node_map, matrix, frequency),
        DeviceKind::Inductor => {
            stamp_inductor(device, index, node_map, branch_map, matrix, frequency)
        }
        _ => Err(SimError::Internal("not a passive device".to_string())),
    }
}

fn stamp_source(
    device: &impl Device,
    index: usize,
    node_map: &[Option<usize>],
    branch_map: &[Option<usize>],
    matrix: &mut ComplexMatrix,
    rhs: &mut [Complex64],
) -> Result<(), SimError> {
    match device.kind() {
        DeviceKind::VoltageSource => {
            stamp_voltage(device, index, node_map, branch_map, matrix, rhs)
        }
        DeviceKind::CurrentSource => stamp_current(device, node_map, rhs),
        _ => Err(SimError::Internal("not a source device".to_string())),
    }
}

fn stamp_resistor(
    device: &impl Device,
    nodes: &[Option<usize>],
    matrix: &mut ComplexMatrix,
) -> Result<(), SimError> {
    let resistance = parameter(device, "resistance")?;
    add_conductance(nodes, device.pins(), 1.0 / resistance, matrix)
}

fn stamp_capacitor(
    device: &impl Device,
    nodes: &[Option<usize>],
    matrix: &mut ComplexMatrix,
    frequency: f64,
) -> Result<(), SimError> {
    let capacitance = parameter(device, "capacitance")?;
    let value = Complex64::new(0.0, 2.0 * std::f64::consts::PI * frequency * capacitance);
    stamp_pair(matrix, node_pair(nodes, device.pins()), value)
}

fn stamp_inductor(
    device: &impl Device,
    index: usize,
    nodes: &[Option<usize>],
    branches: &[Option<usize>],
    matrix: &mut ComplexMatrix,
    frequency: f64,
) -> Result<(), SimError> {
    let inductance = parameter(device, "inductance")?;
    let impedance = Complex64::new(0.0, 2.0 * std::f64::consts::PI * frequency * inductance);
    stamp_branch(
        matrix,
        nodes,
        branches,
        device.pins(),
        index,
        Some(-impedance),
    )
}

fn stamp_voltage(
    device: &impl Device,
    index: usize,
    nodes: &[Option<usize>],
    branches: &[Option<usize>],
    matrix: &mut ComplexMatrix,
    rhs: &mut [Complex64],
) -> Result<(), SimError> {
    stamp_branch(matrix, nodes, branches, device.pins(), index, None)?;
    let branch = *branches
        .get(index)
        .and_then(|value| value.as_ref())
        .ok_or_else(|| SimError::Internal("missing AC voltage branch".to_string()))?;
    rhs[branch] = source_value(device, "voltage")?;
    Ok(())
}

fn stamp_current(
    device: &impl Device,
    nodes: &[Option<usize>],
    rhs: &mut [Complex64],
) -> Result<(), SimError> {
    let value = source_value(device, "current")?;
    let pair = node_pair(nodes, device.pins());
    add_rhs(rhs, pair[0], -value)?;
    add_rhs(rhs, pair[1], value)
}

fn stamp_branch(
    matrix: &mut ComplexMatrix,
    nodes: &[Option<usize>],
    branches: &[Option<usize>],
    pins: &[NodeId],
    device: usize,
    impedance: Option<Complex64>,
) -> Result<(), SimError> {
    let pair = node_pair(nodes, pins);
    let branch = branch_index(branches, device)?;
    stamp_branch_entries(matrix, pair, branch)?;
    if let Some(value) = impedance {
        add_entry(matrix, Some(branch), Some(branch), value)?;
    }
    Ok(())
}

fn branch_index(branches: &[Option<usize>], device: usize) -> Result<usize, SimError> {
    branches
        .get(device)
        .and_then(|value| *value)
        .ok_or_else(|| SimError::Internal("missing AC branch".to_string()))
}

fn stamp_branch_entries(
    matrix: &mut ComplexMatrix,
    pair: [Option<usize>; 2],
    branch: usize,
) -> Result<(), SimError> {
    let one = Complex64::from_real(1.0);
    add_entry(matrix, pair[0], Some(branch), one)?;
    add_entry(matrix, pair[1], Some(branch), -one)?;
    add_entry(matrix, Some(branch), pair[0], one)?;
    add_entry(matrix, Some(branch), pair[1], -one)
}

fn add_conductance(
    nodes: &[Option<usize>],
    pins: &[NodeId],
    scale: f64,
    matrix: &mut ComplexMatrix,
) -> Result<(), SimError> {
    stamp_pair(matrix, node_pair(nodes, pins), Complex64::from_real(scale))
}

fn stamp_pair(
    matrix: &mut ComplexMatrix,
    pair: [Option<usize>; 2],
    value: Complex64,
) -> Result<(), SimError> {
    let negative = -value;
    add_entry(matrix, pair[0], pair[0], value)?;
    add_entry(matrix, pair[0], pair[1], negative)?;
    add_entry(matrix, pair[1], pair[0], negative)?;
    add_entry(matrix, pair[1], pair[1], value)
}

fn add_entry(
    matrix: &mut ComplexMatrix,
    row: Option<usize>,
    col: Option<usize>,
    value: Complex64,
) -> Result<(), SimError> {
    if let (Some(row), Some(col)) = (row, col) {
        matrix.add(row, col, value)?;
    }
    Ok(())
}

fn add_rhs(rhs: &mut [Complex64], index: Option<usize>, value: Complex64) -> Result<(), SimError> {
    if let Some(index) = index {
        let slot = rhs
            .get_mut(index)
            .ok_or(SimError::IndexOutOfBounds { row: index, col: 0 })?;
        *slot = *slot + value;
    }
    Ok(())
}

fn node_pair(nodes: &[Option<usize>], pins: &[NodeId]) -> [Option<usize>; 2] {
    [nodes[pins[0].raw() as usize], nodes[pins[1].raw() as usize]]
}

fn parameter(device: &impl Device, name: &str) -> Result<f64, SimError> {
    let value = device
        .parameters()
        .get(name)
        .copied()
        .ok_or_else(|| SimError::InvalidNetlist(format!("missing parameter: {name}")))?;
    if !value.is_finite() || value <= 0.0 {
        return Err(SimError::InvalidNetlist(format!(
            "parameter {name} must be positive"
        )));
    }
    Ok(value)
}

fn source_value(device: &impl Device, fallback: &str) -> Result<Complex64, SimError> {
    let magnitude = device
        .parameters()
        .get("ac")
        .copied()
        .or_else(|| device.parameters().get(fallback).copied())
        .unwrap_or(1.0);
    let phase = device.parameters().get("ac_phase").copied().unwrap_or(0.0);
    let radians = phase.to_radians();
    Ok(Complex64::new(
        magnitude * radians.cos(),
        magnitude * radians.sin(),
    ))
}

#[cfg(test)]
mod tests {
    use super::{ac_analysis, ac_sweep};
    use crate::test_support::{
        common_emitter, common_source_n, current_source_circuit, diode_clamp, matched_line,
        rc_circuit, rl_circuit,
    };

    #[test]
    fn rc_low_pass_at_cutoff_has_expected_response() {
        let (mut circuit, output) = rc_circuit();
        circuit.set_device_parameter("v1", "ac", 1.0).expect("ac");
        let cutoff = 1.0 / (2.0 * std::f64::consts::PI * 1000.0 * 1e-6);
        let result = ac_analysis(&circuit, cutoff).expect("ac");
        let magnitude = result.magnitude(output).expect("magnitude");
        let phase = result.phase_degrees(output).expect("phase");
        assert!(
            (magnitude - std::f64::consts::FRAC_1_SQRT_2).abs() < 0.01,
            "magnitude={magnitude}"
        );
        assert!((phase + 45.0).abs() < 1.0, "phase={phase}");
    }

    #[test]
    fn ac_sweep_returns_requested_points() {
        let (circuit, _) = rc_circuit();
        let points = ac_sweep(&circuit, 10.0, 1000.0, 5).expect("sweep");
        assert_eq!(points.len(), 5);
        assert!((points[0].frequency() - 10.0).abs() < 1e-12);
        assert!((points[4].frequency() - 1000.0).abs() < 1e-6);
    }

    #[test]
    fn ac_rejects_nonlinear_device() {
        let (circuit, _) = diode_clamp();
        assert!(ac_analysis(&circuit, 1000.0).is_err());
    }

    #[test]
    fn ac_rejects_bjt() {
        let (circuit, _) = common_emitter();
        assert!(ac_analysis(&circuit, 1000.0).is_err());
    }

    #[test]
    fn ac_rejects_jfet() {
        let (circuit, _) = common_source_n();
        assert!(ac_analysis(&circuit, 1000.0).is_err());
    }

    #[test]
    fn ac_rejects_transmission_line() {
        let (circuit, _) = matched_line();
        assert!(ac_analysis(&circuit, 1000.0).is_err());
    }

    #[test]
    fn ac_rejects_invalid_frequency() {
        let (circuit, _) = rc_circuit();
        assert!(ac_analysis(&circuit, 0.0).is_err());
        assert!(ac_sweep(&circuit, 1000.0, 10.0, 2).is_err());
    }

    #[test]
    fn rl_low_pass_at_cutoff_has_expected_response() {
        let (mut circuit, output) = rl_circuit();
        circuit.set_device_parameter("v1", "ac", 1.0).expect("ac");
        let cutoff = 1000.0 / (2.0 * std::f64::consts::PI * 1e-3);
        let result = ac_analysis(&circuit, cutoff).expect("ac");
        let magnitude = result.magnitude(output).expect("magnitude");
        let phase = result.phase_degrees(output).expect("phase");
        assert!(
            (magnitude - std::f64::consts::FRAC_1_SQRT_2).abs() < 0.01,
            "magnitude={magnitude}"
        );
        assert!((phase - 45.0).abs() < 1.0, "phase={phase}");
    }

    #[test]
    fn source_phase_rotates_response() {
        let (mut circuit, output) = rc_circuit();
        circuit.set_device_parameter("v1", "ac", 1.0).expect("ac");
        circuit
            .set_device_parameter("v1", "ac_phase", 90.0)
            .expect("phase");
        let cutoff = 1.0 / (2.0 * std::f64::consts::PI * 1000.0 * 1e-6);
        let result = ac_analysis(&circuit, cutoff).expect("ac");
        let phase = result.phase_degrees(output).expect("phase");
        assert!((phase - 45.0).abs() < 1.0, "phase={phase}");
    }

    #[test]
    fn current_source_uses_current_as_ac_magnitude() {
        let (circuit, output) = current_source_circuit();
        let result = ac_analysis(&circuit, 1000.0).expect("ac");
        let magnitude = result.magnitude(output).expect("magnitude");
        assert!((magnitude - 1.0).abs() < 1e-9, "magnitude={magnitude}");
    }
}
