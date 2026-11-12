//! AC small-signal analysis with DC bias linearization.

use crate::dc::{DcConfig, dc_operating_point};
use crate::system::{SimulationSystem, build_branch_map, build_node_map, count_nodes};
use sim_core::{Circuit, Complex64, ComplexMatrix, Device, DeviceKind, NodeId, SimError, Solution};
use sim_device::{diode_capacitance, mosfet_capacitances};

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
    ensure_supported(circuit)?;
    let bias = dc_operating_point(circuit, &DcConfig::default())?.solution;
    let node_map = build_node_map(circuit);
    let node_count = count_nodes(&node_map);
    let (branch_map, branch_count) = build_branch_map(circuit);
    let size = node_count + branch_count;
    let mut matrix = bias_matrix(circuit, &bias, size)?;
    let reactive = ReactiveContext::new(&bias, frequency, &node_map, &branch_map);
    add_reactive(circuit, &reactive, &mut matrix)?;
    let mut rhs = vec![Complex64::default(); size];
    stamp_sources(circuit, &node_map, &branch_map, &mut rhs)?;
    let values = matrix.solve(&rhs)?;
    Ok(build_solution(
        frequency, node_map, branch_map, values, node_count,
    ))
}

fn ensure_supported(circuit: &Circuit) -> Result<(), SimError> {
    for device in circuit.devices() {
        if supported(device.kind()) {
            continue;
        }
        return Err(SimError::UnsupportedDevice(format!(
            "AC does not support device: {}",
            device.kind().name()
        )));
    }
    Ok(())
}

fn supported(kind: &DeviceKind) -> bool {
    matches!(
        kind,
        DeviceKind::Resistor
            | DeviceKind::Capacitor
            | DeviceKind::Inductor
            | DeviceKind::VoltageSource
            | DeviceKind::CurrentSource
            | DeviceKind::Diode
            | DeviceKind::Nmos
            | DeviceKind::Pmos
            | DeviceKind::Npn
            | DeviceKind::Pnp
            | DeviceKind::JfetN
            | DeviceKind::JfetP
    )
}

fn bias_matrix(
    circuit: &Circuit,
    bias: &sim_core::Solution,
    size: usize,
) -> Result<ComplexMatrix, SimError> {
    let mut system = SimulationSystem::new(circuit)?;
    let context = system.stamp_dc(bias)?;
    let source = context.matrix();
    if source.rows() != size || source.cols() != size {
        return Err(SimError::Internal("AC bias matrix mismatch".to_string()));
    }
    copy_matrix(source, size)
}

fn copy_matrix(source: &sim_core::DenseMatrix, size: usize) -> Result<ComplexMatrix, SimError> {
    let mut matrix = ComplexMatrix::zeros(size)?;
    for row in 0..size {
        for col in 0..size {
            let value = source.get(row, col).unwrap_or(0.0);
            if value != 0.0 {
                matrix.add(row, col, Complex64::from_real(value))?;
            }
        }
    }
    Ok(matrix)
}

struct ReactiveContext<'a> {
    bias: &'a Solution,
    frequency: f64,
    nodes: &'a [Option<usize>],
    branches: &'a [Option<usize>],
}

impl<'a> ReactiveContext<'a> {
    fn new(
        bias: &'a Solution,
        frequency: f64,
        nodes: &'a [Option<usize>],
        branches: &'a [Option<usize>],
    ) -> Self {
        Self {
            bias,
            frequency,
            nodes,
            branches,
        }
    }
}

impl ReactiveContext<'_> {
    fn node_voltage(&self, node: NodeId) -> f64 {
        self.bias.voltage(node).unwrap_or(0.0)
    }

    fn stamp(
        &self,
        matrix: &mut ComplexMatrix,
        first: NodeId,
        second: NodeId,
        capacitance: f64,
    ) -> Result<(), SimError> {
        let pair = [
            self.nodes[first.raw() as usize],
            self.nodes[second.raw() as usize],
        ];
        stamp_pair(matrix, pair, reactance(capacitance, self.frequency))
    }
}

fn add_reactive(
    circuit: &Circuit,
    context: &ReactiveContext<'_>,
    matrix: &mut ComplexMatrix,
) -> Result<(), SimError> {
    for (index, device) in circuit.devices().iter().enumerate() {
        add_device_reactance(device, index, context, matrix)?;
    }
    Ok(())
}

fn add_device_reactance(
    device: &impl Device,
    index: usize,
    context: &ReactiveContext<'_>,
    matrix: &mut ComplexMatrix,
) -> Result<(), SimError> {
    match device.kind() {
        DeviceKind::Capacitor => stamp_capacitor(device, context.nodes, matrix, context.frequency),
        DeviceKind::Inductor => {
            stamp_inductor(device, index, context.branches, matrix, context.frequency)
        }
        DeviceKind::Diode => add_diode_capacitance(device, context, matrix),
        DeviceKind::Nmos | DeviceKind::Pmos => add_mosfet_capacitances(device, context, matrix),
        _ => Ok(()),
    }
}

fn add_diode_capacitance(
    device: &impl Device,
    context: &ReactiveContext<'_>,
    matrix: &mut ComplexMatrix,
) -> Result<(), SimError> {
    let pins = device.pins();
    let voltage = context.node_voltage(pins[0]) - context.node_voltage(pins[1]);
    let capacitance = diode_capacitance(device.parameters(), voltage)?;
    context.stamp(matrix, pins[0], pins[1], capacitance)
}

fn add_mosfet_capacitances(
    device: &impl Device,
    context: &ReactiveContext<'_>,
    matrix: &mut ComplexMatrix,
) -> Result<(), SimError> {
    let pins = device.pins();
    let body = body_pin(pins);
    let (cgs, cgd, cgb) = mosfet_capacitances(
        device.kind(),
        device.parameters(),
        context.node_voltage(pins[0]),
        context.node_voltage(pins[1]),
        context.node_voltage(pins[2]),
        context.node_voltage(body),
    )?;
    context.stamp(matrix, pins[0], pins[2], cgs)?;
    context.stamp(matrix, pins[0], pins[1], cgd)?;
    context.stamp(matrix, pins[0], body, cgb)
}

fn body_pin(pins: &[NodeId]) -> NodeId {
    if pins.len() == 4 {
        return pins[3];
    }
    pins[2]
}

fn reactance(capacitance: f64, frequency: f64) -> Complex64 {
    Complex64::new(0.0, 2.0 * std::f64::consts::PI * frequency * capacitance)
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
    branches: &[Option<usize>],
    matrix: &mut ComplexMatrix,
    frequency: f64,
) -> Result<(), SimError> {
    let inductance = parameter(device, "inductance")?;
    let value = Complex64::new(0.0, -2.0 * std::f64::consts::PI * frequency * inductance);
    let branch = branch_index(branches, index)?;
    add_entry(matrix, Some(branch), Some(branch), value)
}

fn stamp_sources(
    circuit: &Circuit,
    nodes: &[Option<usize>],
    branches: &[Option<usize>],
    rhs: &mut [Complex64],
) -> Result<(), SimError> {
    for (index, device) in circuit.devices().iter().enumerate() {
        match device.kind() {
            DeviceKind::VoltageSource => {
                stamp_voltage_rhs(device, index, branches, rhs)?;
            }
            DeviceKind::CurrentSource => {
                stamp_current_rhs(device, nodes, rhs)?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn stamp_voltage_rhs(
    device: &impl Device,
    index: usize,
    branches: &[Option<usize>],
    rhs: &mut [Complex64],
) -> Result<(), SimError> {
    let branch = branch_index(branches, index)?;
    rhs[branch] = source_value(device)?;
    Ok(())
}

fn stamp_current_rhs(
    device: &impl Device,
    nodes: &[Option<usize>],
    rhs: &mut [Complex64],
) -> Result<(), SimError> {
    let value = source_value(device)?;
    let pair = node_pair(nodes, device.pins());
    add_rhs(rhs, pair[0], -value)?;
    add_rhs(rhs, pair[1], value)
}

fn build_solution(
    frequency: f64,
    node_map: Vec<Option<usize>>,
    branch_map: Vec<Option<usize>>,
    values: Vec<Complex64>,
    node_count: usize,
) -> AcSolution {
    let branch_map = branch_map
        .into_iter()
        .map(|slot| slot.map(|index| index - node_count))
        .collect();
    AcSolution {
        frequency,
        node_map,
        branch_map,
        node_voltages: values[..node_count].to_vec(),
        branch_currents: values[node_count..].to_vec(),
    }
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

fn branch_index(branches: &[Option<usize>], device: usize) -> Result<usize, SimError> {
    branches
        .get(device)
        .and_then(|value| *value)
        .ok_or_else(|| SimError::Internal("missing AC branch".to_string()))
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

fn source_value(device: &impl Device) -> Result<Complex64, SimError> {
    let magnitude = device
        .parameters()
        .get("ac")
        .copied()
        .or_else(|| device.parameters().get("voltage").copied())
        .or_else(|| device.parameters().get("current").copied())
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
        common_emitter, common_source_n_biased, current_source_circuit, diode_clamp,
        diode_varactor, inverter, matched_line, pulse_divider, rc_circuit, rl_circuit,
        switch_circuit,
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
    fn voltage_source_branch_current_is_observable() {
        let (mut circuit, _) = rc_circuit();
        circuit.set_device_parameter("v1", "ac", 1.0).expect("ac");
        let result = ac_analysis(&circuit, 1000.0).expect("ac");
        let current = result.branch_current(0).expect("current");
        let magnitude = current.magnitude();
        assert!(
            magnitude > 9e-4 && magnitude < 1.1e-3,
            "current={current:?}"
        );
    }

    #[test]
    fn inductor_branch_current_is_observable() {
        let (mut circuit, _) = rl_circuit();
        circuit.set_device_parameter("v1", "ac", 1.0).expect("ac");
        let result = ac_analysis(&circuit, 1.0).expect("ac");
        let current = result.branch_current(2).expect("current");
        assert!(
            (current.magnitude() - 1e-3).abs() < 1e-5,
            "current={current:?}"
        );
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
    fn ac_rejects_switch() {
        let (circuit, _) = switch_circuit(true);
        assert!(ac_analysis(&circuit, 1000.0).is_err());
    }

    #[test]
    fn ac_rejects_transmission_line() {
        let (circuit, _) = matched_line();
        assert!(ac_analysis(&circuit, 1000.0).is_err());
    }

    #[test]
    fn ac_rejects_pulse_source() {
        let (circuit, _) = pulse_divider();
        assert!(ac_analysis(&circuit, 1000.0).is_err());
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

    #[test]
    fn diode_small_signal_uses_dc_bias() {
        let (mut circuit, output) = diode_clamp();
        circuit.set_device_parameter("v1", "ac", 1.0).expect("ac");
        let result = ac_analysis(&circuit, 1000.0).expect("ac");
        let magnitude = result.magnitude(output).expect("magnitude");
        assert!(magnitude > 1e-6 && magnitude < 0.1, "magnitude={magnitude}");
    }

    #[test]
    fn mosfet_inverter_has_gain() {
        let (mut circuit, input, output) = inverter();
        circuit
            .set_device_parameter("vin", "voltage", 2.5)
            .expect("bias");
        circuit.set_device_parameter("vin", "ac", 1.0).expect("ac");
        let result = ac_analysis(&circuit, 1000.0).expect("ac");
        let magnitude = result.magnitude(output).expect("magnitude");
        assert!(magnitude > 1.0, "magnitude={magnitude}");
        assert!(result.magnitude(input).is_some());
    }

    #[test]
    fn diode_capacitance_reduces_response_at_high_frequency() {
        let (mut circuit, output) = diode_varactor();
        circuit.set_device_parameter("v1", "ac", 1.0).expect("ac");
        circuit
            .set_device_parameter("vc", "ac", 0.0)
            .expect("cathode ac");
        let low = ac_analysis(&circuit, 1e3).expect("low");
        let high = ac_analysis(&circuit, 1e8).expect("high");
        let low = low.magnitude(output).expect("low magnitude");
        let high = high.magnitude(output).expect("high magnitude");
        assert!(low > 0.99, "low={low}");
        assert!(high < 0.5, "high={high}");
    }

    #[test]
    fn mosfet_capacitance_reduces_gain_at_high_frequency() {
        let (mut circuit, _, output) = inverter();
        circuit
            .set_device_parameter("vin", "voltage", 2.5)
            .expect("bias");
        circuit.set_device_parameter("vin", "ac", 1.0).expect("ac");
        circuit
            .set_device_parameter("mp", "cox", 1e-12)
            .expect("mp");
        circuit
            .set_device_parameter("mn", "cox", 1e-12)
            .expect("mn");
        let low = ac_analysis(&circuit, 1e3).expect("low");
        let high = ac_analysis(&circuit, 1e9).expect("high");
        let low_gain = low.magnitude(output).expect("low gain");
        let high_gain = high.magnitude(output).expect("high gain");
        assert!(high_gain < low_gain, "low={low_gain} high={high_gain}");
    }

    #[test]
    fn jfet_common_source_has_gain() {
        let (mut circuit, output) = common_source_n_biased();
        circuit.set_device_parameter("vg", "ac", 1.0).expect("ac");
        let result = ac_analysis(&circuit, 1000.0).expect("ac");
        let magnitude = result.magnitude(output).expect("magnitude");
        assert!(magnitude > 0.1, "magnitude={magnitude}");
    }

    #[test]
    fn bjt_common_emitter_has_gain() {
        let (mut circuit, output) = common_emitter();
        circuit.set_device_parameter("vb", "ac", 1.0).expect("ac");
        let result = ac_analysis(&circuit, 1000.0).expect("ac");
        let magnitude = result.magnitude(output).expect("magnitude");
        assert!(magnitude > 1.0, "magnitude={magnitude}");
    }
}
