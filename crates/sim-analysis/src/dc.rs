//! DC operating point and sweep analysis.

use crate::system::SimulationSystem;
use sim_core::tolerance::approx_eq;
use sim_core::{Circuit, SimError, Solution};

/// DC solver configuration.
#[derive(Clone, Debug)]
pub struct DcConfig {
    /// Maximum Newton iterations.
    pub max_iterations: usize,
    /// Relative convergence tolerance.
    pub rel_tol: f64,
    /// Absolute convergence tolerance.
    pub abs_tol: f64,
    /// Minimum conductance added to each node.
    pub gmin: f64,
}

impl Default for DcConfig {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            rel_tol: 1e-6,
            abs_tol: 1e-9,
            gmin: 1e-12,
        }
    }
}

/// A DC operating point result.
#[derive(Clone, Debug)]
pub struct DcResult {
    /// Converged solution.
    pub solution: Solution,
    /// Number of iterations used.
    pub iterations: usize,
}

/// One point in a DC sweep.
#[derive(Clone, Debug)]
pub struct DcPoint {
    /// Sweep value.
    pub sweep: f64,
    /// Solution at this value.
    pub solution: Solution,
}

/// Solves the DC operating point of a circuit.
pub fn dc_operating_point(circuit: &Circuit, config: &DcConfig) -> Result<DcResult, SimError> {
    let mut system = SimulationSystem::new(circuit)?;
    let initial = system.zero_solution();
    solve_dc(&mut system, initial, config)
}

/// Sweeps a device parameter and solves the DC operating point at each step.
pub fn dc_sweep(
    circuit: &Circuit,
    device: &str,
    parameter: &str,
    start: f64,
    stop: f64,
    step: f64,
    config: &DcConfig,
) -> Result<Vec<DcPoint>, SimError> {
    let sweep = SweepSpec {
        device,
        parameter,
        start,
        stop,
        step,
    };
    run_sweep(circuit, &sweep, config)
}

struct SweepSpec<'a> {
    device: &'a str,
    parameter: &'a str,
    start: f64,
    stop: f64,
    step: f64,
}

fn run_sweep(
    circuit: &Circuit,
    sweep: &SweepSpec<'_>,
    config: &DcConfig,
) -> Result<Vec<DcPoint>, SimError> {
    validate_sweep(sweep.start, sweep.stop, sweep.step)?;
    collect_sweep_points(circuit, sweep, config)
}

fn collect_sweep_points(
    circuit: &Circuit,
    sweep: &SweepSpec<'_>,
    config: &DcConfig,
) -> Result<Vec<DcPoint>, SimError> {
    let mut points = Vec::new();
    let mut value = sweep.start;
    while value <= sweep.stop + sweep.step * 1e-9 {
        append_sweep_point(&mut points, circuit, sweep, value, config)?;
        value += sweep.step;
    }
    Ok(points)
}

fn append_sweep_point(
    points: &mut Vec<DcPoint>,
    circuit: &Circuit,
    sweep: &SweepSpec<'_>,
    value: f64,
    config: &DcConfig,
) -> Result<(), SimError> {
    if points.len() >= 1_000_000 {
        return Err(SimError::InvalidNetlist("sweep limit exceeded".to_string()));
    }
    points.push(solve_sweep_point(
        circuit,
        sweep.device,
        sweep.parameter,
        value,
        config,
    )?);
    Ok(())
}

fn solve_sweep_point(
    circuit: &Circuit,
    device: &str,
    parameter: &str,
    value: f64,
    config: &DcConfig,
) -> Result<DcPoint, SimError> {
    let mut swept = circuit.clone();
    swept.set_device_parameter(device, parameter, value)?;
    let result = dc_operating_point(&swept, config)?;
    Ok(DcPoint {
        sweep: value,
        solution: result.solution,
    })
}

fn solve_dc(
    system: &mut SimulationSystem,
    initial: Solution,
    config: &DcConfig,
) -> Result<DcResult, SimError> {
    let mut solution = initial;
    for iteration in 0..config.max_iterations {
        let next = system.solve(&solution, None, config.gmin)?;
        let converged = solution_close(&solution, &next, config);
        solution = next;
        if converged {
            return finish_dc(system, solution, iteration + 1);
        }
    }
    Err(SimError::ConvergenceFailure {
        iteration: config.max_iterations,
    })
}

fn finish_dc(
    system: &mut SimulationSystem,
    solution: Solution,
    iterations: usize,
) -> Result<DcResult, SimError> {
    system.update(&solution)?;
    Ok(DcResult {
        solution,
        iterations,
    })
}

fn solution_close(first: &Solution, second: &Solution, config: &DcConfig) -> bool {
    let voltages = first
        .node_voltages
        .iter()
        .zip(&second.node_voltages)
        .all(|(left, right)| approx_eq(*left, *right, config.rel_tol, config.abs_tol));
    let currents = first
        .branch_currents
        .iter()
        .zip(&second.branch_currents)
        .all(|(left, right)| approx_eq(*left, *right, config.rel_tol, config.abs_tol));
    voltages && currents
}

fn validate_sweep(start: f64, stop: f64, step: f64) -> Result<(), SimError> {
    if !start.is_finite() || !stop.is_finite() || !step.is_finite() || step <= 0.0 {
        return Err(SimError::InvalidNetlist(
            "sweep values must be finite and step must be positive".to_string(),
        ));
    }
    if stop < start {
        return Err(SimError::InvalidNetlist(
            "sweep stop must be greater than or equal to start".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{DcConfig, dc_operating_point, dc_sweep};
    use crate::test_support::{diode_clamp, inverter, rc_circuit, rl_circuit, voltage_divider};

    #[test]
    fn voltage_divider_is_half_supply() {
        let (circuit, output) = voltage_divider();
        let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
        let voltage = result.solution.voltage(output).expect("output");
        assert!((voltage - 5.0).abs() < 1e-6);
    }

    #[test]
    fn dc_sweep_visits_expected_points() {
        let (circuit, output) = voltage_divider();
        let points = dc_sweep(
            &circuit,
            "v1",
            "voltage",
            0.0,
            10.0,
            5.0,
            &DcConfig::default(),
        )
        .expect("sweep");
        assert_eq!(points.len(), 3);
        let middle = points[1].solution.voltage(output).expect("out");
        assert!((middle - 2.5).abs() < 1e-6);
    }

    #[test]
    fn dc_sweep_rejects_zero_step() {
        let (circuit, _) = voltage_divider();
        let result = dc_sweep(
            &circuit,
            "v1",
            "voltage",
            0.0,
            1.0,
            0.0,
            &DcConfig::default(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn dc_sweep_rejects_reverse_range() {
        let (circuit, _) = voltage_divider();
        let result = dc_sweep(
            &circuit,
            "v1",
            "voltage",
            5.0,
            1.0,
            1.0,
            &DcConfig::default(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn dc_sweep_rejects_non_finite_step() {
        let (circuit, _) = voltage_divider();
        let result = dc_sweep(
            &circuit,
            "v1",
            "voltage",
            0.0,
            1.0,
            f64::NAN,
            &DcConfig::default(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn dc_config_default_is_reasonable() {
        let config = DcConfig::default();
        assert!(config.max_iterations > 0);
        assert!(config.rel_tol > 0.0);
        assert!(config.abs_tol > 0.0);
        assert!(config.gmin > 0.0);
    }

    #[test]
    fn inverter_vtc_switches() {
        let (circuit, _, output) = inverter();
        let points = dc_sweep(
            &circuit,
            "vin",
            "voltage",
            0.0,
            5.0,
            0.5,
            &DcConfig::default(),
        )
        .expect("sweep");
        let first = points.first().expect("first");
        let last = points.last().expect("last");
        assert!(first.solution.voltage(output).expect("high") > 4.0);
        assert!(last.solution.voltage(output).expect("low") < 1.0);
    }

    #[test]
    fn inverter_operating_point_converges() {
        let (circuit, _, _) = inverter();
        let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
        assert!(result.iterations > 0);
    }

    #[test]
    fn capacitor_is_open_at_dc() {
        let (circuit, output) = rc_circuit();
        let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
        let voltage = result.solution.voltage(output).expect("output");
        assert!((voltage - 5.0).abs() < 1e-6);
    }

    #[test]
    fn inductor_is_short_at_dc() {
        let (circuit, output) = rl_circuit();
        let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
        let voltage = result.solution.voltage(output).expect("output");
        assert!(voltage.abs() < 1e-6);
    }

    #[test]
    fn dc_sweep_unknown_device_is_rejected() {
        let (circuit, _) = voltage_divider();
        let result = dc_sweep(
            &circuit,
            "missing",
            "voltage",
            0.0,
            1.0,
            0.5,
            &DcConfig::default(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn dc_sweep_single_point_has_solution() {
        let (circuit, _) = voltage_divider();
        let points = dc_sweep(
            &circuit,
            "v1",
            "voltage",
            3.0,
            3.0,
            1.0,
            &DcConfig::default(),
        )
        .expect("sweep");
        assert_eq!(points.len(), 1);
        assert!((points[0].sweep - 3.0).abs() < 1e-12);
    }

    #[test]
    fn diode_clamp_limits_voltage() {
        let (circuit, output) = diode_clamp();
        let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
        let voltage = result.solution.voltage(output).expect("output");
        assert!(voltage > 0.4 && voltage < 0.9, "voltage={voltage}");
    }
}
